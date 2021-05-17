use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use bevy::prelude::*;
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};

use crate::{
    loading::GameState,
    sea::{
        map::{Islands, TileKind},
        player::PlayerPositionUpdate,
        worldgen::Island,
        TILE_SIZE,
    },
};

use super::{
    collision::{LandCollisionTree, LandId, LandValue},
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
        app.on_state_update(GameState::STAGE, GameState::Land, mob_movement.system())
            .on_state_enter(GameState::STAGE, GameState::Land, load_mobs.system())
            .init_resource::<MobsConfig>()
            .on_state_exit(GameState::STAGE, GameState::Land, unload_mobs.system());
    }
}

#[derive(Default, Clone)]
pub struct Mob {
    pub kind: String,
    pub material: Handle<ColorMaterial>,
    pub speed: f32,
    pub pathfinder: Option<Pathfinder>,
    pub collider: ColliderType,
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ColliderType {
    Ball(f32),
    None,
}
impl Default for ColliderType {
    fn default() -> Self {
        ColliderType::None
    }
}
impl ColliderType {
    fn bounding_box(&self) -> Vec2 {
        match self {
            ColliderType::None => Vec2::new(0., 0.),
            ColliderType::Ball(diam) => Vec2::new(*diam, *diam),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MobConfig {
    pub kind: String,
    pub sprite_path: String,
    pub speed: f32,
    pub scale: f32,
    pub collider: ColliderType,
    pub pathfinding: PathfindingType,
    pub spawn: Vec<SpawnConfig>,
}

fn mob_movement(
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
                if should_update
                    && pathfinder
                        .find_path(mob_translation, player_translation)
                        .is_err()
                {
                    continue;
                }
                if let Ok(next_pos) = pathfinder.step(speed, time.delta_seconds()) {
                    mob_transform.translation.x = next_pos.x;
                    mob_transform.translation.y = next_pos.y;
                }
            }
        }
    }
}
fn unload_mobs(
    commands: &mut Commands,
    query: Query<(Entity, &Mob, &Transform)>,
    sea_player_pos: Res<PlayerPositionUpdate>,
    mut islands: ResMut<Islands>,
) {
    let island = &mut islands.0[sea_player_pos.island_id.unwrap() as usize];
    for (entity, mob, transform) in query.iter() {
        commands.despawn_recursive(entity);
        island.mobs.push((mob.clone(), *transform));
    }
}

fn load_mobs(
    commands: &mut Commands,
    sea_player_pos: Res<PlayerPositionUpdate>,
    mut islands: ResMut<Islands>,
    mut collisions: ResMut<LandCollisionTree>,
) {
    let island = &mut islands.0[sea_player_pos.island_id.unwrap() as usize];

    for (mob, transform) in island.mobs.drain(..) {
        let bounding_box = mob.collider.bounding_box();
        let entity = commands //mob
            .spawn(SpriteBundle {
                material: mob.material.clone(),
                transform,
                ..Default::default()
            })
            .with(mob)
            .current_entity()
            .unwrap();
        collisions.0.insert(LandValue {
            min_x: transform.translation.x,
            max_x: transform.translation.x + bounding_box.x,
            min_y: transform.translation.y,
            max_y: transform.translation.y + bounding_box.y,
            id: LandId::Mob(entity),
        })
    }
}

//should use spawn tables at some point. Json config should be used for lots of things actually
pub fn generate_mobs(mobs_config: &MobsConfig, island: &mut Island, mut hasher: SeaHasher) {
    "generate_mobs".to_string().hash(&mut hasher); //to shuffle things a bit between different rng gen
                                                   //iterate over the tiles and their coordinates
    let mut island_hasher = hasher;
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
        let mut tile_hasher = island_hasher;
        tile_hasher.write_usize(coord.0);
        tile_hasher.write_usize(coord.1);
        let hash = tile_hasher.finish() % MAX_SPAWN_RATE as u64;
        let tile_kind: TileKind = tile.kind;
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
                                collider: mob_config.collider.clone(),
                            },
                            Transform {
                                translation: Vec3::new(
                                    LAND_SCALING * TILE_SIZE as f32 * coord.0 as f32,
                                    LAND_SCALING * TILE_SIZE as f32 * coord.1 as f32,
                                    100.,
                                ),
                                scale: mob_config.scale * Vec3::one(),
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
