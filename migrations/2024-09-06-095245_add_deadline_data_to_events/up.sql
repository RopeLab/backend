-- Your SQL goes here
ALTER TABLE "event"
ADD "register_deadline" TIMESTAMP NOT NULL;

ALTER TABLE "event"
ADD "visible_date" TIMESTAMP NOT NULL;

ALTER TABLE "event"
ADD "archive_date" TIMESTAMP NOT NULL;

ALTER TABLE "event"
ADD "description" TEXT NOT NULL;