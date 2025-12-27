pub mod actions;
pub mod chat;
pub mod database;
pub mod logs;
pub mod settings;
pub mod stats;

use eframe::egui;
use egui_commonmark::{CommonMarkCache, commonmark_str};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use crate::ui::{
    state::AppState,
    tabs::{chat::show_chat_ui, settings::show_settings_ui},
};

#[derive(Clone, Serialize, Deserialize, Display, EnumIter, EnumString)]
pub enum Tabs {
    Chat,
    Stats,
    Actions,
    Logs,
    Database,
    Settings,
    Docs,
}

pub struct TabViewer<'s> {
    pub state: &'s mut AppState,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tabs;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        return tab.to_string().into();
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tabs::Chat => show_chat_ui(ui, self.state),
            Tabs::Settings => show_settings_ui(ui, self.state),
            Tabs::Docs => {
                commonmark_str!(ui, &mut CommonMarkCache::default(), "./assets/documentation.md");
            }
            _ => {
                ui.label("unimplemented");
            }
        };
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        return false;
    }
}
