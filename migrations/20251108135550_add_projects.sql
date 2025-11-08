-- Add migration script here

PRAGMA foreign_keys=off;

CREATE TABLE projects (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL
);

INSERT INTO projects (
  name
) VALUES ( 'Default' );

ALTER TABLE tasks RENAME TO tasks_old;

CREATE TABLE tasks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  description TEXT,
  lane TEXT NOT NULL,
  in_backlog BOOLEAN NOT NULL DEFAULT 1,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES tasks(id)
);

INSERT INTO tasks (id, title, description, lane, in_backlog, project_id)
SELECT id, title, description, lane, in_backlog, 1 FROM tasks_old;

DROP TABLE tasks_old;

PRAGMA foreign_keys=on;
