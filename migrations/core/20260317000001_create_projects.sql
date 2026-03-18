CREATE TABLE IF NOT EXISTS projects (
    id         TEXT NOT NULL PRIMARY KEY,   -- UUID stored as text
    name       TEXT NOT NULL,
    db_path    TEXT NOT NULL,               -- absolute path to the project's .db file
    created_at TEXT NOT NULL                -- ISO 8601 timestamp
);
