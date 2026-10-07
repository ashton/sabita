CREATE TABLE integrations (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    integration_type TEXT NOT NULL CHECK (integration_type IN ('kavita')),
    url TEXT,
    api_key TEXT
);
