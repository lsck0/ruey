use diesel::{prelude::*, sqlite::SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

const DATABASE_URL: &str = env!("DATABASE_URL");
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn initialize_database() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(DATABASE_URL).expect("Failed to connect to the database.");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations.");

    return conn;
}

#[derive(Debug, Clone, Queryable, Selectable, Default)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Settings {
    pub id: i32,
    pub channel: Option<String>,
    pub user_refresh_token: Option<String>,
    pub tree: Option<String>,
    pub zoom_factor: Option<f32>,
}

#[derive(Debug, Clone, Insertable, Default)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewSettings {
    pub channel: Option<String>,
    pub user_refresh_token: Option<String>,
    pub tree: Option<String>,
    pub zoom_factor: Option<f32>,
}
