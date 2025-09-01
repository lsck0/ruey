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
    state::{AppStateDiff, TwitchAccount},
    ui::util::show_toast,
};

const RUEY_CLIENT_ID: &str = env!("RUEY_CLIENT_ID");

pub fn twitch_link_account(diff_tx: &mpsc::Sender<AppStateDiff>) {
    let diff_tx = diff_tx.clone();

    tokio::spawn(async move {
        let client: HelixClient<reqwest::Client> = HelixClient::with_client(ClientDefault::default_client());
        let mut builder = DeviceUserTokenBuilder::new(RUEY_CLIENT_ID, Scope::all());
        let code = builder
            .start(&client)
            .await
            .expect("Failed to start device user token builder");

        open::that(code.verification_uri.clone()).expect("Failed to open verification URI");

        let Ok(token) = builder.wait_for_code(&client, tokio::time::sleep).await else {
            return;
        };

        diff_tx
            .send(AppStateDiff::AccountLinked(client, token))
            .expect("Failed to send account linked diff");
    });
}

pub fn twitch_relink_account(diff_tx: &mpsc::Sender<AppStateDiff>, access_token: &str, refresh_token: &str) {
    let diff_tx = diff_tx.clone();
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
                diff_tx
                    .send(AppStateDiff::AccountLinked(client, user_token))
                    .expect("Failed to send account linked diff");
            }
            Err(err) => {
                warn!("Failed to relink account: {}", err);
                show_toast(&diff_tx, ToastKind::Error, "Failed to relink account.");
            }
        }
    });
}

pub fn twitch_get_channel_from_login(diff_tx: &mpsc::Sender<AppStateDiff>, account: &TwitchAccount, channel: &str) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let channel = channel.trim().to_string();

    tokio::spawn(async move {
        let maybe_info = match client.get_channel_from_login(&channel, &token).await {
            Ok(info) => info,
            Err(err) => {
                warn!("Failed to get channel information: {}", err);
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        match maybe_info {
            Some(channel_info) => {
                show_toast(
                    &diff_tx,
                    ToastKind::Success,
                    &format!("Connected to channel {}.", channel),
                );
                diff_tx
                    .send(AppStateDiff::ChannelInfoUpdated(channel_info))
                    .expect("Failed to send channel information");
            }
            None => {
                diff_tx
                    .send(AppStateDiff::SetSettingsChannelError(String::from(
                        "Channel not found.",
                    )))
                    .expect("Failed to send channel not found error");
            }
        }
    });
}

pub fn twitch_send_message(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    message: &str,
) {
    let diff_tx = diff_tx.clone();
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to send message.");
            }
        }
    });
}

pub fn twitch_send_announcement(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    message: &str,
) {
    let diff_tx = diff_tx.clone();
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to send announcement.");
            }
        }
    });
}

pub fn twitch_delete_message(
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &TwitchAccount,
    channel: &ChannelInformation,
    message_id: &str,
) {
    let diff_tx = diff_tx.clone();
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to delete message.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to timeout user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .unban_user(target_user_info.id, broadcaster_id, user_id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to untimeout user: {}", err);
                show_toast(&diff_tx, ToastKind::Error, "Failed to untimeout user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .ban_user(target_user_info.id, "", None, broadcaster_id, user_id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to ban user: {}", err);
                show_toast(&diff_tx, ToastKind::Error, "Failed to ban user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .unban_user(target_user_info.id, broadcaster_id, user_id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to unban user: {}", err);
                show_toast(&diff_tx, ToastKind::Error, "Failed to unban user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to shoutout user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .add_channel_vip(broadcaster_id, target_user_info.id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to vip user: {}", err);
                show_toast(&diff_tx, ToastKind::Error, "Failed to vip user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .remove_channel_vip(broadcaster_id, target_user_info.id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to unvip user: {}", err);
                show_toast(&diff_tx, ToastKind::Error, "Failed to unvip user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to mod user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to get user information.");
                return;
            }
        };

        let Some(target_user_info) = target_user_info else {
            show_toast(&diff_tx, ToastKind::Error, "User not found.");
            return;
        };

        match client
            .remove_channel_moderator(broadcaster_id, target_user_info.id, &token)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to unmod user: {}", err);
                show_toast(&diff_tx, ToastKind::Error, "Failed to unmod user.");
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
                show_toast(&diff_tx, ToastKind::Error, "Failed to update chat settings.");
            }
        }
    });
}
