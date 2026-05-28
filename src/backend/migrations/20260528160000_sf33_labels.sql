CREATE TABLE labels (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    company_id UUID REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT labels_tenant_xor
        CHECK ((company_id IS NULL) <> (user_id IS NULL)),
    CONSTRAINT labels_color_format
        CHECK (color ~ '^#[0-9A-Fa-f]{6}$')
);

CREATE UNIQUE INDEX labels_unique_company_name
    ON labels (company_id, lower(name))
    WHERE company_id IS NOT NULL;

CREATE UNIQUE INDEX labels_unique_user_name
    ON labels (user_id, lower(name))
    WHERE user_id IS NOT NULL;

CREATE INDEX labels_company_id_idx ON labels(company_id);
CREATE INDEX labels_user_id_idx ON labels(user_id);

CREATE TABLE task_labels (
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, label_id)
);

CREATE INDEX task_labels_label_id_idx ON task_labels(label_id);
