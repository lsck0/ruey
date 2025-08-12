pub mod events;

use std::sync::mpsc;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::AbortHandle;
use tracing::trace;
use twitch_irc::{
    ClientConfig, SecureTCPTransport, TwitchIRCClient, login::StaticLoginCredentials, message::ServerMessage,
};

use crate::{state::AppStateDiff, twitch::events::TwitchEvent};

pub fn initialize_twitch_worker(
    channel_name: String,
    txs: Vec<mpsc::Sender<TwitchEvent>>,
    state_diff_tx: mpsc::Sender<AppStateDiff>,
) -> Option<AbortHandle> {
    let config = ClientConfig::default();
    let (incoming_messages, client) = TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    if client.join(channel_name).is_err() {
        return None;
    }

    let abort_handle = tokio::spawn(async move {
        let _client = client; // keep alive

        let join_handle = tokio::spawn(async move {
            irc_message_handler(incoming_messages, &txs, state_diff_tx).await;
        });

        join_handle.await.unwrap();
    })
    .abort_handle();

    return Some(abort_handle);
}

async fn irc_message_handler(
    mut incoming_messages: UnboundedReceiver<ServerMessage>,
    txs: &Vec<mpsc::Sender<TwitchEvent>>,
    state_diff_tx: mpsc::Sender<AppStateDiff>,
) {
    while let Some(message) = incoming_messages.recv().await {
        let event = match TwitchEvent::try_from(message) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        trace!("Received Twtich IRC event: {:?}", event);

        for tx in txs {
            tx.send(event.clone()).unwrap();
        }

        state_diff_tx.send(AppStateDiff::NewEvent(event)).unwrap();
    }
}
