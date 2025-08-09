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

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Settings {
    pub id: i32,
}
