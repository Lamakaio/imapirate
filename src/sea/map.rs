use super::{player::PlayerPositionUpdate, CHUNK_SIZE};
use super::{worldgen::generate_chunk, SCALING, TILE_SIZE};
use crate::{
    tilemap::{
        get_layer_components, AnimatedSyncMap, Chunk, ChunkLayer, Layer, LayerComponents,
        Tile as MapTile, TileMapBuilder, TileMapPlugin,
    },
    util::SeededHasher,
};
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Default)]
pub struct SeaHandles {
    pub base_islands_sheet: Handle<TextureAtlas>,
    pub sea_sheet: Handle<TextureAtlas>,
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

pub struct SeaLayerMem {
    layer: LayerComponents,
}

pub struct SeaMapPlugin;
impl Plugin for SeaMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(TileMapPlugin)
            .add_startup_system(setup.system())
            .add_startup_system(draw_chunks_system.system())
            .init_resource::<Time>()
            .init_resource::<SeaHandles>()
            .add_resource(SeaLayerMem {
                layer: LayerComponents::default(),
            })
            .add_system(draw_chunks_system.system())
            .add_system(despawn_chunk_system.system());
    }
}

fn get_sea_layer(handles: &ResMut<SeaHandles>) -> Layer {
    let mut tiles = Vec::new();
    let tile = MapTile::Animated(vec![1, 2, 3]);
    for x in 0..CHUNK_SIZE / 4 {
        tiles.push(Vec::new());
        for _ in 0..CHUNK_SIZE / 4 {
            tiles[x as usize].push(tile.clone())
        }
    }
    Layer {
        tiles,
        atlas_handle: handles.sea_sheet.clone(),
        anim_frame_time: Some(Duration::from_millis(500)),
        sync: true,
        num_frames: 3,
    }
}
fn setup(
    asset_server: Res<AssetServer>,
    mut sea: ResMut<SeaLayerMem>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut handles: ResMut<SeaHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    //loading textures
    let texture_handle_map_spritesheet = asset_server.load("sprites/sea/sheet.png");
    let texture_handle_sea_spritesheet = asset_server.load("sprites/sea/seaTileSheet.png");

    //initializing the sea animation
    let island_atlas =
        TextureAtlas::from_grid(texture_handle_map_spritesheet, Vec2::new(16., 16.), 4, 47);
    let sea_atlas =
        TextureAtlas::from_grid(texture_handle_sea_spritesheet, Vec2::new(64., 64.), 3, 1);
    handles.base_islands_sheet = atlases.add(island_atlas);
    handles.sea_sheet = atlases.add(sea_atlas);
    let layer = get_sea_layer(&handles);
    sea.layer = get_layer_components(
        &*atlases,
        &mut *meshes,
        &mut *materials,
        &layer,
        0,
        &Transform::from_scale(SCALING as f32 * Vec3::one()),
        true,
    );
}

fn draw_chunks_system(
    mut commands: Commands,
    seeded_hasher: Res<SeededHasher>,
    pos_update: Res<PlayerPositionUpdate>,
    handles: Res<SeaHandles>,
    sea: Res<SeaLayerMem>,
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
                let tiles = generate_chunk(*x, *y, seeded_hasher.get_hasher());
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
            let mut sea_chunk = sea.layer.clone();
            sea_chunk.transform.translation += Vec3::new(
                (TILE_SIZE * SCALING * CHUNK_SIZE * x) as f32,
                (TILE_SIZE * SCALING * CHUNK_SIZE * y) as f32,
                0.,
            );
            commands.spawn(sea_chunk).with(AnimatedSyncMap);
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
