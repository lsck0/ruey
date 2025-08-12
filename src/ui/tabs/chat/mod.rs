mod footer;
mod header;
mod history;
pub mod message;

use crate::{
    state::AppState,
    twitch::events::{PrivmsgMessageExt, TwitchEvent},
    ui::tabs::chat::{footer::render_chat_footer, header::render_chat_header, history::render_chat_history},
};
use eframe::egui;

pub fn show_chat_ui(ui: &mut egui::Ui, state: &mut AppState) {
    update_regex_cache(state);
    set_event_filter(state);

    render_chat_header(ui, state);
    ui.separator();
    render_chat_history(ui, state);
    ui.separator();
    render_chat_footer(ui, state);
}

fn update_regex_cache(state: &mut AppState) {
    if state.chat_user_query != state.chat_user_query_last {
        state.chat_user_query_last = state.chat_user_query.clone();

        if state.chat_user_query.is_empty() {
            state.chat_user_query_regex = None;
            state.chat_user_query_valid = true;
        } else {
            let pattern = format!("(?i){}", state.chat_user_query);
            state.chat_user_query_valid = match regex::Regex::new(&pattern) {
                Ok(re) => {
                    state.chat_user_query_regex = Some(re);
                    true
                }
                Err(_) => {
                    state.chat_user_query_regex = None;
                    false
                }
            };
        }
    }

    if state.chat_message_query != state.chat_message_query_last {
        state.chat_message_query_last = state.chat_message_query.clone();

        if state.chat_message_query.is_empty() {
            state.chat_message_regex = None;
            state.chat_message_query_valid = true;
        } else {
            let pattern = format!("(?i){}", state.chat_message_query);
            state.chat_message_query_valid = match regex::Regex::new(&pattern) {
                Ok(re) => {
                    state.chat_message_regex = Some(re);
                    true
                }
                Err(_) => {
                    state.chat_message_regex = None;
                    false
                }
            };
        }
    }
}

fn set_event_filter(state: &mut AppState) {
    let local_chat_show_messages = state.chat_show_messages;
    let local_chat_show_messages_by_broadcaster = state.chat_show_messages_by_broadcaster;
    let local_chat_show_messages_by_moderator = state.chat_show_messages_by_moderator;
    let local_chat_show_messages_by_vip = state.chat_show_messages_by_vip;
    let local_chat_show_messages_by_subscriber = state.chat_show_messages_by_subscriber;
    let local_chat_show_messages_by_regular_viewer = state.chat_show_messages_by_regular_viewer;
    let local_chat_user_query_regex = state.chat_user_query_regex.clone();
    let local_chat_message_regex = state.chat_message_regex.clone();

    state.events.set_filter(move |event| match event {
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
        _ => true,
    });
}
