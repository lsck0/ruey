use egui_file_dialog::FileDialog;
use egui_toast::{Toast, ToastKind, Toasts};
use std::{sync::mpsc, time::Duration};
use tokio::task::AbortHandle;
use twitch_api::{HelixClient, helix::channels::ChannelInformation};
use twitch_irc::message::{ClearChatAction, FollowersOnlyMode, IRCMessage, IRCTags, NoticeMessage};
use twitch_oauth2::UserToken;

use crate::{
    models::{SqlitePool, settings::Settings},
    twitch::{
        api::{twitch_get_channel_from_login, twitch_link_account, twitch_relink_account},
        types::{PrivmsgMessageExt, TwitchEvent},
    },
    ui::{
        tabs::{chat::ChatState, settings::SettingsState},
        util::show_toast,
    },
    workers::twitch_irc::start_twitch_irc_worker,
};

pub struct AppState {
    pub connected_to_internet: bool,

    // global
    pub db_pool: SqlitePool,
    pub zoom_factor: f32,
    pub file_dialog: FileDialog,
    pub toasts: Toasts,

    // twtich worker and information to start/restart them
    pub twitch_irc_worker_handle: Option<AbortHandle>,
    pub twitch_pubsub_worker_handle: Option<AbortHandle>,
    pub diff_tx: mpsc::Sender<AppStateDiff>,
    pub diff_rx: mpsc::Receiver<AppStateDiff>,
    pub event_rx: mpsc::Receiver<TwitchEvent>,
    pub event_worker_txs: Vec<mpsc::Sender<TwitchEvent>>,

    // account and channel
    pub connected_channel_name: Option<String>,
    pub connected_channel_info: Option<ChannelInformation>,
    pub twitch_account: Option<TwitchAccount>,

    // tabs
    pub chat: ChatState,
    pub settings: SettingsState,
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

#[derive(Clone)]
pub struct TwitchAccount {
    pub client: HelixClient<'static, reqwest::Client>,
    pub token: UserToken,
}

impl AppState {
    pub fn new(
        db_pool: SqlitePool,
        toasts: Toasts,
        diff_tx: mpsc::Sender<AppStateDiff>,
        diff_rx: mpsc::Receiver<AppStateDiff>,
        event_rx: mpsc::Receiver<TwitchEvent>,
        twitch_event_txs: Vec<mpsc::Sender<TwitchEvent>>,
    ) -> Self {
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
            diff_tx: diff_tx.clone(),
            diff_rx,
            event_rx,
            event_worker_txs: twitch_event_txs,

            // twitch
            connected_channel_name: None,
            connected_channel_info: None,
            twitch_account: None,

            // tabs
            chat: ChatState::default(),
            settings: SettingsState::default(),
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
            twitch_relink_account(&diff_tx, &access_token, &refresh_token);
        }

        app_state.start_twitch_irc_worker();

        return app_state;
    }

    pub fn start_twitch_irc_worker(&mut self) {
        if self.settings.channel_name.is_empty() {
            return;
        }

        // stop existing worker if it exists
        if let Some(handle) = &self.twitch_irc_worker_handle {
            handle.abort();
            self.chat.events.items.clear();
        }

        // start new worker
        match start_twitch_irc_worker(self.settings.channel_name.clone(), self.event_worker_txs.clone()) {
            Some(handle) => {
                self.twitch_irc_worker_handle = Some(handle);
                self.connected_channel_name = Some(self.settings.channel_name.clone());

                if let Some(account) = &self.twitch_account {
                    twitch_get_channel_from_login(
                        &self.diff_tx,
                        account,
                        self.connected_channel_name.as_ref().expect("unreachable"),
                    );
                }
            }
            None => {
                self.diff_tx
                    .send(AppStateDiff::SetSettingsChannelError(String::from(
                        "Invalid channel name.",
                    )))
                    .expect("unreachable");
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

    pub fn link_twitch_account(&mut self) {
        twitch_link_account(&self.diff_tx);
    }

    pub fn unlink_twitch_account(&mut self) {
        self.twitch_account = None;
    }

    pub fn register_new_event(&mut self, event: TwitchEvent) {
        match event {
            TwitchEvent::Ping(_) => {}
            TwitchEvent::Pong(_) => {}
            TwitchEvent::RoomState(state) => {
                if let Some(duration) = state.slow_mode {
                    if duration.is_zero() {
                        self.chat.is_slow_mode = None;
                    } else {
                        self.chat.is_slow_mode = Some(duration);
                    }
                }

                if let Some(state) = state.emote_only {
                    self.chat.is_emote_only = state;
                }

                if let Some(followers_only_mode) = state.follwers_only {
                    if let FollowersOnlyMode::Enabled(follow_duration) = followers_only_mode {
                        self.chat.is_follow_only = Some(follow_duration);
                    } else {
                        self.chat.is_follow_only = None;
                    }
                }

                if let Some(state) = state.subscribers_only {
                    self.chat.is_subscriber_only = state;
                }
            }
            TwitchEvent::ClearMsg(clear_msg) => {
                for event in self.chat.events.items.iter_mut().rev() {
                    if let TwitchEvent::Privmsg(privmsg) = event
                        && privmsg.message_id == clear_msg.message_id
                    {
                        privmsg.mark_deleted();
                        break;
                    }
                }
            }
            TwitchEvent::ClearChat(clear_chat) => match clear_chat.action {
                ClearChatAction::ChatCleared => {} // ignore
                // low duration timeouts are used to clear messages usually
                ClearChatAction::UserTimedOut {
                    user_id,
                    timeout_length,
                    ..
                } if timeout_length.lt(&Duration::from_secs(5)) => {
                    for event in self.chat.events.items.iter_mut().rev() {
                        if let TwitchEvent::Privmsg(privmsg) = event
                            && privmsg.sender.id == user_id
                        {
                            privmsg.mark_deleted();
                        }
                    }
                }
                ClearChatAction::UserTimedOut {
                    user_login,
                    user_id,
                    timeout_length,
                } => {
                    for event in self.chat.events.items.iter_mut().rev() {
                        if let TwitchEvent::Privmsg(privmsg) = event
                            && privmsg.sender.id == user_id
                        {
                            privmsg.mark_timeouted();
                        }
                    }
                    self.chat.events.items.push(TwitchEvent::Notice(NoticeMessage {
                        channel_login: None,
                        message_id: None,
                        message_text: format!(
                            "{user_login} has been timed out for {} seconds.",
                            timeout_length.as_secs()
                        ),
                        source: IRCMessage {
                            tags: IRCTags::default(),
                            prefix: None,
                            command: String::from("NOTICE"),
                            params: Vec::new(),
                        },
                    }));
                }
                ClearChatAction::UserBanned { user_login, user_id } => {
                    for event in self.chat.events.items.iter_mut().rev() {
                        if let TwitchEvent::Privmsg(privmsg) = event
                            && privmsg.sender.id == user_id
                        {
                            privmsg.mark_banned();
                        }
                    }
                    self.chat.events.items.push(TwitchEvent::Notice(NoticeMessage {
                        channel_login: None,
                        message_id: None,
                        message_text: format!("{user_login} has been banned."),
                        source: IRCMessage {
                            tags: IRCTags::default(),
                            prefix: None,
                            command: String::from("NOTICE"),
                            params: Vec::new(),
                        },
                    }));
                }
            },
            event => {
                self.chat.events.items.push(event);
            }
        }
    }

    pub fn apply_diff(&mut self, diff: AppStateDiff) {
        match diff {
            AppStateDiff::SaveSettings | AppStateDiff::ResetLayout => {} // handled one level up
            AppStateDiff::InternetConnected => {
                self.connected_to_internet = true;
            }
            AppStateDiff::InternetDisconnected => {
                self.connected_to_internet = false;
            }
            AppStateDiff::ShowToast(toast) => {
                self.toasts.add(toast);
            }
            AppStateDiff::AccountLinked(client, token) => {
                show_toast(
                    &self.diff_tx,
                    ToastKind::Success,
                    &format!("Logged in as {}.", token.login),
                );

                self.twitch_account = Some(TwitchAccount { client, token });
                if let Some(connected_channel_name) = &self.connected_channel_name {
                    twitch_get_channel_from_login(
                        &self.diff_tx,
                        self.twitch_account.as_ref().expect("unreachable"),
                        connected_channel_name,
                    );
                } else {
                    self.connected_channel_name = None;
                    self.connected_channel_info = None;
                }
            }
            AppStateDiff::ChannelInfoUpdated(channel_info) => {
                self.connected_channel_info = Some(channel_info);
            }
            AppStateDiff::SetSettingsChannelError(error) => {
                self.connected_channel_name = None;
                self.connected_channel_info = None;
                self.settings.channel_name_error = Some(error);
            }
        }
    }
}
