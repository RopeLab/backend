use axum::extract::Path;
use axum::{Json, Router};
use axum::routing::{get};
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::auth::{AuthSession, ID};
use crate::auth::util::{auth_to_conn_expect_logged_in_check_is_admin, auth_to_is_admin_and_conn};
use crate::backend::{Backend};
use crate::error::{APIError, APIResult};
use crate::events::users::{EventUserState};
use crate::events::util::{get_count_of_event_users_open_with_state, get_count_of_event_users_with_state, get_slots_and_description_of_event_with_admin_check};
use crate::schema::{event};

#[derive(serde::Serialize, serde::Deserialize, ToSchema, Debug, PartialEq)]
pub struct EventDate {
    pub id: ID,
    pub date: NaiveDateTime,
}

#[derive(serde::Serialize, serde::Deserialize, ToSchema, Debug, PartialEq)]
pub struct PublicEventData {
    pub slots: i32,
    pub register_count: i32,
    pub wait_count: i32,
    pub description: String,
}

#[derive(serde::Serialize, serde::Deserialize, ToSchema, Debug, PartialEq)]
pub struct LoggedInEventData {
    pub slots: i32,
    pub register_count: i32,
    pub wait_count: i32,
    pub open_count: i32,
    pub description: String,
}

#[utoipa::path(
    get,
    path = "/event/dates"
)]
pub async fn get_event_dates(
    auth: AuthSession,
) -> APIResult<Json<Vec<EventDate>>> {
    let (admin, mut conn) = auth_to_is_admin_and_conn(auth).await?;

    if admin {
        let event_dates = event::table
            .select((event::id, event::date))
            .get_results(&mut conn.0)
            .await
            .map_err(APIError::internal)?
            .into_iter()
            .map(|(id, date)| {
                EventDate{id, date}
            })
            .collect();

        return Ok(Json(event_dates))
    }

    let event_dates = event::table
        .filter(event::visible.eq(true))
        .select((event::id, event::date))
        .get_results(&mut conn.0)
        .await
        .map_err(APIError::internal)?
        .into_iter()
        .map(|(id, date)| {
            EventDate{id, date}
        })
        .collect();
    Ok(Json(event_dates))
}



#[utoipa::path(
    get,
    path = "/event/{id}/public_data"
)]
pub async fn get_event_public_data(
    auth: AuthSession,
    Path(e_id): Path<ID>,
) -> APIResult<Json<PublicEventData>> {
    let (admin, mut conn) = auth_to_is_admin_and_conn(auth).await?;

    let (slots, description) = get_slots_and_description_of_event_with_admin_check(e_id, admin, &mut conn).await?;
    let register_count = get_count_of_event_users_with_state(e_id, EventUserState::Registered, &mut conn).await?;
    let wait_count = get_count_of_event_users_with_state(e_id, EventUserState::Waiting, &mut conn).await?;

    Ok(Json(PublicEventData {
        slots,
        register_count,
        wait_count,
        description,
    }))
}

#[utoipa::path(
    get,
    path = "/event/{id}/logged_in_data"
)]
pub async fn get_event_logged_in_data(
    auth: AuthSession,
    Path(e_id): Path<ID>,
) -> APIResult<Json<LoggedInEventData>> {
    let (admin, mut conn) = auth_to_conn_expect_logged_in_check_is_admin(auth).await?;

    let (slots, description) = get_slots_and_description_of_event_with_admin_check(e_id, admin, &mut conn).await?;
    let register_count = get_count_of_event_users_with_state(e_id, EventUserState::Registered, &mut conn).await?;
    let wait_count = get_count_of_event_users_with_state(e_id, EventUserState::Waiting, &mut conn).await?;
    let open_count = get_count_of_event_users_open_with_state(e_id, EventUserState::Registered, &mut conn).await?;

    Ok(Json(LoggedInEventData {
        slots,
        register_count,
        wait_count,
        open_count,
        description,
    }))
}

pub fn add_public_event_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/event/dates", get(get_event_dates))
        .route("/event/:id/public_data", get(get_event_public_data))
        .route("/event/:id/logged_in_data", get(get_event_logged_in_data))

}

