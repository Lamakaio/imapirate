use bevy::{ecs::bevy_utils::HashMap, prelude::*};
use hierarchical_pathfinding::{prelude::ManhattanNeighborhood, AbstractPath};

use crate::sea::TILE_SIZE;

use super::{
    islands_from_map::Island, map::CurrentIsland, player::Player, player::PlayerMovedEvent,
    player::PlayerMovedEventReader, LAND_SCALING,
};

pub struct LandMobsPlugin;
impl Plugin for LandMobsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(mob_movement_system.system());
    }
}

#[derive(Debug)]
pub enum MobKind {
    Crab,
}

#[derive(Debug)]
pub struct Mob {
    pub kind: MobKind,
    pub update_pathfinding: bool,
    pub current_path: Option<AbstractPath<ManhattanNeighborhood>>,
    pub last_tile: Vec2,
    pub destination: Vec2,
    pub transition: f32,
    pub speed: f32,
}

impl Default for Mob {
    fn default() -> Self {
        Mob {
            kind: MobKind::Crab,
            update_pathfinding: true,
            current_path: None,
            last_tile: Vec2::default(),
            destination: Vec2::default(),
            transition: 1.,
            speed: 2.,
        }
    }
}

//TODO : remove the duplicate code in that system
fn mob_movement_system(
    mut event_reader: Local<PlayerMovedEventReader>,
    events: Res<Events<PlayerMovedEvent>>,
    time: Res<Time>,
    current_island: Res<CurrentIsland>,
    mut islands: ResMut<HashMap<u64, Island>>,
    mut mob_query: Query<(&mut Mob, &mut Transform)>,
    mut player_query: Query<(&Player, &Transform)>,
) {
    const TILE: f32 = TILE_SIZE as f32 * LAND_SCALING;
    let should_update = event_reader.reader.iter(&events).next().is_none();
    for (_, player_transform) in &mut player_query.iter() {
        let player_x = player_transform.translation.x();
        let player_y = player_transform.translation.y();
        for (mut mob, mut mob_transform) in &mut mob_query.iter() {
            if should_update {
                mob.update_pathfinding = true;
            }
            if mob.current_path.is_none() {
                //It means either the path is not initialized, or there are no path.
                if !mob.update_pathfinding {
                    //to avoid recalculating the path every frame : only when the player moved
                    continue;
                }
                let start = (
                    (mob_transform.translation.x() / TILE).floor() as usize,
                    (mob_transform.translation.y() / TILE).floor() as usize,
                );
                let goal = (
                    (player_x / TILE).floor() as usize,
                    (player_y / TILE).floor() as usize,
                );
                mob.last_tile = Vec2::new(start.0 as f32 * TILE, start.1 as f32 * TILE); //updates the current tile
                mob.update_pathfinding = false;
                let island = islands
                    .get_mut(&current_island.id)
                    .expect("Island does not exist");
                let collision = island.collision.clone(); //get island collision map
                mob.current_path = island.pathcache.find_path(start, goal, |(x, y)| {
                    *collision
                        .get(x)
                        .unwrap_or(&Vec::new())
                        .get(y)
                        .unwrap_or(&-1)
                }); //and the path
                if mob.current_path.is_none() {
                    //if there still isn't a path, does nothing
                    continue;
                }
                //else, start to follow the path
                mob.transition = 0.;
                let next = mob.current_path.as_mut().unwrap().next();
                if let Some((tile_x, tile_y)) = next {
                    *mob.destination.x_mut() = tile_x as f32 * TILE;
                    *mob.destination.y_mut() = tile_y as f32 * TILE;
                }
            }
            mob.transition += mob.speed * time.delta_seconds;
            if mob.transition >= 1. {
                *mob_transform.translation.x_mut() = mob.destination.x();
                *mob_transform.translation.y_mut() = mob.destination.y();
                mob.last_tile = mob.destination;
                mob.transition = 0.;
                if mob.update_pathfinding {
                    mob.update_pathfinding = false;
                    let island = islands
                        .get_mut(&current_island.id)
                        .expect("Island does not exist");
                    let start = (
                        (mob.destination.x() / TILE).floor() as usize,
                        (mob.destination.y() / TILE).floor() as usize,
                    );
                    let goal = (
                        (player_x / TILE).floor() as usize,
                        (player_y / TILE).floor() as usize,
                    );
                    let collision = island.collision.clone();
                    mob.current_path = island.pathcache.find_path(start, goal, |(x, y)| {
                        *collision
                            .get(x)
                            .unwrap_or(&Vec::new())
                            .get(y)
                            .unwrap_or(&-1)
                    });
                }
                let next = mob.current_path.as_mut().unwrap().next();
                if let Some((tile_x, tile_y)) = next {
                    *mob.destination.x_mut() = tile_x as f32 * TILE;
                    *mob.destination.y_mut() = tile_y as f32 * TILE;
                }
            } else {
                let new_pos = Vec2::lerp(mob.last_tile, mob.destination, mob.transition);
                *mob_transform.translation.x_mut() = new_pos.x();
                *mob_transform.translation.y_mut() = new_pos.y();
            }
        }
    }
}
