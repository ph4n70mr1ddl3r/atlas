-- 143_receivables_factoring.sql
-- Oracle Fusion Financial Feature: Receivables Factoring
-- Manages the sale of accounts receivable to a third-party factor at a discount.
-- Supports both recourse and non-recourse factoring with advance tracking,
-- fee calculations, and settlement management.
-- Oracle Fusion: Financials > Treasury > Receivables Factoring

-- ============================================================================
-- Factor Companies (Financial institutions that purchase receivables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.factor_companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    contact_name VARCHAR(200),
    contact_email VARCHAR(200),
    contact_phone VARCHAR(50),
    bank_name VARCHAR(200),
    bank_account_number VARCHAR(100),
    default_advance_rate NUMERIC(5,4) NOT NULL DEFAULT 0.8000,
    default_fee_rate NUMERIC(5,4) NOT NULL DEFAULT 0.0150,
    default_recourse_type VARCHAR(20) NOT NULL DEFAULT 'recourse',
    minimum_invoice_amount NUMERIC(18,2),
    maximum_invoice_amount NUMERIC(18,2),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_fc_org ON financials.factor_companies(organization_id);
CREATE INDEX IF NOT EXISTS idx_fc_active ON financials.factor_companies(is_active);

-- ============================================================================
-- Factoring Agreements (Master contracts with factor companies)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.factoring_agreements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agreement_number VARCHAR(50) NOT NULL,
    factor_company_id UUID NOT NULL REFERENCES financials.factor_companies(id),
    factor_company_code VARCHAR(50),
    agreement_name VARCHAR(200) NOT NULL,
    description TEXT,
    agreement_type VARCHAR(20) NOT NULL DEFAULT 'spot',
    recourse_type VARCHAR(20) NOT NULL DEFAULT 'recourse',
    advance_rate NUMERIC(5,4) NOT NULL DEFAULT 0.8000,
    factoring_fee_rate NUMERIC(5,4) NOT NULL DEFAULT 0.0150,
    late_fee_rate NUMERIC(5,4) NOT NULL DEFAULT 0.0050,
    reserve_rate NUMERIC(5,4) NOT NULL DEFAULT 0.0500,
    minimum_fee NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    start_date DATE NOT NULL,
    end_date DATE,
    credit_limit NUMERIC(18,2),
    total_factored_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    total_advance_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    total_fee_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    total_reserve_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    total_settled_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, agreement_number)
);

CREATE INDEX IF NOT EXISTS idx_fa_org ON financials.factoring_agreements(organization_id);
CREATE INDEX IF NOT EXISTS idx_fa_factor ON financials.factoring_agreements(factor_company_id);
CREATE INDEX IF NOT EXISTS idx_fa_status ON financials.factoring_agreements(status);
CREATE INDEX IF NOT EXISTS idx_fa_number ON financials.factoring_agreements(agreement_number);
CREATE INDEX IF NOT EXISTS idx_fa_dates ON financials.factoring_agreements(start_date, end_date);

-- ============================================================================
-- Factoring Requests (Individual requests to factor receivables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.factoring_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    request_number VARCHAR(50) NOT NULL,
    agreement_id UUID NOT NULL REFERENCES financials.factoring_agreements(id),
    agreement_number VARCHAR(50),
    factor_company_id UUID REFERENCES financials.factor_companies(id),
    factor_company_name VARCHAR(200),
    request_date DATE NOT NULL,
    funding_date DATE,
    settlement_date DATE,
    total_invoice_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    eligible_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    advance_rate NUMERIC(5,4) NOT NULL DEFAULT 0.8000,
    advance_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    factoring_fee_rate NUMERIC(5,4) NOT NULL DEFAULT 0.0150,
    factoring_fee_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    reserve_rate NUMERIC(5,4) NOT NULL DEFAULT 0.0500,
    reserve_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    recourse_type VARCHAR(20) NOT NULL DEFAULT 'recourse',
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    funded_by UUID,
    funded_at TIMESTAMPTZ,
    settled_by UUID,
    settled_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, request_number)
);

CREATE INDEX IF NOT EXISTS idx_fr_org ON financials.factoring_requests(organization_id);
CREATE INDEX IF NOT EXISTS idx_fr_agreement ON financials.factoring_requests(agreement_id);
CREATE INDEX IF NOT EXISTS idx_fr_status ON financials.factoring_requests(status);
CREATE INDEX IF NOT EXISTS idx_fr_number ON financials.factoring_requests(request_number);
CREATE INDEX IF NOT EXISTS idx_fr_date ON financials.factoring_requests(request_date);

-- ============================================================================
-- Factoring Request Lines (Individual receivables being factored)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.factoring_request_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    request_id UUID NOT NULL REFERENCES financials.factoring_requests(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    transaction_id UUID,
    transaction_number VARCHAR(50),
    customer_id UUID,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    invoice_date DATE,
    invoice_due_date DATE,
    invoice_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    eligible_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    days_outstanding INT,
    days_overdue INT,
    advance_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    factoring_fee_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    reserve_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    settlement_amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    is_eligible BOOLEAN NOT NULL DEFAULT true,
    exclusion_reason TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    settled_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_frl_request ON financials.factoring_request_lines(request_id);
CREATE INDEX IF NOT EXISTS idx_frl_transaction ON financials.factoring_request_lines(transaction_id);
CREATE INDEX IF NOT EXISTS idx_frl_customer ON financials.factoring_request_lines(customer_id);
CREATE INDEX IF NOT EXISTS idx_frl_status ON financials.factoring_request_lines(status);

-- ============================================================================
-- Factoring Settlements (When customers pay the factor)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.factoring_settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    settlement_number VARCHAR(50) NOT NULL,
    agreement_id UUID NOT NULL REFERENCES financials.factoring_agreements(id),
    request_id UUID REFERENCES financials.factoring_requests(id),
    settlement_date DATE NOT NULL,
    total_settled NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    total_reserve_released NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    total_chargebacks NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    total_late_fees NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    net_to_customer NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    processed_by UUID,
    processed_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, settlement_number)
);

CREATE INDEX IF NOT EXISTS idx_fs_org ON financials.factoring_settlements(organization_id);
CREATE INDEX IF NOT EXISTS idx_fs_agreement ON financials.factoring_settlements(agreement_id);
CREATE INDEX IF NOT EXISTS idx_fs_request ON financials.factoring_settlements(request_id);
CREATE INDEX IF NOT EXISTS idx_fs_status ON financials.factoring_settlements(status);

-- ============================================================================
-- Factoring Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.factoring_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entity_type VARCHAR(30) NOT NULL,
    entity_id UUID,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_fat_entity ON financials.factoring_audit(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_fat_action ON financials.factoring_audit(action);
CREATE INDEX IF NOT EXISTS idx_fat_performed ON financials.factoring_audit(performed_at);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.factoring_dashboard AS
SELECT
    fa.organization_id,
    COUNT(DISTINCT fc.id) AS total_factor_companies,
    COUNT(DISTINCT fc.id) FILTER (WHERE fc.is_active) AS active_factor_companies,
    COUNT(DISTINCT fa.id) AS total_agreements,
    COUNT(DISTINCT fa.id) FILTER (WHERE fa.status = 'active') AS active_agreements,
    COUNT(DISTINCT fr.id) AS total_requests,
    COUNT(DISTINCT fr.id) FILTER (WHERE fr.status = 'draft') AS draft_requests,
    COUNT(DISTINCT fr.id) FILTER (WHERE fr.status = 'approved') AS approved_requests,
    COUNT(DISTINCT fr.id) FILTER (WHERE fr.status = 'funded') AS funded_requests,
    COUNT(DISTINCT fr.id) FILTER (WHERE fr.status = 'settled') AS settled_requests,
    COALESCE(SUM(fr.total_invoice_amount), 0) AS total_invoices_factored,
    COALESCE(SUM(fr.advance_amount), 0) AS total_advances,
    COALESCE(SUM(fr.factoring_fee_amount), 0) AS total_fees,
    COALESCE(SUM(fr.reserve_amount), 0) AS total_reserves,
    COALESCE(SUM(fst.total_settled), 0) AS total_settled
FROM financials.factor_companies fc
LEFT JOIN financials.factoring_agreements fa ON fa.factor_company_id = fc.id
LEFT JOIN financials.factoring_requests fr ON fr.agreement_id = fa.id
LEFT JOIN financials.factoring_settlements fst ON fst.agreement_id = fa.id
GROUP BY fa.organization_id;

COMMENT ON TABLE financials.factor_companies IS 'Financial institutions (factors) that purchase receivables';
COMMENT ON TABLE financials.factoring_agreements IS 'Master factoring agreements with factor companies defining terms and rates';
COMMENT ON TABLE financials.factoring_requests IS 'Individual requests to factor batches of receivables';
COMMENT ON TABLE financials.factoring_request_lines IS 'Individual receivable lines within a factoring request';
COMMENT ON TABLE financials.factoring_settlements IS 'Settlement records when customers pay the factor';
COMMENT ON TABLE financials.factoring_audit IS 'Audit trail for all factoring actions';
COMMENT ON VIEW financials.factoring_dashboard IS 'Dashboard aggregation for receivables factoring overview';
