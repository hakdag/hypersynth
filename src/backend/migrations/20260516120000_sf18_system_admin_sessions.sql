ALTER TABLE sessions
    ALTER COLUMN user_id DROP NOT NULL;

ALTER TABLE sessions
    ADD COLUMN is_system_admin BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE sessions
    ADD COLUMN system_admin_email TEXT NULL;

ALTER TABLE sessions
    ADD CONSTRAINT sessions_system_admin_shape CHECK (
        (
            is_system_admin = false
            AND user_id IS NOT NULL
            AND system_admin_email IS NULL
        )
        OR (
            is_system_admin = true
            AND user_id IS NULL
            AND system_admin_email IS NOT NULL
            AND char_length(trim(system_admin_email)) >= 1
        )
    );
