-- This file should undo anything in `up.sql`
ALTER TABLE "event"
RENAME COLUMN "custom_workshop" to "description";