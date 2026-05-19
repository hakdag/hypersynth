CREATE TABLE project_ai_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider IN ('anthropic', 'openai')),
    encrypted_api_key BYTEA NOT NULL,
    allowed_models TEXT[] NOT NULL DEFAULT '{}',
    monthly_token_limit BIGINT NULL CHECK (
        monthly_token_limit IS NULL
        OR monthly_token_limit > 0
    ),
    usage_tracking_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (array_length(allowed_models, 1) IS NULL OR array_length(allowed_models, 1) > 0)
);

CREATE INDEX idx_project_ai_settings_project_id ON project_ai_settings(project_id);

INSERT INTO project_ai_settings (
    project_id,
    provider,
    encrypted_api_key,
    allowed_models,
    usage_tracking_enabled
)
SELECT
    id,
    'anthropic',
    encrypted_api_key,
    '{}',
    TRUE
FROM projects
WHERE encrypted_api_key IS NOT NULL;

ALTER TABLE projects DROP COLUMN encrypted_api_key;
