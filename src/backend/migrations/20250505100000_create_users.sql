CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fullname TEXT NOT NULL CHECK (
        char_length(fullname) >= 1
        AND char_length(fullname) <= 512
    ),
    email TEXT NOT NULL CHECK (char_length(email) <= 320),
    password_hash TEXT NOT NULL CHECK (char_length(password_hash) >= 1),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_users_email_lower ON users (lower(email));
