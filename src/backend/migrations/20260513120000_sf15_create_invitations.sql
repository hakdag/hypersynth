CREATE TABLE invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invitation_token_hash TEXT NOT NULL,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    project_id UUID NULL REFERENCES projects(id) ON DELETE SET NULL,
    invited_email TEXT NOT NULL,
    invited_role TEXT NOT NULL CHECK (invited_role IN ('company_admin', 'project_manager', 'contributor', 'viewer')),
    invited_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'expired', 'cancelled')),
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT invitations_invited_email_length CHECK (char_length(invited_email) <= 320)
);

CREATE UNIQUE INDEX idx_invitations_token_hash ON invitations (invitation_token_hash);

CREATE UNIQUE INDEX idx_invitations_pending_company_email
    ON invitations (company_id, lower(invited_email))
    WHERE status = 'pending';

CREATE INDEX idx_invitations_company_status ON invitations (company_id, status);
