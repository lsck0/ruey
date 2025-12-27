use std::sync::mpsc;

use eframe::egui::WidgetText;
use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::ui::state::AppStateDiff;

pub fn show_toast(diff_tx: &mpsc::Sender<AppStateDiff>, kind: ToastKind, message: &str) {
    diff_tx
        .send(AppStateDiff::ShowToast(Toast {
            kind,
            text: WidgetText::Text(String::from(message)),
            options: ToastOptions::default().duration_in_seconds(3.0),
            ..Toast::default()
        }))
        .expect("Failed to send toast");
}
