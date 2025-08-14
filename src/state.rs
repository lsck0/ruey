use egui_file_dialog::FileDialog;
use egui_toast::{Toast, Toasts};
use std::{sync::mpsc, time::Duration};
use tokio::task::AbortHandle;
use twitch_api::{HelixClient, client::ClientDefault, helix::channels::ChannelInformation};
use twitch_irc::message::{ClearChatAction, FollowersOnlyMode, IRCMessage, IRCTags, NoticeMessage};
use twitch_oauth2::{DeviceUserTokenBuilder, Scope, UserToken};

use diesel::{OptionalExtension, RunQueryDsl, SqliteConnection};

use crate::{
    models::{NewSettings, Settings},
    schema::{self},
    twitch::{
        events::{PrivmsgMessageExt, TwitchEvent},
        initialize_twitch_worker,
    },
    ui::tabs::{chat::ChatState, settings::SettingsState},
};

const MY_CLIENT_ID: &str = "cqrt6xvgzu8au325zz4zpk6uueg15u";

pub struct AppState {
    pub connected_to_internet: bool,

    // global
    pub db: SqliteConnection,
    pub zoom_factor: f32,
    pub file_dialog: FileDialog,
    pub toasts: Toasts,

    // twtich worker and information to start/restart them
    pub connected_channel: Option<String>,
    pub twitch_worker_handle: Option<AbortHandle>,
    pub diff_tx: mpsc::Sender<AppStateDiff>,
    pub diff_rx: mpsc::Receiver<AppStateDiff>,
    pub event_worker_txs: Vec<mpsc::Sender<TwitchEvent>>,

    // account
    pub twitch_account: Option<TwitchAccount>,

    // tabs
    pub chat: ChatState,
    pub settings: SettingsState,
}

#[allow(clippy::large_enum_variant)]
pub enum AppStateDiff {
    InternetConnected,
    InternetDisconnected,
    SaveSettings,
    ResetLayout,

    ShowToast(Toast),

    AccountLinked(HelixClient<'static, reqwest::Client>, UserToken),
    ChannelInfoUpdated(ChannelInformation),
    SetSettingsChannelError(String),

    NewEvent(TwitchEvent),
}

pub struct TwitchAccount {
    pub client: HelixClient<'static, reqwest::Client>,
    pub token: UserToken,
    pub channel: Option<ChannelInformation>,
}

impl AppState {
    pub fn new(
        mut db: SqliteConnection,
        toasts: Toasts,
        diff_tx: mpsc::Sender<AppStateDiff>,
        diff_rx: mpsc::Receiver<AppStateDiff>,
        event_worker_txs: Vec<mpsc::Sender<TwitchEvent>>,
    ) -> Self {
        let stored_settings = Self::get_stored_settings(&mut db);

        let mut app_state = Self {
            connected_to_internet: true,

            // global
            db,
            zoom_factor: 1.0,
            file_dialog: FileDialog::new(),
            toasts,

            // twitch worker
            connected_channel: None,
            twitch_worker_handle: None,
            diff_tx: diff_tx.clone(),
            diff_rx,
            event_worker_txs,

            // twitch
            twitch_account: None,

            // tabs
            chat: ChatState::default(),
            settings: SettingsState::default(),
        };

        let _ = app_state.try_start_twitch_worker();

        if let Some(channel_name) = stored_settings.channel {
            app_state.settings.channel_name = channel_name;
        }

        if let Some(refresh_token) = stored_settings.user_refresh_token {
            tokio::spawn(async move {
                let client: HelixClient<reqwest::Client> =
                    twitch_api::HelixClient::with_client(ClientDefault::default_client());

                if let Ok(user_token) =
                    UserToken::from_refresh_token(&client, refresh_token.into(), MY_CLIENT_ID.into(), None).await
                {
                    diff_tx.send(AppStateDiff::AccountLinked(client, user_token)).unwrap();
                }
            });
        }

        if let Some(zoom_factor) = stored_settings.zoom_factor {
            app_state.zoom_factor = zoom_factor;
        }

        return app_state;
    }

    #[allow(clippy::result_unit_err)]
    pub fn try_start_twitch_worker(&mut self) -> Result<(), String> {
        if self.settings.channel_name.is_empty() {
            return Err(String::from("Empty field."));
        }

        // stop existing worker if it exists
        if let Some(handle) = &self.twitch_worker_handle {
            handle.abort();
            self.chat.events.items.clear();
        }

        // start new worker
        match initialize_twitch_worker(
            self.settings.channel_name.clone(),
            self.event_worker_txs.clone(),
            self.diff_tx.clone(),
        ) {
            Some(handle) => {
                self.twitch_worker_handle = Some(handle);
                self.connected_channel = Some(self.settings.channel_name.clone());

                if let Some(account) = &self.twitch_account {
                    let local_diff_tx = self.diff_tx.clone();
                    let local_connected_channel = self.connected_channel.clone();
                    let local_client = account.client.clone();
                    let local_token = account.token.clone();

                    tokio::spawn(async move {
                        if let Some(connected_channel) = local_connected_channel
                            && let Ok(maybe_channel_info) = local_client
                                .get_channel_from_login(&connected_channel, &local_token)
                                .await
                        {
                            if let Some(channel_info) = maybe_channel_info {
                                local_diff_tx
                                    .send(AppStateDiff::ChannelInfoUpdated(channel_info))
                                    .unwrap();
                            } else {
                                local_diff_tx
                                    .send(AppStateDiff::SetSettingsChannelError(String::from(
                                        "Channel not found.",
                                    )))
                                    .unwrap();
                            }
                        }
                    });
                }
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
    }

    pub fn link_twitch_account(&mut self) {
        let local_connected_channel = self.connected_channel.clone();
        let local_diff_tx = self.diff_tx.clone();

        tokio::spawn(async move {
            let client: HelixClient<reqwest::Client> = HelixClient::with_client(ClientDefault::default_client());
            let mut builder = DeviceUserTokenBuilder::new(MY_CLIENT_ID, Scope::all());
            let code = builder.start(&client).await.unwrap();

            open::that(code.verification_uri.clone()).unwrap();

            let Ok(token) = builder.wait_for_code(&client, tokio::time::sleep).await else {
                return;
            };

            local_diff_tx
                .send(AppStateDiff::AccountLinked(client.clone(), token.clone()))
                .unwrap();

            if let Some(connected_channel) = local_connected_channel
                && let Ok(maybe_channel_info) = client.get_channel_from_login(&connected_channel, &token).await
            {
                if let Some(channel_info) = maybe_channel_info {
                    local_diff_tx
                        .send(AppStateDiff::ChannelInfoUpdated(channel_info))
                        .unwrap();
                } else {
                    local_diff_tx
                        .send(AppStateDiff::SetSettingsChannelError(String::from(
                            "Channel not found.",
                        )))
                        .unwrap();
                }
            }
        });
    }

    pub fn unlink_twitch_account(&mut self) {
        self.twitch_account = None;
    }

    pub fn apply_diff(&mut self, diff: AppStateDiff) {
        match diff {
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
                self.twitch_account = Some(TwitchAccount {
                    client,
                    token,
                    channel: None,
                });
            }
            AppStateDiff::ChannelInfoUpdated(channel_info) => {
                if let Some(account) = &mut self.twitch_account {
                    account.channel = Some(channel_info);
                }
            }
            AppStateDiff::SetSettingsChannelError(error) => {
                self.connected_channel = None;
                self.settings.channel_name_error = Some(error);
            }
            AppStateDiff::NewEvent(event) => match event {
                TwitchEvent::Ping(_) => {}
                TwitchEvent::Pong(_) => {}
                TwitchEvent::RoomState(state) => {
                    self.chat.is_slow_mode = state.slow_mode;
                    self.chat.is_emote_only = state.emote_only.unwrap_or(false);

                    // thank you twitch
                    if let Some(followers_only_mode) = state.follwers_only
                        && let FollowersOnlyMode::Enabled(follow_duration) = followers_only_mode
                        && !follow_duration.is_zero()
                    {
                        self.chat.is_follow_only = Some(follow_duration);
                    } else {
                        self.chat.is_follow_only = None;
                    }

                    self.chat.is_subscriber_only = state.subscribers_only.unwrap_or(false);
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
            },
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
                channel: Some(self.settings.channel_name.clone()),
                user_refresh_token: self
                    .twitch_account
                    .as_ref()
                    .and_then(|acc| acc.token.refresh_token.clone())
                    .map(|token| token.take()),
                tree: Some(tree_str),
                zoom_factor: Some(self.zoom_factor),
            })
            .execute(&mut self.db)
            .unwrap();
    }
}
