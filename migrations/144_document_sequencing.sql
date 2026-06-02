-- 144_document_sequencing.sql
-- Oracle Fusion Cloud ERP Feature: Document Sequencing
-- Provides automatic sequential document numbering for regulatory compliance,
-- supporting gapless and gap-permitted sequences, configurable reset frequency,
-- prefix/suffix formatting, sequence assignments, and full audit trail.
-- Oracle Fusion: General Ledger > Setup > Document Sequencing

-- ============================================================================
-- Document Sequences
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.document_sequences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    sequence_type VARCHAR(20) NOT NULL DEFAULT 'gap_permitted',
    document_type VARCHAR(50) NOT NULL DEFAULT 'custom',
    initial_value BIGINT NOT NULL DEFAULT 1,
    current_value BIGINT NOT NULL DEFAULT 0,
    increment_by INT NOT NULL DEFAULT 1,
    max_value BIGINT,
    cycle_flag BOOLEAN NOT NULL DEFAULT false,
    prefix VARCHAR(50),
    suffix VARCHAR(50),
    pad_length INT NOT NULL DEFAULT 0,
    pad_character VARCHAR(1) NOT NULL DEFAULT '0',
    reset_frequency VARCHAR(20),
    last_reset_date DATE,
    effective_from DATE,
    effective_to DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_docseq_org ON _atlas.document_sequences(organization_id);
CREATE INDEX IF NOT EXISTS idx_docseq_status ON _atlas.document_sequences(status);
CREATE INDEX IF NOT EXISTS idx_docseq_doc_type ON _atlas.document_sequences(document_type);

-- ============================================================================
-- Sequence Assignments (map sequences to document categories)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.document_sequence_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    sequence_id UUID NOT NULL REFERENCES _atlas.document_sequences(id) ON DELETE CASCADE,
    sequence_code VARCHAR(100) NOT NULL,
    document_category VARCHAR(100) NOT NULL,
    business_unit_id UUID,
    ledger_id UUID,
    method VARCHAR(20) NOT NULL DEFAULT 'automatic',
    effective_from DATE,
    effective_to DATE,
    priority INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_docseqassign_org ON _atlas.document_sequence_assignments(organization_id);
CREATE INDEX IF NOT EXISTS idx_docseqassign_seq ON _atlas.document_sequence_assignments(sequence_id);
CREATE INDEX IF NOT EXISTS idx_docseqassign_cat ON _atlas.document_sequence_assignments(document_category);
CREATE INDEX IF NOT EXISTS idx_docseqassign_status ON _atlas.document_sequence_assignments(status);

-- ============================================================================
-- Sequence Audit Trail (every generated number is logged)
-- ============================================================================

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
    generated_at TIMESTAMPTZ DEFAULT now(),
    generated_by UUID,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_docseqaudit_org ON _atlas.document_sequence_audit(organization_id);
CREATE INDEX IF NOT EXISTS idx_docseqaudit_seq ON _atlas.document_sequence_audit(sequence_id);
CREATE INDEX IF NOT EXISTS idx_docseqaudit_cat ON _atlas.document_sequence_audit(document_category);
CREATE INDEX IF NOT EXISTS idx_docseqaudit_doc ON _atlas.document_sequence_audit(document_id);
CREATE INDEX IF NOT EXISTS idx_docseqaudit_gen ON _atlas.document_sequence_audit(generated_at);

COMMENT ON TABLE _atlas.document_sequences IS 'Defines document numbering sequences for regulatory compliance';
COMMENT ON TABLE _atlas.document_sequence_assignments IS 'Maps sequences to document categories, business units, and ledgers';
COMMENT ON TABLE _atlas.document_sequence_audit IS 'Complete audit trail of every document number generated';
