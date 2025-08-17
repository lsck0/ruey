use std::sync::mpsc;

use crate::{state::AppStateDiff, twitch::types::TwitchEvent};

pub fn start_action_worker(event_rx: mpsc::Receiver<TwitchEvent>, _state_diff_tx: mpsc::Sender<AppStateDiff>) {
    tokio::task::spawn_blocking(move || {
        while let Ok(_event) = event_rx.recv() {
            // todo
        }
    });
}
