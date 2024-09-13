-- Your SQL goes here
CREATE TYPE EventUserAction AS ENUM ('register', 'unregister', 'get_slot', 'rejected', 'change_guests');

CREATE TABLE "user_action"(
    "id" SERIAL PRIMARY KEY NOT NULL,
    "user_id" INT NOT NULL REFERENCES users(id),
    "event_id" INT NOT NULL REFERENCES event(id),
    "date" TIMESTAMP NOT NULL,
    "action" EventUserAction NOT NULL,
    "in_waiting" BOOL NOT NULL,
    "in_new" BOOL NOT NULL,
    "guests" INT NOT NULL
);