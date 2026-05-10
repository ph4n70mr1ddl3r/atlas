-- Finance Charge Management (Oracle Fusion: Receivables > Finance Charges)
-- Manages automatic assessment of late payment charges on overdue customer invoices.

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Finance Charge Terms
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.finance_charge_terms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    term_code VARCHAR(50) NOT NULL,
    term_name VARCHAR(200) NOT NULL,
    description TEXT,
    charge_type VARCHAR(20) NOT NULL DEFAULT 'percentage',
    charge_rate DOUBLE PRECISION,
    minimum_charge DOUBLE PRECISION,
    maximum_charge DOUBLE PRECISION,
    grace_period_days INT NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    calculation_basis VARCHAR(20) NOT NULL DEFAULT 'monthly',
    include_tax BOOLEAN NOT NULL DEFAULT false,
    compound_charges BOOLEAN NOT NULL DEFAULT false,
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    auto_assess BOOLEAN NOT NULL DEFAULT false,
    revenue_account_code VARCHAR(50),
    receivable_account_code VARCHAR(50),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, term_code)
);

-- ============================================================================
-- Finance Charge Tiers (for tiered charge types)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.finance_charge_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    term_id UUID NOT NULL REFERENCES _atlas.finance_charge_terms(id) ON DELETE CASCADE,
    from_days_overdue INT NOT NULL,
    to_days_overdue INT,
    charge_rate DOUBLE PRECISION NOT NULL,
    flat_fee DOUBLE PRECISION,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- ============================================================================
-- Finance Charge Runs
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.finance_charge_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(20) NOT NULL,
    run_date DATE NOT NULL,
    gl_date DATE NOT NULL,
    term_id UUID REFERENCES _atlas.finance_charge_terms(id),
    term_code VARCHAR(50),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    total_invoices_assessed INT NOT NULL DEFAULT 0,
    total_charges_assessed DOUBLE PRECISION NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, run_number)
);

-- ============================================================================
-- Finance Charge Invoices
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.finance_charge_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    charge_invoice_number VARCHAR(20) NOT NULL,
    customer_id UUID,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    invoice_date DATE NOT NULL,
    gl_date DATE NOT NULL,
    due_date DATE NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    total_charge_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    amount_applied DOUBLE PRECISION NOT NULL DEFAULT 0,
    amount_remaining DOUBLE PRECISION NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    run_id UUID REFERENCES _atlas.finance_charge_runs(id),
    revenue_account_code VARCHAR(50),
    receivable_account_code VARCHAR(50),
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, charge_invoice_number)
);

-- ============================================================================
-- Finance Charge Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.finance_charge_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.finance_charge_runs(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    customer_id UUID,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    invoice_id UUID,
    invoice_number VARCHAR(50),
    invoice_date DATE,
    invoice_due_date DATE,
    days_overdue INT NOT NULL DEFAULT 0,
    invoice_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    outstanding_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    charge_type VARCHAR(20) NOT NULL DEFAULT 'percentage',
    charge_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    charge_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    term_id UUID REFERENCES _atlas.finance_charge_terms(id),
    term_code VARCHAR(50),
    charge_invoice_id UUID REFERENCES _atlas.finance_charge_invoices(id),
    charge_invoice_number VARCHAR(20),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    waived_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- ============================================================================
-- Finance Charge Activity Log
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.finance_charge_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entity_type VARCHAR(20) NOT NULL,
    entity_id UUID NOT NULL,
    activity_type VARCHAR(50) NOT NULL,
    description TEXT,
    old_status VARCHAR(20),
    new_status VARCHAR(20),
    performed_by UUID,
    performed_by_name VARCHAR(200),
    created_at TIMESTAMPTZ DEFAULT now()
);

-- ============================================================================
-- Indexes
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_fc_terms_org ON _atlas.finance_charge_terms(organization_id);
CREATE INDEX IF NOT EXISTS idx_fc_terms_active ON _atlas.finance_charge_terms(organization_id, is_active);
CREATE INDEX IF NOT EXISTS idx_fc_tiers_term ON _atlas.finance_charge_tiers(term_id);
CREATE INDEX IF NOT EXISTS idx_fc_runs_org ON _atlas.finance_charge_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_fc_runs_status ON _atlas.finance_charge_runs(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_fc_lines_run ON _atlas.finance_charge_lines(run_id);
CREATE INDEX IF NOT EXISTS idx_fc_lines_customer ON _atlas.finance_charge_lines(customer_id);
CREATE INDEX IF NOT EXISTS idx_fc_invoices_org ON _atlas.finance_charge_invoices(organization_id);
CREATE INDEX IF NOT EXISTS idx_fc_invoices_status ON _atlas.finance_charge_invoices(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_fc_invoices_customer ON _atlas.finance_charge_invoices(customer_id);
CREATE INDEX IF NOT EXISTS idx_fc_activities_entity ON _atlas.finance_charge_activities(entity_type, entity_id);
