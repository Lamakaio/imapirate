use std::{time::Duration, collections::VecDeque};

use bevy::{
    prelude::*, 
    render::render_graph::base::MainPass,  
    render::pipeline::RenderPipeline, 
    render::pipeline::PipelineSpecialization, 
    render::pipeline::DynamicBinding, 
    sprite::QUAD_HANDLE, 
    render::pipeline::PrimitiveTopology, 
    render::mesh::VertexAttribute, 
    ecs::bevy_utils::HashMap, sprite::SPRITE_PIPELINE_HANDLE
};
pub const SCALING : i32 = 4;
struct AnimatedMap;

pub struct AnimatedSyncMap;

#[derive(Default)]
pub struct SyncAnimationRessource (usize, Timer);

#[derive(Clone)]
pub struct ChunkLayer {
    id : i32
}

pub struct Chunk {
    pub drawn : bool,
    pub bundles : VecDeque<LayerComponents>,
    pub collision_map : VecDeque<Vec<Vec<CollisionType>>>
}

#[derive(Debug, Clone, Copy)]
pub enum CollisionType {
    Rigid, 
    Friction, 
    None
}

#[derive(Bundle)]
pub struct LayerComponents {
    pub sprite: Sprite,
    pub mesh: Handle<Mesh>, // TODO: maybe abstract this out
    pub material: Handle<ColorMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform : GlobalTransform,
    pub animation: Vec<Handle<Mesh>>,
    pub flag: ChunkLayer
}

impl Clone for LayerComponents {
    fn clone(&self) -> Self {
        LayerComponents {
            main_pass : MainPass, 
            sprite : Sprite {
                size : self.sprite.size, 
                resize_mode : match self.sprite.resize_mode {
                    SpriteResizeMode::Automatic => SpriteResizeMode::Automatic, 
                    SpriteResizeMode::Manual => SpriteResizeMode::Manual
                }
            },
            material : self.material,
            render_pipelines : self.render_pipelines.clone(), 
            draw : self.draw.clone(),
            mesh : self.mesh,
            animation : self.animation.clone(),
            transform : self.transform,
            global_transform : self.global_transform,
            flag : ChunkLayer {id : self.flag.id}
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
            )]),
            draw: Draw {
                is_transparent: true,
                ..Default::default()
            },
            sprite: Default::default(),
            main_pass: MainPass,
            material: Default::default(),
            transform: Default::default(),
            global_transform : Default::default(),
            flag : ChunkLayer {id : 0}, 
            animation : Vec::new()
        }
    }
}

#[derive(Default, Debug)]
pub struct ImageFile {
    tilesize_x : u32,
    tilesize_y : u32,
    path : String,
}

pub type TileId = u32;

#[derive(Debug, Clone)]
pub enum Tile {
    Static (TileId),
    Animated (Vec<TileId>)
}
#[derive(Debug, Clone)]
pub struct Layer {
    pub tiles : Vec<Vec<Tile>>,
    pub atlas_handle : Handle<TextureAtlas>,
    pub anim_frame_time : Option<Duration>,
    pub num_frames : usize,
    pub sync : bool,
}
impl Default for Layer {
    fn default() -> Self {
        Layer {
            num_frames : 1,
            tiles : Default::default(),
            atlas_handle : Default::default(),
            anim_frame_time : Default::default(),
            sync : Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct TileMapBuilder {
    pub layers : Vec<Layer>,
    pub layer_offset : i32,
    pub transform : Transform,
    pub chunk_x : i32, 
    pub chunk_y : i32,
}

impl Default for TileMapBuilder {
    fn default() -> Self {
        TileMapBuilder {
            layers : Vec::new(),
            transform : Transform::default(),
            layer_offset : 0,
            chunk_x : 0, 
            chunk_y : 0,
        }
    }
}
pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_system(process_loaded_layers.system())
        .add_system(anim_unsync_map_system.system())
        .add_system(anim_sync_map_system.system())
        .add_resource(SyncAnimationRessource (0, Timer::new(Duration::from_millis(500), true)))
        .init_resource::<HashMap<(i32, i32), Chunk>>()
        ;
    }
}

pub fn project_ortho(pos: Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = tile_width * pos.x();
    let y = tile_height * pos.y();
    Vec2::new(x, y)
}

pub fn get_layer_components(
    texture_atlases: &Assets<TextureAtlas>,
    meshes : &mut Assets<Mesh>, 
    materials : &mut Assets<ColorMaterial>, 
    layer : &Layer,
    current_layer : i32,
    transform : &Transform
) -> (LayerComponents, Vec<Vec<CollisionType>>) {
        let atlas = texture_atlases.get(&layer.atlas_handle).unwrap();
        let tile_size = atlas.textures[0].max - atlas.textures[0].min;
        let mut mesh_list = Vec::new();
        let mut collision_map = Vec::new();
        for frame in 0..layer.num_frames {
            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut indices = Vec::new();
            let mut i = 0;
            let chunk_size = Vec2::new(layer.tiles[0].len() as f32* tile_size.y(), layer.tiles.len() as f32* tile_size.x());
            if frame == 0 {
                collision_map = Vec::with_capacity(chunk_size.x() as usize);
            }
            for (y, row) in layer.tiles.iter().rev().enumerate() {
                collision_map.push(Vec::with_capacity(chunk_size.y() as usize));
                for (x, tile) in row.iter().enumerate() {
                    collision_map[y].push(get_collision_type(tile));
                    let tile_pos = {
                        let start = Vec2::new(
                            x as f32*tile_size.x(),
                            y as f32*tile_size.y(),
                        ) - (chunk_size + tile_size)/Vec2::new(2., 2.);
                        
                        let end = Vec2::new(
                            (x + 1) as f32*tile_size.x(),
                            (y + 1) as f32*tile_size.y(),
                        ) - (chunk_size + tile_size)/Vec2::new(2., 2.);
                        Vec4::new(end.x(), end.y(), start.x(), start.y())
                    };
                    
                    let tile_uv = {
                        let rect = match tile {
                            Tile::Static(id) => if *id > 0 {atlas.textures[*id as usize - 1]} else {continue},
                            Tile::Animated(ids) => {
                                match ids.get(frame) {
                                    None => atlas.textures[0],
                                    Some(id) => if *id > 0 {atlas.textures[*id as usize - 1]} else {continue;}
                                }
                            }
                        };
                        Vec4::new(rect.max.x()/atlas.size.x(), rect.min.y()/atlas.size.y(), rect.min.x()/atlas.size.x(), rect.max.y()/atlas.size.y())
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
                let mesh = Mesh {
                    primitive_topology: PrimitiveTopology::TriangleList,
                    attributes: vec![
                    VertexAttribute::position(positions),
                    VertexAttribute::normal(normals),
                    VertexAttribute::uv(uvs),
                    ],
                    indices: Some(indices),
                };
                let mesh_handle = meshes.add(mesh);
                mesh_list.push(mesh_handle);
            }
        }
        let transform = *transform;
        (LayerComponents {
            sprite : Sprite {
                size : Vec2::new(1., 1.),
                resize_mode: SpriteResizeMode::Manual
            },
            transform : transform
            .with_translation(Vec3::new(0., 0.,current_layer as f32))
            .with_scale(SCALING as f32),
            material : materials.add(ColorMaterial::texture(atlas.texture)),
            mesh : mesh_list[0], 
            animation : mesh_list.clone(),
            flag : ChunkLayer {id : current_layer},
            ..Default::default()
        }, collision_map)
}
fn process_loaded_layers(
    mut commands: Commands,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut meshes : ResMut<Assets<Mesh>>, 
    mut materials : ResMut<Assets<ColorMaterial>>, 
    mut chunks : ResMut<HashMap<(i32, i32), Chunk>>,
    mut query : Query<(Entity, &mut TileMapBuilder)>,
) {
    for (entity, mut tilemap) in &mut query.iter() {
        if tilemap.layers.is_empty() {
            commands.despawn(entity);
            return;
        }
        let current_layer = tilemap.layer_offset + (tilemap.layers.len() - 1) as i32;
        let layer = tilemap.layers.pop().unwrap();
        let (layer_components, collision_map) = get_layer_components(
            &*texture_atlases,
            &mut *meshes, 
            &mut *materials, 
            &layer, 
            current_layer, 
            &tilemap.transform
        );
        let chunk_opt = chunks.get_mut(&(tilemap.chunk_x, tilemap.chunk_y));
        let chunk = 
        if let Some (c) = chunk_opt {
            c
        }
        else {
            chunks.insert((tilemap.chunk_x, tilemap.chunk_y), Chunk {
                drawn : true, 
                bundles : VecDeque::new(),
                collision_map : VecDeque::new()
            });
            chunks.get_mut(&(tilemap.chunk_x, tilemap.chunk_y)).unwrap()
        };
        chunk.bundles.push_front(layer_components.clone());
        chunk.collision_map.push_front(collision_map);
        if let Some(duration) = layer.anim_frame_time {
            if layer.sync {
                commands.spawn(layer_components)
                .with(AnimatedSyncMap);
            }
            else {
                commands.spawn(layer_components)
                .with((0, Timer::new(duration, true)))
                .with(AnimatedMap);
            }
        }
        else {
            commands
            .spawn(layer_components);
        }
    }
}

fn anim_unsync_map_system(
    time : Res<Time>,
    mut query : Query<(&mut Handle<Mesh>, &Vec<Handle<Mesh>>, &mut usize, &mut Timer, &AnimatedMap)>
) {
    for (mut mesh, meshes, mut current, mut timer, _) in &mut query.iter() {
        timer.tick(time.delta_seconds);
        if timer.finished {
            *current += 1;
            if *current >= meshes.len() {
                *current = 0;
            }
            *mesh = meshes[*current];
        }
        
    }
}

fn anim_sync_map_system(
    time : Res<Time>,
    mut anim_ressource : ResMut<SyncAnimationRessource>,
    mut query : Query<(&mut Handle<Mesh>, &Vec<Handle<Mesh>>, &AnimatedSyncMap)>,
) {
    let SyncAnimationRessource (current, timer) = &mut *anim_ressource;
    timer.tick(time.delta_seconds);
    if timer.finished {
        *current += 1;
        for (mut mesh, meshes, _) in &mut query.iter() {
            if *current >= meshes.len() {
                *current = 0;
            }
            *mesh = meshes[*current];
        }
        
    }
}


pub fn get_collision_type(tile : &Tile) -> CollisionType {
    let id = match tile {
        Tile::Static(id) => *id, 
        Tile::Animated(l) => l[0]
    };
    match id {
        id if (id <= 112 && id >= 1) || (id <= 180 && id >= 125) => CollisionType::Friction, 
        id if (id <= 116 && id >= 113) || (id <= 188 && id >= 181) || (id <= 124 && id >= 121) => CollisionType::Rigid, 
        _ => CollisionType::None
    }
}