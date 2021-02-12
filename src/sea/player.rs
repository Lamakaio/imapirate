use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::Camera};
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::{
        dynamics::{RigidBodyBuilder, RigidBodySet},
        geometry::{ColliderBuilder, InteractionGroups},
        math::Vector,
    },
};

use crate::loading::GameState;

use super::loader::{SeaHandles, BOAT_LAYER};
pub struct SeaPlayerPlugin;
impl Plugin for SeaPlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Time>()
            .init_resource::<SeaPlayerSaveState>()
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                player_sprite_system.system(),
            )
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                keyboard_input_system.system(),
            )
            .on_state_enter(GameState::STAGE, GameState::Sea, load_system.system());
    }
}

struct SeaPlayerSaveState {
    player_transform: Transform,
}

impl Default for SeaPlayerSaveState {
    fn default() -> Self {
        SeaPlayerSaveState {
            player_transform: Transform {
                translation: Vec3::new(0., 0., BOAT_LAYER),
                scale: 0.5 * Vec3::one(),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone)]
pub struct PlayerFlag;

#[derive(Bundle)]
pub struct PlayerBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    flag: PlayerFlag,
    rigidbody: RigidBodyBuilder,
    collider: ColliderBuilder,
}
impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            global_transform: Default::default(),
            flag: PlayerFlag,
            rigidbody: RigidBodyBuilder::new_dynamic()
                .linear_damping(1.5)
                .angular_damping(10.)
                .mass(1.)
                .gravity_scale(0.),
            collider: ColliderBuilder::ball(10.)
                .collision_groups(InteractionGroups::new(
                    0b0000_0000_0000_0010,
                    0b0000_0000_0000_0001,
                ))
                .friction(0.)
                .density(0.1),
        }
    }
}

fn load_system(commands: &mut Commands, save: Res<SeaPlayerSaveState>, handles: Res<SeaHandles>) {
    //spawning entities
    commands
        //player
        .spawn(SpriteSheetBundle {
            texture_atlas: handles.boat.clone(),
            transform: save.player_transform,
            ..Default::default()
        })
        .with(PlayerFlag)
        .spawn(PlayerBundle {
            transform: save.player_transform,
            ..Default::default()
        });
}

fn keyboard_input_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&RigidBodyHandleComponent, &Transform), With<PlayerFlag>>,
    mut rigid_body_set: ResMut<RigidBodySet>,
) {
    for (rbdhc, transform) in player_query.iter_mut() {
        let rbd = rigid_body_set.get_mut(rbdhc.handle()).unwrap();
        let mut move_velocity = 0.;
        let move_step = 10000.;
        let mut rotation_velocity = 0.;
        let rotation_step = 20000.;
        if keyboard_input.pressed(KeyCode::Up) {
            move_velocity += move_step;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            move_velocity -= move_step;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            rotation_velocity += rotation_step;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            rotation_velocity -= rotation_step;
        }
        let angle = transform.rotation.to_axis_angle();
        let angle = PI + angle.1 * angle.0.z.signum();
        let sprite_index = (((angle % (2. * PI)) / (2. * PI)) * 8. + 8.5).floor() % 8.;
        let angle = (sprite_index as f32 - 4.) / 4. * PI;
        let (sin, cos) = angle.sin_cos();
        let linvel = rbd.linvel();
        let norm = linvel.norm();
        let linvel_angle = linvel.angle(&Vector::new(sin, cos)).abs();
        let sign = if linvel_angle < PI / 2. { 1. } else { -1. };
        rbd.set_linvel(Vector::new(sin, cos) * norm * sign, true);

        rbd.apply_impulse(
            Vector::new(sin, cos) * move_velocity * time.delta_seconds(),
            true,
        );
        rbd.apply_torque_impulse(rotation_velocity * time.delta_seconds(), true);
    }
}

// fn player_movement(
//     time: Res<Time>,
//     mut stuck_forward: Local<Option<bool>>,
//     mut pos_update: ResMut<PlayerPositionUpdate>,
//     mut player_query: Query<(&mut Player, &mut Transform)>,
//     mut camera_query: Query<(&Camera, &mut Transform)>,
// ) {
//     for (mut player, mut player_transform) in player_query.iter_mut() {
//         player.rotation_speed += (player.rotation_acceleration
//             - player.rotation_speed * player.rotation_friction)
//             * time.delta_seconds();
//         player.speed +=
//             (player.acceleration - player.speed * player.friction) * time.delta_seconds();
//         let rounded_angle = (0.5 + 8. * player.rotation / (2. * PI)).floor() / 8.0 * (2. * PI);
//         let (s, c) = f32::sin_cos(rounded_angle);
//         // match pos_update.collision_status {
//         //     CollisionType::None => {
//         *stuck_forward = None;
//         player.rotation =
//             (player.rotation + player.rotation_speed * time.delta_seconds()) % (2. * PI);
//         player_transform.translation.x += c * player.speed * time.delta_seconds();
//         player_transform.translation.y += s * player.speed * time.delta_seconds();
//         //     }
//         //     CollisionType::Friction(_) => {
//         //         *stuck_forward = None;
//         //         player.rotation =
//         //             (player.rotation + player.rotation_speed * time.delta_seconds()) % (2. * PI);
//         //         player_transform.translation.x += c * player.speed * time.delta_seconds() / 3.;
//         //         player_transform.translation.y += s * player.speed * time.delta_seconds() / 3.;
//         //     }
//         //     CollisionType::Rigid(_) => {
//         //         if stuck_forward.is_none() {
//         //             *stuck_forward = Some(player.speed > 0.)
//         //         }
//         //         if (stuck_forward.unwrap() && player.speed < 0.)
//         //             || (!stuck_forward.unwrap() && player.speed > 0.)
//         //         {
//         //             player_transform.translation.x += c * player.speed * time.delta_seconds() / 3.;
//         //             player_transform.translation.y += s * player.speed * time.delta_seconds() / 3.;
//         //         } else {
//         //             player.speed = 0.;
//         //         }
//         //     }
//         // }
//         pos_update.update(&player_transform.translation);
//         for (_camera, mut camera_transform) in camera_query.iter_mut() {
//             camera_transform.translation.x = player_transform.translation.x;
//             camera_transform.translation.y = player_transform.translation.y;
//         }
//     }
// }
const SPRITE_SIZE: (f32, f32) = (64., 64.);
const SCALE: f32 = 0.5;
fn player_sprite_system(
    mut player_sprite_query: Query<(&mut TextureAtlasSprite, &mut Transform), With<PlayerFlag>>,
    player_query: Query<&Transform, (Without<TextureAtlasSprite>, With<PlayerFlag>)>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let player_transform = player_query.iter().next().copied().unwrap_or_default();
    let angle = player_transform.rotation.to_axis_angle();
    let angle = angle.1 * angle.0.z.signum();
    let translation = player_transform.translation;
    for (mut sprite, mut transform) in player_sprite_query.iter_mut() {
        sprite.index = ((((angle % (2. * PI)) / (2. * PI)) * 8. + 11.5).floor() % 8.) as u32;
        transform.translation = translation + Vec3::new(0., SPRITE_SIZE.1 * SCALE, 0.) / 2.;
    }
    for mut transform in camera_query.iter_mut() {
        transform.translation = translation;
    }
}
