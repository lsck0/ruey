use std::{net::TcpStream, sync::mpsc, time::Duration};

use eframe::CreationContext;
use egui_dock::DockState;
use egui_toast::Toasts;
use strum::IntoEnumIterator;

use crate::{
    actions::initialize_actions_worker,
    models,
    state::{AppState, AppStateDiff},
    stats::initialize_stats_worker,
    ui::{style::setup_style, tabs::Tabs},
};

pub struct App {
    pub tree: DockState<Tabs>,
    pub state: AppState,
}

impl App {
    pub fn new(cctx: &CreationContext) -> Box<Self> {
        setup_style(cctx);

        let db = models::initialize_database();

        let mut tree = DockState::new(Tabs::iter().collect());
        let toasts = Toasts::new();

        let (state_diff_tx, state_diff_rx) = mpsc::channel();
        let mut event_worker_txs = vec![];
        let event_workers = vec![initialize_stats_worker, initialize_actions_worker];

        for worker in event_workers {
            let (tx, rx) = mpsc::channel();
            event_worker_txs.push(tx);
            worker(rx, state_diff_tx.clone());
        }

        Self::start_app_timers(state_diff_tx.clone());

        let mut app_state = AppState::new(db, toasts, state_diff_tx, state_diff_rx, event_worker_txs);

        let settings = AppState::get_stored_settings(&mut app_state.db);

        if let Some(tree_str) = settings.tree
            && let Ok(saved_tree) = serde_json::from_str::<DockState<Tabs>>(&tree_str)
        {
            tree = saved_tree;
        }

        return Box::new(Self { tree, state: app_state });
    }

    fn start_app_timers(state_diff_tx: mpsc::Sender<AppStateDiff>) {
        // save ui related settings every 30 seconds
        let state_diff_tx_1 = state_diff_tx.clone();
        tokio::spawn(async move {
            loop {
                state_diff_tx_1.send(AppStateDiff::SaveSettings).unwrap();
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        // check internet connection every 15 seconds
        let state_diff_tx_2 = state_diff_tx.clone();
        tokio::spawn(async move {
            let addr = String::from("1.1.1.1:53").parse().unwrap();

            loop {
                if TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok() {
                    state_diff_tx_2.send(AppStateDiff::InternetConnected).unwrap();
                } else {
                    state_diff_tx_2.send(AppStateDiff::InternetDisconnected).unwrap();
                }

                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }
}
