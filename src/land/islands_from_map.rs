use std::{cmp::max, cmp::min, hash::Hasher};

use crate::sea::{
    collision::CollisionAddedEvent, collision::CollisionAddedEventReader, map::MapParam, CHUNK_SIZE,
};
use crate::tilemap::{Chunk, CollisionType, Tile as MapTile};
use bevy::{ecs::bevy_utils::HashMap, ecs::bevy_utils::HashSet, prelude::*};
pub struct IslandFromMapPlugin;
impl Plugin for IslandFromMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(island_generation_system.system())
            .init_resource::<HashMap<u64, Island>>()
            .add_event::<IslandsAddedEvent>()
            .init_resource::<IslandsAddedEventReader>();
    }
}

pub struct IslandsAddedEvent {
    pub x: i32,
    pub y: i32,
}
#[derive(Default)]
pub struct IslandsAddedEventReader {
    pub reader: EventReader<IslandsAddedEvent>,
}

struct IslandGenData {
    chunk_x: i32,
    chunk_y: i32,
    collision_map: Vec<Vec<CollisionType>>,
    islands: HashMap<u64, Island>,
}

struct IslandChannel {
    sender: crossbeam_channel::Sender<IslandGenData>,
    receiver: crossbeam_channel::Receiver<IslandGenData>,
}

impl Default for IslandChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        IslandChannel { sender, receiver }
    }
}
#[derive(Debug)]
pub struct Island {
    rect: Rect<usize>,
    map: Vec<Vec<MapTile>>,
}

fn island_generation_system(
    receive_channel: Local<IslandChannel>,
    mut event_reader: Local<CollisionAddedEventReader>,
    param: Res<MapParam>,
    collision_events: Res<Events<CollisionAddedEvent>>,
    mut islands_events: ResMut<Events<IslandsAddedEvent>>,
    mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
    mut islands: ResMut<HashMap<u64, Island>>,
) {
    for event in event_reader.reader.iter(&collision_events) {
        if let Some(chunk) = chunks.get(&(event.x, event.y)) {
            let mut collisions = chunk.collision_map.clone().unwrap();
            let tiles = chunk.layers[0].tiles.clone();
            let channel_sender = receive_channel.sender.clone();
            let chunk_x = event.x;
            let chunk_y = event.y;
            let seed = param.seed;
            std::thread::spawn(move || {
                let islands = separate_islands(&mut collisions, &tiles, seed, chunk_x, chunk_y);
                channel_sender.send(IslandGenData {
                    chunk_x,
                    chunk_y,
                    collision_map: collisions,
                    islands,
                })
            });
        } else {
            panic!();
        }
    }
    loop {
        match receive_channel.receiver.try_recv() {
            Err(_) => break,
            Ok(mut data) => {
                if let Some(chunk) = chunks.get_mut(&(data.chunk_x, data.chunk_y)) {
                    chunk.collision_map = Some(data.collision_map);
                    for (k, v) in data.islands.drain() {
                        islands.insert(k, v);
                    }
                    islands_events.send(IslandsAddedEvent {
                        x: data.chunk_x,
                        y: data.chunk_y,
                    })
                }
            }
        }
    }
}

fn separate_islands(
    tiles_collision: &mut Vec<Vec<CollisionType>>,
    tiles: &Vec<Vec<MapTile>>,
    seed: usize,
    chunk_x: i32,
    chunk_y: i32,
) -> HashMap<u64, Island> {
    let mut island_map = HashMap::default();
    for y in 0..tiles_collision.len() {
        for x in 0..tiles_collision[0].len() {
            match tiles_collision[y][x] {
                CollisionType::Rigid(None) | CollisionType::Friction(None) => {
                    let mut hasher = seahash::SeaHasher::new();
                    hasher.write_usize(seed);
                    hasher.write_i32(chunk_x);
                    hasher.write_i32(chunk_y);
                    hasher.write_usize(x + y * CHUNK_SIZE as usize);
                    let island_id = hasher.finish();
                    let (rect, map) = convex_component(tiles_collision, tiles, x, y, island_id);
                    island_map.insert(island_id, Island { rect, map });
                }
                _ => continue,
            }
        }
    }
    island_map
}

fn convex_component(
    tiles_collision: &mut Vec<Vec<CollisionType>>,
    tiles: &Vec<Vec<MapTile>>,
    start_x: usize,
    start_y: usize,
    island_id: u64,
) -> (Rect<usize>, Vec<Vec<MapTile>>) {
    let mut set = HashSet::default();
    let mut left = start_x;
    let mut right = start_x;
    let mut top = start_y;
    let mut bottom = start_y;
    set.insert((start_x, start_y));
    while !set.is_empty() {
        let (x, y) = set.iter().next().cloned().unwrap();
        left = min(x, left);
        right = max(x, right);
        top = max(y, top);
        bottom = min(y, bottom);
        set.remove(&(x, y));
        let valid_surroundings: Vec<(usize, usize)> = vec![
            (x + 1, y),
            (x + 1, y + 1),
            (x + 1, y - 1),
            (x, y + 1),
            (x, y - 1),
            (x - 1, y + 1),
            (x - 1, y),
            (x - 1, y - 1),
        ]
        .drain(0..8)
        .filter(|(x, y)| {
            if let Some(c_type) = tiles_collision
                .get_mut(*y)
                .unwrap_or(&mut Vec::new())
                .get_mut(*x)
            {
                match c_type {
                    CollisionType::Rigid(x) | CollisionType::Friction(x) if x.is_none() => {
                        *x = Some(island_id);
                        true
                    }
                    _ => false,
                }
            } else {
                false
            }
        })
        .collect();
        for valid_point in valid_surroundings.iter() {
            set.insert(*valid_point);
        }
    }
    let mut map = Vec::new();
    for (y, row) in tiles
        .iter()
        .rev()
        .enumerate()
        .filter(|(y, _)| *y >= bottom && *y <= top)
    {
        map.push(Vec::new());
        for (_, tile) in row
            .iter()
            .enumerate()
            .filter(|(x, _)| *x >= left && *x <= right)
        {
            map[y - bottom].push(tile.clone());
        }
    }
    (
        Rect {
            left,
            right,
            top,
            bottom,
        },
        map,
    )
}
