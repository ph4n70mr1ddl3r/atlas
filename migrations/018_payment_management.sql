-- Atlas ERP - Payment Management
-- Oracle Fusion Cloud ERP: Financials > Payables > Payments
--
-- Manages payment terms, payment batches, scheduled payments,
-- payment processing (checks, EFT, wire, ACH), payment files,
-- void/reissue, and remittance advice.
--
-- This is a core Procure-to-Pay feature that connects
-- AP invoices → payments → bank accounts → GL posting.

-- ============================================================================
-- Payment Terms
-- Oracle Fusion: Payables > Setup > Payment Terms
-- ============================================================================
CREATE TABLE _atlas.payment_terms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Base due days from invoice date
    due_days INT NOT NULL DEFAULT 30,
    -- Discount days for early payment
    discount_days INT,
    -- Discount percentage for early payment
    discount_percentage NUMERIC(10, 4),

    -- Installment support
    is_installment BOOLEAN NOT NULL DEFAULT false,
    installment_count INT,
    installment_frequency VARCHAR(20), -- 'monthly', 'quarterly', 'weekly'

    -- Default payment method
    default_payment_method VARCHAR(50), -- 'check', 'eft', 'wire', 'ach'

    -- Effective dating
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, code)
);

CREATE INDEX idx_payment_terms_org ON _atlas.payment_terms(organization_id);
CREATE INDEX idx_payment_terms_active ON _atlas.payment_terms(organization_id, is_active);

-- ============================================================================
-- Payment Batches
-- Oracle Fusion: Payables > Payments > Payment Batches
-- Groups multiple invoices into a single payment run.
-- ============================================================================
CREATE TABLE _atlas.payment_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(100) NOT NULL,
    name VARCHAR(200),
    description TEXT,

    -- Payment run parameters
    payment_date DATE NOT NULL,
    bank_account_id UUID,
    payment_method VARCHAR(50) NOT NULL DEFAULT 'check', -- 'check', 'eft', 'wire', 'ach'
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',

    -- Selection criteria (stored as JSON for flexibility)
    selection_criteria JSONB DEFAULT '{}',

    -- Counts and totals
    total_invoice_count INT NOT NULL DEFAULT 0,
    total_payment_count INT NOT NULL DEFAULT 0,
    total_payment_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_discount_taken NUMERIC(18, 2) NOT NULL DEFAULT 0,

    -- Status: 'draft', 'selected', 'approved', 'formatted', 'confirmed', 'cancelled'
    status VARCHAR(30) NOT NULL DEFAULT 'draft',

    -- Workflow tracking
    selected_by UUID,
    selected_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    formatted_by UUID,
    formatted_at TIMESTAMPTZ,
    confirmed_by UUID,
    confirmed_at TIMESTAMPTZ,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,

    -- Payment file reference (generated output file)
    payment_file_name VARCHAR(500),
    payment_file_reference VARCHAR(500),

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, batch_number)
);

CREATE INDEX idx_payment_batches_org ON _atlas.payment_batches(organization_id);
CREATE INDEX idx_payment_batches_status ON _atlas.payment_batches(organization_id, status);
CREATE INDEX idx_payment_batches_date ON _atlas.payment_batches(payment_date);

-- ============================================================================
-- Payments (Individual)
-- Oracle Fusion: Payables > Payments > Payments
-- Each payment is one check/EFT/wire issued to a supplier.
-- ============================================================================
CREATE TABLE _atlas.payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_number VARCHAR(100) NOT NULL,
    batch_id UUID REFERENCES _atlas.payment_batches(id),

    -- Supplier info
    supplier_id UUID NOT NULL,
    supplier_number VARCHAR(100),
    supplier_name VARCHAR(500),
    supplier_site VARCHAR(200),

    -- Payment details
    payment_date DATE NOT NULL,
    payment_method VARCHAR(50) NOT NULL DEFAULT 'check',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',

    -- Amounts
    payment_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    discount_taken NUMERIC(18, 2) NOT NULL DEFAULT 0,
    bank_charges NUMERIC(18, 2) NOT NULL DEFAULT 0,

    -- Bank account (source of funds)
    bank_account_id UUID,
    bank_account_name VARCHAR(200),

    -- GL account for cash
    cash_account_code VARCHAR(50),
    -- GL account for AP
    ap_account_code VARCHAR(50),
    -- GL account for discounts taken
    discount_account_code VARCHAR(50),

    -- Status: 'draft', 'issued', 'cleared', 'voided', 'reconciled', 'stopped'
    status VARCHAR(30) NOT NULL DEFAULT 'draft',

    -- Check / reference number
    check_number VARCHAR(100),
    reference_number VARCHAR(200),

    -- Void tracking
    voided_by UUID,
    voided_at TIMESTAMPTZ,
    void_reason TEXT,
    -- Reissue reference
    reissued_from_payment_id UUID REFERENCES _atlas.payments(id),
    reissued_payment_id UUID REFERENCES _atlas.payments(id),

    -- Clearance tracking
    cleared_date DATE,
    cleared_by UUID,
    cleared_at TIMESTAMPTZ,

    -- GL posting
    journal_entry_id UUID,
    posted_at TIMESTAMPTZ,

    -- Remittance
    remittance_sent BOOLEAN NOT NULL DEFAULT false,
    remittance_sent_at TIMESTAMPTZ,
    remittance_method VARCHAR(30), -- 'email', 'print', 'edi'

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, payment_number)
);

CREATE INDEX idx_payments_org ON _atlas.payments(organization_id);
CREATE INDEX idx_payments_batch ON _atlas.payments(batch_id);
CREATE INDEX idx_payments_supplier ON _atlas.payments(organization_id, supplier_id);
CREATE INDEX idx_payments_status ON _atlas.payments(organization_id, status);
CREATE INDEX idx_payments_date ON _atlas.payments(payment_date);
CREATE INDEX idx_payments_check ON _atlas.payments(check_number);

-- ============================================================================
-- Payment Lines (invoices covered by a payment)
-- Oracle Fusion: Links between payments and invoices
-- ============================================================================
CREATE TABLE _atlas.payment_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_id UUID NOT NULL REFERENCES _atlas.payments(id) ON DELETE CASCADE,

    line_number INT NOT NULL DEFAULT 1,

    -- Invoice reference
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(100),
    invoice_date DATE,
    invoice_due_date DATE,

    -- Amounts
    invoice_amount NUMERIC(18, 2),
    amount_paid NUMERIC(18, 2) NOT NULL DEFAULT 0,
    discount_taken NUMERIC(18, 2) NOT NULL DEFAULT 0,

    -- Withholding tax
    withholding_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,

    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(payment_id, line_number)
);

CREATE INDEX idx_payment_lines_payment ON _atlas.payment_lines(payment_id);
CREATE INDEX idx_payment_lines_invoice ON _atlas.payment_lines(invoice_id);

-- ============================================================================
-- Scheduled Payments (automatic payment proposals)
-- Oracle Fusion: Payables > Payments > Scheduled Payments
-- ============================================================================
CREATE TABLE _atlas.scheduled_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,

    -- The invoice this schedule is for
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(100),
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(500),

    -- Scheduling
    scheduled_payment_date DATE NOT NULL,
    scheduled_amount NUMERIC(18, 2) NOT NULL,
    -- Which payment term installment this represents
    installment_number INT NOT NULL DEFAULT 1,

    -- Payment method override
    payment_method VARCHAR(50),
    bank_account_id UUID,

    -- Whether this has been picked up by a batch
    is_selected BOOLEAN NOT NULL DEFAULT false,
    selected_batch_id UUID REFERENCES _atlas.payment_batches(id),
    payment_id UUID REFERENCES _atlas.payments(id),

    -- Status: 'pending', 'selected', 'paid', 'cancelled'
    status VARCHAR(30) NOT NULL DEFAULT 'pending',

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_scheduled_payments_org ON _atlas.scheduled_payments(organization_id);
CREATE INDEX idx_scheduled_payments_date ON _atlas.scheduled_payments(scheduled_payment_date);
CREATE INDEX idx_scheduled_payments_status ON _atlas.scheduled_payments(organization_id, status);
CREATE INDEX idx_scheduled_payments_supplier ON _atlas.scheduled_payments(supplier_id);

-- ============================================================================
-- Payment Formats (output file formats for payment files)
-- Oracle Fusion: Payables > Setup > Payment Formats
-- ============================================================================
CREATE TABLE _atlas.payment_formats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Format type: 'file', 'printed_check', 'edi', 'xml', 'json'
    format_type VARCHAR(30) NOT NULL DEFAULT 'file',
    -- File template reference
    template_reference VARCHAR(500),

    -- Applicable payment methods
    applicable_methods JSONB DEFAULT '["check", "eft", "wire", "ach"]',

    -- Whether this is a system-seeded format
    is_system BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,

    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Remittance Advice Log
-- Oracle Fusion: Payables > Payments > Remittance Advice
-- ============================================================================
CREATE TABLE _atlas.remittance_advices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_id UUID NOT NULL REFERENCES _atlas.payments(id),

    -- Delivery method
    delivery_method VARCHAR(30) NOT NULL DEFAULT 'email', -- 'email', 'print', 'edi', 'xml'
    delivery_address TEXT,

    -- Supplier contact
    contact_name VARCHAR(200),
    contact_email VARCHAR(500),

    -- Content
    subject VARCHAR(500),
    body TEXT,

    -- Status
    status VARCHAR(30) NOT NULL DEFAULT 'pending', -- 'pending', 'sent', 'delivered', 'failed'
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    failure_reason TEXT,

    -- Payment summary (denormalized for the advice)
    payment_summary JSONB DEFAULT '{}',

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_remittance_advices_payment ON _atlas.remittance_advices(payment_id);
CREATE INDEX idx_remittance_advices_org ON _atlas.remittance_advices(organization_id);

-- ============================================================================
-- Payment Dashboard View (convenience)
-- ============================================================================
CREATE OR REPLACE VIEW _atlas.payment_dashboard AS
SELECT
    p.organization_id,
    p.payment_method,
    p.status,
    p.currency_code,
    COUNT(*) AS payment_count,
    SUM(p.payment_amount) AS total_amount,
    SUM(p.discount_taken) AS total_discount,
    MIN(p.payment_date) AS earliest_date,
    MAX(p.payment_date) AS latest_date
FROM _atlas.payments p
GROUP BY p.organization_id, p.payment_method, p.status, p.currency_code;
