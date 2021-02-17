mod loader;
pub(crate) mod map;
//mod mobs;
//mod pathfinding;
mod player;

use bevy::prelude::*;
use loader::LandLoaderPlugin;
use map::LandMapPlugin;
use player::LandPlayerPlugin;

pub const LAND_SCALING: f32 = 10.;
pub struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(LandLoaderPlugin)
            .add_plugin(LandPlayerPlugin)
            .add_plugin(LandMapPlugin);
    }
}
