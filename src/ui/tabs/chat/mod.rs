mod footer;
mod header;
mod history;
pub mod message;

use std::time::Duration;

use crate::{
    state::AppState,
    twitch::types::TwitchEvent,
    ui::tabs::chat::{footer::render_chat_footer, header::render_chat_header, history::render_chat_history},
};
use eframe::egui;
use egui_infinite_scroll::InfiniteScroll;
use regex::Regex;

pub struct ChatState {
    pub events: InfiniteScroll<TwitchEvent, usize>,

    pub is_slow_mode: Option<Duration>,
    pub is_emote_only: bool,
    pub is_follow_only: Option<Duration>, // follow duration
    pub is_subscriber_only: bool,

    pub show_messages_by_broadcaster: bool,
    pub show_messages_by_moderator: bool,
    pub show_messages_by_vip: bool,
    pub show_messages_by_subscriber: bool,
    pub show_messages_by_regular_viewer: bool,

    pub show_notices: bool,
    pub show_messages: bool,
    pub show_follows: bool,
    pub show_subscriptions: bool,
    pub show_bits: bool,
    pub show_raids: bool,

    pub user_query: String,
    pub user_query_regex: Option<Regex>,
    pub user_query_valid: bool,
    pub user_query_last: String,

    pub message_query: String,
    pub message_regex: Option<Regex>,
    pub message_query_valid: bool,
    pub message_query_last: String,

    pub message_input: String,
}

impl Default for ChatState {
    fn default() -> Self {
        let events = InfiniteScroll::new().start_loader(|cursor, callback| {
            let page = cursor.unwrap_or(0);
            let items: Vec<TwitchEvent> = vec![];

            callback(Ok((items, Some(page + 1))));
        });

        return Self {
            events,

            is_slow_mode: None,
            is_emote_only: false,
            is_follow_only: None,
            is_subscriber_only: false,

            show_messages_by_broadcaster: true,
            show_messages_by_moderator: true,
            show_messages_by_vip: true,
            show_messages_by_subscriber: true,
            show_messages_by_regular_viewer: true,

            show_notices: true,
            show_messages: true,
            show_follows: true,
            show_subscriptions: true,
            show_bits: true,
            show_raids: true,

            user_query: String::new(),
            user_query_regex: None,
            user_query_valid: true,
            user_query_last: String::new(),

            message_query: String::new(),
            message_regex: None,
            message_query_valid: true,
            message_query_last: String::new(),

            message_input: String::new(),
        };
    }
}

pub fn show_chat_ui(ui: &mut egui::Ui, state: &mut AppState) {
    update_regex_cache(state);

    render_chat_header(ui, state);
    ui.separator();
    render_chat_history(ui, state);
    ui.separator();
    render_chat_footer(ui, state);
}

fn update_regex_cache(state: &mut AppState) {
    if state.chat.user_query != state.chat.user_query_last {
        state.chat.user_query_last = state.chat.user_query.clone();

        if state.chat.user_query.is_empty() {
            state.chat.user_query_regex = None;
            state.chat.user_query_valid = true;
        } else {
            let pattern = format!("(?i){}", state.chat.user_query);
            state.chat.user_query_valid = match regex::Regex::new(&pattern) {
                Ok(re) => {
                    state.chat.user_query_regex = Some(re);
                    true
                }
                Err(_) => {
                    state.chat.user_query_regex = None;
                    false
                }
            };
        }
    }

    if state.chat.message_query != state.chat.message_query_last {
        state.chat.message_query_last = state.chat.message_query.clone();

        if state.chat.message_query.is_empty() {
            state.chat.message_regex = None;
            state.chat.message_query_valid = true;
        } else {
            let pattern = format!("(?i){}", state.chat.message_query);
            state.chat.message_query_valid = match regex::Regex::new(&pattern) {
                Ok(re) => {
                    state.chat.message_regex = Some(re);
                    true
                }
                Err(_) => {
                    state.chat.message_regex = None;
                    false
                }
            };
        }
    }
}
