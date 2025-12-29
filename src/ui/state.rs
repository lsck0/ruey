use anyhow::Result;
use egui_file_dialog::FileDialog;
use egui_toast::{Toast, Toasts};
use tokio::task::AbortHandle;
use twitch_api::{HelixClient, helix::channels::ChannelInformation};
use twitch_irc::message::{IRCMessage, IRCTags, NoticeMessage};
use twitch_oauth2::UserToken;

use crate::{
    models::SqlitePool,
    twitch::{
        api::{twitch_get_channel_from_login, twitch_link_account},
        types::{TwitchAccount, TwitchEvent},
    },
    ui::tabs::{
        actions::ActionsState, chat::ChatState, database::DatabaseState, docs::DocsState, logs::LogsState,
        settings::SettingsState, stats::StatsState,
    },
    workers::{MPSCChannels, twitch::worker_start_twitch_irc},
};

pub struct AppState {
    pub connected_to_internet: bool,
    pub db_pool: SqlitePool,

    // twitch worker and information to start/restart them
    pub channels: MPSCChannels,
    pub twitch_irc_worker_handle: Option<AbortHandle>,
    pub twitch_pubsub_worker_handle: Option<AbortHandle>,

    // account and channel
    pub did_we_try_to_join: bool,
    pub when_did_we_try_to_join: Option<std::time::Instant>,
    pub did_we_join: bool,

    pub connected_channel_name: Option<String>,
    pub connected_channel_info: Option<ChannelInformation>,
    pub twitch_account: Option<TwitchAccount>,

    // global
    pub zoom_factor: f32,
    pub file_dialog: FileDialog,
    pub toasts: Toasts,

    // tabs
    pub chat: ChatState,
    pub stats: StatsState,
    pub actions: ActionsState,
    pub logs: LogsState,
    pub database: DatabaseState,
    pub settings: SettingsState,
    pub docs: DocsState,
}

#[derive(Clone)]
pub enum AppStateDiff {
    InternetConnected,
    InternetDisconnected,
    SaveSettings,
    ResetLayout,

    ShowToast(Toast),

    AccountLinked(HelixClient<'static, reqwest::Client>, UserToken),
    ChannelInfoUpdated(ChannelInformation),

    SetSettingsChannelError(String),
}

impl AppState {
    pub fn new(db_pool: SqlitePool, channels: MPSCChannels, toasts: Toasts) -> Result<Self> {
        return Ok(Self {
            connected_to_internet: true,

            // global
            db_pool,
            zoom_factor: 1.0,
            file_dialog: FileDialog::new(),
            toasts,

            // twitch worker
            twitch_irc_worker_handle: None,
            twitch_pubsub_worker_handle: None,
            channels,

            // twitch
            did_we_try_to_join: false,
            when_did_we_try_to_join: None,
            did_we_join: false,
            connected_channel_name: None,
            connected_channel_info: None,
            twitch_account: None,

            // tabs
            chat: ChatState::default(),
            stats: StatsState::default(),
            actions: ActionsState::default(),
            logs: LogsState::default(),
            database: DatabaseState::default(),
            settings: SettingsState::default(),
            docs: DocsState::default(),
        });
    }

    pub fn show_notice(&mut self, message: String) {
        self.chat.events.items.push(TwitchEvent::Notice(NoticeMessage {
            channel_login: None,
            message_id: None,
            message_text: message,
            source: IRCMessage {
                tags: IRCTags::default(),
                prefix: None,
                command: String::from("NOTICE"),
                params: Vec::new(),
            },
        }));
    }

    pub fn start_twitch_irc_worker(&mut self) {
        if self.settings.channel_name.is_empty() {
            return;
        }

        // stop existing worker
        if let Some(handle) = &self.twitch_irc_worker_handle {
            handle.abort();
            self.chat.events.items.clear();
        }

        // log attempt
        self.did_we_try_to_join = true;
        self.when_did_we_try_to_join = Some(std::time::Instant::now());

        // start new worker
        match worker_start_twitch_irc(
            self.settings.channel_name.clone(),
            self.channels.twitch_event_txs.clone(),
        ) {
            Ok(handle) => {
                self.twitch_irc_worker_handle = Some(handle);
                self.connected_channel_name = Some(self.settings.channel_name.clone());

                if self.twitch_account.is_some() {
                    unsafe {
                        twitch_get_channel_from_login(self, self.connected_channel_name.as_ref().unwrap());
                    }
                }
            }
            Err(_) => {
                self.channels
                    .ui_diff_tx
                    .send(AppStateDiff::SetSettingsChannelError(String::from(
                        "Invalid channel name.",
                    )))
                    .unwrap();
            }
        }
    }

    pub fn stop_twitch_irc_worker(&mut self) {
        if let Some(handle) = &self.twitch_irc_worker_handle {
            handle.abort();
            self.show_notice(format!("Left channel {}.", self.settings.channel_name));
        }

        self.did_we_try_to_join = false;
        self.when_did_we_try_to_join = None;
        self.did_we_join = false;

        self.twitch_irc_worker_handle = None;
        self.connected_channel_name = None;
    }

    pub fn start_twitch_pubsub_worker(&mut self) {}

    pub fn stop_twitch_pubsub_worker(&mut self) {
        if let Some(handle) = &self.twitch_pubsub_worker_handle {
            handle.abort();
        }

        self.twitch_pubsub_worker_handle = None;
    }

    pub fn link_twitch_account(&mut self) {
        twitch_link_account(self);
    }

    pub fn unlink_twitch_account(&mut self) {
        self.twitch_account = None;
    }
}
