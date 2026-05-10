CREATE TABLE project_api_key_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK (
        event_type IN ('created', 'replaced', 'cleared', 'runtime_use')
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_project_api_key_audit_project_id ON project_api_key_audit (project_id);
CREATE INDEX idx_project_api_key_audit_user_id ON project_api_key_audit (user_id);
