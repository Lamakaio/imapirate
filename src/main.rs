
mod sea_main;
mod worldgen;
mod tmxgen;
use bevy::{
    prelude::*,
};
use bevy::window::WindowMode;
fn main() {
    App::build()
        .add_resource(WindowDescriptor {
        title: "I am a window!".to_string(),
        width: 1920,
        height: 1080,
        vsync: true,
        resizable: false,
        mode: WindowMode::BorderlessFullscreen,
        ..Default::default()
        })
        .add_default_plugins()
        .add_plugin(sea_main::SeaPlugin)
        .run();
        
}

