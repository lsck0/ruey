use std::sync::mpsc;

use eframe::CreationContext;
use egui_dock::DockState;
use strum::IntoEnumIterator;

use crate::{
    actions::initialize_actions_worker,
    models,
    state::AppState,
    stats::initialize_stats_worker,
    twitch::initialize_twitch_worker,
    ui::{style::setup_style, tabs::Tabs},
};

pub struct App {
    pub tree: DockState<Tabs>,
    pub state: AppState,
}

impl App {
    pub fn new(cctx: &CreationContext) -> Box<Self> {
        setup_style(cctx);

        let tree = DockState::new(Tabs::iter().collect());
        let db = models::initialize_database();

        let (state_diff_tx, state_diff_rx) = mpsc::channel();
        let mut event_worker_txs = vec![];
        let event_workers = vec![initialize_stats_worker, initialize_actions_worker];

        for worker in event_workers {
            let (tx, rx) = mpsc::channel();
            event_worker_txs.push(tx);
            worker(rx, state_diff_tx.clone());
        }

        initialize_twitch_worker(event_worker_txs, state_diff_tx.clone());

        return Box::new(Self {
            tree,
            state: AppState::new(db, state_diff_rx),
        });
    }
}
