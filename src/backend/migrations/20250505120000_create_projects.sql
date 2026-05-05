CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (
        char_length(name) >= 1
        AND char_length(name) <= 512
    ),
    requirements TEXT CHECK (
        requirements IS NULL
        OR char_length(requirements) <= 1048576
    ),
    status TEXT NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'In Progress', 'Done')),
    ai_api_key TEXT CHECK (
        ai_api_key IS NULL
        OR char_length(ai_api_key) <= 4096
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_projects_user_id ON projects (user_id);
