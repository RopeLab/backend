use diesel::prelude::*;
use utoipa::ToSchema;
use crate::schema::event_user;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Eventuserstate"]
#[repr(u8)]
pub enum EventUserState {
    Registered, 
    Waiting, 
    Rejected
}

#[derive(serde::Serialize, Queryable, Selectable, ToSchema, Debug, PartialEq)]
#[diesel(table_name = event_user)]
pub struct EventUser {
    pub id: i32,
    pub user_id: i32,
    pub event_id: i32,
    pub slot: i32,
    pub state: EventUserState,
    pub guests: i32,
    pub attended: bool,
}

#[derive(serde::Deserialize, Insertable, AsChangeset, ToSchema, Debug)]
#[diesel(table_name = event_user)]
pub struct NewEventUser {
    pub user_id: i32,
    pub event_id: i32,
    pub slot: i32,
    pub state: EventUserState,
    pub guests: i32,
    pub attended: bool,
}