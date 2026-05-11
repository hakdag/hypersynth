ALTER TABLE users
    ADD COLUMN account_type TEXT NOT NULL DEFAULT 'personal'
    CHECK (account_type IN ('personal', 'company'));

ALTER TABLE users ALTER COLUMN account_type DROP DEFAULT;
