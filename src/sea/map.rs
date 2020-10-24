use super::{player::PlayerPositionUpdate, CHUNK_SIZE};
use super::{worldgen::generate_chunk, SCALING, TILE_SIZE};
use crate::{
    materials::{Pipelines, SeaMaterial},
    tilemap::{AnimatedSyncMap, Chunk, ChunkLayer, Layer, TileMapBuilder, TileMapPlugin},
    util::SeededHasher,
};
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;

#[derive(Default)]
pub struct SeaHandles {
    pub base_islands_sheet: Handle<TextureAtlas>,
    pub sea_material: Handle<SeaMaterial>,
    pub sea_mesh: Handle<Mesh>,
    pub boat: Handle<TextureAtlas>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Sand(bool),
    Forest,
    Sea(bool),
}
impl Default for TileKind {
    fn default() -> Self {
        TileKind::Sea(false)
    }
}

#[derive(Default, Clone, Copy)]
pub struct Tile {
    pub kind: TileKind,
    pub variant: u32,
    pub sprite_id: u32,
}

pub struct SeaMapPlugin;
impl Plugin for SeaMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(TileMapPlugin)
            .add_startup_system(setup.system())
            .add_startup_system(draw_chunks_system.system())
            .init_resource::<Time>()
            .init_resource::<SeaHandles>()
            .add_system(draw_chunks_system.system())
            .add_system(despawn_chunk_system.system());
    }
}

fn setup(
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut handles: ResMut<SeaHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SeaMaterial>>,
) {
    //loading textures
    let texture_handle_map_spritesheet = asset_server.load("sprites/sea/sheet.png");

    //initializing the sea animation
    let island_atlas =
        TextureAtlas::from_grid(texture_handle_map_spritesheet, Vec2::new(16., 16.), 4, 47);
    handles.base_islands_sheet = atlases.add(island_atlas);
    handles.sea_mesh = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(
            (TILE_SIZE * SCALING * CHUNK_SIZE) as f32,
            (TILE_SIZE * SCALING * CHUNK_SIZE) as f32,
        ),
        flip: false,
    }));
    handles.sea_material = materials.add(SeaMaterial::default())
}

fn draw_chunks_system(
    mut commands: Commands,
    pipelines: Res<Pipelines>,
    hasher: Res<SeededHasher>,
    pos_update: Res<PlayerPositionUpdate>,
    handles: Res<SeaHandles>,
    mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
) {
    if pos_update.changed_chunk {
        let chunk_x = pos_update.chunk_x;
        let chunk_y = pos_update.chunk_y;
        let surroundings = [
            (chunk_x, chunk_y),
            (chunk_x + 1, chunk_y),
            (chunk_x + 1, chunk_y + 1),
            (chunk_x + 1, chunk_y - 1),
            (chunk_x, chunk_y + 1),
            (chunk_x, chunk_y - 1),
            (chunk_x - 1, chunk_y + 1),
            (chunk_x - 1, chunk_y),
            (chunk_x - 1, chunk_y - 1),
        ];
        for (x, y) in surroundings.iter() {
            if chunks.contains_key(&(*x, *y)) {
                let chunk = chunks.get_mut(&(*x, *y)).unwrap();
                if chunk.drawn {
                    continue;
                }
                chunk.drawn = true;
                for component in &mut chunk.bundles {
                    commands.spawn(component.clone());
                }
            } else {
                let tiles = generate_chunk(*x, *y, hasher.get_hasher());
                let atlas_handle = handles.base_islands_sheet.clone();
                let layers = vec![Layer {
                    tiles,
                    atlas_handle,
                    ..Default::default()
                }];
                let tilemap_builder = TileMapBuilder {
                    layers,
                    layer_offset: 1,
                    transform: Transform {
                        translation: Vec3::new(
                            (TILE_SIZE * SCALING * CHUNK_SIZE * x) as f32,
                            (TILE_SIZE * SCALING * CHUNK_SIZE * y) as f32,
                            0.,
                        ),
                        scale: SCALING as f32 * Vec3::one(),
                        ..Default::default()
                    },
                    chunk_x: *x,
                    chunk_y: *y,
                    center: true,
                };
                commands.spawn((tilemap_builder,));
            }
            let sea_chunk = MeshComponents {
                mesh: handles.sea_mesh.clone(),
                render_pipelines: pipelines.sea.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        (TILE_SIZE * SCALING * CHUNK_SIZE * x) as f32,
                        (TILE_SIZE * SCALING * CHUNK_SIZE * y) as f32,
                        0.,
                    ),
                    scale: Vec3::one(),
                    ..Default::default()
                },
                ..Default::default()
            };
            commands.spawn(sea_chunk).with(handles.sea_material.clone());
        }
    }
}

fn despawn_chunk_system(
    mut commands: Commands,
    pos_update: Res<PlayerPositionUpdate>,
    mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
    mut chunk_query: Query<(Entity, &Transform, &ChunkLayer)>,
) {
    if pos_update.changed_chunk {
        for (entity, tile_pos, _) in &mut chunk_query.iter() {
            let tile_pos = tile_pos.translation;
            let limit = (CHUNK_SIZE * TILE_SIZE * SCALING) as f32 * 2.5;
            if (tile_pos.x() - pos_update.get_x()).abs() > limit
                || (tile_pos.y() - pos_update.get_y()).abs() > limit
            {
                let chunk_x =
                    (tile_pos.x() / (TILE_SIZE * SCALING * CHUNK_SIZE) as f32).floor() as i32;
                let chunk_y =
                    (tile_pos.y() / (TILE_SIZE * SCALING * CHUNK_SIZE) as f32).floor() as i32;
                if let Some(chunk) = chunks.get_mut(&(chunk_x, chunk_y)) {
                    chunk.drawn = false;
                    commands.despawn(entity);
                } else {
                    panic!("Attempted to despawn nonexistent chunk !");
                }
            }
        }
    }
}
