-- Add migration script here
ALTER TABLE tasks
ADD COLUMN in_backlog BOOLEAN NOT NULL DEFAULT 1;
