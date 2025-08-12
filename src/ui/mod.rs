pub mod style;
pub mod tabs;

use eframe::egui::{self, Key};
use egui_dock::{DockArea, DockState, Style};
use strum::IntoEnumIterator;

use crate::{
    App,
    state::AppStateDiff,
    ui::tabs::{TabViewer, Tabs},
};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // handle events from threads
        while let Ok(message) = self.state.diff_rx.try_recv() {
            if matches!(message, AppStateDiff::SaveSettings) {
                let tree_str = serde_json::to_string(&self.tree).unwrap();
                self.state.store_settings(tree_str);
            }
            if matches!(message, AppStateDiff::ResetLayout) {
                self.tree = DockState::new(Tabs::iter().collect());
            }

            self.state.apply_diff(message);
        }

        // handle zooming
        let input = ctx.input(|i| i.clone());
        if input.modifiers.ctrl && input.key_pressed(Key::Plus) {
            self.state.zoom_factor += 0.1;
        }
        if input.modifiers.ctrl && input.key_pressed(Key::Minus) {
            self.state.zoom_factor -= 0.1;
        }
        ctx.set_zoom_factor(self.state.zoom_factor);

        // main UI
        if self.state.connected_to_internet {
            DockArea::new(&mut self.tree)
                .style(Style::from_egui(ctx.style().as_ref()))
                .show(ctx, &mut TabViewer { state: &mut self.state });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.heading("You are not connected to the internet :(");
                })
            });
        }

        ctx.request_repaint();
    }
}
