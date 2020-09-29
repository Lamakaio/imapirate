use bevy::prelude::*;
pub(crate) mod collision;
mod loader;
pub(crate) mod map;
mod player;
pub(crate) mod worldgen;

use collision::SeaCollisionPlugin;
use loader::SeaLoaderPlugin;
use map::SeaMapPlugin;
use player::SeaPlayerPlugin;

pub struct SeaPlugin;

pub const SCALING: i32 = 4;
pub const CHUNK_SIZE: i32 = 128;
pub const TILE_SIZE: i32 = 16;

impl Plugin for SeaPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(SeaPlayerPlugin)
            .add_plugin(SeaMapPlugin)
            .add_plugin(SeaLoaderPlugin)
            .add_plugin(SeaCollisionPlugin);
    }
}
