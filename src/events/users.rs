use crate::auth::{AuthSession, ID};
use axum::{Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use diesel::prelude::*;
use utoipa::ToSchema;
use crate::auth::util::{auth_to_conn_expect_logged_in, auth_to_id_is_me_or_i_am_admin};
use crate::backend::{Backend};
use diesel_async::RunQueryDsl;
use crate::error::APIError;
use crate::schema::{event_user, user_data};
use crate::user_data::UserData;
use crate::error::APIResult;
use crate::events::slots::{after_unregister, check_change_guests_ok, get_user_slot};
use crate::events::user_action::{EventUserAction, log_user_action_from_event_user};
use crate::events::util::is_user_in_event;
use crate::schema::event_user::guests;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Ord, PartialOrd)]
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

#[derive(Copy, Clone, serde::Serialize, serde::Deserialize, Insertable, AsChangeset, Queryable, Selectable, ToSchema, Debug, PartialEq)]
#[diesel(table_name = event_user)]
pub struct EventUser {
    pub user_id: ID,
    pub event_id: ID,
    pub slot: i32,
    pub new_slot: i32,
    pub state: EventUserState,
    pub guests: i32,
    pub attended: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, ToSchema, Debug, PartialEq)]
pub struct PublicEventUser {
    pub user_id: ID,
    pub name: Option<String>,
    pub fetlife_name: Option<String>,
    pub role_factor: Option<f64>,
    pub open: Option<bool>,
    pub slot: i32,
    pub new_slot: i32,
    pub state: EventUserState,
    pub guests: i32,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, ToSchema, Debug, PartialEq)]
pub struct PublicEventUserLists {
    pub registered: Vec<PublicEventUser>,
    pub new: Vec<PublicEventUser>,
    pub waiting: Vec<PublicEventUser>,
}

fn get_public_user((eu, ud): (EventUser, UserData)) -> PublicEventUser {
    PublicEventUser {
        user_id: ud.user_id,
        name: if ud.show_name { Some(ud.name) } else { None },
        fetlife_name: if ud.show_fetlife { Some(ud.fetlife_name) } else { None },
        role_factor: if ud.show_role { Some(ud.role_factor) } else { None },
        open: if ud.show_open { Some(ud.open) } else { None },
        slot: eu.slot,
        new_slot: eu.new_slot,
        state: eu.state,
        guests: eu.guests,
    }
}

#[utoipa::path(
    get,
    path = "/event/{event_id}/users/{user_id}"
)]
pub async fn get_event_user(
    auth: AuthSession,
    Path((e_id, u_id)): Path<(ID, ID)>,
) -> APIResult<Json<PublicEventUser>> {
    let mut conn = auth_to_conn_expect_logged_in(auth).await?;

    let result = get_public_user(event_user::table
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::user_id.eq(u_id))
        .inner_join(user_data::table.on(event_user::user_id.eq(user_data::user_id)))
        .select((EventUser::as_select(), UserData::as_select()))
        .get_result::<(EventUser, UserData)>(&mut conn.0)
        .await
        .map_err(|_| APIError::UserNotInEvent)?);

    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/event/{id}/users"
)]
pub async fn get_event_users(
    auth: AuthSession,
    Path(e_id): Path<ID>,
) -> APIResult<Json<PublicEventUserLists>> {
    let mut conn = auth_to_conn_expect_logged_in(auth).await?;
    
    let users = event_user::table
        .filter(event_user::event_id.eq(e_id))
        .inner_join(user_data::table.on(event_user::user_id.eq(user_data::user_id)))
        .select((EventUser::as_select(), UserData::as_select()))
        .get_results::<(EventUser, UserData)>(&mut conn.0)
        .await
        .map_err(APIError::internal)?
        .into_iter()
        .map(get_public_user);


    let mut registered = vec![];
    let mut new = vec![];
    let mut waiting = vec![];
    for user in users {
        if user.state == EventUserState::Registered {
            registered.push(user);
            continue 
        }

        if user.state == EventUserState::New {
            new.push(user);
            continue
        }

        if user.state == EventUserState::Waiting || user.state == EventUserState::WaitingNew {
            waiting.push(user);
            continue
        }
    }
    
    registered.sort_by(|a, b| {a.name.cmp(&b.name)});
    new.sort_by(|a, b| {a.name.cmp(&b.name)});
    waiting.sort_by(|a, b| {a.slot.cmp(&b.slot)});
    
    let user_list = PublicEventUserLists {
        registered,
        new,
        waiting,
    };
    
    Ok(Json(user_list))
}

#[utoipa::path(
    post,
    path = "/event/{event_id}/register/{user_id}"
)]
pub async fn register_to_event(
    auth: AuthSession,
    Path((e_id, u_id)): Path<(ID, ID)>,
    Json(g): Json<i32>
) -> APIResult<()> {
    let mut conn = auth_to_id_is_me_or_i_am_admin(auth, u_id).await?;

    if is_user_in_event(e_id, u_id, &mut conn).await {
        return Err(APIError::UserAlreadyRegistered);
    }
    
    let (state, slot, new_slot) = get_user_slot(e_id, u_id, g, &mut conn).await?;
    
    let event_user = EventUser{
        user_id: u_id,
        event_id: e_id,
        slot,
        new_slot,
        state,
        guests: g,
        attended: false,
    };
    
    diesel::insert_into(event_user::table)
        .values(&event_user)
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    log_user_action_from_event_user(event_user, EventUserAction::Register, conn).await?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/event/{event_id}/unregister/{user_id}"
)]
pub async fn unregister_from_event(
    auth: AuthSession,
    Path((e_id, u_id)): Path<(ID, ID)>,
) -> APIResult<()> {
    let mut conn = auth_to_id_is_me_or_i_am_admin(auth, u_id).await?;
    
    let event_user = diesel::delete(event_user::table)
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::user_id.eq(u_id))
        .returning(EventUser::as_select())
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    after_unregister(event_user, &mut conn).await?;
    log_user_action_from_event_user(event_user, EventUserAction::Unregister, conn).await?;
    
    
    
    Ok(())
}

#[utoipa::path(
    post,
    path = "/event/{event_id}/change_guests"
)]
pub async fn change_guests(
    auth: AuthSession,
    Path((e_id, u_id)): Path<(ID, ID)>,
    Json(g): Json<i32>
) -> APIResult<()> {
    let mut conn = auth_to_id_is_me_or_i_am_admin(auth, u_id).await?;
    
    if !check_change_guests_ok(e_id, u_id, g, &mut conn).await? {
        return Err(APIError::ChangeGuestsDenied)
    }
    
    let event_user = diesel::update(event_user::table)
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::user_id.eq(u_id))
        .set(guests.eq(g))
        .returning(EventUser::as_select())
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    log_user_action_from_event_user(event_user, EventUserAction::ChangeGuests, conn).await?;

    Ok(())
}

pub fn add_event_user_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/event/:event_id/users/:user_id", get(get_event_user))
        .route("/event/:event_id/users", get(get_event_users))
        .route("/event/:event_id/register/:user_id", post(register_to_event))
        .route("/event/:event_id/unregister/:user_id", post(unregister_from_event))
        .route("/event/:event_id/change_guests/:user_id", post(change_guests))
}