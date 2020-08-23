use bevy::{
    asset::{HandleId, LoadState},
    prelude::*,
    render::camera::Camera
};
use bevy::prelude::Translation;
use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    hash::Hasher
    };
use bevy_tiled::TiledMapPlugin;

const TILE_SIZE : u32 = 200;
const SEA_LAYER : f32 = 0.;
const BOAT_LAYER : f32 = 1.;

pub struct SeaPlugin;

impl Plugin for SeaPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(setup.system() )
        .add_system(animate_sprite_system.system())
        .add_system(loading_system.system())
        .add_system(init_scene_system.system())
        .init_resource::<Time>()
        .init_resource::<AssetHandles>()
        .init_resource::<LoadingEventListenerState>()
        .init_resource::<DrawnWindow>()
        .init_resource::<Map>()
        .init_resource::<MapParam>()
        .add_system(player_movement.system())
        .add_system(keyboard_input_system.system())
        //.add_system(draw_tile_system.system())
        .add_event::<LoadingEvent>()
        .add_plugin(TiledMapPlugin);
    }
}

#[derive(Default)]
struct AssetHandles {
    handles : Vec<HandleId>,
    loaded : bool
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
             rotation_friction : 2.5}
    }
}

struct LoadingScreen;

struct LoadingEvent {
    status : u32 //100 = loaded
}

struct DrawnWindow {
    center : Translation, 
    center_pos : TilePos,
    win_height : f32, 
    win_width : f32, 
    tiles_height : i32, 
    tiles_width : i32
}
impl Default for DrawnWindow {
    fn default() -> DrawnWindow {
        DrawnWindow {
            center : Translation::new(0., 0., 0.), 
            center_pos : TilePos::default(),
            win_height : 0., 
            win_width : 0., 
            tiles_height : 0, 
            tiles_width : 0
        }
    }
}
impl DrawnWindow {
    fn init(&mut self, win : &WindowDescriptor) {
        self.win_height = win.height as f32;
        self.win_width = win.width as f32;
        self.center_pos = TilePos {x:0, y:0, dim:0};
        self.tiles_height = ((win.height as f32)/(TILE_SIZE as f32)).ceil() as i32 + 2;
        self.tiles_width = ((win.width as f32)/(TILE_SIZE as f32)).ceil() as i32 + 2;
        self.center = Translation::new(0., 0., 0.);
        
    }

    fn to_draw(&mut self, center : &Translation) -> Vec<TilePos>{
        let mut vec = Vec::new();
        if (center.x() - self.center.x()).abs() >= TILE_SIZE as f32 {
            if center.x() > self.center.x() {
                self.center_pos.x += 1;
                *self.center.x_mut() += TILE_SIZE as f32;
                for i in 0..(self.tiles_height+2) {
                    vec.push(TilePos {
                        x : self.center_pos.x + self.tiles_width/2,
                        y : self.center_pos.y + i - self.tiles_height/2 - 1,
                        dim : 0
                    })
                }
            }
            else {
                self.center_pos.x -= 1;
                *self.center.x_mut() -= TILE_SIZE as f32;
                for i in 0..(self.tiles_height+2) {
                    vec.push(TilePos {
                        x : self.center_pos.x - self.tiles_width/2,
                        y : self.center_pos.y + i - self.tiles_height/2 - 1,
                        dim : 0
                    })
                }
            }
        }
        
        if (center.y() - self.center.y()).abs() >= TILE_SIZE as f32 {
            if center.y() > self.center.y() {
                self.center_pos.y += 1;
                *self.center.y_mut() += TILE_SIZE as f32;
                self.center.set_y(center.y());
                for i in 0..(self.tiles_width+2) {
                    vec.push(TilePos {
                        x : self.center_pos.x + i - self.tiles_width/2 - 1,
                        y : self.center_pos.y + self.tiles_height/2,
                        dim : 0
                    })
                }
            }
            else {
                self.center_pos.y -= 1;
                *self.center.y_mut() -= TILE_SIZE as f32;
                for i in 0..(self.tiles_width+2) {
                    vec.push(TilePos {
                        x : self.center_pos.x + i - self.tiles_width/2 - 1,
                        y : self.center_pos.y - self.tiles_height/2,
                        dim : 0
                    })
                }
            }
        }
        vec
    }
}


#[derive(Default)]
struct LoadingEventListenerState {
    loading_event_reader: EventReader<LoadingEvent>,
}


#[derive(Default)]
pub struct MapParam {
    seed : u64,
}
#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TilePos {
    pub x : i32,
    pub y : i32, 
    pub dim : i8
}

impl TilePos {
    fn hash_seed(&self, seed : u64) -> u64 {
        let mut hasher = seahash::SeaHasher::new();
        hasher.write_u64(seed);
        hasher.write_i32(self.x);
        hasher.write_i32(self.y);
        hasher.write_i8(self.dim);
        return hasher.finish()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum TileKind {
    Island, 
    Sea
}
impl Default for TileKind {
    fn default() -> Self {
        TileKind::Sea
    }
}
#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub struct Tile {
    kind : TileKind,
    drawn : bool
}
#[derive(Default, Serialize, Deserialize )]
pub struct Map {
    tiles : HashMap<TilePos, Tile>
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    win : Res<WindowDescriptor>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut asset_handles: ResMut<AssetHandles>, 
    mut drawn_window : ResMut<DrawnWindow>,
) {
    drawn_window.init(&win);
    let texture_handle = asset_server
        .load_sync(&mut textures, "assets/sprites/loading.png")
        .unwrap();
    
    commands
        .spawn(Camera2dComponents::default())
        .spawn(SpriteComponents {
            material: materials.add(texture_handle.into()),
            //scale : Scale(12.0),
            ..Default::default()
        })
        .with(LoadingScreen);
    asset_handles.handles = asset_server
        .load_asset_folder("assets")
        .unwrap();
    let texture_handle = asset_server.load("assets/ortho.png").unwrap();
    commands
        .spawn(bevy_tiled::TiledMapComponents {
                map_asset: asset_server.load("assets/ortho-map.tmx").unwrap(),
                material: materials.add(texture_handle.into()),
                center: true,
                ..Default::default()
        })
        .spawn(Camera2dComponents::default());
}

fn loading_system(
    asset_server: Res<AssetServer>,
    mut asset_handles: ResMut<AssetHandles>, 
    mut loading_event: ResMut<Events<LoadingEvent>>,
    mut query : Query<(&LoadingScreen, &mut SpriteComponents)>, 
)
{
    if asset_handles.loaded {
        return;
    }
    if let Some(LoadState::Loaded(_)) =
        asset_server.get_group_load_state(&asset_handles.handles)
    {
        for (_, mut sprite) in &mut query.iter() {
            sprite.draw.is_visible = false;
        }
        asset_handles.loaded = true;
        print!("loading assets ...");
        loading_event.send(LoadingEvent{status : 100});
    }
}

fn init_scene_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<LoadingEventListenerState>,
    loading_events: Res<Events<LoadingEvent>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    for my_event in state.loading_event_reader.iter(&loading_events) {
        if my_event.status == 100 {
            println!("finished");
            let sea_handle = asset_server
                .get_handle("assets/sprites/sea/sea.png")
                .unwrap();
            let boat_handle = asset_server
                .get_handle("assets/sprites/sea/boat.png")
                .unwrap();
            commands
                .spawn(
                    SpriteComponents {
                        material: materials.add(boat_handle.into()),
                        translation : Translation::new(0., 0., BOAT_LAYER),
                        ..Default::default() }
                )
                .with(Player::new())
                .spawn(
                    SpriteComponents {
                        material: materials.add(sea_handle.into()),
                        translation : Translation::new(0., 0., SEA_LAYER),
                        ..Default::default() }
                    );
        }
    }
}
fn animate_sprite_system(
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query.iter() {
        if timer.finished {
            let texture_atlas = texture_atlases.get(&texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
            timer.reset();
        }
    }
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
            player.acceleration = 80.;
        }
        else if keyboard_input.just_pressed(KeyCode::Down) {
            player.acceleration = -80.;
        }
    
        if keyboard_input.just_pressed(KeyCode::Right) {
            player.rotation_acceleration = -3.;
        }
        else if keyboard_input.just_pressed(KeyCode::Left) {
            player.rotation_acceleration = 3.;
        }
    } 
}

fn player_movement(
    time : Res<Time>,
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

        for (_camera, mut camera_translation) in &mut camera_query.iter() {
            camera_translation.set_x(player_translation.x());
            camera_translation.set_y(player_translation.y());
        }
    } 
}

fn draw_tile_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    param : Res<MapParam>,
    mut drawn_window : ResMut<DrawnWindow>, 
    mut map: ResMut<Map>, 
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_query : Query<(&mut Player, &Translation)>,
) {
    for (_, translation) in &mut player_query.iter() {
        let to_draw = drawn_window.to_draw(&translation);
        for pos in to_draw.iter() {
            match map.tiles.get(pos) {
                Some (tile) if tile.drawn => {}
                opt => {
                let tile;
                if let Some (t) = opt {
                    tile = *t
                } 
                else {
                    tile = get_or_generate_tile(*pos, &mut map.tiles, param.seed);
                }
                let sprite_handle;
                match tile.kind {
                    TileKind::Sea => {
                        sprite_handle = asset_server
                        .get_handle("assets/sprites/sea/sea.png")
                        .unwrap();

                    }
                    TileKind::Island => {
                        sprite_handle = asset_server
                        .get_handle("assets/sprites/sea/island.png")
                        .unwrap();

                    }
                }
                commands
                .spawn(
                    SpriteComponents {
                        material: materials.add(sprite_handle.into()),
                        translation : Translation::new((pos.x * TILE_SIZE as i32) as f32, (pos.y * TILE_SIZE as i32) as f32, SEA_LAYER),
                        ..Default::default() }
                );
                }
            }
        }
    }
    
}

fn get_or_generate_tile(
    pos : TilePos, 
    map : &mut HashMap<TilePos, Tile>,
    seed : u64, 
) -> Tile {
    let hash = pos.hash_seed(seed);
    if hash % 4 == 0 {
        let tile = Tile {kind : TileKind::Island, drawn : false};
        map.insert(pos, tile);
        tile
    }
    else {
        let tile = Tile {kind : TileKind::Sea, drawn : false};
        map.insert(pos, tile);
        tile
    }
}