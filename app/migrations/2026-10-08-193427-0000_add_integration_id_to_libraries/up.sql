ALTER TABLE libraries ADD COLUMN integration_id TEXT REFERENCES integrations(id);
