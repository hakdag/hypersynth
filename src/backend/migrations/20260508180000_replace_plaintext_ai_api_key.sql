ALTER TABLE projects DROP COLUMN ai_api_key;
ALTER TABLE projects ADD COLUMN encrypted_api_key BYTEA NULL;
