use std::sync::mpsc;

use eframe::egui::WidgetText;
use egui_toast::{Toast, ToastKind};
use twitch_api::{HelixClient, client::ClientDefault, helix::channels::ChannelInformation};
use twitch_oauth2::{DeviceUserTokenBuilder, Scope, UserToken};

use crate::state::{AppStateDiff, TwitchAccount};

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

pub fn twitch_relink_account(diff_tx: &mpsc::Sender<AppStateDiff>, token: &str) {
    let diff_tx = diff_tx.clone();
    let token = token.to_owned();

    tokio::spawn(async move {
        let client: HelixClient<reqwest::Client> =
            twitch_api::HelixClient::with_client(ClientDefault::default_client());

        if let Ok(user_token) = UserToken::from_refresh_token(&client, token.into(), RUEY_CLIENT_ID.into(), None).await
        {
            diff_tx
                .send(AppStateDiff::AccountLinked(client, user_token))
                .expect("Failed to send account linked diff");
        } else {
            diff_tx
                .send(AppStateDiff::ShowToast(Toast {
                    kind: ToastKind::Error,
                    text: WidgetText::Text(String::from("Failed to relink account.")),
                    ..Toast::default()
                }))
                .expect("Failed to send toast");
        }
    });
}

pub fn twitch_get_channel_from_login(diff_tx: &mpsc::Sender<AppStateDiff>, account: &TwitchAccount, channel: &str) {
    let diff_tx = diff_tx.clone();
    let client = account.client.clone();
    let token = account.token.clone();
    let channel = channel.trim().to_string();

    tokio::spawn(async move {
        let Ok(maybe_info) = client.get_channel_from_login(&channel, &token).await else {
            diff_tx
                .send(AppStateDiff::ShowToast(Toast {
                    kind: ToastKind::Error,
                    text: WidgetText::Text(String::from("Failed to get channel information.")),
                    ..Toast::default()
                }))
                .expect("Failed to send toast");
            return;
        };

        match maybe_info {
            Some(channel_info) => {
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
        if client
            .send_chat_message(broadcaster_id, user_id, &*message, &token)
            .await
            .is_err()
        {
            diff_tx
                .send(AppStateDiff::ShowToast(Toast {
                    kind: ToastKind::Error,
                    text: WidgetText::Text(String::from("Failed to send message.")),
                    ..Toast::default()
                }))
                .expect("Failed to send toast");
        }
    });
}

pub fn twitch_delete_message(
    _diff_tx: &mpsc::Sender<AppStateDiff>,
    _account: &TwitchAccount,
    _channel: &ChannelInformation,
    _message_id: &str,
) {
}

pub fn twitch_timeout_user() {}

pub fn twitch_untimeout_user() {}

pub fn twitch_ban_user() {}

pub fn twitch_unban_user() {}

pub fn twitch_shoutout_user() {}
