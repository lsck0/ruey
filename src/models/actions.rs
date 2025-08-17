use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionConfig {}

#[derive(Debug, Default, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::actions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Action {
    pub id: i32,
    pub name: String,
    pub script: Vec<u8>,
    pub config: Vec<u8>,
}

#[derive(Debug, Default, Clone, Insertable)]
#[diesel(table_name = crate::schema::actions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewAction {
    pub name: String,
    pub script: Vec<u8>,
    pub config: Vec<u8>,
}
