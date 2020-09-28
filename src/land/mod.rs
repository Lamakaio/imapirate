use bevy::prelude::*;
use map::SeaMapPlugin;
mod loader;
pub(crate) mod map;
mod player;

pub struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(SeaMapPlugin);
    }
}
