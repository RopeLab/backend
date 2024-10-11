use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::backend::DBConnection;
use crate::error::{APIError, APIResult};
use crate::events::users::{EventUser, EventUserState};
use crate::events::util::{get_count_of_event_users_with_state, get_count_of_event_users_with_state_expect_user_id, get_max_slot_index_with_state, get_next_waiting_event_users, get_next_waiting_new_event_users, get_slots_and_new_slots_of_event, is_event_user_states, is_user_new};
use crate::schema::event_user;


pub async fn get_user_slot(e_id: i32, u_id: i32, guests: i32, conn: &mut DBConnection) -> APIResult<(EventUserState, i32, i32)> {
    let register_count = get_count_of_event_users_with_state(e_id, &[EventUserState::Registered], conn).await?;
    let (slots, new_slots) = get_slots_and_new_slots_of_event(e_id, conn).await?;
    
    // If there is still space in the register list use that.
    if (register_count + guests + 1) <= slots {
        return Ok((EventUserState::Registered, 0, 0))
    }
    
    if new_slots != 0 && is_user_new(u_id, conn).await? {
        let new_register_count = get_count_of_event_users_with_state(e_id, &[EventUserState::New], conn).await?;

        // If there is still space in the extra slots for new people register list use that.
        if (new_register_count + guests + 1) <= new_slots {
            return Ok((EventUserState::New, 0, 0))
        }

        let slot_index = get_max_slot_index_with_state(e_id, EventUserState::Waiting, conn).await?;
        let new_slot_index = get_max_slot_index_with_state(e_id, EventUserState::WaitingNew, conn).await?;
        return Ok((EventUserState::WaitingNew, slot_index, new_slot_index));
    }

    let slot_index = get_max_slot_index_with_state(e_id, EventUserState::Waiting, conn).await?;
    return Ok((EventUserState::Waiting, slot_index, 0));
}

pub async fn move_up_register(e_id: i32, conn: &mut DBConnection) -> APIResult<()> {
    let (slots, _) = get_slots_and_new_slots_of_event(e_id, conn).await?;
    
    let mut register_count = get_count_of_event_users_with_state(e_id, &[EventUserState::Registered], conn).await?;
    let mut next_user = get_next_waiting_event_users(e_id, conn).await.ok();
    
    while next_user.is_some() && (register_count + next_user.unwrap().guests + 1) <= slots {
        let user = next_user.unwrap();
        diesel::update(event_user::table)
            .filter(event_user::event_id.eq(e_id))
            .filter(event_user::user_id.eq(user.user_id))
            .set(event_user::state.eq(EventUserState::Registered))
            .execute(&mut conn.0)
            .await
            .map_err(APIError::internal)?;

        diesel::update(event_user::table)
            .filter(event_user::event_id.eq(e_id))
            .filter(event_user::state.eq_any([EventUserState::Waiting, EventUserState::WaitingNew]))
            .set(event_user::slot.eq(event_user::slot - 1))
            .execute(&mut conn.0)
            .await
            .map_err(APIError::internal)?;

        register_count = get_count_of_event_users_with_state(e_id, &[EventUserState::Registered], conn).await?;
        next_user = get_next_waiting_event_users(e_id, conn).await.ok();
    }
    
    Ok(())
}

pub async fn move_up_new(e_id: i32, conn: &mut DBConnection) -> APIResult<()> {
    let (_, new_slots) = get_slots_and_new_slots_of_event(e_id, conn).await?;

    let mut new_count = get_count_of_event_users_with_state(e_id, &[EventUserState::New], conn).await?;
    let mut next_user = get_next_waiting_new_event_users(e_id, conn).await.ok();

    while next_user.is_some() && (new_count + next_user.unwrap().guests + 1) <= new_slots {
        let user = next_user.unwrap();
        diesel::update(event_user::table)
            .filter(event_user::event_id.eq(e_id))
            .filter(event_user::user_id.eq(user.user_id))
            .set(event_user::state.eq(EventUserState::New))
            .execute(&mut conn.0)
            .await
            .map_err(APIError::internal)?;

        diesel::update(event_user::table)
            .filter(event_user::event_id.eq(e_id))
            .filter(event_user::state.eq(EventUserState::WaitingNew))
            .set(event_user::new_slot.eq(event_user::new_slot - 1))
            .execute(&mut conn.0)
            .await
            .map_err(APIError::internal)?;

        new_count = get_count_of_event_users_with_state(e_id, &[EventUserState::New], conn).await?;
        next_user = get_next_waiting_new_event_users(e_id, conn).await.ok();
    }

    Ok(())
}

async fn move_up_waiting(e_id: i32, slot: i32, conn: &mut DBConnection) -> APIResult<()> {
    diesel::update(event_user::table)
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::slot.ge(slot))
        .set(event_user::slot.eq(event_user::slot - 1))
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    
    Ok(())
}

async fn move_up_waiting_new(e_id: i32, new_slot: i32, conn: &mut DBConnection) -> APIResult<()> {
    diesel::update(event_user::table)
        .filter(event_user::event_id.eq(e_id))
        .filter(event_user::new_slot.ge(new_slot))
        .set(event_user::new_slot.eq(event_user::new_slot - 1))
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(())
}

pub async fn after_unregister(event_user: EventUser, conn: &mut DBConnection) -> APIResult<()> {
    if event_user.state == EventUserState::Registered {
        move_up_register(event_user.event_id, conn).await?;
        return Ok(())
    }

    if event_user.state == EventUserState::Registered {
        move_up_new(event_user.event_id, conn).await?;
        return Ok(())
    }

    if event_user.state == EventUserState::Waiting {
        move_up_waiting(event_user.event_id, event_user.slot, conn).await?;
        return Ok(())
    }

    if event_user.state == EventUserState::WaitingNew {
        move_up_waiting(event_user.event_id, event_user.slot, conn).await?;
        move_up_waiting_new(event_user.event_id, event_user.new_slot, conn).await?;
        return Ok(())
    }
    
    Ok(())
}

pub async fn check_change_guests_ok(e_id: i32, u_id: i32, guests: i32, conn: &mut DBConnection) -> APIResult<bool> {
    let (slots, new_slots) = get_slots_and_new_slots_of_event(e_id, conn).await?;
    
    if is_event_user_states(e_id, u_id, &[EventUserState::Registered], conn).await {
        let register_count = get_count_of_event_users_with_state_expect_user_id(
            e_id, u_id, &[EventUserState::Registered], conn).await?;
        
        return Ok((register_count + guests + 1 ) <= slots)
    }

    if is_event_user_states(e_id, u_id, &[EventUserState::New], conn).await {
        let new_count = get_count_of_event_users_with_state_expect_user_id(
            e_id, u_id, &[EventUserState::New], conn).await?;

        return Ok((new_count + guests + 1 ) <= new_slots)
    }
    
    Ok(true)
}