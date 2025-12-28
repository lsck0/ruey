pub mod actions;
pub mod kv_store;
pub mod settings;

use anyhow::Result;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

const DATABASE_URL: &str = env!("DATABASE_URL");
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

pub fn create_database_pool() -> Result<SqlitePool> {
    let manager = ConnectionManager::<SqliteConnection>::new(DATABASE_URL);

    let pool = Pool::builder().build(manager)?;

    let mut connection = pool.get()?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations.");

    return Ok(pool);
}
