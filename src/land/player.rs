use bevy::{prelude::*, render::camera::Camera};
use std::f32::consts::PI;

use crate::{
    loading::GameState,
    sea::{loader::SeaHandles, player::PlayerPositionUpdate, ISLAND_SCALING, TILE_SIZE},
};

use super::{
    loader::{LandHandles, UnloadLandFlag},
    LAND_SCALING,
};

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
            .on_state_update(GameState::STAGE, GameState::Land, shoot.system())
            .on_state_update(GameState::STAGE, GameState::Land, bullets.system())
            .on_state_update(GameState::STAGE, GameState::Land, sprite_index.system())
            .add_event::<PlayerMovedEvent>()
            .insert_resource(CameraTransition {
                t: 0.,
                destination: Vec3::default(),
            });
        // .add_system(player_orientation.system())
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    rotation: f32,
    speed: f32,
    action: Option<Action>,
    sprite_id: u32,
}
impl Default for Player {
    fn default() -> Player {
        Player {
            speed: 0.,
            rotation: 0.,
            action: None,
            sprite_id: 0,
        }
    }
}
impl Player {
    fn fire(&mut self) -> bool {
        if self.action.is_none() {
            self.action = Some(Action {
                kind: ActionKind::Firing,
                timer: Timer::from_seconds(0.5, false),
            });
            true
        } else {
            false
        }
    }
    fn slash(&mut self) -> bool {
        if self.action.is_none() {
            self.action = Some(Action {
                kind: ActionKind::Slashing,
                timer: Timer::from_seconds(0.5, false),
            });
            true
        } else {
            false
        }
    }
    fn tick(&mut self, delta_seconds: f32) {
        if let Some(action) = &mut self.action {
            action.timer.tick(delta_seconds);
            if action.timer.finished() {
                self.action = None;
            }
        }
    }
    fn is_firing(&self) -> bool {
        matches!(
            self.action,
            Some(Action {
                kind: ActionKind::Firing,
                timer: _,
            }),
        )
    }
    fn is_slashing(&self) -> bool {
        matches!(
            self.action,
            Some(Action {
                kind: ActionKind::Slashing,
                timer: _,
            }),
        )
    }
}
#[derive(Clone, Debug)]
pub struct Action {
    kind: ActionKind,
    timer: Timer,
}

#[derive(Clone, Debug)]
pub enum ActionKind {
    Firing,
    Slashing,
}
pub struct PlayerMovedEvent;
pub struct GunSheet;

pub struct SwordSheet;
pub struct Bullet {
    direction: Vec2,
    speed: f32,
    timer: Timer,
}
impl Bullet {
    pub fn from_id(sprite_id: u32) -> Self {
        let direction = match sprite_id {
            PLAYER_DOWN => Vec2::new(0., -1.),
            PLAYER_UP => Vec2::new(0., 1.),
            PLAYER_LEFT => Vec2::new(-1., 0.),
            PLAYER_RIGHT => Vec2::new(1., 0.),
            _ => Vec2::new(0., 0.),
        };
        Self {
            direction,
            speed: 20.,
            timer: Timer::from_seconds(1., false),
        }
    }
}
fn load_system(
    commands: &mut Commands,
    handles: Res<LandHandles>,
    sea_handles: Res<SeaHandles>,
    sea_player_pos: Res<PlayerPositionUpdate>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut transition: ResMut<CameraTransition>,
) {
    let (x, y, normal) = sea_player_pos.contact.unwrap();
    let player_x = (x - normal.x * 10.) * LAND_SCALING;
    let player_y = (y - normal.y * 10.) * LAND_SCALING;
    let boat_x = (x + normal.x * 10.) * LAND_SCALING;
    let boat_y = (y + normal.y * 10.) * LAND_SCALING;
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
                translation: Vec3::new(player_x, player_y, 100.),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Player::default())
        .with_children(|child_builder| {
            child_builder
                .spawn(SpriteSheetBundle {
                    texture_atlas: handles.player_sword.clone(),
                    transform: Transform {
                        translation: Vec3::new(0., 0., 1.),
                        ..Default::default()
                    },
                    visible: Visible {
                        is_visible: false,
                        is_transparent: true,
                    },
                    ..Default::default()
                })
                .with(UnloadLandFlag)
                .with(SwordSheet)
                .spawn(SpriteSheetBundle {
                    texture_atlas: handles.player_gun.clone(),
                    transform: Transform {
                        translation: Vec3::new(0., 0., 1.),
                        ..Default::default()
                    },
                    visible: Visible {
                        is_visible: false,
                        is_transparent: true,
                    },
                    ..Default::default()
                })
                .with(GunSheet)
                .with(UnloadLandFlag);
        })
        .spawn(SpriteSheetBundle {
            texture_atlas: sea_handles.boat.clone(),
            transform: Transform {
                translation: Vec3::new(boat_x, boat_y, 99.),
                scale: Vec3::new(
                    LAND_SCALING / ISLAND_SCALING,
                    LAND_SCALING / ISLAND_SCALING,
                    1.,
                ),
                ..Default::default()
            },
            sprite: TextureAtlasSprite {
                index: sea_player_pos.sprite_id,
                ..Default::default()
            },
            visible: Visible {
                is_visible: true,
                is_transparent: true,
            },
            ..Default::default()
        })
        .with(UnloadLandFlag);
}

fn unload_system(commands: &mut Commands, query: Query<Entity, With<Player>>) {
    for entity in query.iter() {
        commands.despawn_recursive(entity);
    }
}
fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Player>,
) {
    for mut player in player_query.iter_mut() {
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
        player.speed = speed;
        if sprite_id != NO_ID {
            player.rotation = rotation;
            player.sprite_id = sprite_id;
        }
        if keyboard_input.just_pressed(KeyCode::X) {
            player.fire();
        }
        if keyboard_input.just_pressed(KeyCode::C) {
            player.slash();
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

fn sprite_index(
    time: Res<Time>,
    mut player_query: Query<(&mut Player, &mut TextureAtlasSprite)>,
    mut sword_query: Query<(&mut TextureAtlasSprite, &mut Visible), With<SwordSheet>>,
    mut gun_query: Query<(&mut TextureAtlasSprite, &mut Visible), With<GunSheet>>,
) {
    for (mut player, mut sprite) in player_query.iter_mut() {
        player.tick(time.delta_seconds());
        sprite.index = player.sprite_id;
        for (mut sprite, mut visible) in sword_query.iter_mut() {
            if player.is_slashing() {
                visible.is_visible = true;
                sprite.index = player.sprite_id;
            } else {
                visible.is_visible = false;
            }
        }
        for (mut sprite, mut visible) in gun_query.iter_mut() {
            if player.is_firing() {
                visible.is_visible = true;
                sprite.index = player.sprite_id;
            } else {
                visible.is_visible = false;
            }
        }
    }
}

fn shoot(
    commands: &mut Commands,
    player_query: Query<(&Player, &Transform)>,
    mut firing: Local<bool>,
    handles: Res<LandHandles>,
) {
    for (player, transform) in player_query.iter() {
        if player.is_firing() && !*firing {
            let bullet = Bullet::from_id(player.sprite_id);
            let mut transform = *transform;
            transform.rotation = Quat::from_rotation_z(player.rotation + PI);
            commands
                .spawn(SpriteBundle {
                    material: handles.bullet_material.clone(),
                    transform,
                    visible: Visible {
                        is_visible: true,
                        is_transparent: true,
                    },
                    ..Default::default()
                })
                .with(bullet);
        }
        *firing = player.is_firing();
    }
}

fn bullets(
    commands: &mut Commands,
    mut bullet_query: Query<(Entity, &mut Bullet, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut bullet, mut transform) in bullet_query.iter_mut() {
        bullet.timer.tick(time.delta_seconds());
        if bullet.timer.finished() {
            commands.despawn(entity);
        } else {
            transform.translation += bullet.direction.extend(0.) * bullet.speed
        }
    }
}
