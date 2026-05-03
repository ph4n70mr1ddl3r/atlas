-- Suspense Account Processing
-- Oracle Fusion: General Ledger > Suspense Accounts
-- Enables automatic posting of unbalanced journal differences to suspense accounts

CREATE TABLE IF NOT EXISTS _atlas.suspense_account_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    balancing_segment VARCHAR(100) NOT NULL,
    suspense_account VARCHAR(500) NOT NULL,
    enabled BOOLEAN DEFAULT true,
    status VARCHAR(20) DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.suspense_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    suspense_definition_id UUID NOT NULL REFERENCES _atlas.suspense_account_definitions(id),
    journal_entry_id UUID,
    journal_batch_id UUID,
    balancing_segment_value VARCHAR(200) NOT NULL,
    suspense_account VARCHAR(500) NOT NULL,
    suspense_amount NUMERIC(18,2) NOT NULL,
    original_amount NUMERIC(18,2),
    entry_type VARCHAR(20) NOT NULL DEFAULT 'auto',
    entry_date DATE NOT NULL,
    currency_code VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    cleared_by_journal_id UUID,
    clearing_date DATE,
    resolution_notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.suspense_clearing_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    description TEXT,
    clearing_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    total_entries INT DEFAULT 0,
    total_cleared_amount NUMERIC(18,2) DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

CREATE TABLE IF NOT EXISTS _atlas.suspense_clearing_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    clearing_batch_id UUID NOT NULL REFERENCES _atlas.suspense_clearing_batches(id) ON DELETE CASCADE,
    suspense_entry_id UUID NOT NULL REFERENCES _atlas.suspense_entries(id),
    clearing_account VARCHAR(500) NOT NULL,
    cleared_amount NUMERIC(18,2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    resolution_notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.suspense_aging_snapshot (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    snapshot_date DATE NOT NULL,
    total_open_entries INT DEFAULT 0,
    total_open_amount NUMERIC(18,2) DEFAULT 0,
    aging_0_30 NUMERIC(18,2) DEFAULT 0,
    aging_31_60 NUMERIC(18,2) DEFAULT 0,
    aging_61_90 NUMERIC(18,2) DEFAULT 0,
    aging_91_180 NUMERIC(18,2) DEFAULT 0,
    aging_over_180 NUMERIC(18,2) DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_suspense_entries_org_status ON _atlas.suspense_entries(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_suspense_entries_definition ON _atlas.suspense_entries(suspense_definition_id);
CREATE INDEX IF NOT EXISTS idx_suspense_clearing_batch_org ON _atlas.suspense_clearing_batches(organization_id);
