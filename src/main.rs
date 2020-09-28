mod land;
mod loading;
mod sea;
mod tilemap;

use bevy::prelude::*;
use bevy::window::WindowMode;
use land::LandPlugin;
fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "I am a window!".to_string(),
            width: 1920,
            height: 1080,
            vsync: true,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .add_default_plugins()
        .add_plugin(sea::SeaPlugin)
        .add_plugin(LandPlugin)
        .add_plugin(loading::LoaderPlugin)
        .run();
}
