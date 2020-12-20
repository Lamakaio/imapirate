pub(crate) mod islands_from_map;
mod loader;
pub(crate) mod map;
mod mobs;
mod pathfinding;
mod player;
mod worldgen;

use bevy::prelude::*;
use islands_from_map::IslandFromMapPlugin;
use loader::LandLoaderPlugin;
use map::LandMapPlugin;
use player::LandPlayerPlugin;

use self::mobs::LandMobsPlugin;
pub const LAND_SCALING: f32 = 24.;
pub struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(IslandFromMapPlugin)
            .add_plugin(LandLoaderPlugin)
            .add_plugin(LandPlayerPlugin)
            .add_plugin(LandMapPlugin)
            .add_plugin(LandMobsPlugin);
    }
}
