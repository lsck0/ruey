use eframe::egui;

use crate::ui::state::AppState;

#[derive(Default)]
pub struct LogsState {}

pub fn show_logs_ui(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.label("unimplemented");
}
