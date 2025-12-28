use eframe::egui;

use crate::ui::state::AppState;

#[derive(Default)]
pub struct DatabaseState {}

pub fn show_database_ui(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.label("unimplemented");
}
