pub mod style;
pub mod tabs;

use eframe::egui::{self, Key};
use egui_dock::{DockArea, DockState, Style};
use serde_binary::binary_stream::Endian;
use strum::IntoEnumIterator;

use crate::{
    App,
    models::settings::Settings,
    state::AppStateDiff,
    ui::tabs::{TabViewer, Tabs},
};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // handle events from twitch
        while let Ok(event) = self.state.event_rx.try_recv() {
            self.state.register_new_event(event);
        }
        // handle ui changes from threads/asynchronous tasks
        while let Ok(message) = self.state.diff_rx.try_recv() {
            if matches!(message, AppStateDiff::SaveSettings) {
                let settings = Settings {
                    id: 1,
                    zoom_factor: Some(self.state.zoom_factor),
                    tree: Some(serde_binary::to_vec(&self.tree, Endian::Big).expect("Failed to serialize dock state")),
                    channel: Some(self.state.settings.channel_name.clone()),
                    user_refresh_token: self
                        .state
                        .twitch_account
                        .clone()
                        .and_then(|account| account.token.refresh_token)
                        .map(|token| token.take()),
                };
                settings.store_settings(&self.state.db_pool);
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

        // toasts
        self.state.toasts.show(ctx);

        ctx.request_repaint();
    }
}
