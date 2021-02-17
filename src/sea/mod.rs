use bevy::prelude::*;

//pub(crate) mod collision;
mod collision;
mod loader;
pub(crate) mod map;
pub mod player;
pub(crate) mod worldgen;
//use collision::SeaCollisionPlugin;
use loader::SeaLoaderPlugin;
use map::SeaMapPlugin;
use player::SeaPlayerPlugin;
use worldgen::SeaWorldGenPlugin;

use self::collision::SeaCollisionPlugin;
pub const ISLAND_SCALING: f32 = 2.;
pub struct SeaPlugin;

pub const TILE_SIZE: i32 = 16;

impl Plugin for SeaPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(SeaLoaderPlugin)
            .add_plugin(SeaPlayerPlugin)
            .add_plugin(SeaMapPlugin)
            .add_plugin(SeaCollisionPlugin)
            .add_plugin(SeaWorldGenPlugin);
    }
}
