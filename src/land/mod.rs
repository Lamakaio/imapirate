pub(crate) mod islands_from_map;
mod loader;
pub(crate) mod map;
mod player;

use bevy::prelude::*;
use islands_from_map::IslandFromMapPlugin;

pub struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(IslandFromMapPlugin);
    }
}
