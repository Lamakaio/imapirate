use super::{
    player::PlayerPositionUpdate, worldgen::tile_kind_from_sprite_id, worldgen::Biome, CHUNK_SIZE,
};
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
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

#[derive(Default)]
pub struct SeaHandles {
    pub sea_sheet: Handle<TextureAtlas>,
    pub boat: Handle<TextureAtlas>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl From<MapTile> for TileKind {
    fn from(t: MapTile) -> Self {
        let id = match t {
            MapTile::Static(id) => id as i32,
            MapTile::Animated(v) => v[0] as i32,
        };
        tile_kind_from_sprite_id(id)
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
            //.add_startup_system(draw_chunks_system.system())
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
    mut sea: ResMut<SeaLayerMem>,
    atlases: ResMut<Assets<TextureAtlas>>,
    handles: ResMut<SeaHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    //initializing the sea animation
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

struct ChunkGenData {
    tiles: Vec<Vec<MapTile>>,
    chunk_x: i32,
    chunk_y: i32,
    sea_atlas_handle: Handle<TextureAtlas>,
}
struct ChunksChannel {
    sender: crossbeam_channel::Sender<ChunkGenData>,
    receiver: crossbeam_channel::Receiver<ChunkGenData>,
}

impl Default for ChunksChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        ChunksChannel { sender, receiver }
    }
}

fn draw_chunks_system(
    commands: &mut Commands,
    channels: Local<ChunksChannel>,
    seeded_hasher: Res<SeededHasher>,
    pos_update: Res<PlayerPositionUpdate>,
    worldgen_config: Res<Arc<Vec<(Handle<TextureAtlas>, Biome)>>>,
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
                chunks.insert(
                    (*x, *y),
                    Chunk {
                        drawn: true,
                        ..Default::default()
                    },
                );
                let hasher = seeded_hasher.get_hasher();
                let worldgen_config = worldgen_config.clone();
                let channel_sender = channels.sender.clone();
                let x = *x;
                let y = *y;
                std::thread::spawn(move || {
                    let (tiles, handle) = generate_chunk(x, chunk_y, hasher, worldgen_config);
                    channel_sender.send(ChunkGenData {
                        tiles,
                        chunk_x: x,
                        chunk_y: y,
                        sea_atlas_handle: handle,
                    })
                });
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

    loop {
        match channels.receiver.try_recv() {
            Err(_) => break,
            Ok(data) => {
                let layers = vec![Layer {
                    tiles: data.tiles,
                    atlas_handle: data.sea_atlas_handle.clone(),
                    ..Default::default()
                }];
                let tilemap_builder = TileMapBuilder {
                    layers,
                    layer_offset: 1,
                    transform: Transform {
                        translation: Vec3::new(
                            (TILE_SIZE * SCALING * CHUNK_SIZE * data.chunk_x) as f32,
                            (TILE_SIZE * SCALING * CHUNK_SIZE * data.chunk_y) as f32,
                            0.,
                        ),
                        scale: SCALING as f32 * Vec3::one(),
                        ..Default::default()
                    },
                    chunk_x: data.chunk_x,
                    chunk_y: data.chunk_y,
                    center: true,
                    store_chunk: true,
                };
                commands.spawn((tilemap_builder,));
            }
        }
    }
}

fn despawn_chunk_system(
    commands: &mut Commands,
    pos_update: Res<PlayerPositionUpdate>,
    mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
    chunk_query: Query<(Entity, &Transform, &ChunkLayer)>,
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
