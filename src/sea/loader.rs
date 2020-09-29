use crate::tilemap::{Chunk, ChunkLayer};
use bevy::{ecs::bevy_utils::HashMap, prelude::*};

use crate::loading::{GameState, LoadEvent, LoadEventReader};

use super::{
    map::SeaHandles, player::Player, player::PlayerPositionUpdate, CHUNK_SIZE, SCALING, TILE_SIZE,
};

pub const BOAT_LAYER: f32 = 100.;

pub struct SeaFlag;

struct SeaSaveState {
    player: Player,
    player_transform: Transform,
}
impl Default for SeaSaveState {
    fn default() -> Self {
        SeaSaveState {
            player: Player::default(),
            player_transform: Transform::from_translation(Vec3::new(0., 0., BOAT_LAYER))
                .with_scale(2.),
        }
    }
}

pub struct SeaLoaderPlugin;
impl Plugin for SeaLoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(unload_system.system())
            .add_system(load_system.system())
            .init_resource::<SeaSaveState>();
    }
}

fn unload_system(
    mut commands: Commands,
    events: Res<Events<LoadEvent>>,
    mut save: ResMut<SeaSaveState>,
    mut event_reader: Local<LoadEventReader>,
    mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
    mut player_query: Query<(Entity, &Transform, &Player)>,
    mut chunk_query: Query<(Entity, &Transform, &ChunkLayer)>,
    mut flag_query: Query<(Entity, &SeaFlag)>,
) {
    for event in event_reader.reader.iter(&events) {
        if event.state != GameState::Sea {
            for (flag_entity, _) in &mut flag_query.iter() {
                //only despawn things if the flag is there
                commands.despawn(flag_entity);
                for (entity, transform, player) in &mut player_query.iter() {
                    save.player = player.clone();
                    save.player_transform = *transform;
                    commands.despawn(entity);
                }
                for (entity, tile_pos, _) in &mut chunk_query.iter() {
                    let tile_pos = tile_pos.translation();
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
}

fn load_system(
    mut commands: Commands,
    events: Res<Events<LoadEvent>>,
    save: Res<SeaSaveState>,
    handles: Res<SeaHandles>,
    mut pos_update: ResMut<PlayerPositionUpdate>,
    mut event_reader: Local<LoadEventReader>,
) {
    for event in event_reader.reader.iter(&events) {
        if event.state == GameState::Sea {
            pos_update.force_update();
            //spawning entities
            commands
                //player
                .spawn(SpriteSheetComponents {
                    texture_atlas: handles.boat,
                    transform: save.player_transform,
                    ..Default::default()
                })
                .with(save.player.clone())
                //flag
                .spawn((SeaFlag,));
        }
    }
}
