-- Lockbox Processing
-- Oracle Fusion: AR > Lockbox
-- Automated receipt application from bank lockbox files

CREATE TABLE IF NOT EXISTS financials.lockbox_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    lockbox_number VARCHAR(50) NOT NULL,
    bank_name VARCHAR(200),
    deposit_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'imported', -- imported, validated, applied, partial, error, completed
    total_amount VARCHAR(20) NOT NULL DEFAULT '0',
    total_receipts INT NOT NULL DEFAULT 0,
    applied_amount VARCHAR(20) NOT NULL DEFAULT '0',
    unapplied_amount VARCHAR(20) NOT NULL DEFAULT '0',
    on_account_amount VARCHAR(20) NOT NULL DEFAULT '0',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    source_file_name VARCHAR(500),
    error_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

CREATE TABLE IF NOT EXISTS financials.lockbox_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL REFERENCES financials.lockbox_batches(id) ON DELETE CASCADE,
    receipt_number VARCHAR(50) NOT NULL,
    customer_number VARCHAR(50),
    customer_id UUID,
    receipt_date DATE NOT NULL,
    receipt_amount VARCHAR(20) NOT NULL DEFAULT '0',
    applied_amount VARCHAR(20) NOT NULL DEFAULT '0',
    unapplied_amount VARCHAR(20) NOT NULL DEFAULT '0',
    on_account_amount VARCHAR(20) NOT NULL DEFAULT '0',
    status VARCHAR(20) NOT NULL DEFAULT 'unapplied', -- unapplied, applied, partial, on_account, error
    match_type VARCHAR(20), -- invoice_number, customer, auto, manual
    remittance_reference VARCHAR(200),
    error_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS financials.lockbox_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    receipt_id UUID NOT NULL REFERENCES financials.lockbox_receipts(id) ON DELETE CASCADE,
    invoice_id UUID,
    invoice_number VARCHAR(50),
    applied_amount VARCHAR(20) NOT NULL DEFAULT '0',
    application_date DATE NOT NULL DEFAULT CURRENT_DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'applied', -- applied, unapplied, reversed
    applied_by UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS financials.lockbox_transmission_formats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    format_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    format_type VARCHAR(20) NOT NULL DEFAULT 'BAI2', -- BAI2, MT940, OFX, custom
    field_delimiter VARCHAR(5) DEFAULT ',',
    record_delimiter VARCHAR(5) DEFAULT E'\n',
    header_identifier VARCHAR(20),
    detail_identifier VARCHAR(20),
    trailer_identifier VARCHAR(20),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, format_code)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_lockbox_batches_org ON financials.lockbox_batches(organization_id);
CREATE INDEX IF NOT EXISTS idx_lockbox_batches_status ON financials.lockbox_batches(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_lockbox_receipts_batch ON financials.lockbox_receipts(batch_id);
CREATE INDEX IF NOT EXISTS idx_lockbox_receipts_customer ON financials.lockbox_receipts(organization_id, customer_number);
CREATE INDEX IF NOT EXISTS idx_lockbox_receipts_status ON financials.lockbox_receipts(status);
CREATE INDEX IF NOT EXISTS idx_lockbox_applications_receipt ON financials.lockbox_applications(receipt_id);
CREATE INDEX IF NOT EXISTS idx_lockbox_applications_invoice ON financials.lockbox_applications(invoice_number);
CREATE INDEX IF NOT EXISTS idx_lockbox_formats_org ON financials.lockbox_transmission_formats(organization_id);
