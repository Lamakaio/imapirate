use super::sea_main;
use fuss::Simplex;
use sea_main::{TileKind, Tile};
use sea_main::TileKind::*;
use std::hash::Hasher;
pub const CHUNK_SIZE : i32 = 128;
pub const TILE_SIZE : i32 = 64;
pub const NORM : f32 = 32.;
//bisous <3

fn get_sprite_id(surroundings : [TileKind; 9]) -> u32 {
    match surroundings {
        //outer corners
        [Forest, Sea, _, Forest, _, Forest, _, Sea, _] | 
        [Forest, Forest, Sea, Forest, _, Forest, _, Sea, Sea] | 
        [Forest, Sea, _, Forest, _, Forest, Sea, Forest, Sea] => 41, //NW
        [Forest, Sea, _, Sea, _, Forest, _, Forest, _] | 
        [Forest, Sea, Sea, Forest, Sea, Forest, _, Forest, _] | 
        [Forest, Forest, Sea, Sea, _, Forest, _, Forest, Sea] => 48, //NE
        [Forest, Forest, _, Sea, _, Sea, _, Forest, _] | 
        [Forest, Forest, _, Sea, Sea, Forest, Sea, Forest, _] | 
        [Forest, Forest, Sea, Forest, Sea, Sea, _, Forest, _] => 56, //SE
        [Forest, Forest, _, Forest, _, Sea, _, Sea, _] | 
        [Forest, Forest, _, Forest, _, Sea, Sea, Forest, Sea] | 
        [Forest, Forest, _, Forest, Sea, Forest, Sea, Sea, _] => 53, //SW

        [Forest, Sand, _, Forest, _, Forest, _, Sand, _] | 
        [Forest, Forest, Sand, Forest, _, Forest, _, Sand, Sand] | 
        [Forest, Sand, _, Forest, _, Forest, Sand, Forest, Sand] => 57, //NW
        [Forest, Sand, _, Sand, _, Forest, _, Forest, _] | 
        [Forest, Sand, Sand, Forest, Sand, Forest, _, Forest, _] | 
        [Forest, Forest, Sand, Sand, _, Forest, _, Forest, Sand] => 60, //NE
        [Forest, Forest, _, Sand, _, Sand, _, Forest, _] | 
        [Forest, Forest, _, Sand, Sand, Forest, Sand, Forest, _] | 
        [Forest, Forest, Sand, Forest, Sand, Sand, _, Forest, _] => 72, //SE
        [Forest, Forest, _, Forest, _, Sand, _, Sand, _] | 
        [Forest, Forest, _, Forest, _, Sand, Sand, Forest, Sand] | 
        [Forest, Forest, _, Forest, Sand, Forest, Sand, Sand, _] => 69, //SW

        [Sand, Sea, _, Sand, _, Sand, _, Sea, _] | 
        [Sand, Sand, Sea, Sand, _, Sand, _, Sea, Sea] | 
        [Sand, Sea, _, Sand, _, Sand, Sea, Sand, Sea] => 17, //NW
        [Sand, Sea, _, Sea, _, Sand, _, Sand, _] | 
        [Sand, Sea, Sea, Sand, Sea, Sand, _, Sand, _] | 
        [Sand, Sand, Sea, Sea, _, Sand, _, Sand, Sea] => 20, //NE
        [Sand, Sand, _, Sea, _, Sea, _, Sand, _] | 
        [Sand, Sand, _, Sea, Sea, Sand, Sea, Sand, _] | 
        [Sand, Sand, Sea, Sand, Sea, Sea, _, Sand, _] => 4, //SE
        [Sand, Sand, _, Sand, _, Sea, _, Sea, _] | 
        [Sand, Sand, _, Sand, _, Sea, Sea, Sand, Sea] | 
        [Sand, Sand, _, Sand, Sea, Sand, Sea, Sea, _,] => 2, //SW

        //sides
        [Forest, Sea, _, Forest, _, _, _, Forest, _] | [Forest, Forest, Sea, Forest, _, _, _, Forest, Sea] => 42, //N
        [Forest, Forest, _, Sea, _, Forest, _, _, _] | [Forest, Forest, Sea, Forest, Sea, Forest, _, _, _] => 48, //E
        [Forest,  _, _, Forest, _, Sea, _, Forest, _] | [Forest, _, _, Forest, Sea, Forest, Sea, Forest, _] => 55, //S
        [Forest, Forest, _, _, _, Forest, _, Sea, _] | [Forest, Forest, _, _, _, Forest, Sea, Forest, Sea] => 49, //W

        [Forest, Sand, _, Forest, _, _, _, Forest, _] | [Forest, Forest, Sand, Forest, _, _, _, Forest, Sand] => 58, //N
        [Forest, Forest, _, Sand, _, Forest, _, _, _] | [Forest, Forest, Sand, Forest, Sand, Forest, _, _, _] => 64, //E
        [Forest,  _, _, Forest, _, Sand, _, Forest, _] | [Forest, _, _, Forest, Sand, Forest, Sand, Forest, _] => 71, //S
        [Forest, Forest, _, _, _, Forest, _, Sand, _] | [Forest, Forest, _, _, _, Forest, Sand, Forest, Sand] => 65, //W

        [Sand, Sea, _, Sand, _, _, _, Sand, _] | [Sand, Sand, Sea, Sand, _, _, _, Sand, Sea] => 22, //N
        [Sand, Sand, _, Sea, _, Sand, _, _, _] | [Sand, Sand, Sea, Sand, Sea, Sand, _, _, _] => 15, //E
        [Sand,  _, _, Sand, _, Sea, _, Sand, _] | [Sand, _, _, Sand, Sea, Sand, Sea, Sand, _] => 6, //S
        [Sand, Sand, _, _, _, Sand, _, Sea, _] | [Sand, Sand, _, _, _, Sand, Sea, Sand, Sea] => 10, //W

        //inner corners
        [Forest, Forest, _, Forest, Sea, Forest, _, Forest, _] => 33, 
        [Forest, Forest, _, Forest, _, Forest, Sea, Forest, _] => 34,
        [Forest, Forest, _, Forest, _, Forest, _, Forest, Sea] => 38,
        [Forest, Forest, Sea, Forest, _, Forest, _, Forest, _] => 37,

        [Forest, Forest, _, Forest, Sand, Forest, _, Forest, _] => 35,
        [Forest, Forest, _, Forest, _, Forest, Sand, Forest, _] => 36,
        [Forest, Forest, _, Forest, _, Forest, _, Forest, Sand] => 40,
        [Forest, Forest, Sand, Forest, _, Forest, _, Forest, _] => 39,

        [Sand, _, _, Sand, Sea, Sand, _, _, _] => 30,
        [Sand, _, _, _, _, Sand, Sea, Sand, _] => 31,
        [Sand, Sand, _, _, _, _, _, Sand, Sea] => 27,
        [Sand, Sand, Sea, Sand, _, _, _, _, _] => 26,

        //triple
        [_, Sand, Sand, Sand, Sand, Sand, _, _, _] |
        [_, _, _, Sand, Sand, Sand, Sand, Sand, _] |
        [_, Sand, _, _, _, Sand, Sand, Sand, Sand] |
        [_, Sand, Sand, Sand, _, _, _, Sand, Sand] => 1,

        [_, Sea, Sea, Sea, Sea, Sea, _, _, _] |
        [_, _, _, Sea, Sea, Sea, Sea, Sea, _] |
        [_, Sea, _, _, _, Sea, Sea, Sea, Sea] |
        [_, Sea, Sea, Sea, _, _, _, Sea, Sea] => 0,
        
        //inside
        [Forest, _, _, _, _, _, _, _, _] => 46,
        [Sand, _, _, _, _, _, _, _, _] => 1
        ,

        _ => 0
    }
}

pub fn generate_chunk(pos_x : i32, pos_y : i32, world_seed : usize) -> Vec<Vec<sea_main::Tile>>{
    let sn = Simplex::from_seed(vec![pos_x as usize, pos_y as usize, world_seed]);
    let seed = sn.seed.clone();
    let mut map : Vec<Vec<sea_main::Tile>> = Vec::new();
    let hash = hash_vec(seed);
    for i in 0..CHUNK_SIZE {
        map.push(Vec::new());
        for j in 0..CHUNK_SIZE {
            let height = sn.noise_2d(
                (CHUNK_SIZE * pos_x + i) as f32/NORM, 
                (CHUNK_SIZE * pos_y + j) as f32/NORM);
            const LIM : i32 = CHUNK_SIZE - 1;
            let offset = match (i, j) {
                (0, _) | (CHUNK_SIZE, _) | (_, 0) | (_, CHUNK_SIZE)
                => 0.4,
                (1, _) | (LIM, _) | (_ , 1) | (_, LIM)
                => 0.2,
                _ => 0.
            };
            map[i as usize].push(get_tile_type(
                    height - offset,
                    hash)
            )
        }
    }
    fill_sprites_id(&mut map);
    map
}

fn fill_sprites_id(map : &mut Vec<Vec<sea_main::Tile>>) {
    for i in 1..(CHUNK_SIZE as usize-1) {
        for j in 1..(CHUNK_SIZE as usize-1) {
            map[i][j].sprite_id = get_sprite_id([
                map[i][j].kind,
                map[i-1][j].kind,
                map[i-1][j+1].kind,
                map[i][j+1].kind,
                map[i+1][j+1].kind,
                map[i+1][j].kind,
                map[i+1][j-1].kind,
                map[i][j-1].kind,
                map[i-1][j-1].kind,
            ])
        }
    }
}

fn hash_vec(seed : Vec<usize>) -> u64 {
    let mut hasher = seahash::SeaHasher::new();
    for i in seed {
        hasher.write_usize(i);
    }
    hasher.finish()
}
fn get_tile_type(height : f32, hash : u64) -> Tile {
    let var;
    let kind;
    if height > 0.6  && height <= 0.8 {
        kind = TileKind::Sand;
        let decision = (hash as f32 * height) as u64;
        if decision % 7 <= 3 {
            var = 1;
        }
        else {
            var = 2;
        }
    }
    else if height > 0.8 {
        let decision = (hash as f32 * height) as u64;
        kind = TileKind::Forest;
        if decision % 7 <= 3 {
            var = 1;
        }
        else {
            var = 2;
        }
    }
    else {
        kind = TileKind::Sea;
        var = 1;
    }
    Tile {
        variant : var,
        kind, 
        sprite_id : 0
    }
}
