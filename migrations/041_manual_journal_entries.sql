-- Manual Journal Entries (Oracle Fusion GL > Journals > New Journal)
-- Create and post manual journal entries to the General Ledger.
-- Supports journal batches containing multiple entries, each with balanced debit/credit lines.
-- Lifecycle: Draft → Submitted → Approved → Posted, with reversal support.

-- Journal batches
CREATE TABLE IF NOT EXISTS _atlas.journal_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, submitted, approved, posted, reversed
    ledger_id UUID,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    accounting_date DATE,
    period_name VARCHAR(20),
    total_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    entry_count INT NOT NULL DEFAULT 0,
    source VARCHAR(50) DEFAULT 'manual', -- manual, import, api
    is_automatic_post BOOLEAN NOT NULL DEFAULT false,
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    rejection_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

-- Journal entries within a batch
CREATE TABLE IF NOT EXISTS _atlas.journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL REFERENCES _atlas.journal_batches(id) ON DELETE CASCADE,
    entry_number VARCHAR(50) NOT NULL,
    name VARCHAR(200),
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, submitted, approved, posted, reversed
    ledger_id UUID,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    accounting_date DATE,
    period_name VARCHAR(20),
    journal_category VARCHAR(100) DEFAULT 'manual',
    journal_source VARCHAR(100) DEFAULT 'manual',
    total_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    line_count INT NOT NULL DEFAULT 0,
    is_balanced BOOLEAN NOT NULL DEFAULT false,
    is_reversal BOOLEAN NOT NULL DEFAULT false,
    reversal_of_entry_id UUID,
    reversed_by_entry_id UUID,
    reference_number VARCHAR(100),
    external_reference VARCHAR(200),
    statistical_entry BOOLEAN NOT NULL DEFAULT false,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    posted_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Journal entry lines (debits and credits)
CREATE TABLE IF NOT EXISTS _atlas.journal_entry_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entry_id UUID NOT NULL REFERENCES _atlas.journal_entries(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    line_type VARCHAR(20) NOT NULL DEFAULT 'debit', -- debit, credit
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(200),
    description TEXT,
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    entered_amount NUMERIC(18,2),
    entered_currency_code VARCHAR(10) DEFAULT 'USD',
    exchange_rate NUMERIC(18,6) DEFAULT 1,
    tax_code VARCHAR(50),
    cost_center VARCHAR(100),
    department_id UUID,
    project_id UUID,
    intercompany_entity_id UUID,
    statistical_amount NUMERIC(18,2),
    reference1 VARCHAR(200),
    reference2 VARCHAR(200),
    reference3 VARCHAR(200),
    reference4 VARCHAR(200),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_jb_org ON _atlas.journal_batches (organization_id);
CREATE INDEX IF NOT EXISTS idx_jb_status ON _atlas.journal_batches (organization_id, status);
CREATE INDEX IF NOT EXISTS idx_je_batch ON _atlas.journal_entries (batch_id);
CREATE INDEX IF NOT EXISTS idx_je_status ON _atlas.journal_entries (organization_id, status);
CREATE INDEX IF NOT EXISTS idx_je_date ON _atlas.journal_entries (accounting_date);
CREATE INDEX IF NOT EXISTS idx_je_reversal ON _atlas.journal_entries (reversal_of_entry_id);
CREATE INDEX IF NOT EXISTS idx_jel_entry ON _atlas.journal_entry_lines (entry_id);
CREATE INDEX IF NOT EXISTS idx_jel_account ON _atlas.journal_entry_lines (account_code);
