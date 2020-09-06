use bevy::prelude::*;
use bevy_tiled::{TiledMapPlugin, 
                 Map, 
                 map::spawn_map};
use std::{path::Path,
          collections::HashMap,
          time::Duration};

use super::player::PlayerPositionUpdate;
use super::worldgen::{CHUNK_SIZE, TILE_SIZE, generate_chunk, SCALING};

use super::player::{Player, FrictionType};

const SEA_LAYER : f32 = 0.;

struct MapParam {
    seed : usize
}

pub struct SeaMapPlugin;
impl Plugin for SeaMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(setup.system() )
        .add_startup_system(draw_chunks_system.system() )
        .add_plugin(TiledMapPlugin)
        .init_resource::<Time>()
        .init_resource::<SeaHandles>()
        .add_resource(MapParam {seed : 1234})
        .add_resource(HashMap::<(i32, i32), Chunk>::new())
        .add_resource(TilesAnimationTimer 
            {timer : Timer::new(Duration::from_millis(500), true),
                current : 0, 
                num_images : 3
            }
        )
        .add_system(animate_tile_system.system())
        .add_system(draw_chunks_system.system())
        .add_system(despawn_chunk_system.system())
        .add_system(collision_system.system())
        ;
    }
}

pub struct Chunk {
    drawn : bool,
    map_handle : Handle<Map>
}

pub struct SeaChunk;

#[derive(Default)]
pub struct SeaHandles {
    map_sheet : HashMap<u32, Handle<ColorMaterial>>,
    sea_chunk1 : Handle<ColorMaterial>,
    sea_chunk2 : Handle<ColorMaterial>,
    sea_chunk3 : Handle<ColorMaterial>,
}

struct TilesAnimationTimer {
    timer : Timer, 
    current : usize, 
    num_images : usize
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    mut maps: ResMut<Assets<Map>>,
    mut handles: ResMut<SeaHandles>,
) {
    //loading textures
    let texture_handle_map_spritesheet = asset_server.load("assets/sprites/sea/sheet.png").unwrap();
    let texture_handle_sea1 = asset_server.load_sync(&mut textures,"assets/sprites/sea/sea_chunk_1.png").unwrap();
    let texture_handle_sea2 = asset_server.load_sync(&mut textures,"assets/sprites/sea/sea_chunk_2.png").unwrap();
    let texture_handle_sea3 = asset_server.load_sync(&mut textures,"assets/sprites/sea/sea_chunk_3.png").unwrap();
    
    //initializing the sea animation
    let mut material_map = HashMap::new();
    material_map.insert(1, materials.add(texture_handle_map_spritesheet.into()));
    handles.map_sheet = material_map;
    handles.sea_chunk1 = materials.add(texture_handle_sea1.into());
    handles.sea_chunk2 = materials.add(texture_handle_sea2.into());
    handles.sea_chunk3 = materials.add(texture_handle_sea3.into());

    //generating the first chunk
    let map = bevy_tiled::loader::map_from_tiled_map(
        generate_chunk(0, 0, param.seed), 
        Path::new("assets/sprites/sea/sheet.png")).unwrap();
    let map_handle = maps.add(map);
     chunks.insert((0, 0), Chunk {
         drawn  : true,
         map_handle : map_handle.clone()
     });

    //spawning entities
    commands
    //first chunk map
        .spawn(bevy_tiled::TiledMapComponents {
                map_asset: map_handle,
                materials: handles.map_sheet.clone(),
                center: true,
                origin: Translation::new(0., 0., SEA_LAYER)
        })
    //first chunk sea
        .spawn(SpriteComponents {
            material: handles.sea_chunk1,
            translation : Translation::new(0., 0., SEA_LAYER),
            scale : Scale (4.),
            ..Default::default()
        })
        .with(vec![
            handles.sea_chunk1,
            handles.sea_chunk2,
            handles.sea_chunk3
        ])
        .with(SeaChunk)
        .with(0 as usize);
}


fn animate_tile_system(
    time : Res<Time>,
    mut state : ResMut<TilesAnimationTimer>,
    mut query: Query<(&mut Handle<ColorMaterial>, &Vec<Handle<ColorMaterial>>)>,
) {
    state.timer.tick(time.delta_seconds);
    if state.timer.finished {
        state.current += 1;
        if state.current >= state.num_images {
            state.current = 0;
        }
        for (mut material, anim_vec) in &mut query.iter() {
            *material = anim_vec[state.current];
        }
    }
}

fn draw_chunks_system(
    mut commands: Commands,
    param : Res<MapParam>,
    pos_update : Res<PlayerPositionUpdate>,
    handles: Res<SeaHandles>,
    mut chunks : ResMut<HashMap<(i32, i32), Chunk>>, 
    mut maps: ResMut<Assets<Map>>,
) {
    if pos_update.changed_chunk {
        let chunk_x = pos_update.chunk_x;
        let chunk_y = pos_update.chunk_y;
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
            if chunks.contains_key(&(*x, *y)) {
                let chunk = chunks.get_mut(&(*x, *y)).unwrap();
                if chunk.drawn {
                    continue;
                }
                chunk.drawn = true;
                spawn_map(
                    &mut commands, 
                    maps.get(&chunk.map_handle).unwrap(),
                    &handles.map_sheet.clone(),
                    &Translation::new((TILE_SIZE*SCALING*CHUNK_SIZE*x) as f32,
                                                       (TILE_SIZE*SCALING*CHUNK_SIZE*y) as f32, 
                                                       0.),
                    true
                )
            }
            else {
                let map = bevy_tiled::loader::map_from_tiled_map(
                    generate_chunk(*x, *y, param.seed), 
                    Path::new("assets/sprites/sea/sheet.png")).unwrap();
                let map_handle = maps.add(map);
                chunks.insert((*x, *y), Chunk {
                    drawn : true,
                    map_handle : map_handle.clone()
                });
                commands
                .spawn(
                    bevy_tiled::TiledMapComponents {
                        map_asset: map_handle,
                        materials: handles.map_sheet.clone(),
                        center: true,
                        origin : Translation::new((TILE_SIZE*SCALING*CHUNK_SIZE*x) as f32,
                                                       (TILE_SIZE*SCALING*CHUNK_SIZE*y) as f32, 
                                                       0.),
                    });
            }
            commands
                .spawn(SpriteComponents {
                    material: handles.sea_chunk1,
                    translation :Translation::new((TILE_SIZE*SCALING*CHUNK_SIZE*x) as f32,
                    (TILE_SIZE*SCALING*CHUNK_SIZE*y) as f32, 
                    SEA_LAYER),
                    scale : Scale (4.),
                    ..Default::default()
                })
                .with(SeaChunk)
                .with(0 as usize)
                .with(vec![
                    handles.sea_chunk1,
                    handles.sea_chunk2,
                    handles.sea_chunk3
                ]);
            }
        }
    }
fn uncenter(pos : &Translation) -> Translation {
    let center = Vec2::new((CHUNK_SIZE*TILE_SIZE*SCALING) as f32 / 2.0, (CHUNK_SIZE*TILE_SIZE*SCALING) as f32 / 2.0);
    Translation::new(
        pos.x() + center.x(),
        pos.y() - center.y(),
        pos.z()

    )
}
fn despawn_chunk_system(
    mut commands: Commands,
    pos_update : Res<PlayerPositionUpdate>,
    mut chunks : ResMut<HashMap<(i32, i32), Chunk>>, 
    mut sea_chunk_query : Query<(Entity, &Translation, &SeaChunk)>,
    mut island_chunk_query : Query<(Entity, &Translation, &bevy_tiled::TileMapChunk)>
) {
    if pos_update.changed_chunk {
        let player_pos = pos_update.last_pos;
        for (entity, tile_pos, _) in &mut sea_chunk_query.iter() {
            let limit = (CHUNK_SIZE*TILE_SIZE*SCALING) as f32 * 2.5;
            if (tile_pos.x() - player_pos.x()).abs() > limit || (tile_pos.y() - player_pos.y()).abs() > limit {
                commands.despawn(entity);
            }
        }
        for (entity, centered_tile_pos, _) in &mut island_chunk_query.iter() {
            let tile_pos = uncenter(centered_tile_pos);
            let limit = (CHUNK_SIZE*TILE_SIZE*SCALING) as f32 * 2.5;
            if (tile_pos.x() - player_pos.x()).abs() > limit || (tile_pos.y() - player_pos.y()).abs() > limit {
                let chunk_x = (tile_pos.x()/(TILE_SIZE*SCALING*CHUNK_SIZE) as f32).floor() as i32;
                let chunk_y = (tile_pos.y()/(TILE_SIZE*SCALING*CHUNK_SIZE) as f32).floor() as i32;
                if let Some(chunk) = chunks.get_mut(&(chunk_x, chunk_y)) {
                    chunk.drawn = false;
                }
                commands.despawn(entity);
            }
        }
    }
}

fn collision_system(
    pos_update : Res<PlayerPositionUpdate>,
    chunks : Res<HashMap<(i32, i32), Chunk>>, 
    maps: Res<Assets<Map>>,
    mut player : Mut<Player>,
) {
    if pos_update.changed_tile {
        let chunk = &chunks[&(pos_update.chunk_x, pos_update.chunk_y)];
        let map = maps.get(&chunk.map_handle)
                            .expect(&format!("Map doesnt't exist {} {}", pos_update.chunk_x, pos_update.chunk_y));
        let tiles = &map.map.layers[0].tiles;
        println!("{} {}", pos_update.tile_x, pos_update.tile_y);
        let current_tile_id = tiles[(CHUNK_SIZE - 1 - pos_update.tile_y) as usize][pos_update.tile_x as usize].gid;
        if current_tile_id == 0 {
            player.set_friction(FrictionType::Sea);
        }
        else if (current_tile_id >= 33 && current_tile_id <= 48) || (current_tile_id >= 81 && current_tile_id <= 96) {
            player.set_friction(FrictionType::Shore);
        }
        else {
            player.set_friction(FrictionType::Land);
        }
    }
}
