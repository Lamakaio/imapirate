use bevy::{
    prelude::*,
    render::camera::Camera
};
use bevy::prelude::Translation;
struct MapParam {
    seed : usize
}
use bevy_tiled::TiledMapPlugin;
use super::worldgen;
use super::tmxgen;
use std::time::Duration;
use std::collections::HashMap;
use worldgen::CHUNK_SIZE;
use worldgen::TILE_SIZE;

const BOAT_LAYER : f32 = 100.;
const SEA_LAYER : f32 = 0.;
pub struct SeaPlugin;

impl Plugin for SeaPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(setup.system() )
        .add_system(animate_sprite_system.system())
        .init_resource::<Time>()
        .init_resource::<SeaHandles>()
        .add_resource(MapParam {seed : 1})
        .add_resource(HashMap::<(i32, i32), Chunk>::new())
        .add_system(player_movement.system())
        .add_system(keyboard_input_system.system())
        .add_plugin(TiledMapPlugin)
        .add_system(animate_tile_system.system())
        .add_system(draw_chunks_system.system());
    }
}

pub struct Chunk {
    drawn : bool
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

#[derive(Default)]
pub struct SeaHandles {
    map_sheet : HashMap<u32, Handle<ColorMaterial>>,
    sea_chunk1 : Handle<ColorMaterial>,
    sea_chunk2 : Handle<ColorMaterial>,
    sea_chunk3 : Handle<ColorMaterial>,
}

#[derive(Clone, Copy)]
pub enum TileKind {
    Sand,
    Forest,
    Sea
}
impl Default for TileKind {
    fn default() -> Self {
        TileKind::Sea
    }
}

#[derive(Default, Clone, Copy)]
pub struct Tile {
    pub kind : TileKind,
    pub variant : u32,
    pub sprite_id : u32
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    param : Res<MapParam>,
    mut chunks : ResMut<HashMap<(i32, i32), Chunk>>, 
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handles: ResMut<SeaHandles>,
) {
    let texture_handle_boat = asset_server.load("assets/sprites/sea/boat.png").unwrap();
    let texture_handle_map_spritesheet = asset_server.load("assets/sprites/sea/sheet.png").unwrap();
    let texture_handle_sea1 = asset_server.load_sync(&mut textures,"assets/sprites/sea/sea_chunk_1.png").unwrap();
    let texture_handle_sea2 = asset_server.load_sync(&mut textures,"assets/sprites/sea/sea_chunk_2.png").unwrap();
    let texture_handle_sea3 = asset_server.load_sync(&mut textures,"assets/sprites/sea/sea_chunk_3.png").unwrap();
    //let texture_handle_sheet = asset_server.load("assets/sprites/sea/sheet.png").unwrap();
    let map = worldgen::generate_chunk(0, 0, param.seed);
    tmxgen::generate_tmx(map, "world/sea/chunk-sea-00.tmx");
    let mut material_map = HashMap::new();
    material_map.insert(1, materials.add(texture_handle_map_spritesheet.into()));
    handles.map_sheet = material_map;
    handles.sea_chunk1 = materials.add(texture_handle_sea1.into());
    handles.sea_chunk2 = materials.add(texture_handle_sea2.into());
    handles.sea_chunk3 = materials.add(texture_handle_sea3.into());

     chunks.insert((0, 0), Chunk {
         drawn  :true
     });

    commands
        .spawn(Camera2dComponents::default())
        .spawn(bevy_tiled::TiledMapComponents {
                map_asset: asset_server.load("world/sea/chunk-sea-00.tmx").unwrap(),
                materials: handles.map_sheet.clone(),
                center: true,
                ..Default::default()
        })
        .spawn(SpriteComponents {
            material: handles.sea_chunk1,
            translation : Translation::new(0., 0., SEA_LAYER),
            scale : Scale (4.),
            ..Default::default()
        })
        .with(Timer::new(Duration::from_millis(500), true))
        .with(0 as usize)
        .with(vec![
            handles.sea_chunk1,
            handles.sea_chunk2,
            handles.sea_chunk3
        ])
        .spawn(
            SpriteComponents {
                material: materials.add(texture_handle_boat.into()),
                translation : Translation::new(0., 0., BOAT_LAYER),
                ..Default::default()
            })
            .with(Player::new());
}
fn animate_tile_system(
    mut query: Query<(&mut Handle<ColorMaterial>, &Timer, &mut usize, &Vec<Handle<ColorMaterial>>)>,
) {
    
    for (mut material, timer, mut current, anim_vec) in &mut query.iter() {
        if timer.finished {
            *current += 1;
            if *current >= anim_vec.len() {
                *current = 0;
            }
            *material = anim_vec[*current];
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
            player.acceleration = 580.;
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

fn draw_chunks_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    param : Res<MapParam>,
    handles: Res<SeaHandles>,
    mut chunks : ResMut<HashMap<(i32, i32), Chunk>>, 
    mut player_query : Query<(& Player, &Translation)>,
) {
    for (_, translation) in &mut player_query.iter() {
        let chunk_x = (0.5 + translation.x()/(TILE_SIZE*CHUNK_SIZE) as f32).floor() as i32;
        let chunk_y = (0.5 + translation.y()/(TILE_SIZE*CHUNK_SIZE) as f32).floor() as i32;
        let surroundings = [(chunk_x + 1, chunk_y),
                                                 (chunk_x + 1, chunk_y + 1),
                                                 (chunk_x + 1, chunk_y - 1),
                                                 (chunk_x, chunk_y + 1),
                                                 (chunk_x, chunk_y - 1),
                                                 (chunk_x - 1, chunk_y + 1),
                                                 (chunk_x - 1, chunk_y),
                                                 (chunk_x - 1, chunk_y - 1),
        ];
        for (x, y) in surroundings.iter() {
            if !chunks.contains_key(&(*x, *y)) {
                chunks.insert((*x, *y), Chunk {
                    drawn : false
                });
            }
            let chunk = chunks.get_mut(&(*x, *y)).unwrap();
            if !chunk.drawn {
                chunk.drawn = true;
                let map = worldgen::generate_chunk(*x, *y, param.seed);
                tmxgen::generate_tmx(map, &format!("world/sea/chunk-sea-{}{}.tmx", x, y));
                commands
                .spawn(
                    bevy_tiled::TiledMapComponents {
                        map_asset: asset_server.load(format!("world/sea/chunk-sea-{}{}.tmx", x, y)).unwrap(),
                        materials: handles.map_sheet.clone(),
                        center: true,
                        origin : Translation::new((TILE_SIZE*CHUNK_SIZE*x) as f32,
                                                       (TILE_SIZE*CHUNK_SIZE*y) as f32, 
                                                       0.),
                        ..Default::default()
                    }
                )
                .spawn(SpriteComponents {
                    material: handles.sea_chunk1,
                    translation :Translation::new((TILE_SIZE*CHUNK_SIZE*x) as f32,
                    (TILE_SIZE*CHUNK_SIZE*y) as f32, 
                    SEA_LAYER),
                    scale : Scale (4.),
                    ..Default::default()
                })
                .with(Timer::new(Duration::from_millis(500), true))
                .with(0 as usize)
                .with(vec![
                    handles.sea_chunk1,
                    handles.sea_chunk2,
                    handles.sea_chunk3
                ]);
            }
        }
    }
}