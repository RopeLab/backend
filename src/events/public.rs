use axum::extract::Path;
use axum::{Json, Router};
use axum::routing::{get, post};
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel::dsl::{count_star, sum};
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::auth::{AuthSession, ID};
use crate::auth::util::{auth_to_conn_expect_logged_in_check_is_admin, auth_to_is_admin_and_conn};
use crate::backend::{Backend, DBConnection};
use crate::error::{APIError, APIResult};
use crate::events::event_user::{change_guests, EventUserState, get_event_users, register_to_event, unregister_from_event};
use crate::schema::{event, event_user, user_data};

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

async fn get_count_of_event_users_with_state(id: ID, state: EventUserState, conn: &mut DBConnection) -> APIResult<i32> {
    let (register_count, guest_count) = event_user::table
        .filter(event_user::event_id.eq(id))
        .filter(event_user::state.eq(state))
        .select((count_star(), sum(event_user::guests)))
        .get_result::<(i64, Option<i64>)>(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok((register_count + guest_count.unwrap_or_default()) as i32)
}

async fn get_slots_and_description_of_event(e_id: ID, admin: bool, conn: &mut DBConnection) -> APIResult<(i32, String)> {
    let (slots, description) = if admin {
        event::table
            .filter(event::id.eq(e_id))
            .select((event::slots, event::description))
            .get_result(&mut conn.0)
            .await
            .map_err(APIError::internal)?
    } else {
        event::table
            .filter(event::id.eq(e_id))
            .filter(event::visible.eq(true))
            .select((event::slots, event::description))
            .get_result(&mut conn.0)
            .await
            .map_err(APIError::internal)?
    };

    Ok((slots, description))
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

    let (slots, description) = get_slots_and_description_of_event(e_id, admin, &mut conn).await?;
    let register_count = get_count_of_event_users_with_state(e_id, EventUserState::Registered, &mut conn).await?;
    let wait_count = get_count_of_event_users_with_state(e_id, EventUserState::Waiting, &mut conn).await?;

    Ok(Json(PublicEventData {
        slots,
        register_count,
        wait_count,
        description,
    }))
}


async fn get_count_of_event_users_open_with_state(id: ID, state: EventUserState, conn: &mut DBConnection) -> APIResult<i32> {
    let open_count = event_user::table
        .filter(event_user::event_id.eq(id))
        .filter(event_user::state.eq(state))
        .inner_join(user_data::table.on(event_user::user_id.eq(user_data::user_id)))
        .filter(user_data::open.eq(true))
        .count()
        .get_result::<i64>(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(open_count as i32)
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

    let (slots, description) = get_slots_and_description_of_event(e_id, admin, &mut conn).await?;
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

