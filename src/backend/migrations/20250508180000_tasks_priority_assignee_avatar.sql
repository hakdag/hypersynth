ALTER TABLE users ADD COLUMN avatar_url TEXT
    CHECK (
        avatar_url IS NULL
        OR (
            char_length(avatar_url) >= 1
            AND char_length(avatar_url) <= 2048
        )
    );

ALTER TABLE tasks ADD COLUMN priority TEXT NOT NULL DEFAULT 'Standard'
    CHECK (priority IN ('Standard', 'Elevated', 'Critical'));

ALTER TABLE tasks ADD COLUMN assignee_user_id UUID REFERENCES users (id) ON DELETE SET NULL;

ALTER TABLE tasks ADD COLUMN creator_user_id UUID REFERENCES users (id) ON DELETE SET NULL;

CREATE INDEX idx_tasks_assignee_user_id ON tasks (assignee_user_id);
