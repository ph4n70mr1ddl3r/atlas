-- ============================================================================
-- Payment Settlement & Clearing
-- Oracle Fusion Cloud ERP: Financials > Payables > Settlement
--
-- Manages the settlement of payments against invoices:
-- - Settlement batches (grouped payment confirmations)
-- - Settlement lines (individual invoice settlements within a batch)
-- - Discount capture (early payment, trade discounts)
-- - Partial payment handling
-- - Settlement audit trail
-- - Full lifecycle: draft → submitted → approved → settled → cancelled
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Settlement Batches
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.settlement_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    batch_name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Bank details
    bank_account_id UUID,
    bank_account_name VARCHAR(200),
    bank_reference VARCHAR(100),

    -- Financial
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    exchange_rate_type VARCHAR(20) DEFAULT 'daily'
        CHECK (exchange_rate_type IS NULL OR exchange_rate_type IN ('daily', 'spot', 'corporate', 'user')),
    exchange_rate DOUBLE PRECISION,

    -- Settlement control
    settlement_date DATE NOT NULL,
    gl_date DATE NOT NULL,
    settlement_method VARCHAR(20) NOT NULL DEFAULT 'electronic'
        CHECK (settlement_method IN ('check', 'electronic', 'wire', 'ach', 'sepa', 'manual')),

    -- Running totals (denormalized for performance)
    total_invoices INT NOT NULL DEFAULT 0,
    total_invoice_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_discount_taken DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_settled_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_charges DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_net_payment DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- Workflow
    status VARCHAR(20) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'submitted', 'approved', 'settled', 'cancelled')),
    settlement_type VARCHAR(20) NOT NULL DEFAULT 'full'
        CHECK (settlement_type IN ('full', 'partial', 'prepayment')),

    -- Approval
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    settled_by UUID,
    settled_at TIMESTAMPTZ,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    cancel_reason TEXT,

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(organization_id, batch_number)
);

CREATE INDEX IF NOT EXISTS idx_settlement_batches_org
    ON _atlas.settlement_batches(organization_id);
CREATE INDEX IF NOT EXISTS idx_settlement_batches_status
    ON _atlas.settlement_batches(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_settlement_batches_date
    ON _atlas.settlement_batches(organization_id, settlement_date);

-- ============================================================================
-- Settlement Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.settlement_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL REFERENCES _atlas.settlement_batches(id) ON DELETE CASCADE,
    line_number INT NOT NULL,

    -- Invoice reference
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(50),
    invoice_date DATE,
    invoice_amount DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- Supplier
    supplier_id UUID,
    supplier_number VARCHAR(50),
    supplier_name VARCHAR(200),
    supplier_site VARCHAR(100),

    -- Original amounts
    original_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    amount_due DOUBLE PRECISION NOT NULL DEFAULT 0,
    amount_paid DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- Discounts
    discount_available DOUBLE PRECISION NOT NULL DEFAULT 0,
    discount_taken DOUBLE PRECISION NOT NULL DEFAULT 0,
    discount_date DATE,
    discount_reason VARCHAR(200),

    -- Adjustments & charges
    bank_charges DOUBLE PRECISION NOT NULL DEFAULT 0,
    adjustment_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    adjustment_reason TEXT,

    -- Net settlement
    net_settlement DOUBLE PRECISION NOT NULL DEFAULT 0,
    remaining_balance DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- Settlement type for this line
    settlement_type VARCHAR(20) NOT NULL DEFAULT 'full'
        CHECK (settlement_type IN ('full', 'partial', 'prepayment', 'write_off')),

    -- GL accounts
    liability_account VARCHAR(50),
    discount_account VARCHAR(50),
    charges_account VARCHAR(50),

    -- Line status
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'settled', 'cancelled', 'error')),
    error_message TEXT,

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_settlement_lines_batch
    ON _atlas.settlement_lines(batch_id);
CREATE INDEX IF NOT EXISTS idx_settlement_lines_org
    ON _atlas.settlement_lines(organization_id);
CREATE INDEX IF NOT EXISTS idx_settlement_lines_invoice
    ON _atlas.settlement_lines(invoice_id);
CREATE INDEX IF NOT EXISTS idx_settlement_lines_supplier
    ON _atlas.settlement_lines(supplier_id);

-- ============================================================================
-- Settlement Activities (Audit Trail)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.settlement_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL,
    line_id UUID,

    activity_type VARCHAR(50) NOT NULL,
    description TEXT,
    old_status VARCHAR(20),
    new_status VARCHAR(20),
    performed_by UUID,
    performed_by_name VARCHAR(200),
    details JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_settlement_activities_batch
    ON _atlas.settlement_activities(batch_id);
CREATE INDEX IF NOT EXISTS idx_settlement_activities_org
    ON _atlas.settlement_activities(organization_id);
