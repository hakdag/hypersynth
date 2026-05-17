ALTER TABLE users
    ADD COLUMN username TEXT NULL,
    ADD COLUMN display_name TEXT NULL,
    ADD COLUMN role TEXT NULL CHECK (role IN ('company_admin','project_manager','contributor','viewer')),
    ADD COLUMN status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','disabled','pending_invitation')),
    ADD COLUMN timezone TEXT NULL,
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();

CREATE UNIQUE INDEX idx_users_username_lower ON users (lower(username)) WHERE username IS NOT NULL;
