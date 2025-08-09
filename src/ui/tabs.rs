use eframe::egui;
use egui_commonmark::{CommonMarkCache, commonmark_str};
use egui_dock::{DockArea, Style};
use strum::{Display, EnumIter, EnumString};

use crate::{App, state::State};

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
    state: &'s mut State,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tabs;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        return tab.to_string().into();
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut TabViewer { state: &mut self.state });
    }
}
