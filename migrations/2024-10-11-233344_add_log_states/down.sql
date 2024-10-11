-- This file should undo anything in `up.sql`
ALTER TYPE EventUserAction DROP VALUE 'attended';
ALTER TYPE EventUserAction DROP VALUE 'not_attended';
ALTER TYPE EventUserAction DROP VALUE 'not_rejected';