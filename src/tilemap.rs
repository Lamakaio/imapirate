use std::{collections::VecDeque, time::Duration};

use bevy::{
    ecs::bevy_utils::HashMap, prelude::*, render::mesh::Indices, render::mesh::VertexAttribute,
    render::pipeline::DynamicBinding, render::pipeline::PipelineSpecialization,
    render::pipeline::PrimitiveTopology, render::pipeline::RenderPipeline,
    render::render_graph::base::MainPass, sprite::QUAD_HANDLE, sprite::SPRITE_PIPELINE_HANDLE,
};
//HOW TO USE : spawn a TileMapBuilder bundle with the components you want and a LayerComponents should be generated from it, and display on screen hopefully

//event raised when a chunk is drawn on screen
pub struct ChunkDrawnEvent {
    pub x: i32,
    pub y: i32,
}
#[derive(Default)]
pub struct ChunkDrawnEventReader {
    pub reader: EventReader<ChunkDrawnEvent>,
}

//Flags for layers that must be animated, all in sync or separatly.
#[derive(Clone, Copy, Properties, Default)]
struct AnimatedMap {
    current: usize,
}
#[derive(Clone, Copy, Properties, Default)]
pub struct AnimatedSyncMap;

//the ressource to sync the animation
#[derive(Default)]
pub struct SyncAnimationRessource(usize, Timer);

//Flag for a chunk layer component
#[derive(Clone, Copy, Properties, Default)]
pub struct ChunkLayer {
    id: i32,
}

//the chunk data stored in a hashmap for future use.
pub struct Chunk {
    pub drawn: bool,
    pub bundles: VecDeque<LayerComponents>,
    pub collision_map: Option<Vec<Vec<CollisionType>>>,
    pub layers: VecDeque<Layer>,
}

#[derive(Debug, Clone, Copy)]
pub enum CollisionType {
    Rigid(Option<u64>), //rigid and friction collision types store the islands to which they belong, to be filled later.
    Friction(Option<u64>),
    None,
}

//An animation is a vec of mesh, the texture atlas is always the same
#[derive(Clone, Default, Properties)]
pub struct TileAnimation(Vec<Handle<Mesh>>);

//The bundle for a chunk layer. It is based on SpriteComponents, just with aditional components
#[derive(Bundle)]
pub struct LayerComponents {
    pub sprite: Sprite,
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub animation: TileAnimation,
    pub flag: ChunkLayer,
}

impl Clone for LayerComponents {
    fn clone(&self) -> Self {
        LayerComponents {
            main_pass: MainPass,
            sprite: Sprite {
                size: self.sprite.size,
                resize_mode: match self.sprite.resize_mode {
                    SpriteResizeMode::Automatic => SpriteResizeMode::Automatic,
                    SpriteResizeMode::Manual => SpriteResizeMode::Manual,
                }, //SpriteResizeMode doesn't derive Clone
            },
            material: self.material.clone(),
            render_pipelines: self.render_pipelines.clone(),
            draw: self.draw.clone(),
            mesh: self.mesh.clone(),
            animation: self.animation.clone(),
            transform: self.transform,
            global_transform: self.global_transform,
            flag: ChunkLayer { id: self.flag.id },
        }
    }
}

impl Default for LayerComponents {
    fn default() -> Self {
        Self {
            mesh: QUAD_HANDLE,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
                SPRITE_PIPELINE_HANDLE,
                PipelineSpecialization {
                    dynamic_bindings: vec![
                        // Transform
                        DynamicBinding {
                            bind_group: 2,
                            binding: 0,
                        },
                        // Sprite
                        DynamicBinding {
                            bind_group: 2,
                            binding: 1,
                        },
                    ],
                    ..Default::default()
                },
            )]), //the default sprite render pipeline
            draw: Draw {
                is_transparent: true,
                ..Default::default()
            },
            sprite: Sprite {
                size: Vec2::new(1., 1.),
                resize_mode: SpriteResizeMode::Manual,
            }, //SpriteResizeMode must be set to manual because we use a spritesheet and not an individual sprite.
            main_pass: MainPass,
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            flag: ChunkLayer { id: 0 },
            animation: TileAnimation::default(),
        }
    }
}

pub type TileId = u32;
//a tile can be either static or animated
#[derive(Debug, Clone)]
pub enum Tile {
    Static(TileId),
    Animated(Vec<TileId>),
}

//each layer an have its own texture atlas
#[derive(Debug, Clone)]
pub struct Layer {
    pub tiles: Vec<Vec<Tile>>,
    pub atlas_handle: Handle<TextureAtlas>,
    pub anim_frame_time: Option<Duration>,
    pub num_frames: usize,
    pub sync: bool,
}
impl Default for Layer {
    fn default() -> Self {
        Layer {
            num_frames: 1,
            tiles: Default::default(),
            atlas_handle: Default::default(),
            anim_frame_time: Default::default(),
            sync: Default::default(),
        }
    }
}

//If this component is added, a LayerComponent will be build for it in the background, then it will be deleted.
// It is computed one layer pe frame
#[derive(Debug)]
pub struct TileMapBuilder {
    pub layers: Vec<Layer>,
    pub layer_offset: i32, //If we want the first layer to be higher than 0
    pub transform: Transform,
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub center: bool,
}

impl Default for TileMapBuilder {
    fn default() -> Self {
        TileMapBuilder {
            layers: Vec::new(),
            transform: Transform::default(),
            layer_offset: 0,
            chunk_x: 0,
            chunk_y: 0,
            center: true,
        }
    }
}
pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(process_loaded_layers.system())
            .add_system(anim_unsync_map_system.system())
            .add_system(anim_sync_map_system.system())
            .add_resource(SyncAnimationRessource(
                0,
                Timer::new(Duration::from_millis(500), true),
            ))
            .init_resource::<HashMap<(i32, i32), Chunk>>()
            .register_component::<TileAnimation>()
            .register_component::<AnimatedMap>()
            .register_component::<AnimatedSyncMap>()
            .register_component::<ChunkLayer>()
            .init_resource::<ChunkDrawnEventReader>()
            .add_event::<ChunkDrawnEvent>();
    }
}
//builds a LayerComponents from a layer and a transform (plus some assets)
pub fn get_layer_components(
    texture_atlases: &Assets<TextureAtlas>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    layer: &Layer,
    current_layer: i32,
    transform: &Transform,
    center: bool,
) -> LayerComponents {
    let atlas = texture_atlases.get(&layer.atlas_handle).unwrap();
    let tile_size = atlas.textures[0].max - atlas.textures[0].min; //we assume all tiles are the same size
    let mut mesh_list = Vec::new();
    for frame in 0..layer.num_frames {
        let mut positions = Vec::new(); //everything that must be constructed
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut i = 0;
        let chunk_size = Vec2::new(
            layer.tiles[0].len() as f32 * tile_size.y(),
            layer.tiles.len() as f32 * tile_size.x(),
        );
        for (y, row) in layer.tiles.iter().enumerate() {
            //the iteration order is weird, well it works like this
            for (x, tile) in row.iter().enumerate() {
                //compute the 4 vertex of the tile
                let center_offset = if center {
                    chunk_size / Vec2::new(2., 2.)
                } else {
                    Vec2::new(0., 0.)
                };
                let tile_pos = {
                    let start = Vec2::new(x as f32 * tile_size.x(), y as f32 * tile_size.y())
                        - center_offset;

                    let end = Vec2::new(
                        (x + 1) as f32 * tile_size.x(),
                        (y + 1) as f32 * tile_size.y(),
                    ) - center_offset;
                    Vec4::new(end.x(), end.y(), start.x(), start.y())
                };
                //compute the UVs for the tile
                let tile_uv = {
                    let rect = match tile {
                        Tile::Static(id) => {
                            if *id > 0 {
                                atlas.textures[*id as usize - 1]
                            } else {
                                continue;
                            }
                        }
                        Tile::Animated(ids) => match ids.get(frame) {
                            None => atlas.textures[0],
                            Some(id) => {
                                if *id > 0 {
                                    atlas.textures[*id as usize - 1]
                                } else {
                                    continue;
                                }
                            }
                        },
                    };
                    Vec4::new(
                        rect.max.x() / atlas.size.x(),
                        rect.min.y() / atlas.size.y(),
                        rect.min.x() / atlas.size.x(),
                        rect.max.y() / atlas.size.y(),
                    )
                };

                // X, Y
                positions.push([tile_pos.x(), tile_pos.y(), 0.0]);
                normals.push([0.0, 0.0, 1.0]);
                uvs.push([tile_uv.x(), tile_uv.y()]);

                // X, Y + 1
                positions.push([tile_pos.z(), tile_pos.y(), 0.0]);
                normals.push([0.0, 0.0, 1.0]);
                uvs.push([tile_uv.z(), tile_uv.y()]);

                // X + 1, Y + 1
                positions.push([tile_pos.z(), tile_pos.w(), 0.0]);
                normals.push([0.0, 0.0, 1.0]);
                uvs.push([tile_uv.z(), tile_uv.w()]);

                // X + 1, Y
                positions.push([tile_pos.x(), tile_pos.w(), 0.0]);
                normals.push([0.0, 0.0, 1.0]);
                uvs.push([tile_uv.x(), tile_uv.w()]);

                let mut new_indices = vec![i + 0, i + 2, i + 1, i + 0, i + 3, i + 2];
                indices.append(&mut new_indices);

                i += 4;
            }
        }
        if !positions.is_empty() {
            //if there are points
            let mesh = Mesh {
                primitive_topology: PrimitiveTopology::TriangleList,
                attributes: vec![
                    VertexAttribute::position(positions),
                    VertexAttribute::normal(normals),
                    VertexAttribute::uv(uvs),
                ],
                indices: Some(Indices::U16(indices)),
            };
            let mesh_handle = meshes.add(mesh);
            mesh_list.push(mesh_handle);
        }
    }
    let mut transform = transform.clone();
    *transform.translation.z_mut() += current_layer as f32;
    LayerComponents {
        transform: transform,
        material: materials.add(ColorMaterial::texture(atlas.texture.clone())),
        mesh: mesh_list[0].clone(),
        animation: TileAnimation(mesh_list),
        flag: ChunkLayer { id: current_layer },
        ..Default::default()
    }
}
//build and spawn layers from a TileMapBuilder
fn process_loaded_layers(
    mut commands: Commands,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut events: ResMut<Events<ChunkDrawnEvent>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
    mut query: Query<(Entity, &mut TileMapBuilder)>,
) {
    for (entity, mut tilemap) in &mut query.iter() {
        if tilemap.layers.is_empty() {
            commands.despawn(entity);
            events.send(ChunkDrawnEvent {
                x: tilemap.chunk_x,
                y: tilemap.chunk_y,
            });
            return;
        }
        let current_layer = tilemap.layer_offset + (tilemap.layers.len() - 1) as i32;
        let layer = tilemap.layers.pop().unwrap();
        let layer_components = get_layer_components(
            &*texture_atlases,
            &mut *meshes,
            &mut *materials,
            &layer,
            current_layer,
            &tilemap.transform,
            tilemap.center,
        );
        let chunk_opt = chunks.get_mut(&(tilemap.chunk_x, tilemap.chunk_y));
        let chunk = if let Some(c) = chunk_opt {
            c
        } else {
            chunks.insert(
                (tilemap.chunk_x, tilemap.chunk_y),
                Chunk {
                    drawn: true,
                    bundles: VecDeque::new(),
                    collision_map: None,
                    layers: VecDeque::new(),
                },
            );
            chunks.get_mut(&(tilemap.chunk_x, tilemap.chunk_y)).unwrap()
        };
        if let Some(duration) = layer.anim_frame_time {
            if layer.sync {
                commands
                    .spawn(layer_components.clone())
                    .with(AnimatedSyncMap);
            } else {
                commands
                    .spawn(layer_components.clone())
                    .with(AnimatedMap { current: 0 })
                    .with(Timer::new(duration, true));
            }
        } else {
            commands.spawn(layer_components.clone());
        }
        chunk.bundles.push_front(layer_components.clone());
        chunk.layers.push_front(layer);
    }
}

fn anim_unsync_map_system(
    time: Res<Time>,
    mut query: Query<(
        &mut Handle<Mesh>,
        &Vec<Handle<Mesh>>,
        &mut AnimatedMap,
        &mut Timer,
    )>,
) {
    for (mut mesh, meshes, mut map, mut timer) in &mut query.iter() {
        timer.tick(time.delta_seconds);
        if timer.finished {
            map.current += 1;
            if map.current >= meshes.len() {
                map.current = 0;
            }
            *mesh = meshes[map.current].clone();
        }
    }
}

fn anim_sync_map_system(
    time: Res<Time>,
    mut anim_ressource: ResMut<SyncAnimationRessource>,
    mut query: Query<(&mut Handle<Mesh>, &TileAnimation, &AnimatedSyncMap)>,
) {
    let SyncAnimationRessource(current, timer) = &mut *anim_ressource;
    timer.tick(time.delta_seconds);
    if timer.finished {
        *current += 1;
        for (mut mesh, meshes, _) in &mut query.iter() {
            let TileAnimation(meshes) = meshes;
            if *current >= meshes.len() {
                *current = 0;
            }
            *mesh = meshes[*current].clone();
        }
    }
}
