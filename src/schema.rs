// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "eventuseraction"))]
    pub struct Eventuseraction;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "eventuserstate"))]
    pub struct Eventuserstate;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "userpermission"))]
    pub struct Userpermission;
}

diesel::table! {
    event (id) {
        id -> Int4,
        date -> Timestamp,
        slots -> Int4,
        visible -> Bool,
        archive -> Bool,
        register_deadline -> Timestamp,
        visible_date -> Timestamp,
        archive_date -> Timestamp,
        description -> Text,
        new_slots -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Eventuserstate;

    event_user (id) {
        id -> Int4,
        user_id -> Int4,
        event_id -> Int4,
        slot -> Int4,
        state -> Eventuserstate,
        guests -> Int4,
        attended -> Bool,
        new_slot -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Userpermission;

    permission (id) {
        id -> Int4,
        user_id -> Int4,
        user_permission -> Userpermission,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Eventuseraction;

    user_action (id) {
        id -> Int4,
        user_id -> Int4,
        event_id -> Int4,
        date -> Timestamp,
        action -> Eventuseraction,
        in_waiting -> Bool,
        in_new -> Bool,
        guests -> Int4,
    }
}

diesel::table! {
    user_data (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Text,
        fetlife_name -> Text,
        experience_text -> Text,
        found_us_text -> Text,
        goal_text -> Text,
        role_factor -> Float8,
        open -> Bool,
        show_name -> Bool,
        show_role -> Bool,
        show_open -> Bool,
        new -> Bool,
        show_fetlife -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Text,
        pw_hash -> Text,
    }
}

diesel::joinable!(permission -> users (user_id));
diesel::joinable!(user_action -> event (event_id));
diesel::joinable!(user_action -> users (user_id));
diesel::joinable!(user_data -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    event,
    event_user,
    permission,
    user_action,
    user_data,
    users,
);
