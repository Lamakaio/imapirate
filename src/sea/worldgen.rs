use fuss::Simplex;
use super::map::{TileKind, Tile};
use super::map::TileKind::*;
use std::hash::Hasher;
use tiled_builder::{TiledMapBuilder, LayerBuilder, TilesetBuilder, Orientation, Image};
pub const CHUNK_SIZE : i32 = 128;
//pub const SEA_TILE_SIZE : i32 = 64;
pub const SCALING : i32 = 4;
pub const TILE_SIZE : i32 = 16;
pub const NORM : f32 = 40.;
//bisous <3
const SEA : i32 = -1;

const FOREST_SEA_NW : i32 = 0;
const FOREST_SEA_NE : i32 = 4;
const FOREST_SEA_SE : i32 = 8;
const FOREST_SEA_SW : i32 = 12;

const FOREST_SAND_NW : i32 = 16;
const FOREST_SAND_NE : i32 = 20;
const FOREST_SAND_SE : i32 = 24;
const FOREST_SAND_SW : i32 = 28;

const SAND_SEA_NW : i32 = 32;
const SAND_SEA_NE : i32 = 36;
const SAND_SEA_SE : i32 = 40;
const SAND_SEA_SW : i32 = 44;

const FOREST_SEA_N : i32 = 48;
const FOREST_SEA_E : i32 = 52;
const FOREST_SEA_S : i32 = 56;
const FOREST_SEA_W : i32 = 60;

const FOREST_SAND_N : i32 = 64;
const FOREST_SAND_E : i32 = 68;
const FOREST_SAND_S : i32 = 72;
const FOREST_SAND_W : i32 = 76;

const SAND_SEA_N : i32 = 80;
const SAND_SEA_E : i32 = 84;
const SAND_SEA_S : i32 = 88;
const SAND_SEA_W : i32 = 92;

const FOREST_SEA_INNER_NW : i32 = 96;
const FOREST_SEA_INNER_NE : i32 = 100;
const FOREST_SEA_INNER_SE : i32 = 104;
const FOREST_SEA_INNER_SW : i32 = 108;

const FOREST_SAND_INNER_NW : i32 = 112;
const FOREST_SAND_INNER_NE : i32 = 116;
const FOREST_SAND_INNER_SE : i32 = 120;
const FOREST_SAND_INNER_SW : i32 = 124;

const SAND_SEA_INNER_NW : i32 = 128;
const SAND_SEA_INNER_NE : i32 = 132;
const SAND_SEA_INNER_SE : i32 = 136;
const SAND_SEA_INNER_SW : i32 = 140;

const FOREST : i32 = 144;
const SAND : i32 = 148;

const SAND_ROCK : i32 = 152;
const SEA_ROCK : i32 = 156;


const FOREST_SEA_NESW : i32 = 160;
const FOREST_SEA_NWSE : i32 = 160;

const FOREST_SAND_NESW : i32 = 164;
const FOREST_SAND_NWSE : i32 = 164;

const SAND_SEA_NESW : i32 = 168;
const SAND_SEA_NWSE : i32 = 168;

fn get_sprite_id(surroundings : [TileKind; 9], variant : u32) -> u32 {
    variant + (1 + match surroundings {
        //double corners 
        [Forest, Forest, _, Forest, Sea, Forest, _, Forest, Sea] => FOREST_SEA_NESW,
        [Forest, Forest, Sea, Forest, _, Forest, Sea, Forest, _] => FOREST_SEA_NWSE,

        [Forest, Forest, _, Forest, Sand, Forest, _, Forest, Sand] => FOREST_SAND_NESW,
        [Forest, Forest, Sand, Forest, _, Forest, Sand, Forest, _] => FOREST_SAND_NWSE,

        [Sand, Sand, _, Sand, Sea, Sand, _, Sand, Sea] => SAND_SEA_NESW,
        [Sand, Sand, Sea, Sand, _, Sand, Sea, Sand, _] => SAND_SEA_NWSE,
        //outer corners
        [Sea, _, _, _, _, _, _, _, _] => -1,
        [Forest, Sea, _, Forest, _, Forest, _, Sea, _] | 
        [Forest, Forest, Sea, Forest, _, Forest, _, Sea, Sea] | 
        [Forest, Sea, _, Forest, _, Forest, Sea, Forest, Sea] => FOREST_SEA_NW, //NW
        [Forest, Sea, _, Sea, _, Forest, _, Forest, _] | 
        [Forest, Sea, Sea, Forest, Sea, Forest, _, Forest, _] | 
        [Forest, Forest, Sea, Sea, _, Forest, _, Forest, Sea] => FOREST_SEA_NE, //NE
        [Forest, Forest, _, Sea, _, Sea, _, Forest, _] | 
        [Forest, Forest, _, Sea, Sea, Forest, Sea, Forest, _] | 
        [Forest, Forest, Sea, Forest, Sea, Sea, _, Forest, _] => FOREST_SEA_SE, //SE
        [Forest, Forest, _, Forest, _, Sea, _, Sea, _] | 
        [Forest, Forest, _, Forest, _, Sea, Sea, Forest, Sea] | 
        [Forest, Forest, _, Forest, Sea, Forest, Sea, Sea, _] => FOREST_SEA_SW, //SW

        [Forest, Sand, _, Forest, _, Forest, _, Sand, _] | 
        [Forest, Forest, Sand, Forest, _, Forest, _, Sand, Sand] | 
        [Forest, Sand, _, Forest, _, Forest, Sand, Forest, Sand] => FOREST_SAND_NW, //NW
        [Forest, Sand, _, Sand, _, Forest, _, Forest, _] | 
        [Forest, Sand, Sand, Forest, Sand, Forest, _, Forest, _] | 
        [Forest, Forest, Sand, Sand, _, Forest, _, Forest, Sand] => FOREST_SAND_NE, //NE
        [Forest, Forest, _, Sand, _, Sand, _, Forest, _] | 
        [Forest, Forest, _, Sand, Sand, Forest, Sand, Forest, _] | 
        [Forest, Forest, Sand, Forest, Sand, Sand, _, Forest, _] => FOREST_SAND_SE, //SE
        [Forest, Forest, _, Forest, _, Sand, _, Sand, _] | 
        [Forest, Forest, _, Forest, _, Sand, Sand, Forest, Sand] | 
        [Forest, Forest, _, Forest, Sand, Forest, Sand, Sand, _] => FOREST_SAND_SW, //SW

        [Sand, Sea, _, Sand, _, Sand, _, Sea, _] | 
        [Sand, Sand, Sea, Sand, _, Sand, _, Sea, Sea] | 
        [Sand, Sea, _, Sand, _, Sand, Sea, Sand, Sea] => SAND_SEA_NW, //NW
        [Sand, Sea, _, Sea, _, Sand, _, Sand, _] | 
        [Sand, Sea, Sea, Sand, Sea, Sand, _, Sand, _] | 
        [Sand, Sand, Sea, Sea, _, Sand, _, Sand, Sea] => SAND_SEA_NE, //NE
        [Sand, Sand, _, Sea, _, Sea, _, Sand, _] | 
        [Sand, Sand, _, Sea, Sea, Sand, Sea, Sand, _] | 
        [Sand, Sand, Sea, Sand, Sea, Sea, _, Sand, _] => SAND_SEA_SE, //SE
        [Sand, Sand, _, Sand, _, Sea, _, Sea, _] | 
        [Sand, Sand, _, Sand, _, Sea, Sea, Sand, Sea] | 
        [Sand, Sand, _, Sand, Sea, Sand, Sea, Sea, _,] => SAND_SEA_SW, //SW

        //sides
        [Forest, Sea, _, Forest, _, _, _, Forest, _] | [Forest, Forest, Sea, Forest, _, _, _, Forest, Sea] => FOREST_SEA_N, //N
        [Forest, Forest, _, Sea, _, Forest, _, _, _] | [Forest, Forest, Sea, Forest, Sea, Forest, _, _, _] => FOREST_SEA_E, //E
        [Forest,  _, _, Forest, _, Sea, _, Forest, _] | [Forest, _, _, Forest, Sea, Forest, Sea, Forest, _] => FOREST_SEA_S, //S
        [Forest, Forest, _, _, _, Forest, _, Sea, _] | [Forest, Forest, _, _, _, Forest, Sea, Forest, Sea] => FOREST_SEA_W, //W

        [Forest, Sand, _, Forest, _, _, _, Forest, _] | [Forest, Forest, Sand, Forest, _, _, _, Forest, Sand] => FOREST_SAND_N, //N
        [Forest, Forest, _, Sand, _, Forest, _, _, _] | [Forest, Forest, Sand, Forest, Sand, Forest, _, _, _] => FOREST_SAND_E, //E
        [Forest,  _, _, Forest, _, Sand, _, Forest, _] | [Forest, _, _, Forest, Sand, Forest, Sand, Forest, _] => FOREST_SAND_S, //S
        [Forest, Forest, _, _, _, Forest, _, Sand, _] | [Forest, Forest, _, _, _, Forest, Sand, Forest, Sand] => FOREST_SAND_W, //W

        [Sand, Sea, _, Sand, _, _, _, Sand, _] | [Sand, Sand, Sea, Sand, _, _, _, Sand, Sea] => SAND_SEA_N, //N
        [Sand, Sand, _, Sea, _, Sand, _, _, _] | [Sand, Sand, Sea, Sand, Sea, Sand, _, _, _] => SAND_SEA_E, //E
        [Sand,  _, _, Sand, _, Sea, _, Sand, _] | [Sand, _, _, Sand, Sea, Sand, Sea, Sand, _] => SAND_SEA_S, //S
        [Sand, Sand, _, _, _, Sand, _, Sea, _] | [Sand, Sand, _, _, _, Sand, Sea, Sand, Sea] => SAND_SEA_W, //W

        //inner corners
        [Forest, Forest, _, Forest, Sea, Forest, _, Forest, _] => FOREST_SEA_INNER_NW, 
        [Forest, Forest, _, Forest, _, Forest, Sea, Forest, _] => FOREST_SEA_INNER_NE,
        [Forest, Forest, _, Forest, _, Forest, _, Forest, Sea] => FOREST_SEA_INNER_SE,
        [Forest, Forest, Sea, Forest, _, Forest, _, Forest, _] => FOREST_SEA_INNER_SW,

        [Forest, Forest, _, Forest, Sand, Forest, _, Forest, _] => FOREST_SAND_INNER_NW,
        [Forest, Forest, _, Forest, _, Forest, Sand, Forest, _] => FOREST_SAND_INNER_NE,
        [Forest, Forest, _, Forest, _, Forest, _, Forest, Sand] => FOREST_SAND_INNER_SE,
        [Forest, Forest, Sand, Forest, _, Forest, _, Forest, _] => FOREST_SAND_INNER_SW,

        [Sand, _, _, Sand, Sea, Sand, _, _, _] => SAND_SEA_INNER_NW,
        [Sand, _, _, _, _, Sand, Sea, Sand, _] => SAND_SEA_INNER_NE,
        [Sand, Sand, _, _, _, _, _, Sand, Sea] => SAND_SEA_INNER_SE,
        [Sand, Sand, Sea, Sand, _, _, _, _, _] => SAND_SEA_INNER_SW,

        //triple 
        [_, Sand, Sand, Sand, Sand, Sand, _, _, _] |
        [_, _, _, Sand, Sand, Sand, Sand, Sand, _] |
        [_, Sand, _, _, _, Sand, Sand, Sand, Sand] |
        [_, Sand, Sand, Sand, _, _, _, Sand, Sand] => SAND,

        [_, Sea, Sea, Sea, Sea, Sea, _, _, _] |
        [_, _, _, Sea, Sea, Sea, Sea, Sea, _] |
        [_, Sea, _, _, _, Sea, Sea, Sea, Sea] |
        [_, Sea, Sea, Sea, _, _, _, Sea, Sea] => SEA - variant as i32,
        
        //inside
        [Forest, Forest, _, Forest, _, Forest, _, Forest, _] => FOREST,
        [Sand, x, _, y, _, z, _, w, _] 
        if x != Sea && y != Sea && z != Sea && w != Sea => SAND,
        
        [Sand, _, _, _, _, _, _, _, _] => SEA_ROCK,
        [Forest, _, _, _, _, _, _, _, _] => SAND_ROCK,

        //_ => SEA - variant as i32
    }) as u32
}

fn convert_to_tiled_map(tiles : Vec<Vec<Tile>>) -> tiled_builder::Map {
    let tiles_iterator = 
    tiles.iter()
         .flat_map(|i| {i.iter().map(|t| {t.sprite_id})});
    TiledMapBuilder::build(
        Orientation::Orthogonal, 
        TILE_SIZE as u32, 
        TILE_SIZE as u32, 
        CHUNK_SIZE as u32, 
        CHUNK_SIZE as u32
    )
    .add_tileset(
        TilesetBuilder::build(
            "island_tiles".to_string(), 
            TILE_SIZE as u32,
            TILE_SIZE as u32, 
            1,
            172, 
        )
        .add_image(
            Image {
                source : "assets/sprites/sea/sheet.png".to_string(),
                width : 64,
                height : 688,
                transparent_colour : None
            }
        )
        .finish()
    )
    .add_layer(
        LayerBuilder::build("island_layer".to_string(), CHUNK_SIZE as usize)
        .push_tiles_from_iterator(tiles_iterator)
        .finish()
    )
    .finish()
    
}
fn mexp(x : i32) -> f32 {
    (2.*(x as f32 - CHUNK_SIZE as f32/2.)/CHUNK_SIZE as f32).abs().powi(20).exp()
}
fn mountain(pos_x : i32, pos_y : i32) -> f32 {
    2. - mexp(pos_x) - mexp(pos_y)
}
pub fn generate_chunk(pos_x : i32, pos_y : i32, world_seed : usize) -> tiled_builder::Map {
    let sn = Simplex::from_seed(vec![pos_x as usize, pos_y as usize, world_seed]);
    let mut seed = sn.seed.clone();
    let mut map : Vec<Vec<Tile>> = Vec::new();
    seed.push(0);
    let last = seed.len() - 1;
    for i in 0..CHUNK_SIZE {
        map.push(Vec::new());
        for j in 0..CHUNK_SIZE {
            let height = sn.noise_2d(
                (CHUNK_SIZE * pos_x + i) as f32/NORM, 
                (CHUNK_SIZE * pos_y + j) as f32/NORM)
                + mountain(i, j);
            seed[last] = (height * 100000.) as usize;
            let hash = hash_vec(&seed);
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
    uniformization_pass(&mut map);
    fill_sprites_id(&mut map);
    convert_to_tiled_map(map)
}

fn uniformization_pass(map : &mut Vec<Vec<Tile>>) {
    for i in 1..(CHUNK_SIZE as usize-1) {
        for j in 1..(CHUNK_SIZE as usize-1) {
            match [
                map[i][j].kind,
                map[i-1][j].kind,
                map[i-1][j+1].kind,
                map[i][j+1].kind,
                map[i+1][j+1].kind,
                map[i+1][j].kind,
                map[i+1][j-1].kind,
                map[i][j-1].kind,
                map[i-1][j-1].kind,
            ] {
                [Forest, Sand, Sand, Sand, Sand, Sand, _, _, _] |
                [Forest, _, _, Sand, Sand, Sand, Sand, Sand, _] |
                [Forest, Sand, _, _, _, Sand, Sand, Sand, Sand] |
                [Forest, Sand, Sand, Sand, _, _, _, Sand, Sand] => {
                    map[i][j].kind = Sand;
                    map[i][j].variant = 4;
                },

                [Sand, Sea, Sea, Sea, Sea, Sea, _, _, _] |
                [Sand, _, _, Sea, Sea, Sea, Sea, Sea, _] |
                [Sand, Sea, _, _, _, Sea, Sea, Sea, Sea] |
                [Sand, Sea, Sea, Sea, _, _, _, Sea, Sea] => {
                    map[i][j].kind = Sea;
                    map[i][j].variant = 158;
                },

                _ => ()
            }
        }
    }
}

fn fill_sprites_id(map : &mut Vec<Vec<Tile>>) {
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
            ], 
            map[i][j].variant)
        }
    }
}

fn hash_vec(seed : &Vec<usize>) -> u64 {
    let mut hasher = seahash::SeaHasher::new();
    for i in seed {
        hasher.write_usize(*i);
    }
    hasher.finish()
}
fn get_tile_type(height : f32, hash : u64) -> Tile {
    let var;
    let kind;
    if height > 0.6  && height <= 0.8 {
        kind = TileKind::Sand;
        var = hash * 7 % 4;
    }
    else if height > 0.8 {
        kind = TileKind::Forest;
        var = hash * 7 % 4;
    }
    else {
        kind = TileKind::Sea;
        var = 0;
    }
    Tile {
        variant : var as u32,
        kind, 
        sprite_id : 0
    }
}
