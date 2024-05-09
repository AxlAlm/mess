-- Deploy mess:muck to sqlite

BEGIN;

CREATE TABLE muck (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    registered_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deregistered_at DATETIME,
    alive INTEGER NOT NULL,

    UNIQUE(name)
);

COMMIT;
