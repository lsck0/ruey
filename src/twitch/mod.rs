pub mod events;

use std::sync::mpsc;

use tokio::sync::mpsc::UnboundedReceiver;
use tracing::trace;
use tracing::warn;
use twitch_irc::{
    ClientConfig, SecureTCPTransport, TwitchIRCClient, login::StaticLoginCredentials, message::ServerMessage,
};

use crate::{state::AppStateDiff, twitch::events::TwitchEvent};

pub fn initialize_twitch_worker(
    txs: Vec<mpsc::Sender<TwitchEvent>>,
    state_diff_tx: mpsc::Sender<crate::state::AppStateDiff>,
) {
    tokio::spawn(async {
        let config = ClientConfig::default();
        let (incoming_messages, client) = TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        client.join("fregepaul".to_string()).unwrap();

        let join_handle = tokio::spawn(async move {
            irc_message_handler(incoming_messages, &txs, state_diff_tx).await;
        });

        join_handle.await.unwrap();
    });
}

async fn irc_message_handler(
    mut incoming_messages: UnboundedReceiver<ServerMessage>,
    txs: &Vec<mpsc::Sender<TwitchEvent>>,
    state_diff_tx: mpsc::Sender<crate::state::AppStateDiff>,
) {
    while let Some(message) = incoming_messages.recv().await {
        let event = match TwitchEvent::try_from(message) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        trace!("Received Twtich IRC event: {:?}", event);

        for tx in txs {
            if let Err(e) = tx.send(event.clone()) {
                warn!("Failed to send event to workers from irc_message_handler: {}", e);
            }
        }

        if let Err(e) = state_diff_tx.send(AppStateDiff::NewEvent(event)) {
            warn!("Failed to send state diff from irc_message_handler: {}", e);
        }
    }
}
