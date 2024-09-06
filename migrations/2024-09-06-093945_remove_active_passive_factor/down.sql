-- This file should undo anything in `up.sql`
ALTER TABLE "user_data"
ADD "active_factor" FLOAT NOT NULL DEFAULT 0;

ALTER TABLE "user_data"
ADD "passive_factor" FLOAT NOT NULL DEFAULT 0;

ALTER TABLE "user_data"
ADD "show_experience" BOOL NOT NULL DEFAULT FALSE;


