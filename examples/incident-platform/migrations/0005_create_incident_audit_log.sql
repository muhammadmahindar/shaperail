CREATE TABLE IF NOT EXISTS incident_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID NOT NULL REFERENCES incidents(id),
    org_id UUID NOT NULL,
    action TEXT NOT NULL,
    record_data JSONB NOT NULL,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_incident_audit_log_incident_created_at
    ON incident_audit_log (incident_id, created_at DESC);
