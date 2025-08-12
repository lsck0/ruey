use egui_file_dialog::FileDialog;
use egui_infinite_scroll::InfiniteScroll;
use regex::Regex;
use std::sync::mpsc;

use diesel::SqliteConnection;

use crate::twitch::events::TwitchEvent;

pub struct AppState {
    // global
    pub db: SqliteConnection,
    pub diff_rx: mpsc::Receiver<AppStateDiff>,
    pub file_dialog: FileDialog,
    pub events: InfiniteScroll<TwitchEvent, usize>,

    // chat
    pub chat_show_messages_by_broadcaster: bool,
    pub chat_show_messages_by_moderator: bool,
    pub chat_show_messages_by_vip: bool,
    pub chat_show_messages_by_subscriber: bool,
    pub chat_show_messages_by_regular_viewer: bool,

    pub chat_show_messages: bool,
    pub chat_show_follows: bool,
    pub chat_show_subscriptions: bool,
    pub chat_show_bits: bool,

    pub chat_user_query: String,
    pub chat_user_query_regex: Option<Regex>,
    pub chat_user_query_valid: bool,
    pub chat_user_query_last: String,

    pub chat_message_query: String,
    pub chat_message_regex: Option<Regex>,
    pub chat_message_query_valid: bool,
    pub chat_message_query_last: String,

    pub chat_message_input: String,
}

pub enum AppStateDiff {
    NewEvent(TwitchEvent),
}

impl AppState {
    pub fn new(db: SqliteConnection, diff_rx: mpsc::Receiver<AppStateDiff>) -> Self {
        let events = InfiniteScroll::new().start_loader(|cursor, callback| {
            let page = cursor.unwrap_or(0);

            let items: Vec<TwitchEvent> = vec![];

            callback(Ok((items, Some(page + 1))));
        });

        Self {
            db,
            diff_rx,
            file_dialog: FileDialog::new(),
            events,

            chat_show_messages_by_broadcaster: true,
            chat_show_messages_by_moderator: true,
            chat_show_messages_by_vip: true,
            chat_show_messages_by_subscriber: true,
            chat_show_messages_by_regular_viewer: true,

            chat_show_messages: true,
            chat_show_follows: true,
            chat_show_subscriptions: true,
            chat_show_bits: true,

            chat_user_query: String::new(),
            chat_user_query_regex: None,
            chat_user_query_valid: true,
            chat_user_query_last: String::new(),

            chat_message_query: String::new(),
            chat_message_regex: None,
            chat_message_query_valid: true,
            chat_message_query_last: String::new(),

            chat_message_input: String::new(),
        }
    }

    pub fn apply_diff(&mut self, diff: AppStateDiff) {
        match diff {
            AppStateDiff::NewEvent(event) => {
                self.events.items.push(event);
            }
        }
    }
}
