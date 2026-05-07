-- Chargeback Management
-- Oracle Fusion: Financials > Receivables > Chargebacks
-- Manages customer-initiated payment deductions with full lifecycle workflow.
-- Chargebacks represent deductions taken by customers from their payments
-- for reasons such as damaged goods, pricing disputes, promotional allowances,
-- short shipments, returns, and quality issues.

CREATE TABLE IF NOT EXISTS _atlas.chargebacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    chargeback_number VARCHAR(50) NOT NULL,
    customer_id UUID,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    receipt_id UUID,
    receipt_number VARCHAR(50),
    invoice_id UUID,
    invoice_number VARCHAR(50),
    chargeback_date DATE NOT NULL,
    gl_date DATE,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    exchange_rate_type VARCHAR(30),
    exchange_rate DOUBLE PRECISION,
    amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    tax_amount DOUBLE PRECISION DEFAULT 0,
    total_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    open_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    reason_code VARCHAR(50) NOT NULL,
    reason_description TEXT,
    category VARCHAR(50),
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    priority VARCHAR(20) DEFAULT 'medium',
    assigned_to VARCHAR(200),
    assigned_team VARCHAR(200),
    due_date DATE,
    resolution_date DATE,
    resolution_notes TEXT,
    resolved_by UUID,
    reference VARCHAR(100),
    customer_reference VARCHAR(100),
    sales_rep VARCHAR(200),
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, chargeback_number)
);

CREATE TABLE IF NOT EXISTS _atlas.chargeback_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    chargeback_id UUID NOT NULL REFERENCES _atlas.chargebacks(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    line_type VARCHAR(30) NOT NULL DEFAULT 'chargeback',
    description TEXT,
    quantity INT DEFAULT 1,
    unit_price DOUBLE PRECISION DEFAULT 0,
    amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    tax_amount DOUBLE PRECISION DEFAULT 0,
    total_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    reason_code VARCHAR(50),
    reason_description TEXT,
    item_number VARCHAR(100),
    item_description TEXT,
    gl_account_code VARCHAR(50),
    gl_account_name VARCHAR(200),
    reference VARCHAR(100),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.chargeback_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    chargeback_id UUID NOT NULL REFERENCES _atlas.chargebacks(id) ON DELETE CASCADE,
    activity_type VARCHAR(50) NOT NULL,
    description TEXT,
    old_status VARCHAR(30),
    new_status VARCHAR(30),
    performed_by UUID,
    performed_by_name VARCHAR(200),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_chargebacks_org ON _atlas.chargebacks(organization_id);
CREATE INDEX IF NOT EXISTS idx_chargebacks_status ON _atlas.chargebacks(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_chargebacks_customer ON _atlas.chargebacks(organization_id, customer_id);
CREATE INDEX IF NOT EXISTS idx_chargebacks_date ON _atlas.chargebacks(organization_id, chargeback_date);
CREATE INDEX IF NOT EXISTS idx_chargebacks_reason ON _atlas.chargebacks(organization_id, reason_code);
CREATE INDEX IF NOT EXISTS idx_chargeback_lines_chargeback ON _atlas.chargeback_lines(chargeback_id);
CREATE INDEX IF NOT EXISTS idx_chargeback_activities_chargeback ON _atlas.chargeback_activities(chargeback_id);
