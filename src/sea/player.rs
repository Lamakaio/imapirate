
use bevy::{
    prelude::*,
    render::camera::{OrthographicProjection, Camera}
};
use super::worldgen::{TILE_SIZE, CHUNK_SIZE, SCALING};
use std::f32::consts::PI;

const BOAT_LAYER : f32 = 100.;
const ZOOM : f32 = 1.;
pub struct SeaPlayerPlugin;
impl Plugin for SeaPlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(setup.system() )
        .init_resource::<Time>()
        .add_resource(PlayerPositionUpdate::default())
        .add_system(player_movement.system())
        .add_system(keyboard_input_system.system())
        .add_system(player_orientation.system())
        ;
    }
}

pub enum FrictionType {
    Sea, 
    Shore, 
    Land
}

pub struct Player { 
    rotation : f32,
    rotation_speed : f32,
    rotation_acceleration : f32,
    speed : f32,
    acceleration : f32,
    friction : f32,
    rotation_friction : f32
}
impl Player {
    fn new() -> Player{
        Player 
            {speed : 0.,
             acceleration : 0., 
             rotation : 0.,
             rotation_speed : 0., 
             rotation_acceleration : 0.,
             friction : 0.2, 
             rotation_friction : 10.}
    }

    pub fn set_friction(&mut self, friction : FrictionType) {
        match friction {
            FrictionType::Sea => {
                self.friction = 0.2
            }
            FrictionType::Shore => {
                self.friction = 10.
            }
            FrictionType::Land => {
                self.friction = 100.
            }
        }
    }
}


pub struct PlayerPositionUpdate {
    pub last_pos : Translation,
    pub chunk_x : i32, 
    pub chunk_y : i32,
    pub changed_chunk : bool,
    pub tile_x : i32, 
    pub tile_y : i32, 
    pub changed_tile : bool
}
impl PlayerPositionUpdate {
    fn update(&mut self, t : &Translation) {
        self.changed_chunk = false;
        self.changed_tile = false;
        const TILE : f32 = (TILE_SIZE * SCALING) as f32;
        let dx = t.x() - self.last_pos.x();
        let dy = t.y() - self.last_pos.y();
        if dx > TILE {
            self.tile_x += 1;
            *self.last_pos.x_mut() += TILE;
            self.changed_tile = true;
            if self.tile_x >= CHUNK_SIZE as i32 {
                self.tile_x = 0;
                self.chunk_x += 1;
                self.changed_chunk = true;
            }
        }
        else if dx < -TILE {
            self.tile_x -= 1;
            *self.last_pos.x_mut() -= TILE;
            self.changed_tile = true;
            if self.tile_x < 0 {
                self.tile_x = CHUNK_SIZE - 1;
                self.chunk_x -= 1;
                self.changed_chunk = true;
            }
        }
        if dy > TILE {
            self.tile_y += 1;
            *self.last_pos.y_mut() += TILE;
            self.changed_tile = true;
            if self.tile_y >= CHUNK_SIZE as i32 {
                self.tile_y = 0;
                self.chunk_y += 1;
                self.changed_chunk = true;
            }
        }
        else if dy < -TILE {
            self.tile_y -= 1;
            *self.last_pos.y_mut() -= TILE;
            self.changed_tile = true;
            if self.tile_y < 0 {
                self.tile_y = CHUNK_SIZE - 1;
                self.chunk_y -= 1;
                self.changed_chunk = true;
            }
        }
    }
}
impl Default for PlayerPositionUpdate {
    fn default() -> Self {
        PlayerPositionUpdate {
            last_pos : Translation::new(0., 0., 0.),
            chunk_x : 0, 
            chunk_y : 0, 
            tile_x : CHUNK_SIZE/2, 
            tile_y : CHUNK_SIZE/2, 
            changed_chunk : true, 
            changed_tile : true
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>, 
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures : ResMut<Assets<Texture>>
) {
    //loading textures
    let texture_handle = asset_server
        .load_sync(
            &mut textures,
            "assets/sprites/sea/ship_sheet.png",
        )
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
            far : far/ZOOM,
            ..Default::default()
        },
        scale : Scale(ZOOM),
        translation : Translation::new(0., 0., far - 0.1),
        ..Default::default()
        })
    //player
        .spawn(
        SpriteSheetComponents {
            texture_atlas : texture_atlas_handle,
            translation : Translation::new(0., 0., BOAT_LAYER),
            scale : Scale(2.),
            ..Default::default()
        })
        .with(Player::new());
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query : Query<&mut Player>
) {
    for mut player in &mut player_query.iter() {
        if keyboard_input.just_released(KeyCode::Up) || keyboard_input.just_released(KeyCode::Down){
            player.acceleration = 0.;
        }
    
        if keyboard_input.just_released(KeyCode::Right) || keyboard_input.just_released(KeyCode::Left){
            player.rotation_acceleration = 0.;
        }

        if keyboard_input.just_pressed(KeyCode::Up) {
            player.acceleration = 120.;
        }
        else if keyboard_input.just_pressed(KeyCode::Down) {
            player.acceleration = -120.;
        }
    
        if keyboard_input.just_pressed(KeyCode::Right) {
            player.rotation_acceleration = -20.;
        }
        else if keyboard_input.just_pressed(KeyCode::Left) {
            player.rotation_acceleration = 20.;
        }
    } 
}

fn player_movement(
    time : Res<Time>,
    mut pos_update : ResMut<PlayerPositionUpdate>,
    mut player_query : Query<(&mut Player, &mut Translation)>,
    mut camera_query : Query<(&Camera, &mut Translation)>, 
) {
    for (mut player, mut player_translation) in &mut player_query.iter() {
        player.rotation_speed += (player.rotation_acceleration - player.rotation_speed * player.rotation_friction) * time.delta_seconds;
        player.speed += (player.acceleration - player.speed * player.friction) * time.delta_seconds;
        player.rotation = (player.rotation + player.rotation_speed * time.delta_seconds) % (2. * PI);
        let rounded_angle = (0.5 + 8. * player.rotation / (2. * PI)).floor() / 8.0 * (2. * PI);
        let (s, c) = f32::sin_cos(rounded_angle);
        *player_translation.x_mut() += c * player.speed * time.delta_seconds;
        *player_translation.y_mut() += s * player.speed * time.delta_seconds;

        pos_update.update(&player_translation);
        for (_camera, mut camera_translation) in &mut camera_query.iter() {
            camera_translation.set_x(player_translation.x());
            camera_translation.set_y(player_translation.y());
        }
    } 
}

fn player_orientation(
    player  : &Player, 
    mut sprite : Mut<TextureAtlasSprite>
) {
    sprite.index = (((0.5 - 8. * player.rotation / (2. * std::f32::consts::PI)).floor() as i32 + 21) % 8) as u32;
}
