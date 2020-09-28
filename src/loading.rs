use std::path::PathBuf;

use bevy::prelude::*;

#[derive(PartialEq, Eq)]
pub enum GameState {
    Sea,
    Land,
}

pub struct LoadEvent {
    pub state: GameState,
}
#[derive(Default)]
pub struct LoadEventReader {
    pub reader: EventReader<LoadEvent>,
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut events: ResMut<Events<LoadEvent>>,
) {
    if keyboard_input.just_pressed(KeyCode::A) {
        events.send(LoadEvent {
            state: GameState::Land,
        });
    }
    if keyboard_input.just_pressed(KeyCode::B) {
        events.send(LoadEvent {
            state: GameState::Sea,
        });
    }
}

pub struct SavePath(pub PathBuf);
pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(SavePath(PathBuf::from(r"saves/")))
            .init_resource::<LoadEventReader>()
            .add_event::<LoadEvent>()
            .add_system(keyboard_input_system.system());
    }
}
