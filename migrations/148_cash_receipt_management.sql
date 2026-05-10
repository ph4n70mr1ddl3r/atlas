-- 148_cash_receipt_management.sql
-- Oracle Fusion Financial Feature: Cash Receipt Management
-- Manages the full lifecycle of customer cash receipts: creation, confirmation,
-- application to invoices, unapplication, and reversal.
-- Oracle Fusion: Financials > Receivables > Receipts

-- ============================================================================
-- Receipt Batches (group of receipts processed together)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.receipt_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    batch_name VARCHAR(200) NOT NULL,
    description TEXT,
    -- 'bank', 'cash', 'credit_card', 'wire_transfer', 'other'
    receipt_method VARCHAR(30) NOT NULL DEFAULT 'bank',
    bank_account_id UUID,
    -- 'draft', 'confirmed', 'closed', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    receipt_count INT NOT NULL DEFAULT 0,
    gl_posting_date DATE,
    posted_by UUID,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

CREATE INDEX idx_rb_org ON financials.receipt_batches(organization_id);
CREATE INDEX idx_rb_status ON financials.receipt_batches(status);

-- ============================================================================
-- Cash Receipts (individual receipt records)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.cash_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID REFERENCES financials.receipt_batches(id) ON DELETE SET NULL,
    receipt_number VARCHAR(50) NOT NULL,
    customer_id UUID NOT NULL,
    customer_name VARCHAR(200),
    customer_account_number VARCHAR(50),
    -- 'cash', 'check', 'credit_card', 'wire_transfer', 'bank_draft', 'other'
    payment_method VARCHAR(30) NOT NULL DEFAULT 'cash',
    -- 'unidentified', 'identified', 'applied', 'partially_applied', 'unapplied', 'reversed'
    status VARCHAR(20) NOT NULL DEFAULT 'unidentified',
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    applied_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    unapplied_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    exchange_rate NUMERIC(18,6) DEFAULT 1.0,
    receipt_date DATE NOT NULL DEFAULT CURRENT_DATE,
    maturity_date DATE,
    -- Reference / check number
    reference_number VARCHAR(100),
    bank_name VARCHAR(200),
    bank_branch VARCHAR(200),
    -- 'on_account', 'unapplied', 'prepayment'
    deposit_date DATE,
    -- 'available', 'cleared', 'reconciled'
    clearance_status VARCHAR(20) DEFAULT 'available',
    notes TEXT,
    reversal_reason TEXT,
    reversed_from UUID,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, receipt_number)
);

CREATE INDEX idx_cr_org ON financials.cash_receipts(organization_id);
CREATE INDEX idx_cr_batch ON financials.cash_receipts(batch_id);
CREATE INDEX idx_cr_customer ON financials.cash_receipts(customer_id);
CREATE INDEX idx_cr_status ON financials.cash_receipts(status);
CREATE INDEX idx_cr_date ON financials.cash_receipts(receipt_date);
CREATE INDEX idx_cr_ref ON financials.cash_receipts(reference_number);

-- ============================================================================
-- Receipt Applications (linking receipts to invoices/transactions)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.receipt_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    receipt_id UUID NOT NULL REFERENCES financials.cash_receipts(id) ON DELETE CASCADE,
    -- The invoice or transaction being paid
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(50),
    -- Amount applied from the receipt to this invoice
    applied_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Discount taken (early payment discount)
    discount_taken NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- 'applied', 'unapplied', 'reversed'
    status VARCHAR(20) NOT NULL DEFAULT 'applied',
    application_date DATE NOT NULL DEFAULT CURRENT_DATE,
    -- GL accounting date
    gl_date DATE,
    applied_by UUID,
    -- For reversal tracking
    reversed_by UUID,
    reversal_date TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_ra_org ON financials.receipt_applications(organization_id);
CREATE INDEX idx_ra_receipt ON financials.receipt_applications(receipt_id);
CREATE INDEX idx_ra_invoice ON financials.receipt_applications(invoice_id);
CREATE INDEX idx_ra_status ON financials.receipt_applications(status);
CREATE INDEX idx_ra_date ON financials.receipt_applications(application_date);

-- ============================================================================
-- Receipt Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.cash_receipt_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID,
    receipt_id UUID,
    application_id UUID,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_cra_receipt ON financials.cash_receipt_audit(receipt_id);
CREATE INDEX idx_cra_action ON financials.cash_receipt_audit(action);
CREATE INDEX idx_cra_performed ON financials.cash_receipt_audit(performed_at);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.cash_receipt_dashboard AS
SELECT
    r.organization_id,
    COUNT(DISTINCT b.id) AS total_batches,
    COUNT(DISTINCT b.id) FILTER (WHERE b.status = 'draft') AS draft_batches,
    COUNT(DISTINCT b.id) FILTER (WHERE b.status = 'confirmed') AS confirmed_batches,
    COUNT(DISTINCT r.id) AS total_receipts,
    COUNT(DISTINCT r.id) FILTER (WHERE r.status = 'applied') AS applied_receipts,
    COUNT(DISTINCT r.id) FILTER (WHERE r.status = 'unapplied') AS unapplied_receipts,
    COUNT(DISTINCT r.id) FILTER (WHERE r.status = 'partially_applied') AS partially_applied_receipts,
    COUNT(DISTINCT r.id) FILTER (WHERE r.status = 'reversed') AS reversed_receipts,
    COALESCE(SUM(r.amount), 0) AS total_receipt_amount,
    COALESCE(SUM(r.applied_amount), 0) AS total_applied_amount,
    COALESCE(SUM(r.unapplied_amount), 0) AS total_unapplied_amount
FROM financials.cash_receipts r
LEFT JOIN financials.receipt_batches b ON b.id = r.batch_id
GROUP BY r.organization_id;

COMMENT ON TABLE financials.receipt_batches IS 'Receipt batches - groups of customer receipts processed together';
COMMENT ON TABLE financials.cash_receipts IS 'Cash receipts - individual customer payment records with application tracking';
COMMENT ON TABLE financials.receipt_applications IS 'Receipt applications - links between receipts and invoices showing payment allocation';
COMMENT ON TABLE financials.cash_receipt_audit IS 'Audit trail for cash receipt management actions';
COMMENT ON VIEW financials.cash_receipt_dashboard IS 'Dashboard aggregation for cash receipt statistics';
