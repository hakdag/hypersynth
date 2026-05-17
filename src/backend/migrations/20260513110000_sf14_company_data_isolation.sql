ALTER TABLE projects
    RENAME COLUMN user_id TO owner_user_id;

ALTER TABLE projects
    ALTER COLUMN owner_user_id DROP NOT NULL;

ALTER TABLE projects
    ADD COLUMN company_id UUID NULL REFERENCES companies(id) ON DELETE CASCADE,
    ADD COLUMN created_by_user_id UUID NULL REFERENCES users(id) ON DELETE SET NULL;

WITH company_owner AS (
    SELECT DISTINCT ON (cu.user_id)
        cu.user_id,
        cu.company_id
    FROM company_users cu
    ORDER BY cu.user_id, cu.created_at ASC
)
UPDATE projects p
SET
    company_id = co.company_id,
    created_by_user_id = p.owner_user_id,
    owner_user_id = NULL
FROM company_owner co
WHERE p.owner_user_id = co.user_id;

UPDATE projects
SET created_by_user_id = owner_user_id
WHERE created_by_user_id IS NULL;

ALTER TABLE projects
    ADD CONSTRAINT projects_tenant_xor
    CHECK ((company_id IS NULL) <> (owner_user_id IS NULL));

DROP INDEX IF EXISTS idx_projects_user_id;
CREATE INDEX idx_projects_owner_user_id ON projects (owner_user_id);
CREATE INDEX idx_projects_company_id ON projects (company_id);
CREATE INDEX idx_projects_created_by_user_id ON projects (created_by_user_id);

CREATE TABLE project_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, user_id)
);

CREATE INDEX idx_project_memberships_user_id ON project_memberships (user_id);
