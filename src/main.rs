mod background;
mod land;
mod loading;
mod sea;
mod util;
use background::SeaBackgroundPlugin;
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use bevy_dylib;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::{Level, LogSettings},
    window::WindowMode,
};
use bevy::{prelude::*, render::camera::OrthographicProjection};
use land::LandPlugin;
use util::SeededHasher;

pub const ZOOM: f32 = 1.;
fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "I am a window!".to_string(),
            width: 1920.,
            height: 1080.,
            vsync: false,
            resizable: false,
            mode: WindowMode::Fullscreen { use_size: false },
            ..Default::default()
        })
        .init_resource::<Time>()
        //.add_resource(DefaultTaskPoolOptions::with_num_threads(12))
        .insert_resource(LogSettings {
            filter: "wgpu=error".to_string(),
            level: Level::ERROR,
        })
        //.add_resource(Msaa { samples: 4 })
        .add_plugin(loading::LoaderPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(sea::SeaPlugin)
        //.add_plugin(LandPlugin)
        .insert_resource(SeededHasher::new(1))
        .add_startup_system(setup.system())
        //Adds frame time diagnostics
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(SeaBackgroundPlugin)
        .add_plugin(LandPlugin)
        // Any plugin can register diagnostics
        .run();
}
fn setup(commands: &mut Commands) {
    //spawning camera
    let far = 1000.;
    commands
        //camera
        .spawn(OrthographicCameraBundle {
            orthographic_projection: OrthographicProjection {
                far,
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., ZOOM * far - 0.1),
                scale: ZOOM as f32 * Vec3::one(),
                ..Default::default()
            },
            ..OrthographicCameraBundle::new_2d()
        });
}
