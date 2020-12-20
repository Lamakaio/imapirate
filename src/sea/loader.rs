use std::sync::Arc;

use crate::{
    land::map::CurrentIsland,
    tilemap::{Chunk, ChunkLayer, CollisionType},
};
use bevy::{ecs::bevy_utils::HashMap, prelude::*};

use crate::loading::{GameState, LoadEvent, LoadEventReader};

use super::{
    map::SeaHandles, player::Player, player::PlayerPositionUpdate, worldgen::Biome, CHUNK_SIZE,
    SCALING, TILE_SIZE,
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
            player_transform: Transform {
                translation: Vec3::new(0., 0., BOAT_LAYER),
                scale: 2. * Vec3::one(),
                ..Default::default()
            },
        }
    }
}

pub struct SeaLoaderPlugin;
impl Plugin for SeaLoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(unload_system.system())
            .add_system(load_system.system())
            .add_system(enter_island_system.system())
            .add_startup_system(setup.system())
            .init_resource::<SeaSaveState>()
            .init_resource::<Arc<Vec<(Handle<TextureAtlas>, Biome)>>>();
    }
}

fn read_worldgen_config() -> Vec<Biome> {
    let worldgen_config_string =
        std::fs::read_to_string("config/worldgen.ron").expect("worldgen config file not found");
    ron::from_str(&worldgen_config_string).expect("syntax error on worldgen config file")
}

fn setup(
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut handles: ResMut<SeaHandles>,
    mut worldgen_config: ResMut<Arc<Vec<(Handle<TextureAtlas>, Biome)>>>,
) {
    //loading textures
    let texture_handle_sea_spritesheet = asset_server.load("sprites/sea/seaTileSheet.png");

    //initializing the sea animation
    let sea_atlas =
        TextureAtlas::from_grid(texture_handle_sea_spritesheet, Vec2::new(64., 64.), 3, 1);
    handles.sea_sheet = atlases.add(sea_atlas);

    *worldgen_config = Arc::new(
        read_worldgen_config()
            .drain(..)
            .map(|worldgen_config| {
                let texture_handle =
                    asset_server.load(std::path::Path::new(&worldgen_config.sea_sheet));
                let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16., 16.), 4, 47);
                (atlases.add(atlas), worldgen_config)
            })
            .collect(),
    );
}

fn unload_system(
    commands: &mut Commands,
    events: Res<Events<LoadEvent>>,
    mut save: ResMut<SeaSaveState>,
    mut event_reader: Local<LoadEventReader>,
    mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
    player_query: Query<(Entity, &Transform, &Player)>,
    chunk_query: Query<(Entity, &Transform, &ChunkLayer)>,
    flag_query: Query<(Entity, &SeaFlag)>,
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
                    let tile_pos = tile_pos.translation;
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
    commands: &mut Commands,
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
                    texture_atlas: handles.boat.clone(),
                    transform: save.player_transform,
                    ..Default::default()
                })
                .with(save.player.clone())
                //flag
                .spawn((SeaFlag,));
        }
    }
}

fn enter_island_system(
    keyboard_input: Res<Input<KeyCode>>,
    pos_update: Res<PlayerPositionUpdate>,
    mut events: ResMut<Events<LoadEvent>>,
    mut current_island: ResMut<CurrentIsland>,
) {
    match pos_update.collision_status {
        CollisionType::Friction(Some(id)) | CollisionType::Rigid(Some(id)) => {
            if keyboard_input.just_pressed(KeyCode::A) {
                current_island.id = id;
                current_island.entrance = (pos_update.tile_x, pos_update.tile_y);
                events.send(LoadEvent {
                    state: GameState::Land,
                });
            }
        }
        _ => (),
    }

    if keyboard_input.just_pressed(KeyCode::B) {
        events.send(LoadEvent {
            state: GameState::Sea,
        });
    }
}
