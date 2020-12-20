mod land;
mod loading;
mod sea;
mod tilemap;
mod util;

#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use bevy_dylib;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    window::WindowMode,
};
use bevy::{prelude::*, render::camera::OrthographicProjection};
use land::LandPlugin;
use loading::LoadEvent;
use util::SeededHasher;
pub const ZOOM: f32 = 1.;
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
        .add_plugins(DefaultPlugins)
        .add_plugin(sea::SeaPlugin)
        .add_plugin(LandPlugin)
        .add_plugin(loading::LoaderPlugin)
        .add_resource(SeededHasher::new(1))
        .add_startup_system(setup.system())
        // Adds frame time diagnostics
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        .add_plugin(PrintDiagnosticsPlugin::default())
        // Any plugin can register diagnostics
        .run();
}
fn setup(commands: &mut Commands, mut events: ResMut<Events<LoadEvent>>) {
    //spawning camera
    let far = 1000.;
    commands
        //camera
        .spawn(Camera2dComponents {
            orthographic_projection: OrthographicProjection {
                far,
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., far - 0.1),
                scale: ZOOM * Vec3::one(),
                ..Default::default()
            },
            ..Default::default()
        });
    events.send(LoadEvent {
        state: loading::GameState::Sea,
    })
}
