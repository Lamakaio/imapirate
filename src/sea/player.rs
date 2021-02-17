use bevy::{prelude::*, render::camera::Camera};
use parry2d::{math::Vector, na::Unit};

use std::f32::consts::PI;

use crate::loading::GameState;

use super::{loader::SeaHandles, ISLAND_SCALING, TILE_SIZE};
pub struct SeaPlayerPlugin;
impl Plugin for SeaPlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.on_state_enter(GameState::STAGE, GameState::Sea, load_system.system())
            .on_state_exit(GameState::STAGE, GameState::Sea, unload_system.system())
            .init_resource::<PlayerPositionUpdate>()
            .init_resource::<PlayerSave>()
            .on_state_update(GameState::STAGE, GameState::Sea, player_movement.system())
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                keyboard_input_system.system(),
            )
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                player_orientation.system(),
            );
    }
}

struct PlayerSave {
    translation: Vec3,
    player: Player,
}
impl Default for PlayerSave {
    fn default() -> Self {
        Self {
            translation: Vec3::new(0., 0., 100.),
            player: Player::default(),
        }
    }
}
#[derive(Clone)]
pub struct Player {
    rotation: f32,
    rotation_speed: f32,
    rotation_acceleration: f32,
    speed: f32,
    acceleration: f32,
    friction: f32,
    rotation_friction: f32,
}
impl Default for Player {
    fn default() -> Player {
        Player {
            speed: 0.,
            acceleration: 0.,
            rotation: 0.,
            rotation_speed: 0.,
            rotation_acceleration: 0.,
            friction: 0.2,
            rotation_friction: 10.,
        }
    }
}

pub enum CollisionType {
    None,
    Friction,
    Rigid,
}

pub struct PlayerPositionUpdate {
    pub x: i32,
    pub y: i32,
    pub translation: Vec3,
    pub changed_tile: bool,
    pub collision_status: CollisionType,
    pub island_id: Option<u32>,
    pub contact: Option<(f32, f32, Unit<Vector<f32>>)>,
    pub sprite_id: u32,
}
impl PlayerPositionUpdate {
    fn update(&mut self, t: &Vec3) {
        self.x = (t.x / TILE_SIZE as f32 / ISLAND_SCALING) as i32;
        self.y = (t.y / TILE_SIZE as f32 / ISLAND_SCALING) as i32;
        self.translation = *t;
        //println!("{}, {}", self.x, self.y)
    }
}
impl Default for PlayerPositionUpdate {
    fn default() -> Self {
        PlayerPositionUpdate {
            x: 0,
            y: 0,
            sprite_id: 0,
            island_id: None,
            translation: Vec3::default(),
            changed_tile: true,
            collision_status: CollisionType::None,
            contact: None,
        }
    }
}

fn load_system(commands: &mut Commands, handles: Res<SeaHandles>, save: Res<PlayerSave>) {
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: handles.boat.clone(),
            transform: Transform::from_translation(save.translation),
            ..Default::default()
        })
        .with(save.player.clone());
}

fn unload_system(
    commands: &mut Commands,
    mut save: ResMut<PlayerSave>,
    player_query: Query<(Entity, &Transform, &Player)>,
) {
    for (entity, transform, player) in player_query.iter() {
        save.translation = transform.translation;
        save.player = player.clone();
        commands.despawn_recursive(entity);
    }
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Player>,
) {
    for mut player in player_query.iter_mut() {
        if keyboard_input.just_released(KeyCode::Up) || keyboard_input.just_released(KeyCode::Down)
        {
            player.acceleration = 0.;
        }

        if keyboard_input.just_released(KeyCode::Right)
            || keyboard_input.just_released(KeyCode::Left)
        {
            player.rotation_acceleration = 0.;
        }

        if keyboard_input.just_pressed(KeyCode::Up) {
            player.acceleration = 100.;
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            player.acceleration = -100.;
        }

        if keyboard_input.just_pressed(KeyCode::Right) {
            player.rotation_acceleration = -20.;
        } else if keyboard_input.just_pressed(KeyCode::Left) {
            player.rotation_acceleration = 20.;
        }
    }
}

fn player_movement(
    time: Res<Time>,
    mut stuck_forward: Local<Option<bool>>,
    mut pos_update: ResMut<PlayerPositionUpdate>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut camera_query: Query<(&Camera, &mut Transform)>,
) {
    for (mut player, mut player_transform) in player_query.iter_mut() {
        player.rotation_speed += (player.rotation_acceleration
            - player.rotation_speed * player.rotation_friction)
            * time.delta_seconds();

        let rounded_angle = (0.5 + 8. * player.rotation / (2. * PI)).floor() / 8.0 * (2. * PI);
        let (s, c) = f32::sin_cos(rounded_angle);
        match pos_update.collision_status {
            CollisionType::None => {
                *stuck_forward = None;
                player.rotation =
                    (player.rotation + player.rotation_speed * time.delta_seconds()) % (2. * PI);
                player.speed += (player.acceleration
                    - (1. + player.rotation_speed.abs()) * player.speed * player.friction)
                    * time.delta_seconds();
                player_transform.translation.x += c * player.speed * time.delta_seconds();
                player_transform.translation.y += s * player.speed * time.delta_seconds();
            }
            CollisionType::Friction => {
                *stuck_forward = None;
                player.speed += (player.acceleration - player.speed * player.friction * 20.)
                    * time.delta_seconds();
                player_transform.translation.x += c * player.speed * time.delta_seconds();
                player_transform.translation.y += s * player.speed * time.delta_seconds();
            }
            CollisionType::Rigid => {
                player.speed += (player.acceleration - player.speed * player.friction * 20.)
                    * time.delta_seconds();
                if stuck_forward.is_none() {
                    *stuck_forward = Some(player.speed > 0.)
                }
                if (stuck_forward.unwrap() && player.speed < 0.)
                    || (!stuck_forward.unwrap() && player.speed > 0.)
                {
                    player_transform.translation.x += c * player.speed * time.delta_seconds();
                    player_transform.translation.y += s * player.speed * time.delta_seconds();
                } else {
                    player.speed = 0.;
                }
            }
        }
        pos_update.update(&player_transform.translation);
        for (_camera, mut camera_transform) in camera_query.iter_mut() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}

fn player_orientation(
    mut player_query: Query<(&Player, &mut TextureAtlasSprite)>,
    mut player_pos_update: ResMut<PlayerPositionUpdate>,
) {
    for (player, mut sprite) in player_query.iter_mut() {
        sprite.index = (((0.5 - 8. * player.rotation / (2. * std::f32::consts::PI)).floor() as i32
            + 21)
            % 8) as u32;
        player_pos_update.sprite_id = sprite.index;
    }
}
