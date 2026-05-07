CREATE TABLE features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (
        char_length(title) >= 1
        AND char_length(title) <= 512
    ),
    requirements TEXT CHECK (
        requirements IS NULL
        OR char_length(requirements) <= 1048576
    ),
    status TEXT NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'In Progress', 'Done')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_features_project_id ON features (project_id);
