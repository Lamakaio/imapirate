use bevy::{prelude::*, render::camera::Camera};
use std::f32::consts::PI;

use crate::{
    loading::GameState,
    sea::{player::PlayerPositionUpdate, TILE_SIZE},
};

use super::{loader::LandHandles, LAND_SCALING};

const PLAYER_DOWN: u32 = 0;
const PLAYER_UP: u32 = 1;
const PLAYER_RIGHT: u32 = 2;
const PLAYER_LEFT: u32 = 3;
pub struct LandPlayerPlugin;
impl Plugin for LandPlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Time>()
            .on_state_update(GameState::STAGE, GameState::Land, player_movement.system())
            .on_state_enter(GameState::STAGE, GameState::Land, load_system.system())
            .on_state_exit(GameState::STAGE, GameState::Land, unload_system.system())
            .on_state_update(
                GameState::STAGE,
                GameState::Land,
                keyboard_input_system.system(),
            )
            .on_state_update(GameState::STAGE, GameState::Land, camera_system.system())
            .add_event::<PlayerMovedEvent>()
            .insert_resource(CameraTransition {
                t: 0.,
                destination: Vec3::default(),
            });
        // .add_system(player_orientation.system())
    }
}

#[derive(Clone)]
pub struct Player {
    rotation: f32,
    speed: f32,
}
impl Default for Player {
    fn default() -> Player {
        Player {
            speed: 0.,
            rotation: 0.,
        }
    }
}

pub struct PlayerMovedEvent;

fn load_system(
    commands: &mut Commands,
    handles: Res<LandHandles>,
    sea_player_pos: Res<PlayerPositionUpdate>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut transition: ResMut<CameraTransition>,
) {
    let (x, y) = sea_player_pos.contact.unwrap();
    let player_x = x * LAND_SCALING;
    let player_y = y * LAND_SCALING;
    //spawning entities
    for mut camera_transform in camera_query.iter_mut() {
        let camera_x = player_x;
        let camera_y = player_y;
        camera_transform.translation.x = camera_x;
        camera_transform.translation.y = camera_y;
        transition.t = 1.;
        transition.destination = Vec3::new(0., 0., 0.);
    }
    commands
        //player
        .spawn(SpriteSheetBundle {
            texture_atlas: handles.player.clone(),
            transform: Transform {
                translation: Vec3::new(player_x, player_y, 99.),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Player::default());
}

fn unload_system(commands: &mut Commands, query: Query<Entity, With<Player>>) {
    for entity in query.iter() {
        commands.despawn_recursive(entity);
    }
}
fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut TextureAtlasSprite)>,
) {
    for (mut player, mut sprite) in &mut player_query.iter_mut() {
        const NO_ID: u32 = u32::MAX;
        let base_speed = 300.;
        let (rotation, speed, sprite_id) = match (
            keyboard_input.pressed(KeyCode::Left),
            keyboard_input.pressed(KeyCode::Down),
            keyboard_input.pressed(KeyCode::Right),
            keyboard_input.pressed(KeyCode::Up),
        ) {
            (true, true, false, false) => (-3. * PI / 4., base_speed, PLAYER_DOWN),
            (false, true, true, false) => (-PI / 4., base_speed, PLAYER_DOWN),
            (false, false, true, true) => (PI / 4., base_speed, PLAYER_UP),
            (true, false, false, true) => (3. * PI / 4., base_speed, PLAYER_UP),
            (true, false, false, false) => (PI, base_speed, PLAYER_LEFT),
            (false, true, false, false) => (-PI / 2., base_speed, PLAYER_DOWN),
            (false, false, true, false) => (0., base_speed, PLAYER_RIGHT),
            (false, false, false, true) => (PI / 2., base_speed, PLAYER_UP),
            _ => (0., 0., NO_ID),
        };
        player.rotation = rotation;
        player.speed = speed;
        if sprite_id != NO_ID {
            sprite.index = sprite_id
        }
    }
}

const UPDATES_PER_TILE: f32 = 10.;
fn player_movement(
    mut last_pos: Local<Vec3>,
    time: Res<Time>,
    mut events: ResMut<Events<PlayerMovedEvent>>,
    mut player_query: Query<(&Player, &mut Transform)>,
) {
    const TILE: f32 = TILE_SIZE as f32 * LAND_SCALING;
    for (player, mut player_transform) in player_query.iter_mut() {
        let rounded_angle = (0.5 + 8. * player.rotation / (2. * PI)).floor() / 8.0 * (2. * PI);
        let (s, c) = f32::sin_cos(rounded_angle);
        player_transform.translation.x += c * player.speed * time.delta_seconds();
        player_transform.translation.y += s * player.speed * time.delta_seconds();
        let current_tile = (player_transform.translation / TILE * UPDATES_PER_TILE).floor();
        if current_tile.x as i32 != last_pos.x as i32 || current_tile.y as i32 != last_pos.y as i32
        {
            *last_pos = current_tile;
            events.send(PlayerMovedEvent);
        }
    }
}

pub struct CameraTransition {
    t: f32,
    destination: Vec3,
}

fn camera_system(
    time: Res<Time>,
    window: Res<WindowDescriptor>,
    mut transition: ResMut<CameraTransition>,
    mut camera_position: Local<Vec3>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, With<Player>>,
) {
    const CAMERA_SPEED: f32 = 2.;
    for player_transform in player_query.iter() {
        for mut camera_transform in camera_query.iter_mut() {
            if transition.t < 1. {
                transition.t += CAMERA_SPEED * time.delta_seconds();
                if transition.t >= 1. {
                    *camera_position = transition.destination;
                    let new_pos = transition.destination;
                    camera_transform.translation.x = new_pos.x;
                    camera_transform.translation.y = new_pos.y;
                } else {
                    let new_pos =
                        Vec3::lerp(*camera_position, transition.destination, transition.t);
                    camera_transform.translation.x = new_pos.x;
                    camera_transform.translation.y = new_pos.y;
                }
            } else {
                let window = Vec2::new(window.width, window.height) / 2.;
                let assumed_camera_x = player_transform.translation.x
                    - (player_transform.translation.x % window.x as f32)
                    + window.x / 2.;
                let assumed_camera_y = player_transform.translation.y
                    - (player_transform.translation.y % window.y as f32)
                    + window.y / 2.;
                if assumed_camera_x as i32 != camera_transform.translation.x as i32
                    || assumed_camera_y as i32 != camera_transform.translation.y as i32
                {
                    transition.t = 0.;
                    transition.destination = Vec3::new(assumed_camera_x, assumed_camera_y, 0.);
                }
            }
        }
    }
}

// fn player_orientation(player: &Player, mut sprite: Mut<TextureAtlasSprite>) {
//     sprite.index = (((0.5 - 8. * player.rotation / (2. * std::f32::consts::PI)).floor() as i32
//         + 21)
//         % 8) as u32;
// }
