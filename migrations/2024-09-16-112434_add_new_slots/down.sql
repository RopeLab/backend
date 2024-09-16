-- This file should undo anything in `up.sql`
ALTER TABLE "event"
DROP COLUMN "new_slots";
