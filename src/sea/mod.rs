use bevy::{
    prelude::*,
};
mod worldgen;
mod player;
mod map;
mod minimap;

use map::SeaMapPlugin;
use player::SeaPlayerPlugin;

pub struct SeaPlugin;

impl Plugin for SeaPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_plugin(SeaPlayerPlugin)
        .add_plugin(SeaMapPlugin)
        ;
    }
}

