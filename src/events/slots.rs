use crate::auth::ID;
use crate::backend::DBConnection;
use crate::error::{APIResult};
use crate::events::users::{EventUserState};
use crate::events::util::{get_count_of_event_users_with_state, get_max_slot_index_with_state, get_slots_and_new_slots_of_event, is_user_new};

pub async fn get_user_slot(e_id: ID, u_id: ID, guests: i32, conn: &mut DBConnection) -> APIResult<(EventUserState, i32)> {
    let register_count = get_count_of_event_users_with_state(e_id, EventUserState::Registered, conn).await?;
    let (slots, new_slots) = get_slots_and_new_slots_of_event(e_id, conn).await?;
    
    // If there is still space in the register list use that.
    if (register_count + guests + 1) <= slots {
        let slot_index = get_max_slot_index_with_state(e_id, EventUserState::Registered, conn).await?;
        return Ok((EventUserState::Registered, slot_index))
    }
    
    if is_user_new(u_id, conn).await? {
        let new_register_count = get_count_of_event_users_with_state(e_id, EventUserState::New, conn).await?;

        // If there is still space in the extra slots for new people register list use that.
        if (new_register_count + guests + 1) <= new_slots {
            let slot_index = get_max_slot_index_with_state(e_id, EventUserState::New, conn).await?;
            return Ok((EventUserState::New, slot_index))
        }

        let slot_index = get_max_slot_index_with_state(e_id, EventUserState::WaitingNew, conn).await?;
        return Ok((EventUserState::WaitingNew, slot_index));
    }

    let slot_index = get_max_slot_index_with_state(e_id, EventUserState::Waiting, conn).await?;
    return Ok((EventUserState::Waiting, slot_index));
}