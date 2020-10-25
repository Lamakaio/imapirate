use std::sync::{Arc, Mutex};

use super::{islands_from_map::Island, LAND_SCALING};
use crate::{sea::TILE_SIZE, tilemap::Tile as MapTile};
use bevy::math::Vec2;
use hierarchical_pathfinding::{prelude::Neighborhood, AbstractPath, PathCache};
const TILE: f32 = TILE_SIZE as f32 * LAND_SCALING;

#[derive(Debug, Clone)]
pub struct NoPathError;

#[derive(Debug, Clone)]
pub struct PathFinishedError;

pub trait Pathfinder {
    fn find_path(&mut self, mob_pos: Vec2, player_pos: Vec2) -> Result<(), NoPathError>;

    fn step(&mut self, speed: f32, delta_time: f32) -> Result<Vec2, PathFinishedError>;
}

pub enum PathfindingType {
    HierachicalAStar,
    None,
}

pub struct NoPathfinding;

impl Pathfinder for NoPathfinding {
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
        if self.transition < 1. {
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

pub fn get_pathfinding(
    island: &Island,
    pathfinding_type: PathfindingType,
) -> Box<dyn Pathfinder + Send + Sync> {
    match pathfinding_type {
        PathfindingType::None => Box::new(NoPathfinding),
        PathfindingType::HierachicalAStar => Box::new(HierachicalAStar {
            pathcache: island.pathcache.clone(),
            collision: island.collision.clone(),
            current_path: None,
            last_tile: Vec2::default(),
            destination: Vec2::default(),
            transition: 0.,
        }),
    }
}
