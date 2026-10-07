CREATE TABLE servers (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    server_type TEXT NOT NULL CHECK (server_type IN ('kavita', 'suwayomi', 'opds')),
    url TEXT NOT NULL
);
