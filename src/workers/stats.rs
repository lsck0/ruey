use std::sync::mpsc;

use crate::{twitch::types::TwitchEvent, ui::state::AppStateDiff};

pub fn worker_start_stats(event_rx: mpsc::Receiver<TwitchEvent>, _state_diff_tx: mpsc::Sender<AppStateDiff>) {
    tokio::task::spawn_blocking(move || {
        while let Ok(_event) = event_rx.recv() {
            //
        }
    });
}
