CREATE TABLE task_comments (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT task_comments_content_nonempty CHECK (char_length(trim(content)) >= 1),
    CONSTRAINT task_comments_content_max CHECK (char_length(content) <= 10000)
);

CREATE INDEX task_comments_task_id_created_at_idx
    ON task_comments (task_id, created_at);
