-- Your SQL goes here
CREATE TYPE UserPermission AS ENUM ('admin', 'verified');

CREATE TABLE "permission"(
    "id" SERIAL PRIMARY KEY NOT NULL,
    "user_id" INT NOT NULL REFERENCES users(id),
    "user_permission" UserPermission NOT NULL
);