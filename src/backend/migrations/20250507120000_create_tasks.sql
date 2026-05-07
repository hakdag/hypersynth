CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feature_id UUID NOT NULL REFERENCES features (id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (
        char_length(title) >= 1
        AND char_length(title) <= 512
    ),
    description TEXT CHECK (
        description IS NULL
        OR char_length(description) <= 1048576
    ),
    status TEXT NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'In Progress', 'Done')),
    created_by TEXT NOT NULL DEFAULT 'User' CHECK (created_by IN ('User', 'AI')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tasks_feature_id ON tasks (feature_id);
