pub mod style;
pub mod tabs;

use eframe::egui;
use egui_dock::{DockArea, Style};

use crate::{App, ui::tabs::TabViewer};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.state.diff_rx.try_recv() {
            self.state.apply_diff(message);
        }

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut TabViewer { state: &mut self.state });

        ctx.request_repaint();
    }
}
