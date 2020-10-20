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

pub struct SavePath(pub PathBuf);
pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(SavePath(PathBuf::from(r"saves/")))
            .init_resource::<LoadEventReader>()
            .add_event::<LoadEvent>();
    }
}
