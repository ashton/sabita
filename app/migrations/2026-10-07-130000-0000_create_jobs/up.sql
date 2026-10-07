CREATE TABLE jobs (
    id TEXT NOT NULL PRIMARY KEY,
    job_type TEXT NOT NULL CHECK (job_type IN ('sync_integration_libraries')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    integration_id TEXT,
    error TEXT,
    created_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP,
    finished_at TIMESTAMP
);
