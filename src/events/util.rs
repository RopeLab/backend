use diesel::dsl::{count_star, max, sum};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::backend::DBConnection;
use crate::error::{APIError, APIResult};
use crate::events::users::{EventUser, EventUserState};
use crate::schema::{event, event_user, user_data};


pub async fn is_user_in_event(e_id: i32, u_id: i32, conn: &mut DBConnection) -> bool {
    event_user::table
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::user_id.eq(u_id))
        .select(EventUser::as_select())
        .get_result(&mut conn.0)
        .await
        .is_ok()
}

pub async fn is_user_new(u_id: i32, conn: &mut DBConnection) -> APIResult<bool> {
    user_data::table
        .filter(user_data::user_id.eq(u_id))
        .select(user_data::new)
        .get_result::<bool>(&mut conn.0)
        .await
        .map_err(APIError::internal)
}

pub async fn get_slots_and_new_slots_of_event(e_id: i32, conn: &mut DBConnection) -> APIResult<(i32, i32)> {
     event::table
        .filter(event::id.eq(e_id))
        .filter(event::visible.eq(true))
        .select((event::slots, event::new_slots))
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)
}

pub async fn get_slots_and_description_of_event_with_admin_check(e_id: i32, admin: bool, conn: &mut DBConnection) -> APIResult<(i32, i32, String)> {
    if admin {
        event::table
            .filter(event::id.eq(e_id))
            .select((event::slots, event::new_slots, event::description))
            .get_result(&mut conn.0)
            .await
            .map_err(APIError::internal)
    } else {
        event::table
            .filter(event::id.eq(e_id))
            .filter(event::visible.eq(true))
            .select((event::slots, event::new_slots, event::description))
            .get_result(&mut conn.0)
            .await
            .map_err(APIError::internal)
    }
}

pub async fn get_max_slot_index_with_state(e_id: i32, state: EventUserState, conn: &mut DBConnection) -> APIResult<i32> {
    let max_slot_index = event_user::table
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::state.eq(state))
        .select(max(event_user::slot))
        .get_result::<Option<i32>>(&mut conn.0)
        .await
        .map_err(APIError::internal)?
        .unwrap_or_default();

    Ok(max_slot_index)
}

pub async fn get_count_of_event_users_with_state(id: i32, states: &[EventUserState], conn: &mut DBConnection) -> APIResult<i32> {
    let (register_count, guest_count) = event_user::table
        .filter(event_user::event_id.eq(id))
        .filter(event_user::state.eq_any(states))
        .select((count_star(), sum(event_user::guests)))
        .get_result::<(i64, Option<i64>)>(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok((register_count + guest_count.unwrap_or_default()) as i32)
}

pub async fn get_count_of_event_users_with_state_expect_user_id(e_id: i32, u_id: i32, states: &[EventUserState], conn: &mut DBConnection) -> APIResult<i32> {
    let (register_count, guest_count) = event_user::table
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::user_id.ne(u_id))
        .filter(event_user::state.eq_any(states))
        .select((count_star(), sum(event_user::guests)))
        .get_result::<(i64, Option<i64>)>(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok((register_count + guest_count.unwrap_or_default()) as i32)
}

pub async fn get_count_of_event_users_open_with_state(id: i32, state: &[EventUserState], conn: &mut DBConnection) -> APIResult<i32> {
    let open_count = event_user::table
        .filter(event_user::event_id.eq(id))
        .filter(event_user::state.eq_any(state))
        .inner_join(user_data::table.on(event_user::user_id.eq(user_data::user_id)))
        .filter(user_data::open.eq(true))
        .count()
        .get_result::<i64>(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(open_count as i32)
}

pub async fn get_next_waiting_event_users(e_id: i32, conn: &mut DBConnection) -> APIResult<EventUser> {
    event_user::table
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::state.eq_any(&[EventUserState::Waiting, EventUserState::WaitingNew]))
        .filter(event_user::slot.eq(0))
        .select(EventUser::as_select())
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)
}

pub async fn get_next_waiting_new_event_users(e_id: i32, conn: &mut DBConnection) -> APIResult<EventUser> {
    event_user::table
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::state.eq(EventUserState::WaitingNew))
        .filter(event_user::new_slot.eq(0))
        .select(EventUser::as_select())
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)
}

pub async fn is_event_user_states(e_id: i32, u_id: i32, state: &[EventUserState], conn: &mut DBConnection) -> bool {
    event_user::table
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::user_id.eq(u_id))
        .filter(event_user::state.eq_any(state))
        .select(EventUser::as_select())
        .get_result(&mut conn.0)
        .await
        .is_ok()
}