ALTER TABLE project_memberships
    ADD COLUMN role TEXT NOT NULL DEFAULT 'contributor'
    CHECK (role IN ('project_manager', 'contributor', 'viewer'));

ALTER TABLE project_memberships ALTER COLUMN role DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_project_memberships_project_id ON project_memberships (project_id);

INSERT INTO project_memberships (project_id, user_id, role)
SELECT p.id, p.created_by_user_id, 'project_manager'
FROM projects p
WHERE p.company_id IS NOT NULL
  AND p.created_by_user_id IS NOT NULL
ON CONFLICT (project_id, user_id) DO NOTHING;
