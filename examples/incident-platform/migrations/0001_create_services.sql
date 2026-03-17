CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    slug VARCHAR(200) NOT NULL,
    tier TEXT NOT NULL DEFAULT 'standard'
        CHECK (tier IN ('critical', 'high', 'standard')),
    status TEXT NOT NULL DEFAULT 'healthy'
        CHECK (status IN ('healthy', 'degraded', 'down', 'maintenance')),
    owner_team VARCHAR(120) NOT NULL,
    runbook_url TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_services_org_slug ON services (org_id, slug);
CREATE INDEX IF NOT EXISTS idx_services_org_status ON services (org_id, status);
CREATE INDEX IF NOT EXISTS idx_services_org_tier ON services (org_id, tier);
