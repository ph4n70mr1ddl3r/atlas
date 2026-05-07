-- ============================================================================
-- Recurring Invoice Management
-- Oracle Fusion Cloud ERP: Financials > Payables > Recurring Invoices
--
-- Provides recurring AP invoice templates with automatic generation schedules:
-- - Recurring invoice templates (header + line definitions)
-- - Recurrence schedules (daily, weekly, monthly, quarterly, yearly)
-- - Automatic invoice generation from templates
-- - Hold management for generated invoices
-- - Full lifecycle: draft → active → suspended → completed → cancelled
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Recurring Invoice Templates
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.recurring_invoice_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_number VARCHAR(50) NOT NULL,
    template_name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Supplier details
    supplier_id UUID,
    supplier_number VARCHAR(50),
    supplier_name VARCHAR(200),
    supplier_site VARCHAR(100),

    -- Invoice defaults
    invoice_type VARCHAR(30) NOT NULL DEFAULT 'standard'
        CHECK (invoice_type IN ('standard', 'credit_memo', 'debit_memo', 'prepayment')),
    invoice_currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    payment_currency_code VARCHAR(10),
    exchange_rate_type VARCHAR(20) DEFAULT 'daily'
        CHECK (exchange_rate_type IS NULL OR exchange_rate_type IN ('daily', 'spot', 'corporate', 'user')),

    -- Financial defaults
    payment_terms VARCHAR(30),
    payment_method VARCHAR(30)
        CHECK (payment_method IS NULL OR payment_method IN ('check', 'electronic', 'wire', 'ach', 'swift')),
    payment_due_days INT NOT NULL DEFAULT 30,
    liability_account_code VARCHAR(50),
    expense_account_code VARCHAR(50),

    -- Amount type (how invoice amounts are determined each period)
    amount_type VARCHAR(20) NOT NULL DEFAULT 'fixed'
        CHECK (amount_type IN ('fixed', 'variable', 'adjusted')),

    -- Recurrence schedule
    recurrence_type VARCHAR(20) NOT NULL DEFAULT 'monthly'
        CHECK (recurrence_type IN ('daily', 'weekly', 'monthly', 'quarterly', 'semi_annual', 'annual')),
    recurrence_interval INT NOT NULL DEFAULT 1,
    generation_day INT DEFAULT 1,  -- Day of month/week to generate
    days_in_advance INT NOT NULL DEFAULT 0,  -- Generate N days before target date

    -- Effective dates
    effective_from DATE NOT NULL,
    effective_to DATE,
    maximum_generations INT,  -- Null = unlimited

    -- Auto-generation control
    auto_submit BOOLEAN NOT NULL DEFAULT false,
    auto_approve BOOLEAN NOT NULL DEFAULT false,
    hold_for_review BOOLEAN NOT NULL DEFAULT true,

    -- PO reference (optional)
    po_number VARCHAR(50),

    -- GL defaults
    gl_date_basis VARCHAR(20) NOT NULL DEFAULT 'generation_date'
        CHECK (gl_date_basis IN ('generation_date', 'due_date', 'period_end')),

    -- Status & workflow
    status VARCHAR(20) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'active', 'suspended', 'completed', 'cancelled')),
    last_generation_date DATE,
    next_generation_date DATE,
    generation_count INT NOT NULL DEFAULT 0,

    -- Running totals
    total_generated_amount DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    updated_by UUID
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_rit_template_number
    ON _atlas.recurring_invoice_templates(organization_id, template_number);
CREATE INDEX IF NOT EXISTS idx_rit_org_status
    ON _atlas.recurring_invoice_templates(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_rit_supplier
    ON _atlas.recurring_invoice_templates(organization_id, supplier_id);
CREATE INDEX IF NOT EXISTS idx_rit_next_gen
    ON _atlas.recurring_invoice_templates(organization_id, next_generation_date)
    WHERE status = 'active';

-- ============================================================================
-- Recurring Invoice Template Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.recurring_invoice_template_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES _atlas.recurring_invoice_templates(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    line_number INT NOT NULL,

    -- Line details
    line_type VARCHAR(20) NOT NULL DEFAULT 'item'
        CHECK (line_type IN ('item', 'freight', 'tax', 'miscellaneous')),
    description TEXT,
    item_code VARCHAR(50),
    unit_of_measure VARCHAR(30),

    -- Amount
    amount DOUBLE PRECISION NOT NULL,
    quantity DOUBLE PRECISION DEFAULT 1,
    unit_price DOUBLE PRECISION,

    -- Account distribution
    gl_account_code VARCHAR(50) NOT NULL,
    cost_center VARCHAR(50),
    department VARCHAR(100),

    -- Tax
    tax_code VARCHAR(50),
    tax_amount DOUBLE PRECISION DEFAULT 0,

    -- Project reference
    project_id UUID,
    expenditure_type VARCHAR(50),

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ritl_template
    ON _atlas.recurring_invoice_template_lines(template_id);
CREATE INDEX IF NOT EXISTS idx_ritl_org
    ON _atlas.recurring_invoice_template_lines(organization_id);

-- ============================================================================
-- Recurring Invoice Generations (audit trail of generated invoices)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.recurring_invoice_generations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.recurring_invoice_templates(id),
    generation_number INT NOT NULL,

    -- Generated invoice reference
    generated_invoice_id UUID,
    generated_invoice_number VARCHAR(50),
    invoice_date DATE NOT NULL,
    invoice_due_date DATE NOT NULL,
    gl_date DATE NOT NULL,

    -- Amounts
    invoice_amount DOUBLE PRECISION NOT NULL,
    tax_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_amount DOUBLE PRECISION NOT NULL,

    -- Generation status
    generation_status VARCHAR(20) NOT NULL DEFAULT 'generated'
        CHECK (generation_status IN ('generated', 'submitted', 'approved', 'paid', 'cancelled', 'error')),
    error_message TEXT,

    -- Period info
    period_name VARCHAR(20),
    fiscal_year INT,
    period_number INT,

    -- Audit
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    generated_by UUID
);

CREATE INDEX IF NOT EXISTS idx_rig_template
    ON _atlas.recurring_invoice_generations(template_id);
CREATE INDEX IF NOT EXISTS idx_rig_org_status
    ON _atlas.recurring_invoice_generations(organization_id, generation_status);
CREATE INDEX IF NOT EXISTS idx_rig_date
    ON _atlas.recurring_invoice_generations(invoice_date);

-- ============================================================================
-- Recurring Invoice Generation Log (detailed audit)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.recurring_invoice_generation_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL,
    generation_id UUID REFERENCES _atlas.recurring_invoice_generations(id),
    log_type VARCHAR(20) NOT NULL DEFAULT 'info'
        CHECK (log_type IN ('info', 'warning', 'error')),
    message TEXT NOT NULL,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_rigl_template
    ON _atlas.recurring_invoice_generation_log(template_id);
CREATE INDEX IF NOT EXISTS idx_rigl_org
    ON _atlas.recurring_invoice_generation_log(organization_id);
