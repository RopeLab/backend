-- This file should undo anything in `up.sql`
ALTER TABLE "event"
DROP COLUMN "register_deadline";

ALTER TABLE "event"
DROP COLUMN "visible_date";

ALTER TABLE "event"
DROP COLUMN "archive_date";

ALTER TABLE "event"
DROP COLUMN "description";