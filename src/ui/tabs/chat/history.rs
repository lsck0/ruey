use eframe::egui::{self, Align2, Area, ScrollArea, Ui, scroll_area::ScrollSource};

use crate::{
    twitch::types::{PrivmsgMessageExt, TwitchEvent},
    ui::{state::AppState, tabs::chat::message::render_chat_message},
};

pub fn render_chat_history(ui: &mut Ui, state: &mut AppState) {
    set_event_filter(state);

    Area::new("chat_state".into())
        .anchor(Align2::RIGHT_TOP, egui::vec2(0.0, 60.0))
        .show(ui.ctx(), |ui| {
            ui.set_width(250.0);
            ui.set_height(200.0);

            if let Some(duration) = state.chat.is_slow_mode
                && duration.as_secs() > 0
            {
                ui.label(format!("Chat is in slow-only mode ({} seconds).", duration.as_secs()));
            }

            if state.chat.is_emote_only {
                ui.label("Chat is in emote-only mode.");
            }

            if let Some(duration) = state.chat.is_follow_only {
                let seconds = duration.as_secs();
                let minutes = seconds / 60;
                let hours = minutes / 60;
                let days = hours / 24;

                if days > 0 {
                    ui.label(format!("Chat is in follower-only mode (> {} days).", days));
                } else if hours > 0 {
                    ui.label(format!("Chat is in follower-only mode (> {} hours).", hours));
                } else if minutes > 0 {
                    ui.label(format!("Chat is in follower-only mode (> {} minutes).", minutes));
                } else if seconds > 0 {
                    ui.label(format!("Chat is in follower-only mode (> {} seconds).", seconds));
                } else {
                    ui.label("Chat is in follower-only mode.");
                }
            }

            if state.chat.is_subscriber_only {
                ui.label("Chat is in subscriber-only mode.");
            }
        });

    ScrollArea::vertical()
        .max_height(ui.available_height() - 35.0)
        .max_width(ui.available_width() - 5.0)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .scroll_source(ScrollSource {
            drag: false,
            mouse_wheel: true,
            scroll_bar: true,
        })
        .show(ui, |ui| {
            state.chat.events.ui(ui, 50, |ui, _, event| match event {
                TwitchEvent::Join(join) => {
                    ui.label(format!("Joined channel {}.", join.channel_login));
                }
                TwitchEvent::Notice(notice) => {
                    ui.label(notice.message_text.trim());
                }
                TwitchEvent::Privmsg(msg) => {
                    let logged_in_user_name = state
                        .twitch_account
                        .as_ref()
                        .map(|account| account.token.login.clone().to_string());

                    render_chat_message(
                        ui,
                        msg,
                        &state.diff_tx,
                        &state.twitch_account,
                        &state.connected_channel_info,
                        &mut state.chat.user_query,
                        logged_in_user_name,
                    );
                }
                _ => {}
            });
        });
}

fn set_event_filter(state: &mut AppState) {
    let local_chat_show_notices = state.chat.show_notices;
    let local_chat_show_messages = state.chat.show_messages;
    let local_chat_show_messages_by_broadcaster = state.chat.show_messages_by_broadcaster;
    let local_chat_show_messages_by_moderator = state.chat.show_messages_by_moderator;
    let local_chat_show_messages_by_vip = state.chat.show_messages_by_vip;
    let local_chat_show_messages_by_subscriber = state.chat.show_messages_by_subscriber;
    let local_chat_show_messages_by_regular_viewer = state.chat.show_messages_by_regular_viewer;
    let local_chat_user_query_regex = state.chat.user_query_regex.clone();
    let local_chat_message_regex = state.chat.message_regex.clone();

    state.chat.events.set_filter(move |event| match event {
        TwitchEvent::Join(_) => true,
        TwitchEvent::Notice(_) => local_chat_show_notices,
        TwitchEvent::Privmsg(msg) => {
            if !local_chat_show_messages {
                return false;
            }

            if !local_chat_show_messages_by_broadcaster && msg.is_by_broadcaster() {
                return false;
            }
            if !local_chat_show_messages_by_moderator && msg.is_by_mod() {
                return false;
            }
            if !local_chat_show_messages_by_vip && msg.is_by_vip() {
                return false;
            }
            if !local_chat_show_messages_by_subscriber && msg.is_by_subscriber() {
                return false;
            }
            if !local_chat_show_messages_by_regular_viewer && msg.is_by_regular_viewer() {
                return false;
            }

            if let Some(ref re) = local_chat_user_query_regex
                && !re.is_match(&msg.sender.name)
            {
                return false;
            }

            if let Some(ref re) = local_chat_message_regex
                && !re.is_match(&msg.message_text)
            {
                return false;
            }

            return true;
        }
        _ => false,
    });
}
