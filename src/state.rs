use diesel::SqliteConnection;
use egui_dock::DockState;
use strum::IntoEnumIterator;

use crate::{models, ui::tabs::Tabs};

pub struct App {
    pub tree: DockState<Tabs>,
    pub state: State,
}

pub struct State {
    pub db: SqliteConnection,
}

impl App {
    pub fn new() -> Box<Self> {
        let tree = DockState::new(Tabs::iter().collect());
        let db = models::initialize_database();

        return Box::new(Self {
            tree,
            state: State { db },
        });
    }
}
