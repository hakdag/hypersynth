-- SF-24 Audit Logging
-- Debezium-envelope row-change log plus a small parallel non-data-event log.
-- Row-change capture happens via a generic PL/pgSQL trigger attached to
-- audited business tables. Actor context propagates from the application
-- through transaction-local GUCs (app.actor, app.request_id, app.ip_address,
-- app.user_agent) set by the request middleware.

CREATE SCHEMA audit;

-- Columns whose values must never be persisted in plaintext audit snapshots.
-- New tables that hold secrets must add their columns here in the same
-- migration that introduces the column.
CREATE TABLE audit.masked_columns (
    table_name  TEXT NOT NULL,
    column_name TEXT NOT NULL,
    PRIMARY KEY (table_name, column_name)
);

INSERT INTO audit.masked_columns (table_name, column_name) VALUES
    ('users', 'password_hash'),
    ('project_ai_settings', 'encrypted_api_key'),
    ('sessions', 'token_hash'),
    ('invitations', 'invitation_token_hash');

-- Debezium-envelope row-change log. Partitioned by ts_ms (epoch milliseconds)
-- so that retention/archival can drop month-sized partitions cheaply.
CREATE TABLE audit_row_changes (
    id          BIGSERIAL,
    ts_ms       BIGINT      NOT NULL,
    op          CHAR(1)     NOT NULL CHECK (op IN ('c','u','d')),
    source      JSONB       NOT NULL,
    "before"    JSONB,
    "after"     JSONB,
    actor       JSONB,
    request_id  UUID,
    ip_address  INET,
    user_agent  TEXT,
    PRIMARY KEY (id, ts_ms)
) PARTITION BY RANGE (ts_ms);

CREATE INDEX idx_audit_row_changes_ts ON audit_row_changes (ts_ms DESC);
CREATE INDEX idx_audit_row_changes_request_id ON audit_row_changes (request_id);
CREATE INDEX idx_audit_row_changes_source_table
    ON audit_row_changes ((source->>'table'), ts_ms DESC);
CREATE INDEX idx_audit_row_changes_actor_user
    ON audit_row_changes ((actor->>'user_id'), ts_ms DESC);
CREATE INDEX idx_audit_row_changes_actor_company
    ON audit_row_changes ((actor->>'company_id'), ts_ms DESC);
CREATE INDEX idx_audit_row_changes_after_gin
    ON audit_row_changes USING GIN ("after" jsonb_path_ops);

-- Initial monthly partitions: current month plus the next five.
DO $$
DECLARE
    m_start    timestamptz;
    m_end      timestamptz;
    m_start_ms bigint;
    m_end_ms   bigint;
    pname      text;
BEGIN
    m_start := date_trunc('month', timezone('UTC', now()));
    FOR i IN 0..5 LOOP
        m_end := m_start + interval '1 month';
        m_start_ms := (EXTRACT(EPOCH FROM m_start) * 1000)::bigint;
        m_end_ms   := (EXTRACT(EPOCH FROM m_end)   * 1000)::bigint;
        pname := format('audit_row_changes_%s', to_char(m_start, 'YYYY_MM'));
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF audit_row_changes FOR VALUES FROM (%s) TO (%s)',
            pname, m_start_ms, m_end_ms
        );
        m_start := m_end;
    END LOOP;
END $$;

-- Append-only non-data event log: login attempts, AI request markers, etc.
CREATE TABLE audit_events (
    id          BIGSERIAL PRIMARY KEY,
    ts_ms       BIGINT NOT NULL,
    event_type  TEXT   NOT NULL,
    actor       JSONB,
    payload     JSONB  NOT NULL DEFAULT '{}'::jsonb,
    request_id  UUID,
    ip_address  INET,
    user_agent  TEXT
);

CREATE INDEX idx_audit_events_ts ON audit_events (ts_ms DESC);
CREATE INDEX idx_audit_events_event_type ON audit_events (event_type, ts_ms DESC);
CREATE INDEX idx_audit_events_actor_user
    ON audit_events ((actor->>'user_id'), ts_ms DESC);
CREATE INDEX idx_audit_events_actor_company
    ON audit_events ((actor->>'company_id'), ts_ms DESC);
CREATE INDEX idx_audit_events_request_id ON audit_events (request_id);

-- Hard-fail any attempt to mutate audit history at the SQL layer.
CREATE OR REPLACE FUNCTION audit.deny_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'audit tables are append-only (table=%, op=%)',
        TG_TABLE_NAME, TG_OP;
END;
$$;

CREATE TRIGGER audit_row_changes_deny_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON audit_row_changes
    FOR EACH STATEMENT EXECUTE FUNCTION audit.deny_mutation();

CREATE TRIGGER audit_events_deny_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON audit_events
    FOR EACH STATEMENT EXECUTE FUNCTION audit.deny_mutation();

-- Generic row-change recorder. Builds a Debezium-style envelope from OLD/NEW,
-- masks secrets, reads actor/request context from transaction-local GUCs.
-- Never raises: any failure to parse a GUC value falls back to NULL so that
-- the originating mutation is not blocked by audit-context bookkeeping.
CREATE OR REPLACE FUNCTION audit.record_row_change()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    v_op           char(1);
    v_before       jsonb;
    v_after        jsonb;
    v_actor        jsonb;
    v_request_id   uuid;
    v_ip           inet;
    v_user_agent   text;
    v_actor_text   text;
    v_request_text text;
    v_ip_text      text;
    v_mask_col     text;
BEGIN
    IF TG_OP = 'INSERT' THEN
        v_op := 'c';
        v_before := NULL;
        v_after  := to_jsonb(NEW);
    ELSIF TG_OP = 'UPDATE' THEN
        v_op := 'u';
        v_before := to_jsonb(OLD);
        v_after  := to_jsonb(NEW);
    ELSIF TG_OP = 'DELETE' THEN
        v_op := 'd';
        v_before := to_jsonb(OLD);
        v_after  := NULL;
    END IF;

    FOR v_mask_col IN
        SELECT column_name FROM audit.masked_columns
        WHERE table_name = TG_TABLE_NAME
    LOOP
        IF v_before IS NOT NULL AND v_before ? v_mask_col THEN
            v_before := jsonb_set(v_before, ARRAY[v_mask_col], to_jsonb('***'::text));
        END IF;
        IF v_after IS NOT NULL AND v_after ? v_mask_col THEN
            v_after := jsonb_set(v_after, ARRAY[v_mask_col], to_jsonb('***'::text));
        END IF;
    END LOOP;

    v_actor_text   := current_setting('app.actor', true);
    v_request_text := current_setting('app.request_id', true);
    v_ip_text      := current_setting('app.ip_address', true);
    v_user_agent   := current_setting('app.user_agent', true);

    BEGIN
        v_actor := nullif(v_actor_text, '')::jsonb;
    EXCEPTION WHEN OTHERS THEN
        v_actor := NULL;
    END;

    BEGIN
        v_request_id := nullif(v_request_text, '')::uuid;
    EXCEPTION WHEN OTHERS THEN
        v_request_id := NULL;
    END;

    BEGIN
        v_ip := nullif(v_ip_text, '')::inet;
    EXCEPTION WHEN OTHERS THEN
        v_ip := NULL;
    END;

    INSERT INTO audit_row_changes (
        ts_ms, op, source, "before", "after",
        actor, request_id, ip_address, user_agent
    ) VALUES (
        (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint,
        v_op,
        jsonb_build_object(
            'schema', TG_TABLE_SCHEMA,
            'table',  TG_TABLE_NAME,
            'tx_id',  txid_current()
        ),
        v_before,
        v_after,
        v_actor,
        v_request_id,
        v_ip,
        nullif(v_user_agent, '')
    );

    RETURN NULL;
END;
$$;

-- Attaches the generic row-change trigger to a business table.
-- Future migrations that introduce a new audited table should end with
-- a call to audit.attach('<table>').
CREATE OR REPLACE FUNCTION audit.attach(p_table regclass)
RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    v_rel  text;
    v_name text;
BEGIN
    SELECT relname INTO v_rel FROM pg_class WHERE oid = p_table;
    v_name := format('%s_audit', v_rel);
    EXECUTE format(
        'CREATE TRIGGER %I AFTER INSERT OR UPDATE OR DELETE ON %s '
        'FOR EACH ROW EXECUTE FUNCTION audit.record_row_change()',
        v_name, p_table
    );
END;
$$;

SELECT audit.attach('users');
SELECT audit.attach('companies');
SELECT audit.attach('company_users');
SELECT audit.attach('projects');
SELECT audit.attach('project_memberships');
SELECT audit.attach('features');
SELECT audit.attach('tasks');
SELECT audit.attach('project_documents');
SELECT audit.attach('invitations');
SELECT audit.attach('project_ai_settings');
