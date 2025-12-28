use std::{net::TcpStream, sync::mpsc, time::Duration};

use crate::ui::state::AppStateDiff;

pub fn worker_start_timers(ui_diff_tx: mpsc::Sender<AppStateDiff>) {
    // save settings every 30 seconds
    let diff_tx_1 = ui_diff_tx.clone();
    tokio::spawn(async move {
        loop {
            diff_tx_1.send(AppStateDiff::SaveSettings).unwrap();
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });

    // check internet connection every 15 seconds
    let diff_tx_2 = ui_diff_tx.clone();
    tokio::spawn(async move {
        let cloudflare_dns_addr = String::from("1.1.1.1:53").parse().expect("Failed to parse address");

        loop {
            if TcpStream::connect_timeout(&cloudflare_dns_addr, Duration::from_secs(3)).is_ok() {
                diff_tx_2.send(AppStateDiff::InternetConnected).unwrap();
            } else {
                diff_tx_2.send(AppStateDiff::InternetDisconnected).unwrap();
            }

            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });
}
