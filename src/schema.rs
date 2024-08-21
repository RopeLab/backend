// @generated automatically by Diesel CLI.

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
        salt -> Text,
    }
}

diesel::joinable!(user_data -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    user_data,
    users,
);
