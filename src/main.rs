mod land;
mod loading;
mod sea;
mod tilemap;
mod util;

use bevy::window::WindowMode;
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
        .add_default_plugins()
        .add_plugin(sea::SeaPlugin)
        .add_plugin(LandPlugin)
        .add_plugin(loading::LoaderPlugin)
        .add_resource(SeededHasher::new(1))
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, mut events: ResMut<Events<LoadEvent>>) {
    //spawning camera
    let far = 1000.;
    commands
        //camera
        .spawn(Camera2dComponents {
            orthographic_projection: OrthographicProjection {
                far,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., far - 0.1)), //TODO add back ZOOM
            ..Default::default()
        });
    events.send(LoadEvent {
        state: loading::GameState::Sea,
    })
}
