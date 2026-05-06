-- Remittance Batches
-- Oracle Fusion: Receivables > Receipts > Automatic Receipts > Remittance Batches
-- Groups receipts into batches for bank deposit processing with full lifecycle management.

CREATE TABLE IF NOT EXISTS _atlas.remittance_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    batch_name VARCHAR(200),
    bank_account_id UUID,
    bank_account_name VARCHAR(200),
    bank_name VARCHAR(200),
    remittance_method VARCHAR(30) NOT NULL DEFAULT 'standard',  -- standard, factoring, standard_with_recourse
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    batch_date DATE NOT NULL,
    gl_date DATE,
    receipt_currency_code VARCHAR(10),
    exchange_rate_type VARCHAR(30),
    status VARCHAR(30) NOT NULL DEFAULT 'draft',  -- draft, approved, formatted, transmitted, confirmed, settled, reversed, cancelled
    total_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    receipt_count INT NOT NULL DEFAULT 0,
    format_program VARCHAR(100),
    format_date TIMESTAMPTZ,
    transmission_date TIMESTAMPTZ,
    confirmation_date TIMESTAMPTZ,
    settlement_date TIMESTAMPTZ,
    reversal_date TIMESTAMPTZ,
    reference_number VARCHAR(100),
    remittance_advice_sent BOOLEAN DEFAULT FALSE,
    remittance_advice_date TIMESTAMPTZ,
    notes TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

CREATE TABLE IF NOT EXISTS _atlas.remittance_batch_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL REFERENCES _atlas.remittance_batches(id) ON DELETE CASCADE,
    receipt_id UUID NOT NULL,
    receipt_number VARCHAR(50),
    customer_id UUID,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    receipt_date DATE,
    receipt_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    applied_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    receipt_method VARCHAR(50),
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    exchange_rate DOUBLE PRECISION,
    status VARCHAR(30) NOT NULL DEFAULT 'included',  -- included, excluded, reversed
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_remittance_batches_org ON _atlas.remittance_batches(organization_id);
CREATE INDEX IF NOT EXISTS idx_remittance_batches_status ON _atlas.remittance_batches(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_remittance_batches_date ON _atlas.remittance_batches(organization_id, batch_date DESC);
CREATE INDEX IF NOT EXISTS idx_remittance_batch_receipts_batch ON _atlas.remittance_batch_receipts(batch_id);
CREATE INDEX IF NOT EXISTS idx_remittance_batch_receipts_receipt ON _atlas.remittance_batch_receipts(receipt_id);

-- Sequence for safe concurrent batch number generation
CREATE SEQUENCE IF NOT EXISTS _atlas.remittance_batch_num_seq;
