use egui_file_dialog::FileDialog;
use egui_infinite_scroll::InfiniteScroll;
use regex::Regex;
use std::sync::mpsc;
use tokio::task::AbortHandle;

use diesel::{OptionalExtension, RunQueryDsl, SqliteConnection};

use crate::{
    models::{NewSettings, Settings},
    schema::{self},
    twitch::{events::TwitchEvent, initialize_twitch_worker},
};

pub struct AppState {
    pub connected_to_internet: bool,

    // global
    pub db: SqliteConnection,
    pub zoom_factor: f32,
    pub file_dialog: FileDialog,

    // twtich worker and information to start/restart them
    pub connected_channel: Option<String>,
    pub twitch_worker_handle: Option<AbortHandle>,
    pub diff_tx: mpsc::Sender<AppStateDiff>,
    pub diff_rx: mpsc::Receiver<AppStateDiff>,
    pub event_worker_txs: Vec<mpsc::Sender<TwitchEvent>>,

    // chat
    pub events: InfiniteScroll<TwitchEvent, usize>,

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

    // settings
    pub setting_channel_name: String,
    pub settings_channel_name_error: Option<String>,
}

#[allow(clippy::large_enum_variant)]
pub enum AppStateDiff {
    InternetConnected,
    InternetDisconnected,
    SaveSettings,
    ResetLayout,
    NewEvent(TwitchEvent),
}

impl AppState {
    pub fn new(
        mut db: SqliteConnection,
        diff_tx: mpsc::Sender<AppStateDiff>,
        diff_rx: mpsc::Receiver<AppStateDiff>,
        event_worker_txs: Vec<mpsc::Sender<TwitchEvent>>,
    ) -> Self {
        let events = InfiniteScroll::new().start_loader(|cursor, callback| {
            let page = cursor.unwrap_or(0);

            let items: Vec<TwitchEvent> = vec![];

            callback(Ok((items, Some(page + 1))));
        });

        let stored_settings = Self::get_stored_settings(&mut db);
        let setting_channel_name = stored_settings.channel.unwrap_or_default();
        let zoom_factor = stored_settings.zoom_factor.unwrap_or(1.0);

        return Self {
            connected_to_internet: true,

            // global
            db,
            file_dialog: FileDialog::new(),
            zoom_factor,

            // twitch worker
            connected_channel: None,
            twitch_worker_handle: None,
            diff_tx,
            diff_rx,
            event_worker_txs,

            // chat
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

            // settings
            setting_channel_name,
            settings_channel_name_error: None,
        };
    }

    #[allow(clippy::result_unit_err)]
    pub fn try_start_twitch_worker(&mut self) -> Result<(), String> {
        if self.setting_channel_name.is_empty() {
            return Err(String::from("Empty field."));
        }

        // TODO: api call to check if channel exists

        // stop existing worker if it exists
        if let Some(handle) = &self.twitch_worker_handle {
            handle.abort();
            self.events.items.clear();
        }

        // start new worker
        match initialize_twitch_worker(
            self.setting_channel_name.clone(),
            self.event_worker_txs.clone(),
            self.diff_tx.clone(),
        ) {
            Some(handle) => {
                self.twitch_worker_handle = Some(handle);
                self.connected_channel = Some(self.setting_channel_name.clone());
            }
            None => {
                return Err(String::from("Invalid format."));
            }
        }

        return Ok(());
    }

    pub fn stop_twitch_worker(&mut self) {
        if let Some(handle) = &self.twitch_worker_handle {
            handle.abort();
        }

        self.twitch_worker_handle = None;
        self.connected_channel = None;
        self.events.items.clear();
    }

    pub fn apply_diff(&mut self, diff: AppStateDiff) {
        match diff {
            AppStateDiff::InternetConnected => {
                self.connected_to_internet = true;
            }
            AppStateDiff::InternetDisconnected => {
                self.connected_to_internet = false;
            }
            AppStateDiff::NewEvent(event) => {
                self.events.items.push(event);
            }
            _ => {}
        }
    }

    pub fn get_stored_settings(db: &mut SqliteConnection) -> Settings {
        schema::settings::dsl::settings
            .load::<Settings>(db)
            .optional()
            .unwrap()
            .and_then(|s| s.first().cloned())
            .unwrap_or_else(|| {
                diesel::insert_into(schema::settings::table)
                    .values(NewSettings::default())
                    .execute(db)
                    .unwrap();

                Settings {
                    id: 1,
                    ..Default::default()
                }
            })
    }

    pub fn store_settings(&mut self, tree_str: String) {
        diesel::delete(schema::settings::dsl::settings)
            .execute(&mut self.db)
            .unwrap();
        diesel::insert_into(schema::settings::table)
            .values(NewSettings {
                channel: Some(self.setting_channel_name.clone()),
                tree: Some(tree_str),
                zoom_factor: Some(self.zoom_factor),
            })
            .execute(&mut self.db)
            .unwrap();
    }
}
