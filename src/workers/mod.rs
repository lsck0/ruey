pub mod action;
pub mod asset;
pub mod stats;
pub mod timers;
pub mod twitch;

use std::sync::mpsc;

use crate::{
    twitch::types::TwitchEvent,
    ui::state::AppStateDiff,
    workers::{
        action::worker_start_action, asset::worker_start_assets, stats::worker_start_stats, timers::worker_start_timers,
    },
};

#[derive(Debug)]
pub struct MPSCChannels {
    pub ui_diff_tx: mpsc::Sender<AppStateDiff>,
    pub ui_diff_rx: mpsc::Receiver<AppStateDiff>,
    pub ui_twitch_event_tx: mpsc::Sender<TwitchEvent>,
    pub ui_twitch_event_rx: mpsc::Receiver<TwitchEvent>,
    pub action_worker_tx: mpsc::Sender<TwitchEvent>,
    pub stats_worker_tx: mpsc::Sender<TwitchEvent>,
    pub asset_worker_tx: mpsc::Sender<TwitchEvent>,
    pub twitch_event_txs: Vec<mpsc::Sender<TwitchEvent>>,
}

pub fn create_workers() -> MPSCChannels {
    let (ui_diff_tx, ui_diff_rx) = mpsc::channel::<AppStateDiff>();
    let (ui_twitch_event_tx, ui_twitch_event_rx) = mpsc::channel::<TwitchEvent>();

    let (action_worker_tx, action_worker_rx) = mpsc::channel::<TwitchEvent>();
    let (stats_worker_tx, stats_worker_rx) = mpsc::channel::<TwitchEvent>();
    let (asset_worker_tx, asset_worker_rx) = mpsc::channel::<TwitchEvent>();

    // who wants to hear about twitch events?
    let twitch_event_txs = vec![
        ui_twitch_event_tx.clone(),
        action_worker_tx.clone(),
        stats_worker_tx.clone(),
        asset_worker_tx.clone(),
    ];

    let channels = MPSCChannels {
        ui_diff_tx,
        ui_diff_rx,
        ui_twitch_event_tx,
        ui_twitch_event_rx,
        action_worker_tx,
        stats_worker_tx,
        asset_worker_tx,
        twitch_event_txs,
    };

    worker_start_action(action_worker_rx, channels.ui_diff_tx.clone());
    worker_start_assets(asset_worker_rx, channels.ui_diff_tx.clone());
    worker_start_stats(stats_worker_rx, channels.ui_diff_tx.clone());

    worker_start_timers(channels.ui_diff_tx.clone());

    return channels;
}
