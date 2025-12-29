use std::{sync::mpsc, time::Duration};

use egui_toast::ToastKind;
use tracing::warn;
use twitch_api::{
    HelixClient,
    client::ClientDefault,
    extra::AnnouncementColor,
    helix::{
        channels::ChannelInformation,
        chat::{SendAShoutoutRequest, UpdateChatSettingsBody, UpdateChatSettingsRequest},
    },
};
use twitch_oauth2::{DeviceUserTokenBuilder, Scope, UserToken};

use crate::{
    app::App,
    twitch::types::TwitchAccount,
    ui::state::{AppState, AppStateDiff},
};

const RUEY_CLIENT_ID: &str = env!("RUEY_CLIENT_ID");

pub fn twitch_link_account(state: &AppState) {
    let ui_diff_tx = state.channels.ui_diff_tx.clone();

    tokio::spawn(async move {
        let client: HelixClient<reqwest::Client> = HelixClient::with_client(ClientDefault::default_client());
        let mut builder = DeviceUserTokenBuilder::new(RUEY_CLIENT_ID, Scope::all());
        let code = builder.start(&client).await.unwrap();

        open::that(code.verification_uri.clone()).unwrap();

        let Ok(token) = builder.wait_for_code(&client, tokio::time::sleep).await else {
            return;
        };

        ui_diff_tx.send(AppStateDiff::AccountLinked(client, token)).unwrap();
    });
}

// BUG: Fix? This sometimes does not work.
pub fn twitch_relink_account(state: &AppState, access_token: &str, refresh_token: &str) {
    let ui_diff_tx = state.channels.ui_diff_tx.clone();
    let access_token = access_token.to_owned();
    let refresh_token = refresh_token.to_owned();

    tokio::spawn(async move {
        let client: HelixClient<reqwest::Client> =
            twitch_api::HelixClient::with_client(ClientDefault::default_client());

        match UserToken::from_existing_or_refresh_token(
            &client,
            access_token.into(),
            refresh_token.into(),
            RUEY_CLIENT_ID.into(),
            None,
        )
        .await
        {
            Ok(user_token) => {
                ui_diff_tx
                    .send(AppStateDiff::AccountLinked(client, user_token))
                    .unwrap();
            }
            Err(err) => {
                warn!("Failed to relink account: {}", err);
                App::show_toast(&ui_diff_tx, ToastKind::Error, "Failed to relink account.");
            }
        }
    });
}

/// # Safety
/// This function assumes a valid twitch account is logged in.
pub unsafe fn twitch_get_channel_from_login(state: &AppState, channel: &str) {
    let ui_diff_tx = state.channels.ui_diff_tx.clone();
    let account = state.twitch_account.as_ref().unwrap();

    let client = account.client.clone();
    let token = account.token.clone();
    let channel = channel.trim().to_string();

    tokio::spawn(async move {
        let maybe_info = match client.get_channel_from_login(&channel, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get channel information: {}", err);
                return;
            }
        };

        match maybe_info {
            Some(channel_info) => {
                ui_diff_tx.send(AppStateDiff::ChannelInfoUpdated(channel_info)).unwrap();
            }
            None => {
                warn!("Channel does not exist: {}", channel);
            }
        }
    });
}

/// # Safety
/// This function assumes a valid twitch account is logged in.
pub unsafe fn twitch_send_message(state: &AppState, message: &str) {
    let ui_diff_tx = state.channels.ui_diff_tx.clone();
    let account = state.twitch_account.as_ref().unwrap();
    let channel = state.connected_channel_info.as_ref().unwrap();

    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let message = message.trim().to_string();

    tokio::spawn(async move {
        match client
            .send_chat_message(broadcaster_id, user_id, &*message, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to send message: {}", err);
                App::show_toast(&ui_diff_tx, ToastKind::Error, "Failed to send message.");
            }
        }
    });
}

/// # Safety
/// This function assumes a valid twitch account is logged in.
pub unsafe fn twitch_send_announcement(state: &AppState, message: &str) {
    let ui_diff_tx = state.channels.ui_diff_tx.clone();
    let account = state.twitch_account.as_ref().unwrap();
    let channel = state.connected_channel_info.as_ref().unwrap();

    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let message = message.trim().to_string();

    tokio::spawn(async move {
        match client
            .send_chat_announcement(broadcaster_id, user_id, &*message, AnnouncementColor::Primary, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to send announcement: {}", err);
                App::show_toast(&ui_diff_tx, ToastKind::Error, "Failed to send announcement.");
            }
        }
    });
}

// TODO: continue refactoring this
/// # Safety
/// This function assumes a valid twitch account is logged in.
pub unsafe fn twitch_delete_message(state: &AppState, message_id: &str) {
    let ui_diff_tx = state.channels.ui_diff_tx.clone();
    let account = state.twitch_account.as_ref().unwrap();
    let channel = state.connected_channel_info.as_ref().unwrap();

    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let message_id = message_id.to_owned();

    tokio::spawn(async move {
        match client
            .delete_chat_message(broadcaster_id, user_id, message_id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to delete message: {}", err);
                App::show_toast(&ui_diff_tx, ToastKind::Error, "Failed to delete message.");
            }
        }
    });
}

pub fn twitch_delete_all_messages(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();

    tokio::spawn(async move {
        match client.delete_all_chat_message(broadcaster_id, user_id, &token).await {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to delete all messages: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
            }
        }
    });
}

pub fn twitch_timeout_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    target_user_name: &str,
    duration: Duration,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = target_user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .ban_user(
                target_user_info.id,
                "",
                Some(duration.as_secs() as u32),
                broadcaster_id,
                user_id,
                &token,
            )
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to timeout user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to timeout user.");
            }
        }
    });
}

pub fn twitch_untimeout_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    target_user_name: &str,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = target_user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .unban_user(target_user_info.id, broadcaster_id, user_id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to untimeout user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to untimeout user.");
            }
        }
    });
}

pub fn twitch_ban_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    target_user_name: &str,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = target_user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .ban_user(target_user_info.id, "", None, broadcaster_id, user_id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to ban user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to ban user.");
            }
        }
    });
}

pub fn twitch_unban_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    user_name: &str,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .unban_user(target_user_info.id, broadcaster_id, user_id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to unban user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to unban user.");
            }
        }
    });
}

pub fn twitch_shoutout_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    target_user_name: &str,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let user_id = account.token.user_id.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = target_user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .req_post(
                SendAShoutoutRequest::new(broadcaster_id, target_user_info.id, user_id),
                Default::default(),
                &token,
            )
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to shoutout user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to shoutout user.");
            }
        }
    });
}

pub fn twitch_vip_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    target_user_name: &str,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = target_user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .add_channel_vip(broadcaster_id, target_user_info.id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to vip user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to vip user.");
            }
        }
    });
}

pub fn twitch_unvip_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    target_user_name: &str,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = target_user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .remove_channel_vip(broadcaster_id, target_user_info.id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to unvip user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to unvip user.");
            }
        }
    });
}

pub fn twitch_mod_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    target_user_name: &str,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = target_user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        // BUG: pretty sure this endpoint is wrongly interacted with by the twitch crate
        match client
            .add_channel_moderator(broadcaster_id, target_user_info.id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to mod user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to mod user.");
            }
        }
    });
}

pub fn twitch_unmod_user(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    target_user_name: &str,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let broadcaster_id = channel.broadcaster_id.clone();
    let target_user_name = target_user_name.to_owned();

    tokio::spawn(async move {
        let target_user_info = match client.get_user_from_login(&target_user_name, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get user information: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            App::show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .remove_channel_moderator(broadcaster_id, target_user_info.id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to unmod user: {}", err);
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to unmod user.");
            }
        }
    });
}

pub fn twitch_patch_chat_settings(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    settings_patch: UpdateChatSettingsBody,
) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let broadcaster_id = channel.broadcaster_id.clone();

    tokio::spawn(async move {
        let request = UpdateChatSettingsRequest::new(broadcaster_id, &token.user_id);

        match client.req_patch(request, settings_patch, &token).await {
            Ok(_) => {}
            Err(_) => {
                warn!("Failed to update chat settings.");
                App::show_toast(&diff_tx, ToastKind::Error, "Failed to update chat settings.");
            }
        }
    });
}
