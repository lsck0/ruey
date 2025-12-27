use std::{net::TcpStream, sync::mpsc, time::Duration};

use eframe::{
    CreationContext, NativeOptions,
    egui::{Align2, Direction, ViewportBuilder, pos2},
};
use egui_dock::DockState;
use egui_toast::{ToastKind, Toasts};
use serde_binary::binary_stream::Endian;
use strum::IntoEnumIterator;
use twitch_irc::message::{ClearChatAction, FollowersOnlyMode, IRCMessage, IRCTags, NoticeMessage};

use crate::{
    models::{self, settings::Settings},
    twitch::{
        api::twitch_get_channel_from_login,
        types::{PrivmsgMessageExt, TwitchAccount, TwitchEvent},
    },
    ui::{
        state::{AppState, AppStateDiff},
        style::load_fonts,
        tabs::Tabs,
        util::show_toast,
    },
    workers::action::start_action_worker,
};

pub struct App {
    pub tree: DockState<Tabs>,
    pub state: AppState,
}

impl App {
    pub fn new(cctx: &CreationContext) -> Box<Self> {
        load_fonts(cctx);

        let db_pool = models::initialize_database();

        // BUG: this does not restore the tabs properly
        let tree = if let Some(tree_str) = Settings::get_stored_settings(&db_pool).tree
            && let Ok(saved_tree) = serde_binary::from_vec::<DockState<Tabs>>(tree_str, Endian::Big)
        {
            saved_tree
        } else {
            DockState::new(Tabs::iter().collect())
        };

        let toasts = Toasts::new()
            .anchor(Align2::RIGHT_TOP, pos2(10.0, 10.0))
            .direction(Direction::TopDown);

        let (ui_diff_tx, ui_diff_rx) = mpsc::channel::<AppStateDiff>();
        let (ui_event_tx, ui_event_rx) = mpsc::channel::<TwitchEvent>();
        let (action_worker_tx, action_worker_rx) = mpsc::channel::<TwitchEvent>();
        let twitch_event_txs = vec![ui_event_tx, action_worker_tx];

        start_action_worker(action_worker_rx, ui_diff_tx.clone());
        Self::start_background_tasks(ui_diff_tx.clone());

        return Box::new(Self {
            tree,
            state: AppState::new(db_pool, toasts, ui_diff_tx, ui_diff_rx, ui_event_rx, twitch_event_txs),
        });
    }

    fn start_background_tasks(ui_state_diff_tx: mpsc::Sender<AppStateDiff>) {
        // save settings every 30 seconds
        let state_diff_tx_1 = ui_state_diff_tx.clone();
        tokio::spawn(async move {
            loop {
                state_diff_tx_1
                    .send(AppStateDiff::SaveSettings)
                    .expect("Failed to send SaveSettings event");
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        // check internet connection every 15 seconds
        let state_diff_tx_2 = ui_state_diff_tx.clone();
        tokio::spawn(async move {
            let addr = String::from("1.1.1.1:53").parse().expect("Failed to parse address"); // cloudflare DNS

            loop {
                if TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok() {
                    state_diff_tx_2
                        .send(AppStateDiff::InternetConnected)
                        .expect("Failed to send InternetConnected event");
                } else {
                    state_diff_tx_2
                        .send(AppStateDiff::InternetDisconnected)
                        .expect("Failed to send InternetDisconnected event");
                }

                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }

    pub fn apply_state_diff(&mut self, diff: AppStateDiff) {
        match diff {
            AppStateDiff::SaveSettings => {
                let settings = Settings {
                    id: 1,
                    zoom_factor: Some(self.state.zoom_factor),
                    tree: Some(serde_binary::to_vec(&self.tree, Endian::Big).expect("Failed to serialize dock state")),
                    channel: Some(self.state.settings.channel_name.clone()),
                    user_access_token: self
                        .state
                        .twitch_account
                        .clone()
                        .map(|account| account.token.access_token)
                        .map(|token| token.take()),
                    user_refresh_token: self
                        .state
                        .twitch_account
                        .clone()
                        .and_then(|account| account.token.refresh_token)
                        .map(|token| token.take()),
                };
                settings.store_settings(&self.state.db_pool);
            }
            AppStateDiff::ResetLayout => {
                self.tree = DockState::new(Tabs::iter().collect());
            }
            AppStateDiff::InternetConnected => {
                self.state.connected_to_internet = true;
            }
            AppStateDiff::InternetDisconnected => {
                self.state.connected_to_internet = false;
            }
            AppStateDiff::ShowToast(toast) => {
                self.state.toasts.add(toast);
            }
            AppStateDiff::AccountLinked(client, token) => {
                show_toast(
                    &self.state.diff_tx,
                    ToastKind::Success,
                    &format!("Logged in as {}.", token.login),
                );

                self.state.twitch_account = Some(TwitchAccount { client, token });
                if let Some(connected_channel_name) = &self.state.connected_channel_name {
                    twitch_get_channel_from_login(
                        &self.state.diff_tx,
                        self.state.twitch_account.as_ref().expect("unreachable"),
                        connected_channel_name,
                    );
                } else {
                    self.state.connected_channel_name = None;
                    self.state.connected_channel_info = None;
                }
            }
            AppStateDiff::ChannelInfoUpdated(channel_info) => {
                self.state.connected_channel_info = Some(channel_info);
            }
            AppStateDiff::SetSettingsChannelError(error) => {
                self.state.connected_channel_name = None;
                self.state.connected_channel_info = None;
                self.state.settings.channel_name_error = Some(error);
            }
        }
    }

    pub fn register_new_twitch_event(&mut self, event: TwitchEvent) {
        match event {
            TwitchEvent::Ping(_) => {}
            TwitchEvent::Pong(_) => {}
            TwitchEvent::RoomState(state) => {
                if let Some(duration) = state.slow_mode {
                    if duration.is_zero() {
                        self.state.chat.is_slow_mode = None;
                    } else {
                        self.state.chat.is_slow_mode = Some(duration);
                    }
                }

                if let Some(state) = state.emote_only {
                    self.state.chat.is_emote_only = state;
                }

                if let Some(followers_only_mode) = state.follwers_only {
                    if let FollowersOnlyMode::Enabled(follow_duration) = followers_only_mode {
                        self.state.chat.is_follow_only = Some(follow_duration);
                    } else {
                        self.state.chat.is_follow_only = None;
                    }
                }

                if let Some(state) = state.subscribers_only {
                    self.state.chat.is_subscriber_only = state;
                }
            }
            TwitchEvent::ClearMsg(clear_msg) => {
                for event in self.state.chat.events.items.iter_mut().rev() {
                    if let TwitchEvent::Privmsg(privmsg) = event
                        && privmsg.message_id == clear_msg.message_id
                    {
                        privmsg.mark_deleted();
                        break;
                    }
                }

                self.state.chat.events.items.push(TwitchEvent::Notice(NoticeMessage {
                    channel_login: None,
                    message_id: None,
                    message_text: format!(
                        "Message deleted: {}: {}",
                        clear_msg.sender_login, clear_msg.message_text
                    ),
                    source: IRCMessage {
                        tags: IRCTags::default(),
                        prefix: None,
                        command: String::from("NOTICE"),
                        params: Vec::new(),
                    },
                }));
            }
            TwitchEvent::ClearChat(clear_chat) => match clear_chat.action {
                ClearChatAction::ChatCleared => {
                    // dont actually remove the messages, since this is a streamer tool in the
                    // first place
                    self.state.chat.events.items.push(TwitchEvent::Notice(NoticeMessage {
                        channel_login: None,
                        message_id: None,
                        message_text: String::from("Chat has been cleared."),
                        source: IRCMessage {
                            tags: IRCTags::default(),
                            prefix: None,
                            command: String::from("NOTICE"),
                            params: Vec::new(),
                        },
                    }));
                }
                // low duration timeouts are used to clear messages usually
                ClearChatAction::UserTimedOut {
                    user_login,
                    user_id,
                    timeout_length,
                    ..
                } if timeout_length.lt(&Duration::from_secs(5)) => {
                    for event in self.state.chat.events.items.iter_mut().rev() {
                        if let TwitchEvent::Privmsg(privmsg) = event
                            && privmsg.sender.id == user_id
                        {
                            privmsg.mark_deleted();
                        }
                    }
                    self.state.chat.events.items.push(TwitchEvent::Notice(NoticeMessage {
                        channel_login: None,
                        message_id: None,
                        message_text: format!("{user_login}'s messages have been deleted."),
                        source: IRCMessage {
                            tags: IRCTags::default(),
                            prefix: None,
                            command: String::from("NOTICE"),
                            params: Vec::new(),
                        },
                    }));
                }
                ClearChatAction::UserTimedOut {
                    user_login,
                    user_id,
                    timeout_length,
                } => {
                    for event in self.state.chat.events.items.iter_mut().rev() {
                        if let TwitchEvent::Privmsg(privmsg) = event
                            && privmsg.sender.id == user_id
                        {
                            privmsg.mark_timeouted();
                        }
                    }
                    self.state.chat.events.items.push(TwitchEvent::Notice(NoticeMessage {
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
                    for event in self.state.chat.events.items.iter_mut().rev() {
                        if let TwitchEvent::Privmsg(privmsg) = event
                            && privmsg.sender.id == user_id
                        {
                            privmsg.mark_banned();
                        }
                    }
                    self.state.chat.events.items.push(TwitchEvent::Notice(NoticeMessage {
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
                self.state.chat.events.items.push(event);
            }
        }
    }
}
