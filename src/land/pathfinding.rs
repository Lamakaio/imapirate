use std::sync::Arc;

use super::LAND_SCALING;
use crate::{sea::TILE_SIZE, tilemap::Tile as MapTile};
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
const TILE: f32 = TILE_SIZE as f32 * LAND_SCALING;

#[derive(Debug, Clone)]
pub struct NoPathError;

#[derive(Debug, Clone)]
pub struct PathFinishedError;

pub trait Pathfinder {
    /// a trait that a pathfinding algoritm must implement to be used by mobs.
    /// It must be able to :
    /// -calculate and store a path between two points with the find_path method
    /// -calculate the mob position  for each succesive frame.
    ///
    /// The mob system will update the path using find_path when necessary,
    /// and call the step function every frame.
    fn find_path(&mut self, mob_pos: Vec2, player_pos: Vec2) -> Result<(), NoPathError>;

    fn step(&mut self, speed: f32, delta_time: f32) -> Result<Vec2, PathFinishedError>;
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PathfindingType {
    LineOfSight(f32),
    None,
}

pub struct NoPathfinding;

impl Pathfinder for NoPathfinding {
    /// Empty pathfinding : the mob won't move
    fn find_path(&mut self, _mob_pos: Vec2, _player_pos: Vec2) -> Result<(), NoPathError> {
        Err(NoPathError)
    }

    fn step(&mut self, _speed: f32, _delta_time: f32) -> Result<Vec2, PathFinishedError> {
        Err(PathFinishedError)
    }
}

const SAMPLES_PER_TILE: f32 = 10.;
#[derive(Default)]
pub struct LineOfSight {
    /// line of sight pathfinding
    /// the mobs check the following :
    /// - The player is within its view distance
    /// - There is no obstacle between the mob and the player
    ///
    /// If both are true, it starts a path towards the player.
    /// If one of them becomes false, step calls are still valid and continues towards the position the player was last seen at.
    /// find_path still return NoPathError though
    pub view_distance: f32,
    pub destination: Vec2,
    pub origin: Vec2,
    pub path_len: f32,
    pub transition: f32,
    pub collision: Arc<Vec<Vec<isize>>>,
}
impl Pathfinder for LineOfSight {
    fn find_path(&mut self, mob_pos: Vec2, player_pos: Vec2) -> Result<(), NoPathError> {
        let path_len = (player_pos - mob_pos).length();
        if path_len > self.view_distance {
            return Err(NoPathError);
        }
        let n_samples = path_len / TILE * SAMPLES_PER_TILE;
        let step = (mob_pos - player_pos) / n_samples;
        let clear = (0..n_samples as u32 + 1).into_iter().all(|i| {
            let tile = ((mob_pos + i as f32 * step) / TILE).floor();
            *self
                .collision
                .get(tile.x as usize)
                .unwrap_or(&Vec::new())
                .get(tile.y as usize)
                .unwrap_or(&1)
                > 0
        });
        if clear {
            self.destination = player_pos;
            self.origin = mob_pos;
            self.path_len = path_len;
            self.transition = 0.;
            Ok(())
        } else {
            Err(NoPathError)
        }
    }

    fn step(&mut self, speed: f32, delta_time: f32) -> Result<Vec2, PathFinishedError> {
        self.transition += speed * delta_time / self.path_len * TILE;
        if self.transition <= 1. {
            Ok(Vec2::lerp(self.origin, self.destination, self.transition))
        } else {
            Err(PathFinishedError)
        }
    }
}

//Get the cost of walking over the tile according to the sprite id.
//Might be a good idea to add some sort of parameter to that function so different behaviour can exist.
pub fn get_tile_cost(tile: &MapTile) -> isize {
    let id = match tile {
        MapTile::Static(id) => id,
        MapTile::Animated(v) => &v[0],
    };
    match *id {
        0 => 1,
        _ => 1,
    }
}

//get a pathfinder struct for the current island and with the choosen pathfinding algorithm
pub fn get_pathfinding(
    collision: &Arc<Vec<Vec<isize>>>,
    pathfinding_type: PathfindingType,
) -> Box<dyn Pathfinder + Send + Sync> {
    match pathfinding_type {
        PathfindingType::None => Box::new(NoPathfinding),
        PathfindingType::LineOfSight(view_distance) => Box::new(LineOfSight {
            view_distance,
            collision: collision.clone(),
            ..Default::default()
        }),
    }
}
