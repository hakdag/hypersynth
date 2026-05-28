CREATE TABLE task_comment_mentions (
    comment_id UUID NOT NULL REFERENCES task_comments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (comment_id, user_id)
);

CREATE INDEX task_comment_mentions_user_id_idx
    ON task_comment_mentions (user_id);
