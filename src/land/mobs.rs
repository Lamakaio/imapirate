use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use bevy::prelude::*;
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};

use crate::{
    loading::GameState,
    sea::{map::TileKind, worldgen::Island, TILE_SIZE},
};

use super::{
    loader::MobsConfig,
    pathfinding::get_pathfinding,
    pathfinding::{Pathfinder, PathfindingType},
    player::Player,
    player::PlayerMovedEvent,
    LAND_SCALING,
};

pub struct LandMobsPlugin;
impl Plugin for LandMobsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.on_state_update(
            GameState::STAGE,
            GameState::Land,
            mob_movement_system.system(),
        )
        .init_resource::<MobsConfig>();
    }
}

#[derive(Default, Clone)]
pub struct Mob {
    pub kind: String,
    pub material: Handle<ColorMaterial>,
    pub speed: f32,
    pub pathfinder: Option<Pathfinder>,
}

impl Debug for Mob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Mob {{ kind : {:?}, speed :  {:?} }}",
            self.kind, self.speed
        )
    }
}
#[derive(Debug)]
struct MobSave(Mob, Transform);

#[derive(Serialize, Deserialize)]
pub struct SpawnConfig {
    pub biome: String,
    pub tile_kind: TileKind,
    pub rate: f32,
}

#[derive(Serialize, Deserialize)]
pub struct MobConfig {
    pub kind: String,
    pub sprite_path: String,
    pub speed: f32,
    pub size: f32,
    pub pathfinding: PathfindingType,
    pub spawn: Vec<SpawnConfig>,
}

fn mob_movement_system(
    mut event_reader: EventReader<PlayerMovedEvent>,
    time: Res<Time>,
    mut mob_query: Query<(&mut Mob, &mut Transform)>,
    player_query: Query<(&Player, &Transform)>,
) {
    let should_update = event_reader.iter().next().is_some();
    for (_, player_transform) in player_query.iter() {
        let player_translation = player_transform.translation.truncate();
        for (mut mob, mut mob_transform) in mob_query.iter_mut() {
            let mob_translation = mob_transform.translation.truncate();
            let speed = mob.speed;
            if let Some(pathfinder) = mob.pathfinder.as_mut() {
                if should_update {
                    if pathfinder
                        .find_path(mob_translation, player_translation)
                        .is_err()
                    {
                        continue;
                    }
                }
                if let Ok(next_pos) = pathfinder.step(speed, time.delta_seconds()) {
                    mob_transform.translation.x = next_pos.x;
                    mob_transform.translation.y = next_pos.y;
                }
            }
        }
    }
}

//should use spawn tables at some point. Json config should be used for lots of things actually
pub fn generate_mobs(mobs_config: &MobsConfig, island: &mut Island, mut hasher: SeaHasher) {
    "generate_mobs".to_string().hash(&mut hasher); //to shuffle things a bit between different rng gen

    //iterate over the tiles and their coordinates
    let mut island_hasher = hasher.clone();
    island_hasher.write_i32(island.min_x);
    island_hasher.write_i32(island.max_x);
    island_hasher.write_i32(island.min_y);
    island_hasher.write_i32(island.max_y);
    for (coord, tile) in island
        .tiles
        .iter()
        .enumerate()
        .map(|(i, v)| v.iter().enumerate().map(move |(j, t)| ((i, j), t)))
        .flatten()
    {
        const MAX_SPAWN_RATE: f32 = 10000.;
        let mut tile_hasher = island_hasher.clone();
        tile_hasher.write_usize(coord.0);
        tile_hasher.write_usize(coord.1);
        let hash = tile_hasher.finish() % MAX_SPAWN_RATE as u64;
        let tile_kind: TileKind = tile.kind.clone().into();
        //TODO : handle different biomes
        let mut current_number = 0;
        for (material, mob_config) in mobs_config.0.iter() {
            for spawn_config in mob_config.spawn.iter() {
                if tile_kind == spawn_config.tile_kind {
                    let number = (spawn_config.rate * MAX_SPAWN_RATE) as u64;
                    if hash >= current_number && hash < number + current_number {
                        let pathfinder = Some(get_pathfinding(
                            &island.tiles,
                            mob_config.pathfinding.clone(),
                        ));
                        island.mobs.push((
                            Mob {
                                kind: mob_config.kind.clone(),
                                speed: mob_config.speed,
                                material: material.clone(),
                                pathfinder,
                            },
                            Transform {
                                translation: Vec3::new(
                                    LAND_SCALING * TILE_SIZE as f32 * coord.0 as f32,
                                    LAND_SCALING * TILE_SIZE as f32 * coord.1 as f32,
                                    100.,
                                ),
                                scale: mob_config.size * Vec3::one(),
                                ..Default::default()
                            },
                        ))
                    }
                    current_number += number;
                }
            }
        }
    }
}
