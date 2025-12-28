use std::sync::mpsc;

use anyhow::Result;
use tokio::task::AbortHandle;
use tracing::trace;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient, login::StaticLoginCredentials};

use crate::twitch::types::TwitchEvent;

pub fn worker_start_twitch_irc(channel_name: String, txs: Vec<mpsc::Sender<TwitchEvent>>) -> Result<AbortHandle> {
    let config = ClientConfig::default();
    let (incoming_messages, client) = TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    client.join(channel_name)?;

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
                tx.send(event.clone()).unwrap();
            }
        }
    })
    .abort_handle();

    return Ok(abort_handle);
}

pub fn worker_start_twitch_pubsub(_txs: Vec<mpsc::Sender<TwitchEvent>>) -> Result<AbortHandle> {
    let abort_handle = tokio::spawn(async move {}).abort_handle();

    return Ok(abort_handle);
}
