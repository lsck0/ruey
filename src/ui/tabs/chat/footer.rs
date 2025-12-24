use std::sync::mpsc;

use eframe::egui::{self, Button, TextEdit, Ui};
use egui_flex::{Flex, item};
use egui_toast::ToastKind;
use twitch_api::helix::channels::ChannelInformation;

use crate::{
    state::{AppState, AppStateDiff, TwitchAccount},
    twitch::api::twitch_send_message,
    ui::util::show_toast,
};

pub fn render_chat_footer(ui: &mut Ui, state: &mut AppState) {
    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

    Flex::horizontal().w_full().show(ui, |flex| {
        let input = flex.add(
            item().grow(1.0),
            TextEdit::singleline(&mut state.chat.message_input)
                .hint_text("Chat...")
                .char_limit(255),
        );

        if (flex.add(item(), Button::new("Send")).clicked() || (enter_pressed && input.lost_focus()))
            && !state.chat.message_input.is_empty()
        {
            let Some(channel) = &state.connected_channel_info else {
                show_toast(&state.diff_tx, ToastKind::Error, "You are not connected to a channel");
                return;
            };

            let Some(account) = &state.twitch_account else {
                show_toast(&state.diff_tx, ToastKind::Error, "You are not logged in.");
                return;
            };

            if state.chat.message_input.trim().starts_with('/') {
                run_command(&state.diff_tx, account, channel, &state.chat.message_input);
            } else {
                twitch_send_message(&state.diff_tx, account, channel, &state.chat.message_input);
            }

            state.chat.message_input.clear();
            input.request_focus();
        }
    });
}

fn run_command(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    message: &str,
) -> bool {
    let parts: Vec<&str> = message.split_whitespace().collect();

    // TODO: implement commands
    // - send shoutout
    // - timeout / untimeout
    // - ban / unban
    // - shoutout
    // - vip / unvip
    // - mod / unmod
    match parts[0] {
        "/shoutout" => {
            if parts.len() < 2 {
                show_toast(diff_tx, ToastKind::Error, "Usage: /shoutout <username>");
                return false;
            }
            let target_username = parts[1];
            show_toast(
                diff_tx,
                ToastKind::Info,
                &format!("Shoutout sent to {}", target_username),
            );
            true
        }
        _ => {
            show_toast(diff_tx, ToastKind::Error, "Unknown command");
            return false;
        }
    }
}
