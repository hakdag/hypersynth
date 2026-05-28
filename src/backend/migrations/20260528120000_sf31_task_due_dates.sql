ALTER TABLE tasks
    ADD COLUMN due_date DATE NULL,
    ADD COLUMN due_time TIME NULL;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_due_time_requires_date
    CHECK (due_time IS NULL OR due_date IS NOT NULL);

CREATE INDEX idx_tasks_due_date ON tasks (due_date);
