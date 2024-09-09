
use crate::auth::AuthSession;
use axum::{Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use diesel::prelude::*;
use utoipa::ToSchema;
use crate::auth::util::{id_is_admin_or_me, is_logged_in, parse_path_id};
use crate::backend::{Backend, DBConnection};
use diesel_async::RunQueryDsl;
use crate::error::APIError;
use crate::schema::{event_user, user_data};
use crate::user_data::UserData;
use crate::error::Result;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Eventuserstate"]
#[repr(u8)]
pub enum EventUserState {
    Registered, 
    Waiting, 
    Rejected, 
    New,
    WaitingNew,
}

#[derive(serde::Serialize, serde::Deserialize, Insertable, AsChangeset, Queryable, Selectable, ToSchema, Debug, PartialEq)]
#[diesel(table_name = event_user)]
pub struct EventUser {
    pub user_id: i32,
    pub event_id: i32,
    pub slot: i32,
    pub state: EventUserState,
    pub guests: i32,
    pub attended: bool,
}

#[derive(serde::Serialize, serde::Deserialize, ToSchema, Debug, PartialEq)]
pub struct PublicEventUser {
    pub user_id: i32,
    pub name: Option<String>,
    pub role_factor: Option<f64>,
    pub open: Option<bool>,
    pub slot: i32,
    pub state: EventUserState,
    pub guests: i32,
}

#[derive(serde::Serialize, serde::Deserialize, ToSchema, Debug, PartialEq)]
pub struct RegisterEvent {
    pub user_id: i32,
    pub guests: i32,
}

#[utoipa::path(
    get,
    path = "/event/{id}/users"
)]
pub async fn get_event_users(
    auth: AuthSession,
    path: Path<String>,
) -> Result<Json<Vec<PublicEventUser>>> {
    let mut conn = is_logged_in(auth).await?;
    let event_id = parse_path_id(path)?;
    
    let result = event_user::table
        .filter(event_user::event_id.eq(event_id))
        .inner_join(user_data::table.on(event_user::user_id.eq(user_data::user_id)))
        .select((EventUser::as_select(), UserData::as_select()))
        .get_results::<(EventUser, UserData)>(&mut conn.0)
        .await
        .map_err(APIError::internal)?
        .into_iter()
        .map(|(eu, ud)| PublicEventUser{
            user_id: ud.user_id,
            name: if ud.show_name {Some(ud.name)} else {None},
            role_factor: if ud.show_role {Some(ud.role_factor)} else {None},
            open: if ud.show_open {Some(ud.open)} else {None},
            slot: eu.slot,
            state: eu.state,
            guests: eu.guests,
            
        })
        .collect();
    
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/event/{event_id}/register"
)]
pub async fn register_to_event(
    auth: AuthSession,
    path: Path<String>,
    Json(register_event): Json<RegisterEvent>
) -> Result<()> {
    let (user_id, mut conn) = id_is_admin_or_me(auth, register_event.user_id).await?;
    let event_id = parse_path_id(path)?;

    let event_user = EventUser{
        user_id,
        event_id,
        slot: 0,
        state: EventUserState::Registered,
        guests: register_event.guests,
        attended: false,
    };

    diesel::insert_into(event_user::table)
        .values(&event_user)
        .on_conflict((event_user::event_id, event_user::user_id))
        .do_nothing()
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    Ok(())
}

pub fn add_event_user_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/event/:id/users", get(get_event_users))
        .route("/event/:event_id/register", post(register_to_event))
}