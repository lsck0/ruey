use anyhow::Result;
use diesel::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

use crate::models::SqlitePool;

#[derive(Debug, Default, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::kv_store)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct KvStore {
    pub bucket: String,
    pub key: String,
    pub value: String,
}

impl KvStore {
    pub fn get_value<T>(pool: &SqlitePool, bucket: &str, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        use crate::schema::kv_store;

        let mut db = pool.get()?;

        let value = kv_store::table
            .filter(kv_store::bucket.eq(bucket))
            .filter(kv_store::key.eq(key))
            .select(kv_store::value)
            .first::<String>(&mut db)
            .optional()?;

        let object = value.map(|v| serde_json::from_str::<T>(&v)).transpose()?;

        return Ok(object);
    }

    pub fn set_value<T>(pool: &SqlitePool, bucket: String, key: String, value: T) -> Result<()>
    where
        T: Serialize,
    {
        use crate::schema::kv_store;

        let mut db = pool.get()?;

        let value = serde_json::to_string(&value)?;

        diesel::insert_into(kv_store::table)
            .values(KvStore {
                bucket,
                key,
                value: value.clone(),
            })
            .on_conflict((kv_store::bucket, kv_store::key))
            .do_update()
            .set(kv_store::value.eq(value))
            .execute(&mut db)?;

        return Ok(());
    }
}
