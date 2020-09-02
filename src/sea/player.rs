
use bevy::{
    prelude::*,
    render::camera::Camera
};
const BOAT_LAYER : f32 = 100.;

pub struct SeaPlayerPlugin;
impl Plugin for SeaPlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(setup.system() )
        .init_resource::<Time>()
        .add_resource(PlayerPositionUpdate {
            last_pos : Translation::new(0., 0., BOAT_LAYER),
            should_update : true
        })
        .add_system(player_movement.system())
        .add_system(keyboard_input_system.system())
        ;
    }
}

struct Player { 
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
             rotation_friction : 2.5}
    }
}

const UPDATE_DISTANCE : f32 = 128.;
pub struct PlayerPositionUpdate {
    pub last_pos : Translation,
    pub should_update : bool
}
impl PlayerPositionUpdate {
    fn update(&mut self, t : &Translation) {
        self.should_update = false;
        if (t.x() - self.last_pos.x()).abs() > UPDATE_DISTANCE || (t.y() - self.last_pos.y()).abs() > UPDATE_DISTANCE {
            *self.last_pos.x_mut() = t.x();
            *self.last_pos.y_mut() = t.y();
            self.should_update = true;
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>, 
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    //loading textures
    let texture_handle_boat = asset_server.load("assets/sprites/sea/boat.png").unwrap();

    //spawning entities
    commands
    //camera
        .spawn(Camera2dComponents {
        scale : Scale(30.),
        ..Default::default()
        })
    //player
        .spawn(
        SpriteComponents {
            material: materials.add(texture_handle_boat.into()),
            translation : Translation::new(0., 0., BOAT_LAYER),
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
            player.acceleration = 500.;
        }
        else if keyboard_input.just_pressed(KeyCode::Down) {
            player.acceleration = -500.;
        }
    
        if keyboard_input.just_pressed(KeyCode::Right) {
            player.rotation_acceleration = -10.;
        }
        else if keyboard_input.just_pressed(KeyCode::Left) {
            player.rotation_acceleration = 10.;
        }
    } 
}

fn player_movement(
    time : Res<Time>,
    mut pos_update : ResMut<PlayerPositionUpdate>,
    mut player_query : Query<(&mut Player, &mut Translation, &mut Rotation)>,
    mut camera_query : Query<(&Camera, &mut Translation)>, 
) {
    for (mut player, mut player_translation, mut player_rotation) in &mut player_query.iter() {
        player.rotation_speed += (player.rotation_acceleration - player.rotation_speed * player.rotation_friction) * time.delta_seconds;
        player.speed += (player.acceleration - player.speed * player.friction) * time.delta_seconds;
        player.rotation += player.rotation_speed * time.delta_seconds;
        *player_rotation = Rotation::from_rotation_z(player.rotation);
        let (s, c) = f32::sin_cos(player.rotation);
        *player_translation.x_mut() += c * player.speed * time.delta_seconds;
        *player_translation.y_mut() += s * player.speed * time.delta_seconds;

        pos_update.update(&player_translation);
        for (_camera, mut camera_translation) in &mut camera_query.iter() {
            camera_translation.set_x(player_translation.x());
            camera_translation.set_y(player_translation.y());
        }
    } 
}