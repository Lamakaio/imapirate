
mod sea;
use bevy::{
    prelude::*,
};
use bevy::{diagnostic::{PrintDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, window::WindowMode};
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
        .add_plugin(sea::SeaPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        .add_plugin(PrintDiagnosticsPlugin::default())
        .run();
        
}

