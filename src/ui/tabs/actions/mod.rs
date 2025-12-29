use eframe::egui;

use crate::ui::state::AppState;

#[derive(Default)]
pub struct ActionsState {}

pub fn show_actions_ui(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.label("Coming Soon...");
}
