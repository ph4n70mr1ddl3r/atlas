-- AP/AR Netting (Oracle Fusion: Financials > Netting)
-- Allows organizations to settle payables and receivables with the same trading partner

-- Netting Agreements
CREATE TABLE IF NOT EXISTS _atlas.netting_agreements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agreement_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    partner_id UUID NOT NULL,
    partner_number VARCHAR(50),
    partner_name VARCHAR(200),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    netting_direction VARCHAR(30) NOT NULL DEFAULT 'bi_directional',
    settlement_method VARCHAR(20) NOT NULL DEFAULT 'automatic',
    minimum_netting_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    maximum_netting_amount DOUBLE PRECISION,
    auto_select_transactions BOOLEAN NOT NULL DEFAULT false,
    selection_criteria JSONB NOT NULL DEFAULT '{}',
    netting_clearing_account VARCHAR(50),
    ap_clearing_account VARCHAR(50),
    ar_clearing_account VARCHAR(50),
    approval_required BOOLEAN NOT NULL DEFAULT true,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    effective_from DATE,
    effective_to DATE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, agreement_number)
);

-- Netting Batches
CREATE TABLE IF NOT EXISTS _atlas.netting_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    agreement_id UUID NOT NULL REFERENCES _atlas.netting_agreements(id),
    netting_date DATE NOT NULL,
    gl_date DATE,
    partner_id UUID NOT NULL,
    partner_name VARCHAR(200),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    total_payables_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_receivables_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    net_difference DOUBLE PRECISION NOT NULL DEFAULT 0,
    settlement_direction VARCHAR(10) NOT NULL DEFAULT 'zero',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    payable_transaction_count INT NOT NULL DEFAULT 0,
    receivable_transaction_count INT NOT NULL DEFAULT 0,
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    settlement_payment_id UUID,
    settlement_receipt_id UUID,
    journal_entry_id UUID,
    settled_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

-- Netting Transaction Lines
CREATE TABLE IF NOT EXISTS _atlas.netting_transaction_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL REFERENCES _atlas.netting_batches(id),
    line_number INT NOT NULL,
    source_type VARCHAR(20) NOT NULL,
    source_id UUID NOT NULL,
    source_number VARCHAR(50),
    source_date DATE,
    original_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    netting_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    remaining_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'selected',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Financial Statement Definitions
CREATE TABLE IF NOT EXISTS _atlas.financial_report_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    report_type VARCHAR(30) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    include_comparative BOOLEAN NOT NULL DEFAULT false,
    comparative_period_count INT NOT NULL DEFAULT 0,
    row_definitions JSONB NOT NULL DEFAULT '[]',
    column_definitions JSONB NOT NULL DEFAULT '[]',
    period_name VARCHAR(50),
    fiscal_year INT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Generated Financial Statements
CREATE TABLE IF NOT EXISTS _atlas.financial_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_id UUID REFERENCES _atlas.financial_report_definitions(id),
    report_name VARCHAR(200) NOT NULL,
    report_type VARCHAR(30) NOT NULL,
    as_of_date DATE NOT NULL,
    period_name VARCHAR(50),
    fiscal_year INT,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    lines JSONB NOT NULL DEFAULT '[]',
    totals JSONB NOT NULL DEFAULT '{}',
    is_balanced BOOLEAN NOT NULL DEFAULT false,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    generated_by UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_netting_agreements_org ON _atlas.netting_agreements(organization_id);
CREATE INDEX IF NOT EXISTS idx_netting_agreements_partner ON _atlas.netting_agreements(partner_id);
CREATE INDEX IF NOT EXISTS idx_netting_batches_org ON _atlas.netting_batches(organization_id);
CREATE INDEX IF NOT EXISTS idx_netting_batches_agreement ON _atlas.netting_batches(agreement_id);
CREATE INDEX IF NOT EXISTS idx_netting_lines_batch ON _atlas.netting_transaction_lines(batch_id);
CREATE INDEX IF NOT EXISTS idx_fin_report_defs_org ON _atlas.financial_report_definitions(organization_id);
CREATE INDEX IF NOT EXISTS idx_fin_statements_org ON _atlas.financial_statements(organization_id);
CREATE INDEX IF NOT EXISTS idx_fin_statements_type ON _atlas.financial_statements(report_type);
