-- Your SQL goes here
CREATE TYPE EventUserState AS ENUM ('registered', 'waiting', 'rejected', 'new', 'waiting_new');

CREATE TABLE "event"(
    "id" SERIAL PRIMARY KEY NOT NULL,
    "date" TIMESTAMP NOT NULL,
    "slots" INT NOT NULL,
    "visible" BOOL NOT NULL,
    "archive" BOOL NOT NULL
);

CREATE TABLE "event_user"(
    "id" SERIAL PRIMARY KEY NOT NULL,
    "user_id" INT NOT NULL REFERENCES users(id),
    "event_id" INT NOT NULL REFERENCES event(id),
    "slot" INT NOT NULL,
    "state" EventUserState NOT NULL,
    "guests" INT NOT NULL,
    "attended" BOOL NOT NULL
);