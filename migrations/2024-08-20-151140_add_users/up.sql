-- Your SQL goes here
CREATE TABLE "users"(
    "id" SERIAL PRIMARY KEY NOT NULL,
    "email" TEXT NOT NULL,
    "pw_hash" TEXT NOT NULL,
);

CREATE TABLE "user_data"(
    "id" SERIAL PRIMARY KEY NOT NULL,
    "user_id" SERIAL REFERENCES users(id),

    "name" TEXT NOT NULL,
    "fetlife_name" TEXT NOT NULL,

    "experience_text" TEXT NOT NULL,
    "found_us_text" TEXT NOT NULL,
    "goal_text" TEXT NOT NULL,

    "role_factor" FLOAT NOT NULL,
    "active_factor" FLOAT NOT NULL,
    "passive_factor" FLOAT NOT NULL,
    "open" BOOL NOT NULL,

    "show_name" BOOL NOT NULL,
    "show_role" BOOL NOT NULL,
    "show_experience" BOOL NOT NULL,
    "show_open" BOOL NOT NULL
);