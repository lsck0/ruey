pub mod chat;

use eframe::egui;
use egui_commonmark::{CommonMarkCache, commonmark_str};
use strum::{Display, EnumIter, EnumString};

use crate::{state::AppState, ui::tabs::chat::show_chat_ui};

#[derive(Display, EnumString, EnumIter)]
pub enum Tabs {
    Chat,
    Stats,
    Commands,
    Triggers,
    Events,
    Settings,
    Logs,
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
            Tabs::Logs => egui_logger::logger_ui().show(ui),
            Tabs::Docs => {
                commonmark_str!(ui, &mut CommonMarkCache::default(), "./assets/Docs.md");
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
