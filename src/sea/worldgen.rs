use crate::{loading::GameState, util::SeededHasher};

use super::{loader::BiomeConfig, map::TileKind::*, TILE_SIZE};
use super::{loader::SeaHandles, map::TileKind};
use bevy::{
    prelude::*,
    render::{camera::Camera, pipeline::PrimitiveTopology},
    sprite::TextureAtlas,
    utils::{HashMap, HashSet},
};
use noise::{Fbm, MultiFractal, NoiseFn, Seedable};
use rapier2d::{
    geometry::TriMesh,
    na::{Point, Point2, U2},
};
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::{
    cmp::{max, min},
    collections::VecDeque,
    hash::Hasher,
    ops::{Index, IndexMut},
};
//bisous <3

pub struct SeaWorldGenPlugin;
impl Plugin for SeaWorldGenPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<HashMap<IslandPos, Island>>()
            .init_resource::<HashSet<IslandPos>>()
            .init_resource::<Vec<(Island, Option<TriMesh>, TriMesh)>>()
            .on_state_update(GameState::STAGE, GameState::Sea, worldgen_system.system());
    }
}

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

//Select a biome using the hasher (pre-loaded with the chunk coordinates) and the list of biomes
pub fn select_biome(
    mut hasher: SeaHasher,
    BiomeConfig(config): BiomeConfig,
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

const VIEW_DISTANCE: i32 = 50;
#[derive(Default, Debug)]
struct Ribbon {
    neg: Vec<(i32, i32)>,
    pos: Vec<(i32, i32)>,
}
impl Index<i32> for Ribbon {
    type Output = (i32, i32);

    fn index(&self, index: i32) -> &Self::Output {
        if index < 0 {
            &self.neg[(-(index + 1)) as usize]
        } else {
            &self.pos[index as usize]
        }
    }
}
impl IndexMut<i32> for Ribbon {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        if index < 0 {
            &mut self.neg[(-(index + 1)) as usize]
        } else {
            &mut self.pos[index as usize]
        }
    }
}
impl Ribbon {
    // fn iter<'a>(&'a self) -> impl Iterator + 'a {
    //     self.neg.iter().rev().chain(self.pos.iter())
    // }
    // fn iter_mut<'a>(&'a mut self) -> impl Iterator + 'a {
    //     self.neg.iter_mut().rev().chain(self.pos.iter_mut())
    // }
    fn iter_mut_enumerate<'a>(&'a mut self) -> impl Iterator<Item = (i32, &mut (i32, i32))> + 'a {
        let low_bound = -(self.neg.len() as i32);
        let high_bound = self.pos.len() as i32;
        (low_bound..high_bound)
            .into_iter()
            .zip(self.neg.iter_mut().rev().chain(self.pos.iter_mut()))
    }
    fn len_pos(&self) -> i32 {
        self.pos.len() as i32
    }
    fn len_neg(&self) -> i32 {
        -(self.neg.len() as i32)
    }
    fn expand_pos(&mut self, y_value: i32) {
        self.pos.push((y_value, y_value + 1))
    }
    fn expand_neg(&mut self, y_value: i32) {
        self.neg.push((y_value, y_value + 1))
    }
}
struct GenRessources {
    pub noise: Fbm,
    pub hasher: SeaHasher,
    pub biome: Biome,
}
impl FromResources for GenRessources {
    fn from_resources(resources: &Resources) -> Self {
        let config = resources.get::<BiomeConfig>().unwrap();
        let mut hasher = resources.get::<SeededHasher>().unwrap().get_hasher();
        let noise = noise::Fbm::new();
        hasher.write(&*"sea_island_gen".to_string().into_bytes());
        let hasher = hasher; //prevent mutability
        let (_handle, biome) = select_biome(hasher.clone(), (&*config).clone());
        let noise = noise
            .set_seed(hasher.finish() as u32)
            .set_octaves(biome.generation_parameters.octaves)
            .set_lacunarity(biome.generation_parameters.lacunarity)
            .set_persistence(biome.generation_parameters.persistence)
            .set_frequency(biome.generation_parameters.frequency);
        Self {
            noise,
            hasher,
            biome,
        }
    }
}

fn get_height(noise: &Fbm, (x, y): (i32, i32)) -> f64 {
    noise.get([x as f64, y as f64])
}
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct IslandPos {
    pub x: (i32, i32),
    pub y: (i32, i32),
}

fn worldgen_system(
    camera_query: Query<(&Camera, &Transform)>,
    mut island_map: ResMut<HashSet<IslandPos>>,
    mut islands_to_add: ResMut<Vec<(Island, Option<TriMesh>, TriMesh)>>,
    mut ribbon: Local<Ribbon>,
    gen_ressources: Local<GenRessources>,
    mut meshes: ResMut<Assets<Mesh>>,
    atlases: Res<Assets<TextureAtlas>>,
    handles: Res<SeaHandles>,
) {
    let tile_size = Vec2::new(16., 16.);
    let mut camera_pos = (0, 0);
    for (_, transform) in camera_query.iter() {
        camera_pos = (
            (transform.translation.x / TILE_SIZE as f32) as i32,
            (transform.translation.y / TILE_SIZE as f32) as i32,
        )
    }

    if ribbon.len_pos() - camera_pos.1 <= VIEW_DISTANCE {
        ribbon.expand_pos(camera_pos.1)
    }
    if camera_pos.1 - ribbon.len_neg() <= VIEW_DISTANCE {
        ribbon.expand_neg(camera_pos.1)
    }
    let mut island_tiles = VecDeque::new();
    for (i, (min, max)) in ribbon.iter_mut_enumerate() {
        //if to far off horizontally, skips.
        if (i - camera_pos.1).abs() > VIEW_DISTANCE {
            continue;
        }
        //if the player did a large circle for example, the ribbon can be very far.
        //this discards the far segment and makes a new one closer.
        //it just discards a bit of cache, but the generated islands are kept, no no big deal.
        // if camera_pos.0 - *min <= -VIEW_DISTANCE || *max - camera_pos.0 <= -VIEW_DISTANCE {
        //     *min = camera_pos.0;
        //     *max = camera_pos.0 + 1;
        // }
        //finally, enlarges the ribbon when necessary
        if camera_pos.0 - *min <= VIEW_DISTANCE {
            let height = get_height(&gen_ressources.noise, (i, *min));
            if height >= gen_ressources.biome.generation_parameters.sea_level as f64 {
                island_tiles.push_back((i, *min))
            }
            *min -= 1;
        }
        if *max - camera_pos.0 <= VIEW_DISTANCE {
            let height = get_height(&gen_ressources.noise, (i, *max));
            if height >= gen_ressources.biome.generation_parameters.sea_level as f64 {
                island_tiles.push_back((i, *max))
            }
            *max += 1;
        }
    }
    let mut processed_tiles = HashSet::default();
    while let Some((x, y)) = island_tiles.pop_front() {
        if !processed_tiles.insert((x, y)) {
            continue;
        }
        if let Some((island, rigid_trimesh, friction_trimesh)) = generate_island(
            (x, y),
            &gen_ressources,
            &mut island_tiles,
            &mut processed_tiles,
            &mut island_map,
            &mut ribbon,
            atlases.get(handles.islands_sheet.clone()).unwrap(),
            &mut *meshes,
            tile_size,
        ) {
            islands_to_add.push((island, rigid_trimesh, friction_trimesh))
        };
    }
}
#[derive(Default, Clone, Copy)]
pub struct Tile {
    pub kind: TileKind,
    pub variant: u32,
    pub sprite_id: Option<u32>,
}

impl Tile {
    fn new(mut hasher: SeaHasher, height: f64, position: (i32, i32), biome: &Biome) -> Self {
        hasher.write_i32(position.0);
        hasher.write_i32(position.1);
        Self {
            kind: if height < biome.generation_parameters.sea_level as f64 {
                TileKind::Sea(false)
            } else if height < biome.generation_parameters.high_level as f64 {
                TileKind::Sand(false)
            } else {
                TileKind::Forest
            },
            variant: (hasher.finish() * 7 % 8) as u32,
            sprite_id: None,
        }
    }
}
pub struct Island {
    pub tiles: Vec<Vec<Tile>>,
    pub mesh: Handle<Mesh>,
    pub up: i32,
    pub down: i32,
    pub left: i32,
    pub right: i32,
    pub entity: Option<Entity>,
}
fn get_surroundings(tiles_vec: &Vec<Vec<Tile>>, i: usize, j: usize) -> [TileKind; 9] {
    [
        tiles_vec
            .get(i)
            .map(|v| v.get(j))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
        tiles_vec
            .get(i + 1)
            .map(|v| v.get(j))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
        tiles_vec
            .get(i + 1)
            .map(|v| v.get(j + 1))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
        tiles_vec
            .get(i)
            .map(|v| v.get(j + 1))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
        tiles_vec
            .get(i - 1)
            .map(|v| v.get(j + 1))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
        tiles_vec
            .get(i - 1)
            .map(|v| v.get(j))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
        tiles_vec
            .get(i - 1)
            .map(|v| v.get(j - 1))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
        tiles_vec
            .get(i)
            .map(|v| v.get(j - 1))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
        tiles_vec
            .get(i + 1)
            .map(|v| v.get(j - 1))
            .flatten()
            .copied()
            .unwrap_or_default()
            .kind,
    ]
}

fn add_tile_to_mesh(
    tile_size: Vec2,
    id: u32,
    x: usize,
    y: usize,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u16>,
    atlas: &TextureAtlas,
    i: &mut usize,
) {
    let tile_pos = {
        let start = Vec2::new(x as f32 * tile_size.x, y as f32 * tile_size.y);

        let end = Vec2::new((x + 1) as f32 * tile_size.x, (y + 1) as f32 * tile_size.y);
        Vec4::new(end.x, end.y, start.x, start.y)
    };
    let tile_uv = {
        let rect = atlas.textures[id as usize];
        Vec4::new(
            rect.max.x / atlas.size.x,
            rect.min.y / atlas.size.y,
            rect.min.x / atlas.size.x,
            rect.max.y / atlas.size.y,
        )
    };
    // X, Y
    positions.push([tile_pos.x, tile_pos.y, 0.0]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([tile_uv.x, tile_uv.y]);

    // X, Y + 1
    positions.push([tile_pos.z, tile_pos.y, 0.0]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([tile_uv.z, tile_uv.y]);

    // X + 1, Y + 1
    positions.push([tile_pos.z, tile_pos.w, 0.0]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([tile_uv.z, tile_uv.w]);

    // X + 1, Y
    positions.push([tile_pos.x, tile_pos.w, 0.0]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([tile_uv.x, tile_uv.w]);
    let j = *i as u16;
    let mut new_indices = vec![j + 0, j + 2, j + 1, j + 0, j + 3, j + 2];
    indices.append(&mut new_indices);
    *i += 4;
}

fn add_tile_to_trimesh(
    tile_size: Vec2,
    x: usize,
    y: usize,
    positions: &mut Vec<Point<f32, U2>>,
    indices: &mut Vec<[u32; 3]>,
    i: &mut usize,
) {
    let tile_pos = {
        let start = Vec2::new(x as f32 * tile_size.x, y as f32 * tile_size.y);

        let end = Vec2::new((x + 1) as f32 * tile_size.x, (y + 1) as f32 * tile_size.y);
        Vec4::new(end.x, end.y, start.x, start.y)
    };
    // X, Y
    positions.push(Point2::new(tile_pos.x, tile_pos.y));

    // X, Y + 1
    positions.push(Point2::new(tile_pos.z, tile_pos.y));

    // X + 1, Y + 1
    positions.push(Point2::new(tile_pos.z, tile_pos.w));

    // X + 1, Y
    positions.push(Point2::new(tile_pos.x, tile_pos.w));
    let j = *i as u32;
    indices.push([j + 0, j + 2, j + 1]);
    indices.push([j + 0, j + 3, j + 2]);
    *i += 4;
}
fn generate_island(
    tile: (i32, i32),
    gen_ressources: &GenRessources,
    to_process: &mut VecDeque<(i32, i32)>,
    processed: &mut HashSet<(i32, i32)>,
    generated_islands: &mut HashSet<IslandPos>,
    ribbon: &mut Ribbon,
    atlas: &TextureAtlas,
    meshes: &mut Assets<Mesh>,
    tile_size: Vec2,
) -> Option<(Island, Option<TriMesh>, TriMesh)> {
    let mut up = tile.1;
    let mut down = tile.1;
    let mut right = tile.0;
    let mut left = tile.0;
    let mut island_queue = VecDeque::new();
    let mut tiles = HashMap::default();
    island_queue.push_back(tile);
    while let Some((x, y)) = island_queue.pop_front() {
        up = max(up, y); //TODO: check orientation
        down = min(down, y);
        right = max(right, x);
        left = min(left, x);
        for (nx, ny) in [
            (x + 1, y),
            (x - 1, y),
            (x, y + 1),
            (x, y - 1),
            (x - 1, y + 1),
            (x + 1, y + 1),
            (x + 1, y - 1),
            (x - 1, y - 1),
        ]
        .iter()
        {
            let (nx, ny) = (*nx, *ny);
            //skips the already processed tiles
            if tiles.contains_key(&(nx, ny)) {
                continue;
            }
            //updates the ribbon
            processed.insert((nx, ny));
            if nx >= ribbon.len_pos() {
                ribbon.expand_pos(ny);
                ribbon[nx] = (ny - 1, ny + 1) //should work because nx can only be one more than the max.
            }
            if nx <= ribbon.len_neg() {
                ribbon.expand_neg(ny);
                ribbon[nx] = (ny - 1, ny + 1) //shoukd work because nx can only be one more than the max.
            }
            let (min, max) = &mut ribbon[nx];
            //if there is a gap that is too large, move the ribbon
            // if *min - ny <= 2 * VIEW_DISTANCE || ny - *max <= -2 * VIEW_DISTANCE {
            //     *min = ny - 1;
            //     *max = ny + 1;
            // }
            //if there is a gap, add all tiles in between to be processed.
            if ny <= *min {
                //add all the tiles in between to be processed
                for y in ny + 1..*min + 1 {
                    let height = get_height(&gen_ressources.noise, (nx, y));
                    if height >= gen_ressources.biome.generation_parameters.sea_level as f64 {
                        to_process.push_back((nx, y));
                    }
                }
                *min = ny - 1;
            }
            if ny >= *max {
                //add all the tiles in between to be processed
                for y in *max..ny {
                    let height = get_height(&gen_ressources.noise, (nx, y));
                    if height >= gen_ressources.biome.generation_parameters.sea_level as f64 {
                        to_process.push_back((nx, y));
                    }
                }
                *max = ny + 1;
            }
            //if the tile is sea, skips it
            let height = get_height(&gen_ressources.noise, (nx, ny));
            if height < gen_ressources.biome.generation_parameters.sea_level as f64 {
                continue;
            }
            let tile = Tile::new(
                gen_ressources.hasher.clone(),
                height,
                (nx, ny),
                &gen_ressources.biome,
            );
            tiles.insert((nx, ny), tile);
            island_queue.push_back((nx, ny))
        }
    }
    let size_y = up - down + 1;
    let size_x = right - left + 1;
    if generated_islands.contains(&IslandPos {
        x: (left, right),
        y: (down, up),
    }) {
        return None;
    }
    generated_islands.insert(IslandPos {
        x: (left, right),
        y: (down, up),
    });
    let mut tiles_vec = vec![vec![Tile::default(); size_y as usize]; size_x as usize];
    for ((x, y), t) in tiles.into_iter() {
        tiles_vec[(x - left) as usize][(y - down) as usize] = t;
    }
    //do a first pass where some tiles are deleted to avoid causing problems
    for i in 0..size_x as usize {
        for j in 0..size_y as usize {
            let surroundings = get_surroundings(&tiles_vec, i, j);
            match surroundings {
                [Forest, Sand(_), Sand(_), Sand(_), Sand(_), Sand(_), _, _, _]
                | [Forest, _, _, Sand(_), Sand(_), Sand(_), Sand(_), Sand(_), _]
                | [Forest, Sand(_), _, _, _, Sand(_), Sand(_), Sand(_), Sand(_)]
                | [Forest, Sand(_), Sand(_), Sand(_), _, _, _, Sand(_), Sand(_)] => {
                    tiles_vec[i][j].kind = Sand(true);
                }

                [Sand(_), Sea(_), Sea(_), Sea(_), Sea(_), Sea(_), _, _, _]
                | [Sand(_), _, _, Sea(_), Sea(_), Sea(_), Sea(_), Sea(_), _]
                | [Sand(_), Sea(_), _, _, _, Sea(_), Sea(_), Sea(_), Sea(_)]
                | [Sand(_), Sea(_), Sea(_), Sea(_), _, _, _, Sea(_), Sea(_)] => {
                    tiles_vec[i][j].kind = Sea(true);
                }
                _ => (),
            }
            let sprite_id = get_sprite_id(surroundings, tiles_vec[i][j].variant);
            tiles_vec[i][j].sprite_id = Some(sprite_id);
        }
    }
    let mut rigid_positions = Vec::new(); //everything that must be constructed
    let mut rigid_indices = Vec::new();
    let mut rigid_i = 0;
    let mut friction_positions = Vec::new();
    let mut friction_indices = Vec::new();
    let mut friction_i = 0;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut i = 0;
    //then complete the sprite ids
    for x in 0..size_x as usize {
        for y in 0..size_y as usize {
            let surroundings = get_surroundings(&tiles_vec, x, y);
            let tile = &mut tiles_vec[x][y];
            let sprite_id = get_sprite_id(surroundings, tile.variant);
            tile.sprite_id = Some(sprite_id);
            match tile.kind {
                Sand(false) => add_tile_to_trimesh(
                    tile_size,
                    x,
                    y,
                    &mut friction_positions,
                    &mut friction_indices,
                    &mut friction_i,
                ),
                Forest | Sand(true) | Sea(true) => {
                    add_tile_to_trimesh(
                        tile_size,
                        x,
                        y,
                        &mut rigid_positions,
                        &mut rigid_indices,
                        &mut rigid_i,
                    );
                }
                Sea(false) => {
                    continue;
                }
            }
            add_tile_to_mesh(
                tile_size,
                sprite_id,
                x,
                y,
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                atlas,
                &mut i,
            )
        }
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U16(indices)));
    let rigid_trimesh = if rigid_positions.is_empty() {
        None
    } else {
        Some(TriMesh::new(rigid_positions, rigid_indices))
    };
    let friction_trimesh = TriMesh::new(friction_positions, friction_indices);
    Some((
        Island {
            up,
            down,
            right,
            left,
            tiles: tiles_vec,
            mesh: meshes.add(mesh),
            entity: None,
        },
        rigid_trimesh,
        friction_trimesh,
    ))
}
