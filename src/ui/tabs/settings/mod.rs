use eframe::egui::{self, Color32, RichText, TextEdit};

use crate::state::{AppState, AppStateDiff};

#[derive(Default)]
pub struct SettingsState {
    pub channel_name: String,
    pub channel_name_error: Option<String>,
}

pub fn show_settings_ui(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(RichText::new("Settings").heading().color(Color32::WHITE));

    ui.label(RichText::new("Connection").strong());

    ui.horizontal(|ui| {
        ui.label("Channel:");

        if let Some(channel) = &state.connected_channel {
            ui.label(RichText::new(channel).color(Color32::GREEN));

            if ui.button("Disconnect").clicked() {
                state.stop_twitch_worker();
            }
        } else {
            let mut channel_edit = TextEdit::singleline(&mut state.settings.channel_name).char_limit(25);
            if state.settings.channel_name_error.is_some() {
                channel_edit = channel_edit.text_color(Color32::RED);
            }
            ui.add(channel_edit);

            if ui.button("Connect").clicked() {
                match state.try_start_twitch_worker() {
                    Ok(_) => {
                        state.settings.channel_name_error = None;
                    }
                    Err(e) => {
                        state.settings.channel_name_error = Some(e);
                    }
                }
            }

            if let Some(error) = &state.settings.channel_name_error {
                ui.label(RichText::new(error).color(Color32::RED));
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Account: ");

        if let Some(acc) = &state.twitch_account {
            ui.label(RichText::new(format!("Logged in as {}", acc.token.login)).color(Color32::GREEN));

            if ui.button("Logout").clicked() {
                state.unlink_twitch_account();
            }
        } else if ui.button("Login").clicked() {
            state.link_twitch_account();
        }
    });

    ui.separator();

    ui.label(RichText::new("UI").strong());

    ui.horizontal(|ui| {
        ui.label(format!("UI Zoom: {:.1}x", state.zoom_factor));

        if ui.button("Zoom In").clicked() {
            state.zoom_factor = (state.zoom_factor + 0.1).min(3.0);
        }

        if ui.button("Zoom Out").clicked() {
            state.zoom_factor = (state.zoom_factor - 0.1).max(0.5);
        }

        if ui.button("Reset").clicked() {
            state.zoom_factor = 1.0;
        }

        ui.label("(CTRL + +/- zoomes too)")
    });

    if ui.button("Reset Layout").clicked() {
        state.diff_tx.send(AppStateDiff::ResetLayout).unwrap();
    }

    ui.separator();

    ui.label(RichText::new("Storage").strong());

    ui.horizontal(|ui| {
        if ui.button("Persist Settings").clicked() {
            state.diff_tx.send(AppStateDiff::SaveSettings).unwrap();
        }

        ui.label("(This happens automatically every 30 seconds)")
    });
}
