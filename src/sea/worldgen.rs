use super::map::{Tile, TileKind};
use super::{map::TileKind::*, CHUNK_SIZE};
use crate::tilemap::Tile as MapTile;
use fuss::Simplex;
use std::hash::Hasher;

pub const NORM: f32 = 50.;
//bisous <3
const SEA: i32 = -1;

const FOREST_SEA_NW: i32 = 0;
const FOREST_SEA_NE: i32 = 4;
const FOREST_SEA_SE: i32 = 8;
const FOREST_SEA_SW: i32 = 12;

const FOREST_SEA_N: i32 = 16;
const FOREST_SEA_E: i32 = 20;
const FOREST_SEA_S: i32 = 24;
const FOREST_SEA_W: i32 = 28;

const FOREST_SEA_INNER_NW: i32 = 32;
const FOREST_SEA_INNER_NE: i32 = 36;
const FOREST_SEA_INNER_SE: i32 = 40;
const FOREST_SEA_INNER_SW: i32 = 44;

const FOREST_SEA_NESW: i32 = 48;
const FOREST_SEA_NWSE: i32 = 52;

const FOREST_SAND_NW: i32 = 56;
const FOREST_SAND_NE: i32 = 60;
const FOREST_SAND_SE: i32 = 64;
const FOREST_SAND_SW: i32 = 68;

const FOREST_SAND_N: i32 = 72;
const FOREST_SAND_E: i32 = 76;
const FOREST_SAND_S: i32 = 80;
const FOREST_SAND_W: i32 = 84;

const FOREST_SAND_INNER_NW: i32 = 88;
const FOREST_SAND_INNER_NE: i32 = 92;
const FOREST_SAND_INNER_SE: i32 = 96;
const FOREST_SAND_INNER_SW: i32 = 100;

const FOREST_SAND_NESW: i32 = 104;
const FOREST_SAND_NWSE: i32 = 108;

const FOREST: i32 = 112;

const SAND_ROCK: i32 = 120;

const SAND_SEA_NW: i32 = 124;
const SAND_SEA_NE: i32 = 128;
const SAND_SEA_SE: i32 = 132;
const SAND_SEA_SW: i32 = 136;

const SAND_SEA_N: i32 = 140;
const SAND_SEA_E: i32 = 144;
const SAND_SEA_S: i32 = 148;
const SAND_SEA_W: i32 = 152;

const SAND_SEA_INNER_NW: i32 = 156;
const SAND_SEA_INNER_NE: i32 = 160;
const SAND_SEA_INNER_SE: i32 = 164;
const SAND_SEA_INNER_SW: i32 = 168;

const SAND_SEA_NESW: i32 = 172;
const SAND_SEA_NWSE: i32 = 176;

const SAND: i32 = 180;

const SEA_ROCK: i32 = 184;

fn get_sprite_id(surroundings: [TileKind; 9], variant: u32) -> u32 {
    let half_var = variant / 2;
    half_var
        + (1 + match surroundings {
            [Sea(true), _, _, _, _, _, _, _, _] => SEA_ROCK,
            [Sand(true), _, _, _, _, _, _, _, _] => SAND_ROCK,

            //double corners
            [Forest, Forest, _, Forest, Sea(_), Forest, _, Forest, Sea(_)] => FOREST_SEA_NESW,
            [Forest, Forest, Sea(_), Forest, _, Forest, Sea(_), Forest, _] => FOREST_SEA_NWSE,

            [Forest, Forest, _, Forest, Sand(_), Forest, _, Forest, Sand(_)] => FOREST_SAND_NESW,
            [Forest, Forest, Sand(_), Forest, _, Forest, Sand(_), Forest, _] => FOREST_SAND_NWSE,

            [Sand(_), Sand(_), _, Sand(_), Sea(_), Sand(_), _, Sand(_), Sea(_)] => SAND_SEA_NESW,
            [Sand(_), Sand(_), Sea(_), Sand(_), _, Sand(_), Sea(_), Sand(_), _] => SAND_SEA_NWSE,
            //outer corners
            [Sea(_), _, _, _, _, _, _, _, _] => SEA - half_var as i32,
            [Forest, Sea(_), _, Forest, _, Forest, _, Sea(_), _]
            | [Forest, Forest, Sea(_), Forest, _, Forest, _, Sea(_), Sea(_)]
            | [Forest, Sea(_), _, Forest, _, Forest, Sea(_), Forest, Sea(_)] => FOREST_SEA_NW, //NW
            [Forest, Sea(_), _, Sea(_), _, Forest, _, Forest, _]
            | [Forest, Sea(_), Sea(_), Forest, Sea(_), Forest, _, Forest, _]
            | [Forest, Forest, Sea(_), Sea(_), _, Forest, _, Forest, Sea(_)] => FOREST_SEA_NE, //NE
            [Forest, Forest, _, Sea(_), _, Sea(_), _, Forest, _]
            | [Forest, Forest, _, Sea(_), Sea(_), Forest, Sea(_), Forest, _]
            | [Forest, Forest, Sea(_), Forest, Sea(_), Sea(_), _, Forest, _] => FOREST_SEA_SE, //SE
            [Forest, Forest, _, Forest, _, Sea(_), _, Sea(_), _]
            | [Forest, Forest, _, Forest, _, Sea(_), Sea(_), Forest, Sea(_)]
            | [Forest, Forest, _, Forest, Sea(_), Forest, Sea(_), Sea(_), _] => FOREST_SEA_SW, //SW

            [Forest, Sand(_), _, Forest, _, Forest, _, Sand(_), _]
            | [Forest, Forest, Sand(_), Forest, _, Forest, _, Sand(_), Sand(_)]
            | [Forest, Sand(_), _, Forest, _, Forest, Sand(_), Forest, Sand(_)] => FOREST_SAND_NW, //NW
            [Forest, Sand(_), _, Sand(_), _, Forest, _, Forest, _]
            | [Forest, Sand(_), Sand(_), Forest, Sand(_), Forest, _, Forest, _]
            | [Forest, Forest, Sand(_), Sand(_), _, Forest, _, Forest, Sand(_)] => FOREST_SAND_NE, //NE
            [Forest, Forest, _, Sand(_), _, Sand(_), _, Forest, _]
            | [Forest, Forest, _, Sand(_), Sand(_), Forest, Sand(_), Forest, _]
            | [Forest, Forest, Sand(_), Forest, Sand(_), Sand(_), _, Forest, _] => FOREST_SAND_SE, //SE
            [Forest, Forest, _, Forest, _, Sand(_), _, Sand(_), _]
            | [Forest, Forest, _, Forest, _, Sand(_), Sand(_), Forest, Sand(_)]
            | [Forest, Forest, _, Forest, Sand(_), Forest, Sand(_), Sand(_), _] => FOREST_SAND_SW, //SW

            [Sand(_), Sea(_), _, Sand(_), _, Sand(_), _, Sea(_), _]
            | [Sand(_), Sand(_), Sea(_), Sand(_), _, Sand(_), _, Sea(_), Sea(_)]
            | [Sand(_), Sea(_), _, Sand(_), _, Sand(_), Sea(_), Sand(_), Sea(_)] => SAND_SEA_NW, //NW
            [Sand(_), Sea(_), _, Sea(_), _, Sand(_), _, Sand(_), _]
            | [Sand(_), Sea(_), Sea(_), Sand(_), Sea(_), Sand(_), _, Sand(_), _]
            | [Sand(_), Sand(_), Sea(_), Sea(_), _, Sand(_), _, Sand(_), Sea(_)] => SAND_SEA_NE, //NE
            [Sand(_), Sand(_), _, Sea(_), _, Sea(_), _, Sand(_), _]
            | [Sand(_), Sand(_), _, Sea(_), Sea(_), Sand(_), Sea(_), Sand(_), _]
            | [Sand(_), Sand(_), Sea(_), Sand(_), Sea(_), Sea(_), _, Sand(_), _] => SAND_SEA_SE, //SE
            [Sand(_), Sand(_), _, Sand(_), _, Sea(_), _, Sea(_), _]
            | [Sand(_), Sand(_), _, Sand(_), _, Sea(_), Sea(_), Sand(_), Sea(_)]
            | [Sand(_), Sand(_), _, Sand(_), Sea(_), Sand(_), Sea(_), Sea(_), _] => SAND_SEA_SW, //SW

            //sides
            [Forest, Sea(_), _, Forest, _, _, _, Forest, _]
            | [Forest, Forest, Sea(_), Forest, _, _, _, Forest, Sea(_)] => FOREST_SEA_N, //N
            [Forest, Forest, _, Sea(_), _, Forest, _, _, _]
            | [Forest, Forest, Sea(_), Forest, Sea(_), Forest, _, _, _] => FOREST_SEA_E, //E
            [Forest, _, _, Forest, _, Sea(_), _, Forest, _]
            | [Forest, _, _, Forest, Sea(_), Forest, Sea(_), Forest, _] => FOREST_SEA_S, //S
            [Forest, Forest, _, _, _, Forest, _, Sea(_), _]
            | [Forest, Forest, _, _, _, Forest, Sea(_), Forest, Sea(_)] => FOREST_SEA_W, //W

            [Forest, Sand(_), _, Forest, _, _, _, Forest, _]
            | [Forest, Forest, Sand(_), Forest, _, _, _, Forest, Sand(_)] => FOREST_SAND_N, //N
            [Forest, Forest, _, Sand(_), _, Forest, _, _, _]
            | [Forest, Forest, Sand(_), Forest, Sand(_), Forest, _, _, _] => FOREST_SAND_E, //E
            [Forest, _, _, Forest, _, Sand(_), _, Forest, _]
            | [Forest, _, _, Forest, Sand(_), Forest, Sand(_), Forest, _] => FOREST_SAND_S, //S
            [Forest, Forest, _, _, _, Forest, _, Sand(_), _]
            | [Forest, Forest, _, _, _, Forest, Sand(_), Forest, Sand(_)] => FOREST_SAND_W, //W

            [Sand(_), Sea(_), _, Sand(_), _, _, _, Sand(_), _]
            | [Sand(_), Sand(_), Sea(_), Sand(_), _, _, _, Sand(_), Sea(_)] => SAND_SEA_N, //N
            [Sand(_), Sand(_), _, Sea(_), _, Sand(_), _, _, _]
            | [Sand(_), Sand(_), Sea(_), Sand(_), Sea(_), Sand(_), _, _, _] => SAND_SEA_E, //E
            [Sand(_), _, _, Sand(_), _, Sea(_), _, Sand(_), _]
            | [Sand(_), _, _, Sand(_), Sea(_), Sand(_), Sea(_), Sand(_), _] => SAND_SEA_S, //S
            [Sand(_), Sand(_), _, _, _, Sand(_), _, Sea(_), _]
            | [Sand(_), Sand(_), _, _, _, Sand(_), Sea(_), Sand(_), Sea(_)] => SAND_SEA_W, //W

            //inner corners
            [Forest, Forest, _, Forest, Sea(_), Forest, _, Forest, _] => FOREST_SEA_INNER_NW,
            [Forest, Forest, _, Forest, _, Forest, Sea(_), Forest, _] => FOREST_SEA_INNER_NE,
            [Forest, Forest, _, Forest, _, Forest, _, Forest, Sea(_)] => FOREST_SEA_INNER_SE,
            [Forest, Forest, Sea(_), Forest, _, Forest, _, Forest, _] => FOREST_SEA_INNER_SW,

            [Forest, Forest, _, Forest, Sand(_), Forest, _, Forest, _] => FOREST_SAND_INNER_NW,
            [Forest, Forest, _, Forest, _, Forest, Sand(_), Forest, _] => FOREST_SAND_INNER_NE,
            [Forest, Forest, _, Forest, _, Forest, _, Forest, Sand(_)] => FOREST_SAND_INNER_SE,
            [Forest, Forest, Sand(_), Forest, _, Forest, _, Forest, _] => FOREST_SAND_INNER_SW,

            [Sand(_), _, _, Sand(_), Sea(_), Sand(_), _, _, _] => SAND_SEA_INNER_NW,
            [Sand(_), _, _, _, _, Sand(_), Sea(_), Sand(_), _] => SAND_SEA_INNER_NE,
            [Sand(_), Sand(_), _, _, _, _, _, Sand(_), Sea(_)] => SAND_SEA_INNER_SE,
            [Sand(_), Sand(_), Sea(_), Sand(_), _, _, _, _, _] => SAND_SEA_INNER_SW,

            //triple
            [_, Sand(_), Sand(_), Sand(_), Sand(_), Sand(_), _, _, _]
            | [_, _, _, Sand(_), Sand(_), Sand(_), Sand(_), Sand(_), _]
            | [_, Sand(_), _, _, _, Sand(_), Sand(_), Sand(_), Sand(_)]
            | [_, Sand(_), Sand(_), Sand(_), _, _, _, Sand(_), Sand(_)] => SAND,

            [_, Sea(_), Sea(_), Sea(_), Sea(_), Sea(_), _, _, _]
            | [_, _, _, Sea(_), Sea(_), Sea(_), Sea(_), Sea(_), _]
            | [_, Sea(_), _, _, _, Sea(_), Sea(_), Sea(_), Sea(_)]
            | [_, Sea(_), Sea(_), Sea(_), _, _, _, Sea(_), Sea(_)] => SEA - half_var as i32,

            //inside
            [Forest, Forest, _, Forest, _, Forest, _, Forest, _] => {
                FOREST - half_var as i32 + variant as i32
            }
            [Sand(_), x, _, y, _, z, _, w, _]
                if x != Sea(false)
                    && y != Sea(false)
                    && z != Sea(false)
                    && w != Sea(false)
                    && x != Sea(true)
                    && y != Sea(true)
                    && z != Sea(true)
                    && w != Sea(true) =>
            {
                SAND
            }

            _ => SEA - half_var as i32,
        }) as u32
}

fn mexp(x: i32) -> f32 {
    (2. * (x as f32 - CHUNK_SIZE as f32 / 2.) / CHUNK_SIZE as f32)
        .abs()
        .powi(20)
        .exp()
}
fn mountain(pos_x: i32, pos_y: i32) -> f32 {
    2. - mexp(pos_x) - mexp(pos_y)
}
pub fn generate_chunk(pos_x: i32, pos_y: i32, world_seed: usize) -> Vec<Vec<MapTile>> {
    let sn = Simplex::from_seed(vec![pos_x as usize, pos_y as usize, world_seed]);
    let mut seed = sn.seed.clone();
    let mut map: Vec<Vec<Tile>> = Vec::new();
    seed.push(0);
    let last = seed.len() - 1;
    for i in 0..CHUNK_SIZE {
        map.push(Vec::new());
        for j in 0..CHUNK_SIZE {
            let height = sn.noise_2d(
                (CHUNK_SIZE * pos_x + i) as f32 / NORM,
                (CHUNK_SIZE * pos_y + j) as f32 / NORM,
            ) + mountain(i, j);
            seed[last] = (height * 100000.) as usize;
            let hash = hash_vec(&seed);
            const LIM: i32 = CHUNK_SIZE - 1;
            let offset = match (i, j) {
                (0, _) | (CHUNK_SIZE, _) | (_, 0) | (_, CHUNK_SIZE) => 0.4,
                (1, _) | (LIM, _) | (_, 1) | (_, LIM) => 0.2,
                _ => 0.,
            };
            map[i as usize].push(get_tile_type(height - offset, hash))
        }
    }
    uniformization_pass(&mut map);
    get_id_map(&map)
}

fn uniformization_pass(map: &mut Vec<Vec<Tile>>) {
    for i in 1..(CHUNK_SIZE as usize - 1) {
        for j in 1..(CHUNK_SIZE as usize - 1) {
            match [
                map[i][j].kind,
                map[i - 1][j].kind,
                map[i - 1][j + 1].kind,
                map[i][j + 1].kind,
                map[i + 1][j + 1].kind,
                map[i + 1][j].kind,
                map[i + 1][j - 1].kind,
                map[i][j - 1].kind,
                map[i - 1][j - 1].kind,
            ] {
                [Forest, Sand(_), Sand(_), Sand(_), Sand(_), Sand(_), _, _, _]
                | [Forest, _, _, Sand(_), Sand(_), Sand(_), Sand(_), Sand(_), _]
                | [Forest, Sand(_), _, _, _, Sand(_), Sand(_), Sand(_), Sand(_)]
                | [Forest, Sand(_), Sand(_), Sand(_), _, _, _, Sand(_), Sand(_)] => {
                    map[i][j].kind = Sand(true);
                }

                [Sand(_), Sea(_), Sea(_), Sea(_), Sea(_), Sea(_), _, _, _]
                | [Sand(_), _, _, Sea(_), Sea(_), Sea(_), Sea(_), Sea(_), _]
                | [Sand(_), Sea(_), _, _, _, Sea(_), Sea(_), Sea(_), Sea(_)]
                | [Sand(_), Sea(_), Sea(_), Sea(_), _, _, _, Sea(_), Sea(_)] => {
                    map[i][j].kind = Sea(true);
                }

                _ => (),
            }
        }
    }
}

fn get_id_map(map: &[Vec<Tile>]) -> Vec<Vec<MapTile>> {
    let mut layer = Vec::new();
    for i in 0..(CHUNK_SIZE as usize) {
        layer.push(Vec::new());
        for j in 0..(CHUNK_SIZE as usize) {
            if i == 0 || j == 0 || i as i32 == CHUNK_SIZE - 1 || j as i32 == CHUNK_SIZE - 1 {
                layer[i].push(MapTile::Static((1 + SEA) as u32));
            } else {
                layer[i].push(MapTile::Static(get_sprite_id(
                    [
                        map[i][j].kind,
                        map[i - 1][j].kind,
                        map[i - 1][j + 1].kind,
                        map[i][j + 1].kind,
                        map[i + 1][j + 1].kind,
                        map[i + 1][j].kind,
                        map[i + 1][j - 1].kind,
                        map[i][j - 1].kind,
                        map[i - 1][j - 1].kind,
                    ],
                    map[i][j].variant,
                )));
            }
        }
    }
    layer
}

fn hash_vec(seed: &[usize]) -> u64 {
    let mut hasher = seahash::SeaHasher::new();
    for i in seed {
        hasher.write_usize(*i);
    }
    hasher.finish()
}
fn get_tile_type(height: f32, hash: u64) -> Tile {
    let kind;
    if height > 0.6 && height <= 0.8 {
        kind = Sand(false);
    } else if height > 0.8 {
        kind = Forest;
    } else {
        kind = Sea(false);
    }
    Tile {
        variant: (hash * 7 % 8) as u32,
        kind,
        sprite_id: 0,
    }
}
