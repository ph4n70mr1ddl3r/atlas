-- ============================================================================
-- AP Invoice Batch Processing
-- Oracle Fusion Cloud ERP: Financials > Payables > Invoice Batches
--
-- Manages batch creation, validation, approval and posting of AP invoices:
-- - Invoice batches (grouping invoices for streamlined processing)
-- - Batch activities (audit trail)
-- - Full lifecycle: draft → submitted → approved → posted → cancelled
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Invoice Batches
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.ap_invoice_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    batch_name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Batch totals (denormalized for performance)
    total_invoice_count INT NOT NULL DEFAULT 0,
    total_invoice_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_tax_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    control_total DOUBLE PRECISION,  -- expected total for validation
    control_count INT,               -- expected count for validation

    -- Currency
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    exchange_rate_type VARCHAR(20) DEFAULT 'daily'
        CHECK (exchange_rate_type IS NULL OR exchange_rate_type IN ('daily', 'spot', 'corporate', 'user')),
    exchange_rate DOUBLE PRECISION,

    -- GL / accounting
    gl_date DATE,
    accounting_period VARCHAR(20),

    -- Source
    source VARCHAR(30) NOT NULL DEFAULT 'manual'
        CHECK (source IN ('manual', 'import', 'edi', 'project', 'expense', 'other')),

    -- Workflow
    status VARCHAR(25) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'submitted', 'approved', 'posted', 'cancelled')),

    -- Approval
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    cancel_reason TEXT,

    -- Validation
    validation_status VARCHAR(20) DEFAULT 'pending'
        CHECK (validation_status IN ('pending', 'valid', 'invalid', 'warnings')),
    validation_errors JSONB DEFAULT '[]',

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(organization_id, batch_number)
);

CREATE INDEX IF NOT EXISTS idx_ap_inv_batch_org
    ON _atlas.ap_invoice_batches(organization_id);
CREATE INDEX IF NOT EXISTS idx_ap_inv_batch_status
    ON _atlas.ap_invoice_batches(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_ap_inv_batch_gl_date
    ON _atlas.ap_invoice_batches(organization_id, gl_date);

-- ============================================================================
-- Batch Activities (Audit Trail)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.ap_invoice_batch_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL REFERENCES _atlas.ap_invoice_batches(id) ON DELETE CASCADE,
    activity_type VARCHAR(30) NOT NULL,
    description TEXT,
    old_status VARCHAR(25),
    new_status VARCHAR(25),
    performed_by UUID,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_ap_inv_batch_act_batch
    ON _atlas.ap_invoice_batch_activities(batch_id);
CREATE INDEX IF NOT EXISTS idx_ap_inv_batch_act_type
    ON _atlas.ap_invoice_batch_activities(batch_id, activity_type);
