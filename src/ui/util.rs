use std::sync::mpsc;

use eframe::egui::WidgetText;
use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::state::AppStateDiff;

pub fn show_error_toast(diff_tx: &mpsc::Sender<AppStateDiff>, message: &str) {
    diff_tx
        .send(AppStateDiff::ShowToast(Toast {
            kind: ToastKind::Error,
            text: WidgetText::Text(String::from(message)),
            options: ToastOptions::default().duration_in_seconds(3.0),
            ..Toast::default()
        }))
        .expect("Failed to send toast");
}

pub fn show_not_logged_in_toast(diff_tx: &mpsc::Sender<AppStateDiff>) {
    show_error_toast(diff_tx, "You are not logged in.");
}
