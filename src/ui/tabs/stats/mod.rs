use eframe::egui;

use crate::ui::state::AppState;

#[derive(Default)]
pub struct StatsState {}

pub fn show_stats_ui(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.label("Coming Soon...");
}
