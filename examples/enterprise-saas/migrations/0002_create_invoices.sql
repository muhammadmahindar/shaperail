CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    customer_id UUID NOT NULL REFERENCES customers(id),
    invoice_number TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'pending', 'sent', 'paid', 'overdue', 'void')),
    subtotal_cents BIGINT NOT NULL,
    tax_cents BIGINT NOT NULL DEFAULT 0,
    total_cents BIGINT NOT NULL,
    due_date DATE NOT NULL,
    sent_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_invoices_org_customer ON invoices (org_id, customer_id);
CREATE INDEX idx_invoices_org_status ON invoices (org_id, status);
CREATE INDEX idx_invoices_due_date ON invoices (due_date);
