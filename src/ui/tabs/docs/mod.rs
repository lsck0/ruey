use eframe::egui;
use egui_commonmark::{CommonMarkCache, commonmark_str};

use crate::ui::state::AppState;

#[derive(Default)]
pub struct DocsState {}

pub fn show_docs_ui(ui: &mut egui::Ui, _state: &mut AppState) {
    commonmark_str!(ui, &mut CommonMarkCache::default(), "./assets/documentation.md");
}
