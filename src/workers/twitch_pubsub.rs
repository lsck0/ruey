use std::sync::mpsc;

use tokio::task::AbortHandle;

use crate::twitch::types::TwitchEvent;

pub fn start_twitch_pubsub_worker(_txs: Vec<mpsc::Sender<TwitchEvent>>) -> Option<AbortHandle> {
    let abort_handle = tokio::spawn(async move {}).abort_handle();

    return Some(abort_handle);
}
