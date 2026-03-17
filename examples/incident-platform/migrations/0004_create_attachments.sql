CREATE TABLE IF NOT EXISTS attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    incident_id UUID NOT NULL REFERENCES incidents(id),
    kind TEXT NOT NULL
        CHECK (kind IN ('screenshot', 'log', 'runbook', 'artifact')),
    file_url TEXT NOT NULL,
    file_url_filename TEXT,
    file_url_mime_type TEXT,
    file_url_size BIGINT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_attachments_org_incident ON attachments (org_id, incident_id);
CREATE INDEX IF NOT EXISTS idx_attachments_incident_kind ON attachments (incident_id, kind);
