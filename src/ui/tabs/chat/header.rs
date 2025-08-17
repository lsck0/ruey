use std::fs;

use crate::{
    state::AppState,
    twitch::{api::twitch_delete_all_messages, types::TwitchEvent},
    ui::tabs::chat::message::render_event_for_log,
};
use eframe::egui::{self, Button, TextEdit, Ui};
use egui_flex::{Flex, item};
use tracing::warn;

pub fn render_chat_header(ui: &mut Ui, state: &mut AppState) {
    Flex::horizontal().w_full().show(ui, |flex| {
        // TODO: implement the apis for these buttons
        flex.add_ui(item(), |ui| {
            if let Some(account) = &state.twitch_account
                && let Some(channel) = &state.connected_channel_info
            {
                ui.menu_button("Chat Settings", |ui| {
                    if ui.button("Clear Chat").clicked() {
                        twitch_delete_all_messages(&state.diff_tx, account, channel);
                    }

                    if ui.button("Toggle Emote-Only Chat").clicked() {}

                    ui.menu_button("Follow-Only Chat", |ui| {
                        if ui.button("Off").clicked() {}

                        ui.separator();

                        if ui.button("On").clicked() {}

                        if ui.button("10 Minutes").clicked() {}

                        if ui.button("30 Minutes").clicked() {}

                        if ui.button("1 Hour").clicked() {}

                        if ui.button("1 Day").clicked() {}

                        if ui.button("1 Week").clicked() {}

                        if ui.button("1 Month").clicked() {}

                        if ui.button("3 Months").clicked() {}
                    });

                    if ui.button("Toggle Sub-Only Chat").clicked() {}
                });
            }

            ui.menu_button("Show", |ui| {
                ui.label("Chatters");
                if ui
                    .selectable_label(state.chat.show_messages_by_broadcaster, "Broadcaster")
                    .clicked()
                {
                    state.chat.show_messages_by_broadcaster ^= true;
                }

                if ui
                    .selectable_label(state.chat.show_messages_by_moderator, "Moderators")
                    .clicked()
                {
                    state.chat.show_messages_by_moderator ^= true;
                }

                if ui.selectable_label(state.chat.show_messages_by_vip, "VIPs").clicked() {
                    state.chat.show_messages_by_vip ^= true;
                }

                if ui
                    .selectable_label(state.chat.show_messages_by_subscriber, "Subscribers")
                    .clicked()
                {
                    state.chat.show_messages_by_subscriber ^= true;
                }

                if ui
                    .selectable_label(state.chat.show_messages_by_regular_viewer, "Viewers")
                    .clicked()
                {
                    state.chat.show_messages_by_regular_viewer ^= true;
                }

                ui.separator();
                ui.label("Kinds");

                if ui.selectable_label(state.chat.show_notices, "Notices").clicked() {
                    state.chat.show_notices ^= true;
                }

                if ui.selectable_label(state.chat.show_messages, "Messages").clicked() {
                    state.chat.show_messages ^= true;
                }

                if ui.selectable_label(state.chat.show_follows, "Follows").clicked() {
                    state.chat.show_follows ^= true;
                }

                if ui
                    .selectable_label(state.chat.show_subscriptions, "Subscriptions")
                    .clicked()
                {
                    state.chat.show_subscriptions ^= true;
                }

                if ui.selectable_label(state.chat.show_bits, "Bits").clicked() {
                    state.chat.show_bits ^= true;
                }

                if ui.selectable_label(state.chat.show_raids, "Raids").clicked() {
                    state.chat.show_raids ^= true;
                }
            });
        });

        flex.add_ui(item().grow(1.0), |ui| {
            let mut user_query_input = TextEdit::singleline(&mut state.chat.user_query)
                .hint_text("Name Search")
                .char_limit(75)
                .desired_width(120.0);
            if !state.chat.user_query_valid {
                user_query_input = user_query_input.text_color(egui::Color32::RED);
            }
            user_query_input.show(ui);

            let mut message_query_input = TextEdit::singleline(&mut state.chat.message_query)
                .hint_text("Message Search")
                .char_limit(75)
                .desired_width(120.0);
            if !state.chat.message_query_valid {
                message_query_input = message_query_input.text_color(egui::Color32::RED);
            }
            message_query_input.show(ui);

            if ui.button("Clear Search").clicked() {
                state.chat.user_query.clear();
                state.chat.message_query.clear();
            }
        });

        if flex.add(item(), Button::new("Export Chat Log")).clicked() {
            state.file_dialog.save_file();
        }
    });

    // chat log saving, i failed twice moving this to a worker thread already
    if let Some(path) = state.file_dialog.update(ui.ctx()).picked() {
        let mut buffer = String::new();
        buffer.reserve(state.chat.events.items.len() * size_of::<TwitchEvent>());

        for event in state.chat.events.items.iter() {
            render_event_for_log(&mut buffer, event);
        }

        if let Err(err) = fs::write(path, buffer) {
            warn!("Failed to write chat log to file: {err}");
        }
    }
}
