pub(crate) mod collision;
mod loader;
pub(crate) mod map;
pub(crate) mod mobs;
pub(crate) mod pathfinding;
pub(crate) mod player;
pub(crate) mod ui;
use bevy::prelude::*;
use loader::LandLoaderPlugin;
use map::LandMapPlugin;
use player::LandPlayerPlugin;

use self::{collision::LandCollisionPlugin, mobs::LandMobsPlugin, ui::LandUiPlugin};

pub const LAND_SCALING: f32 = 10.;
pub struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(LandLoaderPlugin)
            .add_plugin(LandPlayerPlugin)
            .add_plugin(LandMapPlugin)
            .add_plugin(LandMobsPlugin)
            .add_plugin(LandCollisionPlugin)
            .add_plugin(LandUiPlugin);
    }
}
