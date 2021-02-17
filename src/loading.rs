use std::path::PathBuf;

use bevy::prelude::*;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum GameState {
    Sea,
    Land,
}
impl GameState {
    pub const STAGE: &'static str = "game_stage";
}

pub struct SavePath(pub PathBuf);
pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(SavePath(PathBuf::from(r"saves/")))
            .insert_resource(State::new(GameState::Sea))
            .add_stage_after(
                stage::UPDATE,
                GameState::STAGE,
                StateStage::<GameState>::default(),
            );
    }
}
