use crate::{
    state::AppState,
    twitch::events::{PrivmsgMessageExt, TwitchEvent},
    ui::tabs::chat::message::render_chat_message,
};
use eframe::egui::{ScrollArea, Ui, scroll_area::ScrollSource};

pub fn render_chat_history(ui: &mut Ui, state: &mut AppState) {
    set_event_filter(state);

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
                    let logged_in_user_name = if let Some(account) = &state.twitch_account {
                        Some(account.token.login.clone().to_string())
                    } else {
                        None
                    };
                    render_chat_message(ui, &mut state.chat.user_query, msg, logged_in_user_name);
                }
                _ => {}
            });
        });
}

fn set_event_filter(state: &mut AppState) {
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
        TwitchEvent::Notice(_) => true,
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
