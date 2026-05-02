-- Payment Terms Management
-- Oracle Fusion: Financials > Payment Terms
-- Defines payment terms with discount schedules for AP and AR

CREATE TABLE IF NOT EXISTS financials.payment_terms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    term_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    base_due_days INT NOT NULL DEFAULT 30,
    due_date_cutoff_day INT, -- e.g., 25 means due by 25th of month
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    term_type VARCHAR(20) NOT NULL DEFAULT 'standard', -- standard, installment, proxima, day_of_month
    default_discount_percent VARCHAR(20) NOT NULL DEFAULT '0',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, term_code)
);

CREATE TABLE IF NOT EXISTS financials.payment_term_discount_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_term_id UUID NOT NULL REFERENCES financials.payment_terms(id) ON DELETE CASCADE,
    discount_percent VARCHAR(20) NOT NULL DEFAULT '0',
    discount_days INT NOT NULL DEFAULT 0,
    discount_day_of_month INT, -- for proxima terms
    discount_basis VARCHAR(20) NOT NULL DEFAULT 'invoice_amount', -- invoice_amount, line_amount
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS financials.payment_term_installments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_term_id UUID NOT NULL REFERENCES financials.payment_terms(id) ON DELETE CASCADE,
    installment_number INT NOT NULL DEFAULT 1,
    due_days_offset INT NOT NULL DEFAULT 0,
    percentage VARCHAR(20) NOT NULL DEFAULT '100',
    discount_percent VARCHAR(20) NOT NULL DEFAULT '0',
    discount_days INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_payment_terms_org ON financials.payment_terms(organization_id);
CREATE INDEX IF NOT EXISTS idx_payment_terms_status ON financials.payment_terms(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_payment_term_discounts ON financials.payment_term_discount_schedules(payment_term_id);
CREATE INDEX IF NOT EXISTS idx_payment_term_installments ON financials.payment_term_installments(payment_term_id);
