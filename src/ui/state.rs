use egui_file_dialog::FileDialog;
use egui_toast::{Toast, Toasts};
use tokio::task::AbortHandle;
use twitch_api::{HelixClient, helix::channels::ChannelInformation};
use twitch_oauth2::UserToken;

use crate::{
    models::{SqlitePool, settings::Settings},
    twitch::{
        api::{twitch_get_channel_from_login, twitch_link_account, twitch_relink_account},
        types::TwitchAccount,
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
    pub fn new(db_pool: SqlitePool, channels: MPSCChannels, toasts: Toasts) -> Self {
        let stored_settings = Settings::get_stored_settings(&db_pool);

        let mut app_state = Self {
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
        };

        if let Some(zoom_factor) = stored_settings.zoom_factor {
            app_state.zoom_factor = zoom_factor;
        }

        if let Some(channel_name) = stored_settings.channel {
            app_state.settings.channel_name = channel_name;
        }

        if let Some(access_token) = stored_settings.user_access_token
            && let Some(refresh_token) = stored_settings.user_refresh_token
        {
            twitch_relink_account(&app_state.channels.ui_diff_tx, &access_token, &refresh_token);
        }

        app_state.start_twitch_irc_worker();

        return app_state;
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

        // start new worker
        match worker_start_twitch_irc(
            self.settings.channel_name.clone(),
            self.channels.twitch_event_txs.clone(),
        ) {
            Ok(handle) => {
                self.twitch_irc_worker_handle = Some(handle);
                self.connected_channel_name = Some(self.settings.channel_name.clone());

                if let Some(account) = &self.twitch_account {
                    twitch_get_channel_from_login(
                        &self.channels.ui_diff_tx,
                        account,
                        self.connected_channel_name.as_ref().unwrap(),
                    );
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
        }

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
        twitch_link_account(&self.channels.ui_diff_tx);
    }

    pub fn unlink_twitch_account(&mut self) {
        self.twitch_account = None;
    }
}
