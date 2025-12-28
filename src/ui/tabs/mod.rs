pub mod actions;
pub mod chat;
pub mod database;
pub mod docs;
pub mod logs;
pub mod settings;
pub mod stats;

use eframe::egui;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use crate::ui::{
    state::AppState,
    tabs::{
        actions::show_actions_ui, chat::show_chat_ui, database::show_database_ui, docs::show_docs_ui,
        logs::show_logs_ui, settings::show_settings_ui, stats::show_stats_ui,
    },
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
            Tabs::Actions => show_actions_ui(ui, self.state),
            Tabs::Chat => show_chat_ui(ui, self.state),
            Tabs::Database => show_database_ui(ui, self.state),
            Tabs::Docs => show_docs_ui(ui, self.state),
            Tabs::Logs => show_logs_ui(ui, self.state),
            Tabs::Settings => show_settings_ui(ui, self.state),
            Tabs::Stats => show_stats_ui(ui, self.state),
        };
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        return false;
    }
}
