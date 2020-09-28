use crate::land::map::Island;

use super::tilemap::{Chunk, CollisionType, SCALING};
use super::worldgen::{CHUNK_SIZE, TILE_SIZE};
use bevy::ecs::bevy_utils::HashMap;
use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
};
use std::f32::consts::PI;

pub const BOAT_LAYER: f32 = 100.;
pub const ZOOM: f32 = 1.;
pub struct SeaPlayerPlugin;
impl Plugin for SeaPlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .init_resource::<Time>()
            .add_resource(PlayerPositionUpdate::default())
            .add_system(player_movement.system())
            .add_system(keyboard_input_system.system())
            .add_system(player_orientation.system())
            .add_system(collision_system.system())
            .register_component::<Player>();
    }
}

#[derive(Properties, Clone)]
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

pub struct PlayerPositionUpdate {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub changed_chunk: bool,
    pub tile_x: i32,
    pub tile_y: i32,
    pub changed_tile: bool,
    force_update: bool,
    collision_status: CollisionType,
}
impl PlayerPositionUpdate {
    pub fn force_update(&mut self) {
        self.force_update = true;
    }
    pub fn get_x(&self) -> f32 {
        const TILE: i32 = TILE_SIZE * SCALING;
        (TILE * self.tile_x + TILE * CHUNK_SIZE * self.chunk_x) as f32
    }
    pub fn get_y(&self) -> f32 {
        const TILE: i32 = TILE_SIZE * SCALING;
        (TILE * self.tile_y + TILE * CHUNK_SIZE * self.chunk_y) as f32
    }
    fn update(&mut self, t: &Vec4) {
        self.changed_chunk = false;
        self.changed_tile = false;
        const TILE: i32 = TILE_SIZE * SCALING;
        let assumed_x =
            TILE * self.tile_x + TILE * CHUNK_SIZE * self.chunk_x - TILE * CHUNK_SIZE / 2;
        let assumed_y =
            TILE * self.tile_y + TILE * CHUNK_SIZE * self.chunk_y - TILE * CHUNK_SIZE / 2;
        if t.x() > (assumed_x + TILE) as f32 {
            self.tile_x += 1;
            self.changed_tile = true;
            if self.tile_x >= CHUNK_SIZE as i32 {
                self.tile_x = 0;
                self.chunk_x += 1;
                self.changed_chunk = true;
            }
        } else if t.x() < assumed_x as f32 {
            self.tile_x -= 1;
            self.changed_tile = true;
            if self.tile_x < 0 {
                self.tile_x = CHUNK_SIZE - 1;
                self.chunk_x -= 1;
                self.changed_chunk = true;
            }
        }
        if t.y() > (assumed_y + TILE) as f32 {
            self.tile_y += 1;
            self.changed_tile = true;
            if self.tile_y >= CHUNK_SIZE as i32 {
                self.tile_y = 0;
                self.chunk_y += 1;
                self.changed_chunk = true;
            }
        } else if t.y() < assumed_y as f32 {
            self.tile_y -= 1;
            self.changed_tile = true;
            if self.tile_y < 0 {
                self.tile_y = CHUNK_SIZE - 1;
                self.chunk_y -= 1;
                self.changed_chunk = true;
            }
        }
        if self.force_update {
            self.force_update = false;
            self.changed_chunk = true;
            self.changed_tile = true;
        }
    }
}
impl Default for PlayerPositionUpdate {
    fn default() -> Self {
        PlayerPositionUpdate {
            chunk_x: 0,
            chunk_y: 0,
            tile_x: CHUNK_SIZE / 2,
            tile_y: CHUNK_SIZE / 2,
            changed_chunk: true,
            changed_tile: true,
            collision_status: CollisionType::None,
            force_update: false,
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    //loading textures
    let texture_handle = asset_server
        .load_sync(&mut textures, "assets/sprites/sea/ship_sheet.png")
        .unwrap();
    let texture = textures.get(&texture_handle).unwrap();
    let texture_atlas = TextureAtlas::from_grid(texture_handle, texture.size, 8, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    //spawning entities
    let far = 1000.;
    commands
        //camera
        .spawn(Camera2dComponents {
            orthographic_projection: OrthographicProjection {
                far,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., far - 0.1)).with_scale(ZOOM),
            ..Default::default()
        })
        //player
        .spawn(SpriteSheetComponents {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_translation(Vec3::new(0., 0., BOAT_LAYER)).with_scale(2.),
            ..Default::default()
        })
        .with(Player::default());
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Player>,
) {
    for mut player in &mut player_query.iter() {
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
            player.acceleration = 900.;
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            player.acceleration = -120.;
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
    islands: Res<HashMap<u64, Island>>,
    mut stuck_forward: Local<Option<bool>>,
    mut pos_update: ResMut<PlayerPositionUpdate>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut camera_query: Query<(&Camera, &mut Transform)>,
) {
    for (mut player, mut player_translation) in &mut player_query.iter() {
        let player_translation = player_translation.translation_mut();
        player.rotation_speed += (player.rotation_acceleration
            - player.rotation_speed * player.rotation_friction)
            * time.delta_seconds;
        player.speed += (player.acceleration - player.speed * player.friction) * time.delta_seconds;
        let rounded_angle = (0.5 + 8. * player.rotation / (2. * PI)).floor() / 8.0 * (2. * PI);
        let (s, c) = f32::sin_cos(rounded_angle);
        match pos_update.collision_status {
            CollisionType::None => {
                *stuck_forward = None;
                player.rotation =
                    (player.rotation + player.rotation_speed * time.delta_seconds) % (2. * PI);
                *player_translation.x_mut() += c * player.speed * time.delta_seconds;
                *player_translation.y_mut() += s * player.speed * time.delta_seconds;
            }
            CollisionType::Friction(x) => {
                println!("{:?} {:?}", x, islands.get(&x.unwrap_or(0)));
                *stuck_forward = None;
                player.rotation =
                    (player.rotation + player.rotation_speed * time.delta_seconds) % (2. * PI);
                *player_translation.x_mut() += c * player.speed * time.delta_seconds / 3.;
                *player_translation.y_mut() += s * player.speed * time.delta_seconds / 3.;
            }
            CollisionType::Rigid(_) => {
                if stuck_forward.is_none() {
                    *stuck_forward = Some(player.speed > 0.)
                }
                if (stuck_forward.unwrap() && player.speed < 0.)
                    || (!stuck_forward.unwrap() && player.speed > 0.)
                {
                    *player_translation.x_mut() += c * player.speed * time.delta_seconds / 3.;
                    *player_translation.y_mut() += s * player.speed * time.delta_seconds / 3.;
                } else {
                    player.speed = 0.;
                }
            }
        }
        pos_update.update(&player_translation);
        for (_camera, mut camera_translation) in &mut camera_query.iter() {
            let camera_translation = camera_translation.translation_mut();
            camera_translation.set_x(player_translation.x());
            camera_translation.set_y(player_translation.y());
        }
    }
}

fn player_orientation(player: &Player, mut sprite: Mut<TextureAtlasSprite>) {
    sprite.index = (((0.5 - 8. * player.rotation / (2. * std::f32::consts::PI)).floor() as i32
        + 21)
        % 8) as u32;
}

fn collision_system(
    chunks: Res<HashMap<(i32, i32), Chunk>>,
    mut pos_update: ResMut<PlayerPositionUpdate>,
) {
    if pos_update.changed_tile {
        let chunk = chunks
            .get(&(pos_update.chunk_x, pos_update.chunk_y))
            .unwrap();
        pos_update.collision_status = *chunk.collision_map[0]
            .get(pos_update.tile_y as usize)
            .unwrap_or(&Vec::new())
            .get(pos_update.tile_x as usize)
            .unwrap_or(&CollisionType::None);
    }
}
