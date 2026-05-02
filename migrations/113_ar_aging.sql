-- AR Aging Analysis
-- Oracle Fusion: AR > Aging Reports
-- Customer balance aging with configurable bucket definitions

CREATE TABLE IF NOT EXISTS financials.ar_aging_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    aging_basis VARCHAR(20) NOT NULL DEFAULT 'invoice_date', -- invoice_date, due_date, trx_date
    num_buckets INT NOT NULL DEFAULT 5,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, definition_code)
);

CREATE TABLE IF NOT EXISTS financials.ar_aging_buckets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_id UUID NOT NULL REFERENCES financials.ar_aging_definitions(id) ON DELETE CASCADE,
    bucket_number INT NOT NULL DEFAULT 0,
    name VARCHAR(100) NOT NULL,
    from_days INT NOT NULL DEFAULT 0,
    to_days INT, -- NULL means open-ended (e.g., "90+")
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS financials.ar_aging_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_id UUID NOT NULL REFERENCES financials.ar_aging_definitions(id) ON DELETE CASCADE,
    snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
    as_of_date DATE NOT NULL DEFAULT CURRENT_DATE,
    total_open_amount VARCHAR(20) NOT NULL DEFAULT '0',
    total_overdue_amount VARCHAR(20) NOT NULL DEFAULT '0',
    total_past_due_count INT NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'completed', -- pending, completed, error
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS financials.ar_aging_snapshot_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    snapshot_id UUID NOT NULL REFERENCES financials.ar_aging_snapshots(id) ON DELETE CASCADE,
    customer_id UUID,
    customer_number VARCHAR(50) NOT NULL,
    customer_name VARCHAR(200),
    invoice_id UUID,
    invoice_number VARCHAR(50) NOT NULL,
    invoice_date DATE NOT NULL,
    due_date DATE NOT NULL,
    original_amount VARCHAR(20) NOT NULL DEFAULT '0',
    open_amount VARCHAR(20) NOT NULL DEFAULT '0',
    days_past_due INT NOT NULL DEFAULT 0,
    bucket_number INT NOT NULL DEFAULT 0,
    bucket_name VARCHAR(100) NOT NULL DEFAULT '',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_ar_aging_defs_org ON financials.ar_aging_definitions(organization_id);
CREATE INDEX IF NOT EXISTS idx_ar_aging_buckets_def ON financials.ar_aging_buckets(definition_id);
CREATE INDEX IF NOT EXISTS idx_ar_aging_snapshots_def ON financials.ar_aging_snapshots(definition_id);
CREATE INDEX IF NOT EXISTS idx_ar_aging_snapshots_date ON financials.ar_aging_snapshots(snapshot_date);
CREATE INDEX IF NOT EXISTS idx_ar_aging_snap_lines_snapshot ON financials.ar_aging_snapshot_lines(snapshot_id);
CREATE INDEX IF NOT EXISTS idx_ar_aging_snap_lines_customer ON financials.ar_aging_snapshot_lines(organization_id, customer_number);
CREATE INDEX IF NOT EXISTS idx_ar_aging_snap_lines_bucket ON financials.ar_aging_snapshot_lines(bucket_number);
