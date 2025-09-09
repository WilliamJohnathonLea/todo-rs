-- Add migration script here
CREATE TABLE tasks_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    lane TEXT NOT NULL
);

DROP TABLE tasks;

ALTER TABLE tasks_new RENAME TO tasks;
