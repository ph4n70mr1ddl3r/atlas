-- 149_direct_debit_mandate.sql
-- Oracle Fusion Financial Feature: Direct Debit Mandate Management
-- Manages customer direct debit mandates (SEPA and non-SEBA): creation,
-- activation, usage tracking, revocation, and expiration.
-- Oracle Fusion: Financials > Receivables > Direct Debit Mandates

-- ============================================================================
-- Direct Debit Mandates
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.direct_debit_mandates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    mandate_number VARCHAR(50) NOT NULL,
    customer_id UUID NOT NULL,
    customer_name VARCHAR(200),
    customer_account_number VARCHAR(50),
    -- 'core', 'b2b', 'business_to_business' (SEPA types), 'ach', 'other'
    mandate_type VARCHAR(30) NOT NULL DEFAULT 'core',
    -- 'draft', 'active', 'used', 'cancelled', 'expired', 'revoked'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Bank account details
    bank_account_holder VARCHAR(200),
    bank_account_number VARCHAR(50),
    bank_account_number_type VARCHAR(20) DEFAULT 'iban',
    bank_code VARCHAR(50),
    bank_code_type VARCHAR(20) DEFAULT 'bic',
    bank_name VARCHAR(200),
    bank_branch VARCHAR(200),
    -- Creditor / scheme details
    creditor_scheme_id VARCHAR(50),
    creditor_name VARCHAR(200),
    -- Mandate reference assigned to customer
    mandate_reference VARCHAR(100),
    -- Dates
    mandate_date DATE NOT NULL DEFAULT CURRENT_DATE,
    activation_date DATE,
    first_collection_date DATE,
    last_collection_date DATE,
    expiry_date DATE,
    cancellation_date DATE,
    -- Tracking
    collection_count INT NOT NULL DEFAULT 0,
    total_collected NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- 'first', 'recurring', 'final'
    next_collection_type VARCHAR(20) DEFAULT 'first',
    -- Max amount per collection (0 = unlimited)
    max_collection_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Reason for status changes
    status_reason TEXT,
    -- Authorization reference / signature info
    authorization_reference VARCHAR(100),
    authorization_method VARCHAR(30),
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, mandate_number)
);

CREATE INDEX idx_ddm_org ON financials.direct_debit_mandates(organization_id);
CREATE INDEX idx_ddm_customer ON financials.direct_debit_mandates(customer_id);
CREATE INDEX idx_ddm_status ON financials.direct_debit_mandates(status);
CREATE INDEX idx_ddm_date ON financials.direct_debit_mandates(mandate_date);
CREATE INDEX idx_ddm_ref ON financials.direct_debit_mandates(mandate_reference);
CREATE INDEX idx_ddm_expiry ON financials.direct_debit_mandates(expiry_date);

-- ============================================================================
-- Mandate Collections (track each debit collection under a mandate)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.mandate_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    mandate_id UUID NOT NULL REFERENCES financials.direct_debit_mandates(id) ON DELETE CASCADE,
    collection_number VARCHAR(50) NOT NULL,
    -- 'first', 'recurring', 'final', 'one_off'
    collection_type VARCHAR(20) NOT NULL DEFAULT 'recurring',
    -- 'pending', 'submitted', 'completed', 'failed', 'returned', 'reversed'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Reference to invoice/receipt being collected
    invoice_id UUID,
    invoice_number VARCHAR(50),
    receipt_id UUID,
    -- Execution details
    scheduled_date DATE NOT NULL DEFAULT CURRENT_DATE,
    execution_date DATE,
    settlement_date DATE,
    -- Return/reversal info
    return_reason_code VARCHAR(50),
    return_reason_text TEXT,
    -- Bank reference from the collection
    bank_reference VARCHAR(100),
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_mc_org ON financials.mandate_collections(organization_id);
CREATE INDEX idx_mc_mandate ON financials.mandate_collections(mandate_id);
CREATE INDEX idx_mc_status ON financials.mandate_collections(status);
CREATE INDEX idx_mc_date ON financials.mandate_collections(scheduled_date);
CREATE INDEX idx_mc_invoice ON financials.mandate_collections(invoice_id);

-- ============================================================================
-- Mandate Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.direct_debit_mandate_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    mandate_id UUID,
    collection_id UUID,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_ddma_mandate ON financials.direct_debit_mandate_audit(mandate_id);
CREATE INDEX idx_ddma_action ON financials.direct_debit_mandate_audit(action);
CREATE INDEX idx_ddma_performed ON financials.direct_debit_mandate_audit(performed_at);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.direct_debit_dashboard AS
SELECT
    m.organization_id,
    COUNT(DISTINCT m.id) AS total_mandates,
    COUNT(DISTINCT m.id) FILTER (WHERE m.status = 'draft') AS draft_mandates,
    COUNT(DISTINCT m.id) FILTER (WHERE m.status = 'active') AS active_mandates,
    COUNT(DISTINCT m.id) FILTER (WHERE m.status = 'used') AS used_mandates,
    COUNT(DISTINCT m.id) FILTER (WHERE m.status = 'cancelled') AS cancelled_mandates,
    COUNT(DISTINCT m.id) FILTER (WHERE m.status = 'expired') AS expired_mandates,
    COUNT(DISTINCT m.id) FILTER (WHERE m.status = 'revoked') AS revoked_mandates,
    COUNT(DISTINCT c.id) AS total_collections,
    COUNT(DISTINCT c.id) FILTER (WHERE c.status = 'completed') AS completed_collections,
    COUNT(DISTINCT c.id) FILTER (WHERE c.status = 'failed') AS failed_collections,
    COUNT(DISTINCT c.id) FILTER (WHERE c.status = 'pending') AS pending_collections,
    COALESCE(SUM(c.amount) FILTER (WHERE c.status = 'completed'), 0) AS total_collected_amount,
    COALESCE(SUM(c.amount) FILTER (WHERE c.status = 'pending'), 0) AS pending_amount
FROM financials.direct_debit_mandates m
LEFT JOIN financials.mandate_collections c ON c.mandate_id = m.id
GROUP BY m.organization_id;

COMMENT ON TABLE financials.direct_debit_mandates IS 'Direct Debit Mandates - customer authorization for automatic bank collection (SEPA, ACH)';
COMMENT ON TABLE financials.mandate_collections IS 'Mandate Collections - individual debit collection attempts under a direct debit mandate';
COMMENT ON TABLE financials.direct_debit_mandate_audit IS 'Audit trail for direct debit mandate management actions';
COMMENT ON VIEW financials.direct_debit_dashboard IS 'Dashboard aggregation for direct debit mandate statistics';
