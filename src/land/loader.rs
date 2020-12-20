use std::sync::Arc;

use bevy::{ecs::bevy_utils::HashMap, prelude::*, render::camera::Camera};

use crate::{
    loading::{GameState, LoadEvent, LoadEventReader},
    sea::TILE_SIZE,
    tilemap::ChunkLayer,
};

use super::{
    islands_from_map::Island,
    map::{CurrentIsland, LoadIslandEvent},
    mobs::Mob,
    mobs::MobConfig,
    player::Player,
    LAND_SCALING,
};

pub const BOAT_LAYER: f32 = 100.;

pub struct LandFlag;

struct LandSaveState {
    player: Player,
    player_transform: Transform,
}

impl Default for LandSaveState {
    fn default() -> Self {
        LandSaveState {
            player: Player::default(),
            player_transform: Transform {
                translation: Vec3::new(0., 0., BOAT_LAYER),
                scale: 2. * Vec3::one(),
                ..Default::default()
            },
        }
    }
}

#[derive(Default)]
pub(crate) struct LandHandles {
    pub player: Handle<TextureAtlas>,
    pub tiles: Handle<TextureAtlas>,
}

pub struct LandLoaderPlugin;
impl Plugin for LandLoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_system(unload_system.system())
            .add_system(load_system.system())
            .init_resource::<LandHandles>()
            .init_resource::<LandSaveState>();
    }
}

fn setup(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handles: ResMut<LandHandles>,
    mut mobs_config: ResMut<Arc<Vec<(Handle<ColorMaterial>, MobConfig)>>>,
) {
    //loading textures
    let player_texture_handle = asset_server.load("sprites/land/chara_green_base.png");
    let texture_atlas = TextureAtlas::from_grid(player_texture_handle, Vec2::new(64., 64.), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    handles.player = texture_atlas_handle;

    let tiles_texture_handle = asset_server.load("sprites/land/sheet.png");
    let texture_atlas = TextureAtlas::from_grid(tiles_texture_handle, Vec2::new(16., 16.), 4, 47);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    handles.tiles = texture_atlas_handle;

    *mobs_config = Arc::new(
        read_mob_config()
            .drain(..)
            .map(|mob_config| {
                let texture_handle =
                    asset_server.load(std::path::Path::new(&mob_config.sprite_path));
                (materials.add(texture_handle.into()), mob_config)
            })
            .collect(),
    );
}

fn unload_system(
    commands: &mut Commands,
    events: Res<Events<LoadEvent>>,
    current_island: Res<CurrentIsland>,
    mut islands: ResMut<HashMap<u64, Island>>,
    mut save: ResMut<LandSaveState>,
    mut event_reader: Local<LoadEventReader>,
    player_query: Query<(Entity, &Transform, &Player)>,
    mut mobs_query: Query<(Entity, &mut Mob, &Transform)>,
    chunk_query: Query<(Entity, &ChunkLayer)>,
    flag_query: Query<(Entity, &LandFlag)>,
) {
    for event in event_reader.reader.iter(&events) {
        if event.state != GameState::Land {
            for (flag_entity, _) in flag_query.iter() {
                //only despawn things if the flag is there
                commands.despawn(flag_entity);
                for (entity, transform, player) in player_query.iter() {
                    save.player = player.clone();
                    save.player_transform = *transform;
                    commands.despawn(entity);
                }
                for (entity, _) in chunk_query.iter() {
                    commands.despawn(entity);
                }
                let island = islands
                    .get_mut(&current_island.id)
                    .expect("island does not exist");
                for (entity, mut mob, transform) in mobs_query.iter_mut() {
                    island.mobs.push((std::mem::take(&mut *mob), *transform));
                    commands.despawn(entity);
                }
            }
        }
    }
}

fn load_system(
    commands: &mut Commands,
    events: Res<Events<LoadEvent>>,
    save: Res<LandSaveState>,
    handles: Res<LandHandles>,
    current_island: Res<CurrentIsland>,
    window: Res<WindowDescriptor>,
    mut islands: ResMut<HashMap<u64, Island>>,
    mut transition: ResMut<(f32, Vec3)>,
    mut event_reader: Local<LoadEventReader>,
    mut island_events: ResMut<Events<LoadIslandEvent>>,
    mut camera_query: Query<(&Camera, &mut Transform)>,
) {
    for event in event_reader.reader.iter(&events) {
        let island_id = current_island.id;
        if event.state == GameState::Land {
            island_events.send(LoadIslandEvent { island_id });
            let island = islands.get_mut(&island_id).expect("Island does no exist");
            let (tile_x, tile_y) = current_island.entrance;
            let (x, y) = (
                tile_x - island.rect.left as i32,
                tile_y - island.rect.bottom as i32,
            );
            let player_x = (x * TILE_SIZE) as f32 * LAND_SCALING;
            let player_y = (y * TILE_SIZE) as f32 * LAND_SCALING;
            //spawning entities
            for (_camera, mut camera_transform) in camera_query.iter_mut() {
                let camera_x =
                    player_x - (player_x % window.width as f32) + window.width as f32 / 2. + 0.5;
                let camera_y =
                    player_y - (player_y % window.height as f32) + window.height as f32 / 2. + 0.5;
                camera_transform.translation.set_x(camera_x);
                camera_transform.translation.set_y(camera_y);
                *transition = (1., Vec3::new(camera_x, camera_y, 0.));
            }
            commands
                //player
                .spawn(SpriteSheetComponents {
                    texture_atlas: handles.player.clone(),
                    transform: Transform {
                        translation: Vec3::new(player_x, player_y, BOAT_LAYER),
                        scale: 2. * Vec3::one(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(save.player.clone())
                //flag
                .spawn((LandFlag,));
            for (mob, transform) in island.mobs.drain(..) {
                commands //mob
                    .spawn(SpriteComponents {
                        material: mob.material.clone(),
                        transform,
                        ..Default::default()
                    })
                    .with(mob);
            }
        }
    }
}

fn read_mob_config() -> Vec<MobConfig> {
    let mob_config_string =
        std::fs::read_to_string("config/mobs.ron").expect("mob config file not found");
    ron::from_str(&mob_config_string).expect("syntax error on mobs config file")
}
