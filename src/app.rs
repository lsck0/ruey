use std::{sync::mpsc, time::Duration};

use anyhow::Result;
use eframe::{
    CreationContext,
    egui::{Align2, Direction, WidgetText, pos2},
};
use egui_dock::DockState;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use strum::IntoEnumIterator;
use twitch_irc::message::{ClearChatAction, FollowersOnlyMode, IRCMessage, IRCTags, NoticeMessage};

use crate::{
    models::{self, settings::Settings},
    twitch::{
        api::twitch_get_channel_from_login,
        types::{PrivmsgMessageExt, TwitchAccount, TwitchEvent},
    },
    ui::{
        fonts::load_fonts,
        state::{AppState, AppStateDiff},
        tabs::Tabs,
    },
    workers,
};

pub struct App {
    pub tree: DockState<Tabs>,
    pub state: AppState,
}

impl App {
    pub fn new(cctx: &CreationContext) -> Result<Box<Self>> {
        load_fonts(cctx);

        let db_pool = models::create_database_pool()?;
        let channels = workers::create_workers();

        let tree = if let Some(tree_str) = Settings::get_stored_settings(&db_pool).tree
            && let Ok(saved_tree) = serde_json::from_str::<DockState<Tabs>>(&tree_str)
        {
            saved_tree
        } else {
            DockState::new(Tabs::iter().collect())
        };

        let toasts = Toasts::new()
            .anchor(Align2::RIGHT_TOP, pos2(10.0, 10.0))
            .direction(Direction::TopDown);

        return Ok(Box::new(Self {
            tree,
            state: AppState::new(db_pool, channels, toasts),
        }));
    }

    pub fn show_toast(diff_tx: &mpsc::Sender<AppStateDiff>, kind: ToastKind, message: &str) {
        diff_tx
            .send(AppStateDiff::ShowToast(Toast {
                kind,
                text: WidgetText::Text(String::from(message)),
                options: ToastOptions::default().duration_in_seconds(3.0),
                ..Toast::default()
            }))
            .unwrap();
    }

    pub fn apply_state_diff(&mut self, diff: AppStateDiff) {
        match diff {
            AppStateDiff::InternetConnected => {
                self.state.connected_to_internet = true;
            }
            AppStateDiff::InternetDisconnected => {
                self.state.connected_to_internet = false;
            }
            AppStateDiff::SaveSettings => {
                let settings = Settings {
                    id: 1,
                    zoom_factor: Some(self.state.zoom_factor),
                    tree: Some(serde_json::to_string_pretty(&self.tree).expect("Failed to serialize dock state")),
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
            AppStateDiff::ShowToast(toast) => {
                self.state.toasts.add(toast);
            }
            AppStateDiff::AccountLinked(client, token) => {
                Self::show_toast(
                    &self.state.channels.ui_diff_tx,
                    ToastKind::Success,
                    &format!("Logged in as {}.", token.login),
                );

                self.state.twitch_account = Some(TwitchAccount { client, token });
                if let Some(connected_channel_name) = &self.state.connected_channel_name {
                    twitch_get_channel_from_login(
                        &self.state.channels.ui_diff_tx,
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
