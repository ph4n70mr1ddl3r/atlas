-- Document Sequencing (Oracle Fusion GL > Setup > Document Sequencing)
-- Automatic sequential document numbering for regulatory compliance.
-- Supports gapless, gap-permitted, and manual sequences with configurable
-- reset frequency, prefix/suffix formatting, and full audit trail.

-- Document sequence definitions
CREATE TABLE IF NOT EXISTS _atlas.document_sequences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    sequence_type VARCHAR(30) NOT NULL DEFAULT 'gap_permitted', -- gapless, gap_permitted, manual
    document_type VARCHAR(100) NOT NULL, -- invoice, purchase_order, journal_entry, payment, receipt, etc.
    initial_value BIGINT NOT NULL DEFAULT 1,
    current_value BIGINT NOT NULL DEFAULT 0,
    increment_by INT NOT NULL DEFAULT 1,
    max_value BIGINT,
    cycle_flag BOOLEAN NOT NULL DEFAULT false,
    prefix VARCHAR(50),
    suffix VARCHAR(50),
    pad_length INT NOT NULL DEFAULT 0,
    pad_character VARCHAR(1) NOT NULL DEFAULT '0',
    reset_frequency VARCHAR(30), -- daily, monthly, quarterly, annually, never
    last_reset_date DATE,
    effective_from DATE,
    effective_to DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, inactive
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Sequence assignments: map sequences to document categories + business units + ledgers
CREATE TABLE IF NOT EXISTS _atlas.document_sequence_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    sequence_id UUID NOT NULL REFERENCES _atlas.document_sequences(id) ON DELETE CASCADE,
    sequence_code VARCHAR(100) NOT NULL,
    document_category VARCHAR(100) NOT NULL, -- e.g. "accounts_payable_invoice", "gl_journal"
    business_unit_id UUID,
    ledger_id UUID,
    method VARCHAR(20) NOT NULL DEFAULT 'automatic', -- automatic, manual
    effective_from DATE,
    effective_to DATE,
    priority INT NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, inactive
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Sequence audit trail: every number generated is recorded for compliance
CREATE TABLE IF NOT EXISTS _atlas.document_sequence_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    sequence_id UUID NOT NULL,
    sequence_code VARCHAR(100) NOT NULL,
    generated_number VARCHAR(200) NOT NULL,
    numeric_value BIGINT NOT NULL,
    document_category VARCHAR(100) NOT NULL,
    document_id UUID,
    document_number VARCHAR(100),
    business_unit_id UUID,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    generated_by UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_ds_org ON _atlas.document_sequences (organization_id);
CREATE INDEX IF NOT EXISTS idx_ds_type ON _atlas.document_sequences (organization_id, document_type);
CREATE INDEX IF NOT EXISTS idx_ds_status ON _atlas.document_sequences (status);
CREATE INDEX IF NOT EXISTS idx_dsa_category ON _atlas.document_sequence_assignments (organization_id, document_category);
CREATE INDEX IF NOT EXISTS idx_dsa_sequence ON _atlas.document_sequence_assignments (sequence_id);
CREATE INDEX IF NOT EXISTS idx_dsau_sequence ON _atlas.document_sequence_audit (sequence_id);
CREATE INDEX IF NOT EXISTS idx_dsau_category ON _atlas.document_sequence_audit (document_category);
CREATE INDEX IF NOT EXISTS idx_dsau_date ON _atlas.document_sequence_audit (generated_at);
CREATE INDEX IF NOT EXISTS idx_dsau_doc ON _atlas.document_sequence_audit (document_id);
