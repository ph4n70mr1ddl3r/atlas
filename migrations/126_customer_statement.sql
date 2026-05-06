-- Customer Account Statement / Balance Forward Billing
-- Oracle Fusion: Receivables > Billing > Balance Forward Billing
-- Generates consolidated customer statements showing opening balance,
-- new charges, payments/credits, closing balance, and aging breakdown.

CREATE TABLE IF NOT EXISTS _atlas.customer_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_number VARCHAR(50) NOT NULL,
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    statement_date DATE NOT NULL,
    billing_period_from DATE NOT NULL,
    billing_period_to DATE NOT NULL,
    billing_cycle VARCHAR(30) NOT NULL DEFAULT 'monthly',  -- monthly, quarterly, weekly, custom
    opening_balance DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_charges DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_payments DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_credits DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_adjustments DOUBLE PRECISION NOT NULL DEFAULT 0,
    closing_balance DOUBLE PRECISION NOT NULL DEFAULT 0,
    amount_due DOUBLE PRECISION NOT NULL DEFAULT 0,
    aging_current DOUBLE PRECISION NOT NULL DEFAULT 0,
    aging_1_30 DOUBLE PRECISION NOT NULL DEFAULT 0,
    aging_31_60 DOUBLE PRECISION NOT NULL DEFAULT 0,
    aging_61_90 DOUBLE PRECISION NOT NULL DEFAULT 0,
    aging_91_120 DOUBLE PRECISION NOT NULL DEFAULT 0,
    aging_121_plus DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    delivery_method VARCHAR(30),                  -- email, print, xml, edi
    delivery_email VARCHAR(200),
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, generated, sent, viewed, archived, cancelled
    generated_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    viewed_at TIMESTAMPTZ,
    notes TEXT,
    previous_statement_id UUID,                  -- link to prior statement
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, statement_number)
);

CREATE TABLE IF NOT EXISTS _atlas.customer_statement_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_id UUID NOT NULL REFERENCES _atlas.customer_statements(id) ON DELETE CASCADE,
    line_type VARCHAR(30) NOT NULL,              -- opening_balance, invoice, payment, credit_memo, debit_memo, adjustment, finance_charge, closing_balance
    transaction_id UUID,                          -- reference to the source transaction
    transaction_number VARCHAR(50),
    transaction_date DATE,
    due_date DATE,
    original_amount DOUBLE PRECISION,
    amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    running_balance DOUBLE PRECISION,
    description TEXT,
    reference_type VARCHAR(50),                   -- invoice, receipt, credit_memo, debit_memo, adjustment
    reference_id UUID,
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_customer_statements_org ON _atlas.customer_statements(organization_id);
CREATE INDEX IF NOT EXISTS idx_customer_statements_customer ON _atlas.customer_statements(organization_id, customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_statements_status ON _atlas.customer_statements(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_customer_statements_date ON _atlas.customer_statements(organization_id, statement_date DESC);
CREATE INDEX IF NOT EXISTS idx_customer_statement_lines_statement ON _atlas.customer_statement_lines(statement_id);
CREATE INDEX IF NOT EXISTS idx_customer_statement_lines_type ON _atlas.customer_statement_lines(statement_id, line_type);
