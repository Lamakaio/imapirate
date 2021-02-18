use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui::Visuals;

use crate::{character::CharacterSheet, loading::GameState};

pub struct LandUiPlugin;

impl Plugin for LandUiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.on_state_update(GameState::STAGE, GameState::Land, ui_system.system());
    }
}
fn ui_system(mut egui_context: ResMut<EguiContext>, mut sheet: ResMut<CharacterSheet>) {
    let ctx = &mut egui_context.ctx;
    let max_life = sheet.stats.max_life;
    let life = sheet.values.life;
    egui::Area::new("")
        .fixed_pos(egui::pos2(4.0, 4.0))
        .show(ctx, |ui| {
            *ui.visuals_mut() = Visuals::light();

            egui::Frame::group(ui.style())
                .fill(egui::Color32::WHITE)
                .show(ui, |ui| {
                    ui.add(egui::Slider::u32(&mut sheet.values.life, 0..=max_life));
                    ui.label(format!("{}/{}", life, max_life));
                });
        });
}
