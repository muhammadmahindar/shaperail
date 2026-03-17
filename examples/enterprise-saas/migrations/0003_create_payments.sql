CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    invoice_id UUID NOT NULL REFERENCES invoices(id),
    amount_cents BIGINT NOT NULL,
    method TEXT NOT NULL CHECK (method IN ('card', 'bank_transfer', 'check')),
    reference_number TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed', 'failed', 'refunded')),
    processed_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_org_invoice ON payments (org_id, invoice_id);
CREATE INDEX idx_payments_org_status ON payments (org_id, status);
