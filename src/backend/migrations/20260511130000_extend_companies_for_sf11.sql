ALTER TABLE companies
    ADD COLUMN name TEXT,
    ADD COLUMN company_email TEXT,
    ADD COLUMN country TEXT,
    ADD COLUMN timezone TEXT,
    ADD COLUMN legal_name TEXT NULL,
    ADD COLUMN website TEXT NULL,
    ADD COLUMN industry TEXT NULL,
    ADD COLUMN company_size TEXT NULL,
    ADD COLUMN phone TEXT NULL,
    ADD COLUMN billing_email TEXT NULL,
    ADD COLUMN address TEXT NULL,
    ADD COLUMN tax_vat_number TEXT NULL,
    ADD COLUMN status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','disabled','pending_verification')),
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();

UPDATE companies
SET
    name = 'Unnamed Company',
    company_email = id::text || '@migration.local',
    country = 'Unknown',
    timezone = 'UTC'
WHERE name IS NULL;

ALTER TABLE companies
    ALTER COLUMN name SET NOT NULL,
    ALTER COLUMN company_email SET NOT NULL,
    ALTER COLUMN country SET NOT NULL,
    ALTER COLUMN timezone SET NOT NULL;

ALTER TABLE companies
    ADD CONSTRAINT companies_name_length CHECK (char_length(name) BETWEEN 1 AND 255),
    ADD CONSTRAINT companies_company_email_length CHECK (char_length(company_email) <= 320);

CREATE UNIQUE INDEX idx_companies_company_email_lower ON companies (lower(company_email));
