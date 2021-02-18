use bevy::prelude::*;

use self::main_menu::MainMenuPlugin;
mod main_menu;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(MainMenuPlugin);
    }
}
