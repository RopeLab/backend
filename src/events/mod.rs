pub mod users;
pub mod user_action;
pub mod public;
pub mod slots;
mod util;

use axum::{Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::backend::{Backend, DBConnection};
use crate::error::APIError;
use crate::schema::event;
use crate::error::APIResult;

#[derive(serde::Serialize, serde::Deserialize, Queryable, Insertable, Selectable, AsChangeset, ToSchema, Debug, PartialEq)]
#[diesel(table_name = event)]
pub struct Event {
    pub id: i32,
    pub visible_date: NaiveDateTime,
    pub register_deadline: NaiveDateTime,
    pub date: NaiveDateTime,
    pub archive_date: NaiveDateTime,
    pub slots: i32,
    pub new_slots: i32,
    pub visible: bool,
    pub archive: bool,
    pub description: String,
}

#[derive(serde::Deserialize, Insertable, ToSchema, Debug, PartialEq)]
#[diesel(table_name = event)]
pub struct NewEvent {
    pub visible_date: NaiveDateTime,
    pub register_deadline: NaiveDateTime,
    pub date: NaiveDateTime,
    pub archive_date: NaiveDateTime,
    pub slots: i32,
    pub new_slots: i32,
    pub visible: bool,
    pub archive: bool,
    pub description: String,
}

#[utoipa::path(
    post,
    path = "/event"
)]
pub async fn post_event(
    mut conn: DBConnection,
    Json(new_event): Json<NewEvent>
) -> APIResult<()> {
    diesel::insert_into(event::table)
        .values(&new_event)
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/event/{id}"
)]
pub async fn update_event(
    mut conn: DBConnection,
    Path(e_id): Path<i32>,
    Json(event): Json<Event>
) -> APIResult<()> {
    if e_id != event.id {
        return Err(APIError::EventIdsDontMatch)
    }
    
    diesel::update(event::table)
        .filter(event::id.eq(event.id))
        .set(event)
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/event/{id}"
)]
pub async fn get_event(
    mut conn: DBConnection,
    Path(e_id): Path<i32>,
) -> APIResult<Json<Event>> {
    let event = event::table
        .filter(event::id.eq(e_id))
        .select(Event::as_select())
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    Ok(Json(event))
}

#[utoipa::path(
    post,
    path = "/event/{id}/delete"
)]
pub async fn delete_event(
    mut conn: DBConnection,
    Path(e_id): Path<i32>,
) -> APIResult<()> {
    diesel::delete(event::table)
        .filter(event::id.eq(e_id))
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/event/all"
)]
pub async fn get_event_all(mut conn: DBConnection) -> APIResult<Json<Vec<Event>>> {
    let event = event::table
        .select(Event::as_select())
        .get_results(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    Ok(Json(event))
}

pub fn add_admin_event_routes(router: Router<Backend>) -> Router<Backend> {
   router.route("/event", post(post_event))
       .route("/event/:id", post(update_event))
       .route("/event/:id", get(get_event))
       .route("/event/all", get(get_event_all))
       .route("/event/:id/delete", post(delete_event))
}