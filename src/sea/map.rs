use super::{
    background::{BackgroundBundle, TileUv},
    player::PlayerPositionUpdate,
    worldgen::tile_kind_from_sprite_id,
    worldgen::Biome,
    CHUNK_SIZE,
};
use super::{worldgen::generate_chunk, SCALING, TILE_SIZE};
use crate::{
    tilemap::{Chunk, ChunkLayer, Layer, Tile as MapTile, TileMapBuilder, TileMapPlugin},
    util::SeededHasher,
};
use bevy::{ecs::bevy_utils::HashMap, render::pipeline::PipelineDescriptor};
use bevy::{prelude::*, render::camera::Camera};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

#[derive(Default)]
pub struct SeaHandles {
    pub sea_pipeline: Handle<PipelineDescriptor>,
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

pub struct SeaMapPlugin;
impl Plugin for SeaMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(TileMapPlugin)
            .add_startup_system(setup.system())
            //.add_startup_system(draw_chunks_system.system())
            .init_resource::<Time>()
            .init_resource::<SeaHandles>()
            .add_system(draw_chunks_system.system())
            .add_system(move_anim_bg_system.system())
            .add_system(despawn_chunk_system.system());
    }
}

fn setup(commands: &mut Commands, handles: ResMut<SeaHandles>, mut meshes: ResMut<Assets<Mesh>>) {
    //initializing the sea animation
    commands
        // cube
        .spawn(BackgroundBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10000.0 })),
            transform: Transform::from_rotation(Quat::from_rotation_x(3.1415926535 / 2.)),
            texture_atlas: handles.sea_sheet.clone(),
            tile_uv: TileUv {
                uv: Vec2::new(0.0, 0.0),
                scale: 5.,
            },
            ..Default::default()
        });
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
            if (tile_pos.x - pos_update.get_x()).abs() > limit
                || (tile_pos.y - pos_update.get_y()).abs() > limit
            {
                let chunk_x =
                    (tile_pos.x / (TILE_SIZE * SCALING * CHUNK_SIZE) as f32).floor() as i32;
                let chunk_y =
                    (tile_pos.y / (TILE_SIZE * SCALING * CHUNK_SIZE) as f32).floor() as i32;
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

fn move_anim_bg_system(
    mut bg_query: Query<(&mut TileUv, &mut Transform)>,
    camera_query: Query<(&Camera, &Transform)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    if timer.is_none() {
        *timer = Some(Timer::new(Duration::from_millis(500), true));
    }
    let timer = timer.as_mut().unwrap();
    for (mut bg, mut bg_transform) in bg_query.iter_mut() {
        for (_, camera_transform) in camera_query.iter() {
            bg_transform.translation.x = camera_transform.translation.x;
            bg_transform.translation.y = camera_transform.translation.y;
        }
        timer.tick(time.delta_seconds());
        if timer.finished() {
            bg.uv += Vec2::new(1. / 3., 0.);
            if bg.uv.x >= 0.99 {
                bg.uv = Vec2::new(0., 0.)
            }
        }
    }
}
