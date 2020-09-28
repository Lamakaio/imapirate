use crate::tilemap::{Chunk, ChunkLayer, SCALING};
use bevy::{ecs::bevy_utils::HashMap, prelude::*};

use crate::loading::{GameState, LoadEvent, LoadEventReader};

use super::{
    player::Player, player::PlayerPositionUpdate, worldgen::CHUNK_SIZE, worldgen::TILE_SIZE,
};

#[derive(Default)]
struct SeaSaveState {
    player: Player,
    player_transform: Transform,
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
) {
    for event in event_reader.reader.iter(&events) {
        if event.state != GameState::Sea {
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
                    println!("should not happen");
                }
            }
        }
    }
}

fn load_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    events: Res<Events<LoadEvent>>,
    save: Res<SeaSaveState>,
    mut pos_update: ResMut<PlayerPositionUpdate>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    mut event_reader: Local<LoadEventReader>,
) {
    for event in event_reader.reader.iter(&events) {
        if event.state == GameState::Sea {
            pos_update.force_update();
            let texture_handle = asset_server
                .load_sync(&mut textures, "assets/sprites/sea/ship_sheet.png")
                .unwrap();
            let texture = textures.get(&texture_handle).unwrap();
            let texture_atlas = TextureAtlas::from_grid(texture_handle, texture.size, 8, 1);
            let texture_atlas_handle = texture_atlases.add(texture_atlas);
            //spawning entities
            commands
                //player
                .spawn(SpriteSheetComponents {
                    texture_atlas: texture_atlas_handle,
                    transform: save.player_transform,
                    ..Default::default()
                })
                .with(save.player.clone());
        }
    }
}
