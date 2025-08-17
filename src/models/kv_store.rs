use diesel::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_binary::binary_stream::Endian;

use crate::models::SqlitePool;

#[derive(Debug, Default, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::kv_store)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct KvStore {
    pub bucket: String,
    pub key: String,
    pub value: Vec<u8>,
}

impl KvStore {
    pub fn get_value<T>(pool: &SqlitePool, bucket: &str, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        use crate::schema::kv_store;

        let mut db = pool.get().expect("Failed to get a connection from the pool.");

        let value = kv_store::table
            .filter(kv_store::bucket.eq(bucket))
            .filter(kv_store::key.eq(key))
            .select(kv_store::value)
            .first::<Vec<u8>>(&mut db)
            .optional()
            .expect("Failed to query kv_store");

        return value.map(|v| serde_binary::from_vec(v, Endian::Big).expect("Failed to decode value from kv_store"));
    }

    pub fn set_value<T>(pool: &SqlitePool, bucket: &str, key: &str, value: T)
    where
        T: Serialize,
    {
        use crate::schema::kv_store;

        let value = serde_binary::to_vec(&value, Endian::Big).expect("Failed to encode value to kv_store");

        let mut db = pool.get().expect("Failed to get a connection from the pool.");

        diesel::insert_into(kv_store::table)
            .values(KvStore {
                bucket: bucket.to_string(),
                key: key.to_string(),
                value: value.clone(),
            })
            .on_conflict((kv_store::bucket, kv_store::key))
            .do_update()
            .set(kv_store::value.eq(value))
            .execute(&mut db)
            .expect("Failed to insert or update value in kv_store");
    }
}
