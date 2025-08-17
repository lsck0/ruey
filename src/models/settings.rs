use diesel::prelude::*;

use crate::models::SqlitePool;

#[derive(Debug, Default, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Settings {
    pub id: i32,
    pub zoom_factor: Option<f32>,
    pub tree: Option<Vec<u8>>,
    pub channel: Option<String>,
    pub user_refresh_token: Option<String>,
}

#[derive(Debug, Default, Clone, Insertable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewSettings {
    pub zoom_factor: Option<f32>,
    pub tree: Option<Vec<u8>>,
    pub channel: Option<String>,
    pub user_refresh_token: Option<String>,
}

impl Settings {
    pub fn get_stored_settings(pool: &SqlitePool) -> Settings {
        use crate::schema::settings::dsl::*;

        let mut db = pool.get().expect("Failed to get a connection from the pool.");

        return settings
            .first::<Settings>(&mut db)
            .optional()
            .expect("Failed to query settings")
            .unwrap_or_else(|| {
                diesel::insert_into(crate::schema::settings::table)
                    .values(NewSettings::default())
                    .execute(&mut db)
                    .expect("Failed to insert default settings");

                return Settings {
                    id: 1,
                    ..Default::default()
                };
            });
    }

    pub fn store_settings(&self, pool: &SqlitePool) {
        use crate::schema::settings::dsl::*;

        let mut db = pool.get().expect("Failed to get a connection from the pool.");

        diesel::delete(settings)
            .execute(&mut db)
            .expect("Failed to delete existing settings");

        diesel::insert_into(crate::schema::settings::table)
            .values(NewSettings {
                zoom_factor: self.zoom_factor,
                tree: self.tree.clone(),
                channel: self.channel.clone(),
                user_refresh_token: self.user_refresh_token.clone(),
            })
            .execute(&mut db)
            .expect("Failed to store settings");
    }
}
