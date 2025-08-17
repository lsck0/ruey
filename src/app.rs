use std::{net::TcpStream, sync::mpsc, time::Duration};

use eframe::CreationContext;
use egui_dock::DockState;
use egui_toast::Toasts;
use serde_binary::binary_stream::Endian;
use strum::IntoEnumIterator;

use crate::{
    models::{self, settings::Settings},
    state::{AppState, AppStateDiff},
    twitch::types::TwitchEvent,
    ui::{style::setup_style, tabs::Tabs},
    workers::action::start_action_worker,
};

pub struct App {
    pub tree: DockState<Tabs>,
    pub state: AppState,
}

impl App {
    pub fn new(cctx: &CreationContext) -> Box<Self> {
        setup_style(cctx);

        let db_pool = models::initialize_database();

        let tree = if let Some(tree_str) = Settings::get_stored_settings(&db_pool).tree
            && let Ok(saved_tree) = serde_binary::from_vec::<DockState<Tabs>>(tree_str, Endian::Big)
        {
            saved_tree
        } else {
            DockState::new(Tabs::iter().collect())
        };

        let toasts = Toasts::new();

        let (ui_diff_tx, ui_diff_rx) = mpsc::channel::<AppStateDiff>();
        let (ui_event_tx, ui_event_rx) = mpsc::channel::<TwitchEvent>();
        let (action_worker_tx, action_worker_rx) = mpsc::channel::<TwitchEvent>();
        let twitch_event_txs = vec![ui_event_tx, action_worker_tx];

        start_action_worker(action_worker_rx, ui_diff_tx.clone());
        Self::start_app_timers(ui_diff_tx.clone());

        return Box::new(Self {
            tree,
            state: AppState::new(db_pool, toasts, ui_diff_tx, ui_diff_rx, ui_event_rx, twitch_event_txs),
        });
    }

    fn start_app_timers(ui_state_diff_tx: mpsc::Sender<AppStateDiff>) {
        // save settings every 30 seconds
        let state_diff_tx_1 = ui_state_diff_tx.clone();
        tokio::spawn(async move {
            loop {
                state_diff_tx_1
                    .send(AppStateDiff::SaveSettings)
                    .expect("Failed to send SaveSettings event");
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        // check internet connection every 15 seconds
        let state_diff_tx_2 = ui_state_diff_tx.clone();
        tokio::spawn(async move {
            let addr = String::from("1.1.1.1:53").parse().expect("Failed to parse address"); // cloudflare DNS

            loop {
                if TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok() {
                    state_diff_tx_2
                        .send(AppStateDiff::InternetConnected)
                        .expect("Failed to send InternetConnected event");
                } else {
                    state_diff_tx_2
                        .send(AppStateDiff::InternetDisconnected)
                        .expect("Failed to send InternetDisconnected event");
                }

                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }
}
