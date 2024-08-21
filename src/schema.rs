// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "userpermission"))]
    pub struct Userpermission;
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
    user_data (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Text,
        fetlife_name -> Text,
        experience_text -> Text,
        found_us_text -> Text,
        goal_text -> Text,
        role_factor -> Float8,
        active_factor -> Float8,
        passive_factor -> Float8,
        open -> Bool,
        show_name -> Bool,
        show_role -> Bool,
        show_experience -> Bool,
        show_open -> Bool,
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
diesel::joinable!(user_data -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    permission,
    user_data,
    users,
);
