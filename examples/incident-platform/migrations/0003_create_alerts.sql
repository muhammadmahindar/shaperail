CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    service_id UUID NOT NULL REFERENCES services(id),
    incident_id UUID REFERENCES incidents(id),
    external_id VARCHAR(200) NOT NULL,
    source TEXT NOT NULL
        CHECK (source IN ('pagerduty', 'prometheus', 'datadog', 'manual')),
    severity TEXT NOT NULL DEFAULT 'sev3'
        CHECK (severity IN ('sev1', 'sev2', 'sev3', 'sev4')),
    status TEXT NOT NULL DEFAULT 'received'
        CHECK (status IN ('received', 'deduped', 'linked', 'ignored')),
    fingerprint VARCHAR(200) NOT NULL,
    summary VARCHAR(500) NOT NULL,
    payload JSONB NOT NULL,
    created_by UUID,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_alerts_org_external_id ON alerts (org_id, external_id);
CREATE INDEX IF NOT EXISTS idx_alerts_org_status ON alerts (org_id, status);
CREATE INDEX IF NOT EXISTS idx_alerts_service_received_at ON alerts (service_id, received_at);
