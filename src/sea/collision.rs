use bevy::{ecs::bevy_utils::HashMap, prelude::*};

use crate::tilemap::{Chunk, ChunkDrawnEvent, ChunkDrawnEventReader, CollisionType, Tile};

use super::{loader::SeaFlag, player::PlayerPositionUpdate};

pub struct SeaCollisionPlugin;
impl Plugin for SeaCollisionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(collision_system.system())
            .add_event::<CollisionAddedEvent>()
            .init_resource::<CollisionAddedEventReader>()
            .add_system(add_collisions_system.system());
    }
}

pub struct CollisionAddedEvent {
    pub x: i32,
    pub y: i32,
}
#[derive(Default)]
pub struct CollisionAddedEventReader {
    pub reader: EventReader<CollisionAddedEvent>,
}

fn add_collisions_system(
    draw_events: Res<Events<ChunkDrawnEvent>>,
    mut collision_events: ResMut<Events<CollisionAddedEvent>>,
    mut event_reader: Local<ChunkDrawnEventReader>,
    mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
    mut flag: Query<&SeaFlag>,
) {
    for _ in &mut flag.iter() {
        //use a flag because it would run even when not in the right gamemode
        for event in event_reader.reader.iter(&draw_events) {
            let mut collision_map = Vec::new();
            if let Some(mut chunk) = chunks.get_mut(&(event.x, event.y)) {
                for (y, row) in chunk.layers[0].tiles.iter().enumerate() {
                    collision_map.push(Vec::new());
                    for (_, tile) in row.iter().enumerate() {
                        collision_map[y].push(get_collision_type(&tile));
                    }
                }
                chunk.collision_map = Some(collision_map);
                collision_events.send(CollisionAddedEvent {
                    x: event.x,
                    y: event.y,
                })
            } else {
                panic!("Draw event even though there is no chunk")
            }
        }
    }
}

pub fn get_collision_type(tile: &Tile) -> CollisionType {
    let id = match tile {
        Tile::Static(id) => *id,
        Tile::Animated(l) => l[0],
    };
    match id {
        id if (id <= 112 && id >= 1) || (id <= 180 && id >= 125) => CollisionType::Friction(None),
        id if (id <= 116 && id >= 113) || (id <= 188 && id >= 181) || (id <= 124 && id >= 121) => {
            CollisionType::Rigid(None)
        }
        _ => CollisionType::None,
    }
}

fn collision_system(
    chunks: Res<HashMap<(i32, i32), Chunk>>,
    mut pos_update: ResMut<PlayerPositionUpdate>,
) {
    if pos_update.changed_tile {
        let chunk = chunks
            .get(&(pos_update.chunk_x, pos_update.chunk_y))
            .unwrap();
        if let Some(map) = &chunk.collision_map {
            pos_update.collision_status = *map
                .get(pos_update.tile_y as usize)
                .unwrap_or(&Vec::new())
                .get(pos_update.tile_x as usize)
                .unwrap_or(&CollisionType::None);
        }
    }
}
