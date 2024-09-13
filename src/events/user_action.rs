use axum::extract::Path;
use axum::{Json, Router};
use axum::routing::{get};
use chrono::{Local, NaiveDateTime};
use diesel::{AsChangeset, ExpressionMethods, Insertable, Queryable, QueryDsl, Selectable, SelectableHelper};
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::auth::AuthSession;
use crate::auth::util::{id_is_admin_or_me, parse_path_id};
use crate::backend::{Backend, DBConnection};
use crate::error::APIError;
use crate::schema::{user_action};
use crate::error::Result;
use crate::events::event_user::{EventUser, EventUserState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Eventuseraction"]
#[repr(u8)]
pub enum EventUserAction {
    Register, 
    Unregister, 
    GetSlot, 
    Rejected, 
    ChangeGuests,
}

#[derive(serde::Serialize, serde::Deserialize, Insertable, AsChangeset, Queryable, Selectable, ToSchema, Debug, PartialEq)]
#[diesel(table_name = user_action)]
pub struct UserAction {
    pub user_id: i32,
    pub event_id: i32,
    pub date: NaiveDateTime,
    pub action: EventUserAction,
    pub in_waiting: bool,
    pub in_new: bool,
    pub guests: i32,
}

pub async fn log_user_action_from_event_user(
    event_user: EventUser,
    action: EventUserAction,
    conn: DBConnection
) -> Result<()> {
    let waiting = event_user.state == EventUserState::Waiting || event_user.state == EventUserState::WaitingNew;
    let new = event_user.state == EventUserState::New || event_user.state == EventUserState::WaitingNew;
    log_user_action(event_user.user_id, event_user.event_id, action, waiting, new, event_user.guests, conn).await?;

    Ok(())
}

pub async fn log_user_action(
    user_id: i32, 
    event_id: i32,
    action: EventUserAction,
    in_waiting: bool, 
    in_new: bool, 
    guests: i32,
    mut conn: DBConnection
) -> Result<()> {
    diesel::insert_into(user_action::table)
        .values(UserAction {
            user_id,
            event_id,
            date: Local::now().naive_local(),
            action,
            in_waiting,
            in_new,
            guests,
        })
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/user_action/{id}/all"
)]
pub async fn get_user_actions(
    auth: AuthSession,
    path: Path<String>,
) -> Result<Json<Vec<UserAction>>> {
    let user_id = parse_path_id(path)?;
    let (_, mut conn) = id_is_admin_or_me(auth, user_id).await?;

    let actions = user_action::table
        .filter(user_action::user_id.eq(user_id))
        .select(UserAction::as_select())
        .get_results(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(Json(actions))
}

pub fn add_user_action_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/user_action/:id/all", get(get_user_actions))
}




