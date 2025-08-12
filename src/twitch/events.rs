use twitch_irc::message::{
    ClearChatMessage, ClearMsgMessage, GlobalUserStateMessage, JoinMessage, NoticeMessage, PartMessage, PingMessage,
    PongMessage, PrivmsgMessage, ReconnectMessage, RoomStateMessage, ServerMessage, UserNoticeMessage,
    UserStateMessage, WhisperMessage,
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TwitchEvent {
    // IRC EVENTS
    ClearChat(ClearChatMessage),
    ClearMsg(ClearMsgMessage),
    GlobalUserState(GlobalUserStateMessage),
    Join(JoinMessage),
    Notice(NoticeMessage),
    Part(PartMessage),
    Ping(PingMessage),
    Pong(PongMessage),
    Privmsg(PrivmsgMessage),
    Reconnect(ReconnectMessage),
    RoomState(RoomStateMessage),
    UserNotice(UserNoticeMessage),
    UserState(UserStateMessage),
    Whisper(WhisperMessage),
    // PUBSUB EVENTS
}

impl TryFrom<ServerMessage> for TwitchEvent {
    type Error = ();

    fn try_from(message: ServerMessage) -> Result<Self, Self::Error> {
        return match message {
            ServerMessage::ClearChat(msg) => Ok(TwitchEvent::ClearChat(msg)),
            ServerMessage::ClearMsg(msg) => Ok(TwitchEvent::ClearMsg(msg)),
            ServerMessage::GlobalUserState(msg) => Ok(TwitchEvent::GlobalUserState(msg)),
            ServerMessage::Join(msg) => Ok(TwitchEvent::Join(msg)),
            ServerMessage::Notice(msg) => Ok(TwitchEvent::Notice(msg)),
            ServerMessage::Part(msg) => Ok(TwitchEvent::Part(msg)),
            ServerMessage::Ping(msg) => Ok(TwitchEvent::Ping(msg)),
            ServerMessage::Pong(msg) => Ok(TwitchEvent::Pong(msg)),
            ServerMessage::Privmsg(msg) => Ok(TwitchEvent::Privmsg(msg)),
            ServerMessage::Reconnect(msg) => Ok(TwitchEvent::Reconnect(msg)),
            ServerMessage::RoomState(msg) => Ok(TwitchEvent::RoomState(msg)),
            ServerMessage::UserNotice(msg) => Ok(TwitchEvent::UserNotice(msg)),
            ServerMessage::UserState(msg) => Ok(TwitchEvent::UserState(msg)),
            ServerMessage::Whisper(msg) => Ok(TwitchEvent::Whisper(msg)),
            _ => Err(()),
        };
    }
}

pub trait PrivmsgMessageExt {
    fn is_by_broadcaster(&self) -> bool;
    fn is_by_mod(&self) -> bool;
    fn is_by_vip(&self) -> bool;
    fn is_by_subscriber(&self) -> bool;
    fn is_by_regular_viewer(&self) -> bool;
    fn is_first_message(&self) -> bool;
}

impl PrivmsgMessageExt for &PrivmsgMessage {
    fn is_by_broadcaster(&self) -> bool {
        self.badges.iter().any(|badge| badge.name == "broadcaster")
    }

    fn is_by_mod(&self) -> bool {
        self.badges.iter().any(|badge| badge.name == "moderator")
    }

    fn is_by_vip(&self) -> bool {
        self.badges.iter().any(|badge| badge.name == "vip")
    }

    fn is_by_subscriber(&self) -> bool {
        self.badges.iter().any(|badge| badge.name == "subscriber")
    }

    fn is_by_regular_viewer(&self) -> bool {
        return !self.is_by_broadcaster() && !self.is_by_mod() && !self.is_by_vip() && !self.is_by_subscriber();
    }

    fn is_first_message(&self) -> bool {
        self.source
            .tags
            .0
            .get("first-msg")
            .is_some_and(|val| val.as_ref().is_some_and(|v| v.eq("1")))
    }
}
