-- SF-28 System Admin Health and Configuration
-- Platform-wide settings (singleton row).

CREATE TABLE platform_config (
    id SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    allowed_ai_providers TEXT[] NOT NULL DEFAULT ARRAY['anthropic', 'openai']::TEXT[],
    default_monthly_token_limit BIGINT NULL CHECK (
        default_monthly_token_limit IS NULL
        OR default_monthly_token_limit > 0
    ),
    platform_announcement TEXT NULL,
    feature_flags JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO platform_config (id) VALUES (1);

SELECT audit.attach('platform_config');
