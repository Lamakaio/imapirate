use std::sync::{Arc, Mutex};

use super::LAND_SCALING;
use crate::{sea::TILE_SIZE, tilemap::Tile as MapTile};
use bevy::math::Vec2;
use hierarchical_pathfinding::{
    prelude::ManhattanNeighborhood, prelude::Neighborhood, AbstractPath, PathCache,
};
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
    HierachicalAStar,
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

pub struct HierachicalAStar<N: Neighborhood> {
    current_path: Option<AbstractPath<N>>,
    pathcache: Arc<Mutex<PathCache<N>>>,
    collision: Arc<Vec<Vec<isize>>>,
    last_tile: Vec2,
    destination: Vec2,
    transition: f32,
}

impl<N: Neighborhood> Pathfinder for HierachicalAStar<N> {
    /// Hierarchical A* pathfinding
    /// Use an inexact, but fast A* algorithm
    /// If there is a path, it will find it.
    /// Should probably only be used for very long path, as short path result weird behaviour.

    fn find_path(&mut self, mob_pos: Vec2, player_pos: Vec2) -> Result<(), NoPathError> {
        let start = (
            (mob_pos.x() / TILE).floor() as usize,
            (mob_pos.y() / TILE).floor() as usize,
        );
        let goal = (
            (player_pos.x() / TILE).floor() as usize,
            (player_pos.y() / TILE).floor() as usize,
        );
        self.current_path = self
            .pathcache
            .lock()
            .expect("two accesses to the mob pathcache at the same time, should not happen")
            .find_path(start, goal, |(x, y)| {
                *self
                    .collision
                    .get(x)
                    .unwrap_or(&Vec::new())
                    .get(y)
                    .unwrap_or(&-1)
            });
        match self.current_path {
            Some(_) => Ok(()),
            None => Err(NoPathError),
        }
    }

    fn step(&mut self, speed: f32, delta_time: f32) -> Result<Vec2, PathFinishedError> {
        self.transition += speed * delta_time;
        if self.transition <= 1. {
            Ok(Vec2::lerp(
                self.last_tile,
                self.destination,
                self.transition,
            ))
        } else {
            self.last_tile = self.destination;
            self.transition = 0.;
            //this just apply next inside the option and flatten it
            if let Some((tile_x, tile_y)) =
                self.current_path.as_mut().map(|path| path.next()).flatten()
            {
                *self.destination.x_mut() = tile_x as f32 * TILE;
                *self.destination.y_mut() = tile_y as f32 * TILE;
                Ok(self.last_tile)
            } else {
                Err(PathFinishedError)
            }
        }
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
                .get(tile.x() as usize)
                .unwrap_or(&Vec::new())
                .get(tile.y() as usize)
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
    pathcache: &Option<Arc<Mutex<PathCache<ManhattanNeighborhood>>>>,
    collision: &Arc<Vec<Vec<isize>>>,
    pathfinding_type: PathfindingType,
) -> Box<dyn Pathfinder + Send + Sync> {
    match pathfinding_type {
        PathfindingType::None => Box::new(NoPathfinding),
        PathfindingType::HierachicalAStar => {
            // if island.pathcache.is_none() {
            //     let dim = (island.collision.len(), island.collision[0].len());
            //     island.pathcache = Some(Arc::new(Mutex::new(PathCache::new(
            //         (dim.0, dim.1),
            //         |(x, y)| island.collision[x][y],
            //         ManhattanNeighborhood::new(dim.0, dim.1),
            //         PathCacheConfig::HIGH_PERFORMANCE,
            //     ))));
            // }
            Box::new(HierachicalAStar {
                pathcache: pathcache.clone().expect("the island has no pathcache"),
                collision: collision.clone(),
                current_path: None,
                last_tile: Vec2::default(),
                destination: Vec2::default(),
                transition: 0.,
            })
        }
        PathfindingType::LineOfSight(view_distance) => Box::new(LineOfSight {
            view_distance,
            collision: collision.clone(),
            ..Default::default()
        }),
    }
}
