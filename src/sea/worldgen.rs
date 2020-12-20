use super::map::{Tile, TileKind};
use super::{map::TileKind::*, CHUNK_SIZE};
use crate::tilemap::Tile as MapTile;
use bevy::{prelude::Handle, sprite::TextureAtlas};
use noise::{MultiFractal, NoiseFn, Seedable};
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::{hash::Hasher, sync::Arc};
//bisous <3

//The sprite ids in the sheet for every tile
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

pub const fn tile_kind_from_sprite_id(id: i32) -> TileKind {
    let id = id - 1;
    match id {
        _ if id >= FOREST_SEA_NW && id < SAND_ROCK => TileKind::Forest,
        _ if id >= SAND_ROCK && id < SAND_SEA_NW => TileKind::Sand(true),
        _ if id >= SAND_SEA_NW && id < SEA_ROCK => TileKind::Sand(false),
        _ if id >= SEA_ROCK => TileKind::Sea(true),
        _ => TileKind::Sea(false),
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerationParameters {
    pub octaves: usize,
    pub lacunarity: f64,
    pub persistence: f64,
    pub frequency: f64,
    pub sea_level: f32,
    pub high_level: f32,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Biome {
    pub generation_parameters: GenerationParameters,
    pub name: String,
    pub sea_sheet: String,
    pub land_sheet: String,
    pub weight: u32,
}
//The world generator only decide which tile type must be used at each coordinate (among sand, sea and forest here)
//This ugly function use the surrounding tiles to determine which sprite must be displayed.
//It still doesn't cover all cases.
//I'm considering using a contraints solver instead.
fn get_sprite_id(surroundings: [TileKind; 9], variant: u32) -> u32 {
    let half_var = variant / 2;
    //Most tiles only have 4 variants, but some have 8. For those who have 8 it should use variant, for the other half_var
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

fn mexp(x: i32) -> f64 {
    (2. * (x as f64 - CHUNK_SIZE as f64 / 2.) / CHUNK_SIZE as f64)
        .abs()
        .powi(20)
        .exp()
}
//A function which is highest in the center, still very high until it is close to the chunk border, and the gets a lot smaller very vast.
//It is used to avoid generating island on chunk borders, as it makes solving the constraints very difficult.
//This solution is not ideal as it changes the generation.
//Ideally it would be better to have a bot of overlap between chunks.
fn mountain(pos_x: i32, pos_y: i32) -> f64 {
    2. - mexp(pos_x) - mexp(pos_y)
}

//Select a biome using the hasher (pre-loaded with the chunk coordinates) and the list of biomes
pub fn select_biome(
    mut hasher: SeaHasher,
    config: Arc<Vec<(Handle<TextureAtlas>, Biome)>>,
) -> (Handle<TextureAtlas>, Biome) {
    hasher.write_u64(0xB107E); //write a constant to change the number.
    let total = config.iter().fold(0, |i, (_h, b)| i + b.weight);
    let hash = hasher.finish() as u32 % total;
    let mut temp_sum = 0;
    for (h, b) in config.iter() {
        temp_sum += b.weight;
        if temp_sum >= hash {
            return (h.clone(), b.clone());
        }
    }
    return config[0].clone();
}

//Generation using the noise crate, and a hasher. No random is used as the map need to be seeded.
//SeaHasher is already seeded using the map seed.
//The hasher should only hash global, unchanging parameter (ie the tile and chunk position),
//so the generation order and the like can't change the map
pub fn generate_chunk(
    pos_x: i32,
    pos_y: i32,
    hasher: SeaHasher,
    config: Arc<Vec<(Handle<TextureAtlas>, Biome)>>,
) -> (Vec<Vec<MapTile>>, Handle<TextureAtlas>) {
    let mut hasher = hasher.clone();
    hasher.write_i32(pos_x);
    hasher.write_i32(pos_y);
    let (handle, biome) = select_biome(hasher.clone(), config);
    let noise = noise::Fbm::new();
    let noise = noise
        .set_seed(hasher.finish() as u32)
        .set_octaves(biome.generation_parameters.octaves)
        .set_lacunarity(biome.generation_parameters.lacunarity)
        .set_persistence(biome.generation_parameters.persistence)
        .set_frequency(biome.generation_parameters.frequency);
    let mut map: Vec<Vec<Tile>> = Vec::new();
    for i in 0..CHUNK_SIZE {
        map.push(Vec::new());
        for j in 0..CHUNK_SIZE {
            let height = noise.get([
                (CHUNK_SIZE * pos_x + i) as f64,
                (CHUNK_SIZE * pos_y + j) as f64,
            ]) + mountain(i, j);
            let mut hasher_tile = hasher.clone();
            hasher_tile.write_i32(i + j * CHUNK_SIZE);
            map[i as usize].push(get_tile_type(
                height as f32,
                biome.generation_parameters.sea_level,
                biome.generation_parameters.high_level,
                hasher_tile.finish(),
            ))
        }
    }
    uniformization_pass(&mut map);
    (get_id_map(&map), handle)
}

//Again, a bit ugly, but it transforms tiles that should be rocks (ie tile configuration too rare to have their own sprite)
//into rocks, which must behave just like sea or sand depending where they are.
//Once the generation is done in its own thread and is non-blocking, multiple passes should be done.
fn uniformization_pass(map: &mut Vec<Vec<Tile>>) {
    for i in 1..(CHUNK_SIZE as usize - 1) {
        for j in 1..(CHUNK_SIZE as usize - 1) {
            match [
                map[i][j].kind,
                map[i + 1][j].kind,
                map[i + 1][j + 1].kind,
                map[i][j + 1].kind,
                map[i - 1][j + 1].kind,
                map[i - 1][j].kind,
                map[i - 1][j - 1].kind,
                map[i][j - 1].kind,
                map[i + 1][j - 1].kind,
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
                        map[i + 1][j].kind,
                        map[i + 1][j + 1].kind,
                        map[i][j + 1].kind,
                        map[i - 1][j + 1].kind,
                        map[i - 1][j].kind,
                        map[i - 1][j - 1].kind,
                        map[i][j - 1].kind,
                        map[i + 1][j - 1].kind,
                    ],
                    map[i][j].variant,
                )));
            }
        }
    }
    layer
}

//Tiles have variants. This gets the variant.
fn get_tile_type(height: f32, sea_level: f32, high_level: f32, hash: u64) -> Tile {
    let kind;
    if height > sea_level && height <= high_level {
        kind = Sand(false);
    } else if height > high_level {
        kind = Forest;
    } else {
        kind = Sea(false);
    }
    Tile {
        variant: (hash * 7 % 8) as u32, // * 7 because the hash is probably not that good and it seemed too regular without it.
        kind,
        sprite_id: 0,
    }
}
