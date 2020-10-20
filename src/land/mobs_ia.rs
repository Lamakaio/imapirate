use super::islands_from_map::Island;
use crate::tilemap::Tile as MapTile;
use bevy::ecs::bevy_utils::HashMap;

pub enum IAType {
    Crab,
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
