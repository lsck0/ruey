use std::sync::mpsc;

use tokio::task::AbortHandle;
use tracing::trace;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient, login::StaticLoginCredentials};

use crate::twitch::types::TwitchEvent;

pub fn start_twitch_irc_worker(channel_name: String, txs: Vec<mpsc::Sender<TwitchEvent>>) -> Option<AbortHandle> {
    let config = ClientConfig::default();
    let (incoming_messages, client) = TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    if client.join(channel_name).is_err() {
        return None;
    }

    let abort_handle = tokio::spawn(async move {
        let mut incoming_messages = incoming_messages;
        let _client = client; // keep alive

        while let Some(message) = incoming_messages.recv().await {
            let event = match TwitchEvent::try_from(message) {
                Ok(msg) => msg,
                Err(_) => continue,
            };

            trace!("Received Twtich IRC event: {:?}", event);

            for tx in &txs {
                tx.send(event.clone()).expect("Failed to send Twitch event");
            }
        }
    })
    .abort_handle();

    return Some(abort_handle);
}
