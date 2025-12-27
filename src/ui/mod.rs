pub mod state;
pub mod style;
pub mod tabs;
pub mod util;

use eframe::egui::{self, Key};
use egui_dock::{DockArea, Style};

use crate::{App, ui::tabs::TabViewer};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // handle events from twitch
        while let Ok(event) = self.state.event_rx.try_recv() {
            self.register_new_twitch_event(event);
        }
        // handle ui changes from threads/asynchronous tasks
        while let Ok(message) = self.state.diff_rx.try_recv() {
            self.apply_state_diff(message);
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

        // toasts
        self.state.toasts.show(ctx);

        ctx.request_repaint();
    }
}
