CREATE TABLE ai_model_pricing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider TEXT NOT NULL CHECK (provider IN ('anthropic', 'openai')),
    model TEXT NOT NULL,
    input_cost_per_1k_micros BIGINT NOT NULL,
    output_cost_per_1k_micros BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider, model)
);

INSERT INTO ai_model_pricing (provider, model, input_cost_per_1k_micros, output_cost_per_1k_micros) VALUES
    ('openai',    'gpt-4o-mini',        150,    600),
    ('openai',    'gpt-4o',             2500,   10000),
    ('anthropic', 'claude-3-5-sonnet',  3000,   15000),
    ('anthropic', 'claude-3-5-haiku',   800,    4000);

CREATE TABLE ai_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID NULL REFERENCES projects(id) ON DELETE SET NULL,
    feature_id UUID NULL REFERENCES features(id) ON DELETE SET NULL,
    operation_type TEXT NOT NULL CHECK (operation_type IN (
        'enhance_project_requirements',
        'split_project_into_features',
        'enhance_feature_requirements',
        'generate_tasks',
        'regenerate_tasks'
    )),
    provider TEXT NOT NULL CHECK (provider IN ('anthropic', 'openai')),
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0 CHECK (input_tokens >= 0),
    output_tokens INTEGER NOT NULL DEFAULT 0 CHECK (output_tokens >= 0),
    estimated_cost_micros BIGINT NOT NULL DEFAULT 0 CHECK (estimated_cost_micros >= 0),
    status TEXT NOT NULL CHECK (status IN ('success', 'failed')),
    error_code TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((status = 'failed') OR error_code IS NULL)
);

CREATE INDEX idx_ai_usage_company_created ON ai_usage(company_id, created_at DESC);
CREATE INDEX idx_ai_usage_user_created ON ai_usage(user_id, created_at DESC);
CREATE INDEX idx_ai_usage_project_created ON ai_usage(project_id, created_at DESC);
CREATE INDEX idx_ai_usage_feature_created ON ai_usage(feature_id, created_at DESC);
