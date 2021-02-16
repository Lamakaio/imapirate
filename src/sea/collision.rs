use crate::loading::GameState;
use bevy::prelude::*;
use kdtree_collisions::KdValue;
use parry2d::{math::Isometry, na::Vector2};

use super::{
    map::Islands,
    player::{CollisionType, PlayerPositionUpdate},
    worldgen::IslandQueue,
    ISLAND_SCALING, TILE_SIZE,
};
#[derive(Debug, Default)]
pub struct SeaCollisionTree(pub kdtree_collisions::KdTree<IslandValue, 16>);
pub struct SeaCollisionPlugin;
impl Plugin for SeaCollisionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.on_state_update(GameState::STAGE, GameState::Sea, collision_system.system())
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                add_islands_system.system(),
            )
            .add_event::<IslandSpawnEvent>()
            .init_resource::<SeaCollisionTree>();
    }
}

#[derive(Debug, Default, Clone)]
pub struct IslandValue {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
    island_id: u32,
}
impl KdValue for IslandValue {
    type Position = i32;

    fn min_x(&self) -> Self::Position {
        self.min_x
    }

    fn min_y(&self) -> Self::Position {
        self.min_y
    }

    fn max_x(&self) -> Self::Position {
        self.max_x
    }

    fn max_y(&self) -> Self::Position {
        self.max_y
    }
}

pub struct IslandSpawnEvent(pub u32);
fn collision_system(
    mut spawn_events: ResMut<Events<IslandSpawnEvent>>,
    mut player_pos_update: ResMut<PlayerPositionUpdate>,
    islands: Res<Islands>,
    kdtree: Res<SeaCollisionTree>,
) {
    for island_to_spawn in kdtree.0.query_rect(
        player_pos_update.x - 100,
        player_pos_update.x + 100,
        player_pos_update.y - 50,
        player_pos_update.y + 50,
    ) {
        spawn_events.send(IslandSpawnEvent(island_to_spawn.island_id))
    }
    player_pos_update.collision_status = CollisionType::None;
    player_pos_update.island_id = None;
    for close_island in kdtree.0.query_rect(
        player_pos_update.x - 2,
        player_pos_update.x + 2,
        player_pos_update.y - 2,
        player_pos_update.y + 2,
    ) {
        let island = &islands.0[close_island.island_id as usize];
        let intersect_rigid = if let Some(rigid_mesh) = &island.rigid_trimesh {
            parry2d::query::intersection_test(
                &Isometry::new(
                    Vector2::new(
                        (island.min_x * TILE_SIZE) as f32 * ISLAND_SCALING,
                        (island.min_y * TILE_SIZE) as f32 * ISLAND_SCALING,
                    ),
                    0.,
                ),
                rigid_mesh,
                &Isometry::new(
                    Vector2::new(
                        player_pos_update.translation.x,
                        player_pos_update.translation.y,
                    ),
                    0.,
                ),
                &parry2d::shape::Ball::new(5.),
            )
            .unwrap_or(false)
        } else {
            false
        };
        if intersect_rigid {
            player_pos_update.collision_status = CollisionType::Rigid;
            player_pos_update.island_id = Some(close_island.island_id);
        } else {
            let intersect_friction = if let Some(friction_mesh) = &island.friction_trimesh {
                parry2d::query::intersection_test(
                    &Isometry::new(
                        Vector2::new(
                            (island.min_x * TILE_SIZE) as f32 * ISLAND_SCALING,
                            (island.min_y * TILE_SIZE) as f32 * ISLAND_SCALING,
                        ),
                        0.,
                    ),
                    friction_mesh,
                    &Isometry::new(
                        Vector2::new(
                            player_pos_update.translation.x,
                            player_pos_update.translation.y - 24.,
                        ),
                        0.,
                    ),
                    &parry2d::shape::Ball::new(10.),
                )
                .unwrap_or(false)
            } else {
                false
            };
            if intersect_friction {
                player_pos_update.collision_status = CollisionType::Friction;
                player_pos_update.island_id = Some(close_island.island_id);
            }
        }
    }
}

fn add_islands_system(
    mut islands_to_add: ResMut<IslandQueue>,
    mut islands: ResMut<Islands>,
    mut kdtree: ResMut<SeaCollisionTree>,
) {
    for island in islands_to_add.0.drain(..) {
        let island_id = islands.0.len() as u32;
        let island_value = IslandValue {
            min_x: island.min_x,
            max_x: island.max_x,
            min_y: island.min_y,
            max_y: island.max_y,
            island_id,
        };
        kdtree.0.insert(island_value);
        islands.0.push(island);
    }
}
