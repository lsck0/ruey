// TODO: REFACTOR THIS FILE
use anyhow::Result;
use diesel::prelude::*;
use egui_dock::DockState;

use crate::{app::App, models::SqlitePool, twitch::api::twitch_relink_account, ui::tabs::Tabs};

#[derive(Debug, Default, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Settings {
    pub id: i32,
    pub zoom_factor: Option<f32>,
    pub tree: Option<String>,
    pub channel: Option<String>,
    pub user_access_token: Option<String>,
    pub user_refresh_token: Option<String>,
}

#[derive(Debug, Default, Clone, Insertable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewSettings {
    pub zoom_factor: Option<f32>,
    pub tree: Option<String>,
    pub channel: Option<String>,
    pub user_access_token: Option<String>,
    pub user_refresh_token: Option<String>,
}

impl Settings {
    pub fn restore_state(app: &mut App) -> Result<()> {
        let stored_settings = Settings::load(&app.state.db_pool)?;

        if let Some(tree_str) = stored_settings.tree {
            let saved_tree = serde_json::from_str::<DockState<Tabs>>(&tree_str)?;
            app.tree = saved_tree;
        }

        if let Some(zoom_factor) = stored_settings.zoom_factor {
            app.state.zoom_factor = zoom_factor;
        }

        if let Some(channel_name) = stored_settings.channel {
            app.state.settings.channel_name = channel_name;
        }

        if let Some(access_token) = stored_settings.user_access_token
            && let Some(refresh_token) = stored_settings.user_refresh_token
        {
            twitch_relink_account(&app.state.channels.ui_diff_tx, &access_token, &refresh_token);
        }

        return Ok(());
    }

    pub fn save_state(app: &App) -> Result<()> {
        let settings = Settings {
            id: 1,
            zoom_factor: Some(app.state.zoom_factor),
            tree: Some(serde_json::to_string_pretty(&app.tree).unwrap()),
            channel: Some(app.state.settings.channel_name.clone()),
            user_access_token: app
                .state
                .twitch_account
                .clone()
                .map(|account| account.token.access_token)
                .map(|token| token.take()),
            user_refresh_token: app
                .state
                .twitch_account
                .clone()
                .and_then(|account| account.token.refresh_token)
                .map(|token| token.take()),
        };
        settings.store(&app.state.db_pool)?;

        return Ok(());
    }

    fn load(pool: &SqlitePool) -> Result<Settings> {
        use crate::schema::settings::dsl::*;

        let mut db = pool.get()?;

        return Ok(settings.first::<Settings>(&mut db).optional()?.unwrap_or_else(|| {
            diesel::insert_into(crate::schema::settings::table)
                .values(NewSettings::default())
                .execute(&mut db)
                .unwrap();

            return Settings {
                id: 1,
                ..Default::default()
            };
        }));
    }

    fn store(&self, pool: &SqlitePool) -> Result<()> {
        use crate::schema::settings::dsl::*;

        let mut db = pool.get()?;

        diesel::delete(settings).execute(&mut db)?;

        diesel::insert_into(crate::schema::settings::table)
            .values(NewSettings {
                zoom_factor: self.zoom_factor,
                tree: self.tree.clone(),
                channel: self.channel.clone(),
                user_access_token: self.user_access_token.clone(),
                user_refresh_token: self.user_refresh_token.clone(),
            })
            .execute(&mut db)?;

        return Ok(());
    }
}
