use std::sync::mpsc;

use crate::{state::AppStateDiff, twitch::events::TwitchEvent};

pub fn initialize_actions_worker(event_rx: mpsc::Receiver<TwitchEvent>, state_diff_tx: mpsc::Sender<AppStateDiff>) {
    tokio::task::spawn_blocking(move || {
        while let Ok(event) = event_rx.recv() {
            handle_event(event, &state_diff_tx);
        }
    });
}

fn handle_event(_event: TwitchEvent, _state_diff_tx: &mpsc::Sender<AppStateDiff>) {}
