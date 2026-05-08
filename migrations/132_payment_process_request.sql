-- ============================================================================
-- Payment Process Request (PPR)
-- Oracle Fusion Cloud ERP: Financials > Payables > Payment Process Requests
--
-- Manages automated batch payment processing:
-- - Payment process requests (batch selection of invoices for payment)
-- - PPR selected documents (invoices selected for payment in a PPR)
-- - PPR activities (audit trail)
-- - Full lifecycle: draft → submitted → selection_complete → formatted → confirmed → cancelled
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Payment Process Requests
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.payment_process_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    request_number VARCHAR(50) NOT NULL,
    request_name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Payment criteria
    payment_date DATE NOT NULL,
    gl_date DATE NOT NULL,
    payment_method VARCHAR(20) NOT NULL DEFAULT 'electronic'
        CHECK (payment_method IN ('check', 'electronic', 'wire', 'ach', 'swift', 'all')),
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    exchange_rate_type VARCHAR(20) DEFAULT 'daily'
        CHECK (exchange_rate_type IS NULL OR exchange_rate_type IN ('daily', 'spot', 'corporate', 'user')),
    exchange_rate DOUBLE PRECISION,

    -- Invoice selection criteria
    selection_criteria VARCHAR(20) NOT NULL DEFAULT 'all_open'
        CHECK (selection_criteria IN ('due_date', 'discount_date', 'all_open', 'supplier', 'pay_group')),
    due_date_from DATE,
    due_date_to DATE,
    supplier_id UUID,
    supplier_name VARCHAR(200),
    pay_group VARCHAR(50),
    minimum_amount DOUBLE PRECISION,
    maximum_amount DOUBLE PRECISION,
    include_on_hold BOOLEAN NOT NULL DEFAULT false,
    take_discount BOOLEAN NOT NULL DEFAULT true,
    pay_only_due BOOLEAN NOT NULL DEFAULT false,

    -- Bank / payment details
    bank_account_id UUID,
    bank_account_name VARCHAR(200),
    payment_document VARCHAR(100),

    -- Running totals (denormalized)
    total_documents INT NOT NULL DEFAULT 0,
    total_invoice_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_discount_taken DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_payment_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_currency_adjustment DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- Workflow
    status VARCHAR(25) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'submitted', 'selection_complete', 'formatted', 'confirmed', 'cancelled')),
    processing_time_ms INT,

    -- Approval
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    selection_completed_by UUID,
    selection_completed_at TIMESTAMPTZ,
    formatted_by UUID,
    formatted_at TIMESTAMPTZ,
    confirmed_by UUID,
    confirmed_at TIMESTAMPTZ,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    cancel_reason TEXT,

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(organization_id, request_number)
);

CREATE INDEX IF NOT EXISTS idx_ppr_org
    ON _atlas.payment_process_requests(organization_id);
CREATE INDEX IF NOT EXISTS idx_ppr_status
    ON _atlas.payment_process_requests(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_ppr_date
    ON _atlas.payment_process_requests(organization_id, payment_date);

-- ============================================================================
-- PPR Selected Documents (Invoices selected for payment)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.ppr_selected_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ppr_id UUID NOT NULL REFERENCES _atlas.payment_process_requests(id) ON DELETE CASCADE,
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

    -- Payment details
    original_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    amount_due DOUBLE PRECISION NOT NULL DEFAULT 0,
    amount_to_pay DOUBLE PRECISION NOT NULL DEFAULT 0,
    discount_available DOUBLE PRECISION NOT NULL DEFAULT 0,
    discount_taken DOUBLE PRECISION NOT NULL DEFAULT 0,
    discount_date DATE,
    currency_code VARCHAR(10) DEFAULT 'USD',

    -- Net payment
    net_payment DOUBLE PRECISION NOT NULL DEFAULT 0,
    remaining_balance DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- GL accounts
    liability_account VARCHAR(50),
    discount_account VARCHAR(50),
    cash_account VARCHAR(50),

    -- Status
    selected_for_payment BOOLEAN NOT NULL DEFAULT true,
    exclude_reason TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'selected'
        CHECK (status IN ('selected', 'excluded', 'paid', 'error')),
    error_message TEXT,

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ppr_docs_ppr
    ON _atlas.ppr_selected_documents(ppr_id);
CREATE INDEX IF NOT EXISTS idx_ppr_docs_org
    ON _atlas.ppr_selected_documents(organization_id);
CREATE INDEX IF NOT EXISTS idx_ppr_docs_invoice
    ON _atlas.ppr_selected_documents(invoice_id);
CREATE INDEX IF NOT EXISTS idx_ppr_docs_supplier
    ON _atlas.ppr_selected_documents(supplier_id);

-- ============================================================================
-- PPR Activities (Audit Trail)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.ppr_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ppr_id UUID NOT NULL,
    document_id UUID,

    activity_type VARCHAR(50) NOT NULL,
    description TEXT,
    old_status VARCHAR(25),
    new_status VARCHAR(25),
    performed_by UUID,
    performed_by_name VARCHAR(200),
    details JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ppr_activities_ppr
    ON _atlas.ppr_activities(ppr_id);
CREATE INDEX IF NOT EXISTS idx_ppr_activities_org
    ON _atlas.ppr_activities(organization_id);
