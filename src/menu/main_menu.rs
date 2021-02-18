use crate::{loading::GameState, util::SeededHasher};
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContext;
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.on_state_update(GameState::STAGE, GameState::Menu, ui_system.system())
            .on_state_exit(GameState::STAGE, GameState::Menu, exit_menu.system())
            .insert_resource(MenuData {
                seed: "default seed".to_string(),
            });
    }
}
#[derive(Default)]
pub struct MenuData {
    seed: String,
}

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut data: ResMut<MenuData>,
    mut state: ResMut<State<GameState>>,
) {
    let ctx = &mut egui_context.ctx;
    ctx.set_visuals(egui::Visuals::light());
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::top_down(egui::Align::Center).with_cross_justify(true),
            |ui| {
                ui.heading("Menu Principal");
                ui.horizontal(|ui| {
                    ui.label("Enter a seed :");
                    ui.add(egui::TextEdit::singleline(&mut data.seed));
                });

                if ui.add(egui::Button::new("start")).clicked() {
                    state.overwrite_next(GameState::Sea).unwrap();
                }
            },
        );
    });
}

fn exit_menu(mut hasher: ResMut<SeededHasher>, data: Res<MenuData>) {
    *hasher = SeededHasher::new(&*data.seed);
}
