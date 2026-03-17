CREATE TABLE IF NOT EXISTS incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    service_id UUID NOT NULL REFERENCES services(id),
    title VARCHAR(200) NOT NULL,
    slug VARCHAR(200) NOT NULL,
    severity TEXT NOT NULL DEFAULT 'sev3'
        CHECK (severity IN ('sev1', 'sev2', 'sev3', 'sev4')),
    status TEXT NOT NULL DEFAULT 'open'
        CHECK (status IN ('open', 'acknowledged', 'mitigated', 'resolved', 'closed')),
    summary VARCHAR(500) NOT NULL,
    commander_id UUID,
    room_key VARCHAR(200) NOT NULL,
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_incidents_org_status ON incidents (org_id, status);
CREATE INDEX IF NOT EXISTS idx_incidents_org_severity ON incidents (org_id, severity);
CREATE INDEX IF NOT EXISTS idx_incidents_service_created_at ON incidents (service_id, created_at);
