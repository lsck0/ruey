pub mod actions;
pub mod kv_store;
pub mod settings;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

const DATABASE_URL: &str = env!("DATABASE_URL");
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

pub fn initialize_database() -> SqlitePool {
    let manager = ConnectionManager::<SqliteConnection>::new(DATABASE_URL);

    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create the database pool.");

    let mut conn = pool.get().expect("Failed to get a connection from the pool.");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations.");

    return pool;
}
