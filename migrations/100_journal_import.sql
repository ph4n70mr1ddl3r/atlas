-- Journal Import (Oracle Fusion GL > Import Journals)
-- Migration 100

-- Import format definitions
CREATE TABLE IF NOT EXISTS _atlas.journal_import_formats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    source_type VARCHAR(50) NOT NULL DEFAULT 'file',
    file_format VARCHAR(50) NOT NULL DEFAULT 'csv',
    delimiter VARCHAR(10),
    header_row BOOLEAN DEFAULT true,
    ledger_id UUID,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    default_date DATE,
    default_journal_type VARCHAR(100),
    balancing_segment VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    validation_enabled BOOLEAN DEFAULT true,
    auto_post BOOLEAN DEFAULT false,
    max_errors_allowed INT DEFAULT 100,
    column_mappings JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Column mappings for import formats
CREATE TABLE IF NOT EXISTS _atlas.journal_import_column_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    format_id UUID NOT NULL REFERENCES _atlas.journal_import_formats(id) ON DELETE CASCADE,
    column_position INT NOT NULL,
    source_column VARCHAR(200) NOT NULL,
    target_field VARCHAR(100) NOT NULL,
    data_type VARCHAR(50) NOT NULL DEFAULT 'string',
    is_required BOOLEAN DEFAULT false,
    default_value TEXT,
    transformation TEXT,
    validation_rule TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Import batches
CREATE TABLE IF NOT EXISTS _atlas.journal_import_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    format_id UUID NOT NULL REFERENCES _atlas.journal_import_formats(id),
    batch_number VARCHAR(100) NOT NULL,
    name VARCHAR(200),
    description TEXT,
    source VARCHAR(200) NOT NULL DEFAULT 'manual',
    source_file_name VARCHAR(500),
    status VARCHAR(50) NOT NULL DEFAULT 'uploaded',
    total_rows INT DEFAULT 0,
    valid_rows INT DEFAULT 0,
    error_rows INT DEFAULT 0,
    imported_rows INT DEFAULT 0,
    ledger_id UUID,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    journal_batch_id UUID,
    total_debit NUMERIC DEFAULT 0,
    total_credit NUMERIC DEFAULT 0,
    is_balanced BOOLEAN DEFAULT false,
    errors JSONB DEFAULT '[]',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

-- Import rows (individual data rows within a batch)
CREATE TABLE IF NOT EXISTS _atlas.journal_import_rows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL REFERENCES _atlas.journal_import_batches(id) ON DELETE CASCADE,
    row_number INT NOT NULL,
    raw_data JSONB DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    account_code VARCHAR(100),
    account_name VARCHAR(200),
    description TEXT,
    entered_dr NUMERIC DEFAULT 0,
    entered_cr NUMERIC DEFAULT 0,
    currency_code VARCHAR(10),
    exchange_rate NUMERIC,
    gl_date DATE,
    reference VARCHAR(200),
    line_type VARCHAR(50),
    cost_center VARCHAR(100),
    department VARCHAR(100),
    project_code VARCHAR(100),
    error_message TEXT,
    error_field VARCHAR(100),
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_ji_formats_org ON _atlas.journal_import_formats(organization_id);
CREATE INDEX IF NOT EXISTS idx_ji_formats_status ON _atlas.journal_import_formats(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_ji_mappings_format ON _atlas.journal_import_column_mappings(format_id);
CREATE INDEX IF NOT EXISTS idx_ji_batches_org ON _atlas.journal_import_batches(organization_id);
CREATE INDEX IF NOT EXISTS idx_ji_batches_format ON _atlas.journal_import_batches(format_id);
CREATE INDEX IF NOT EXISTS idx_ji_batches_status ON _atlas.journal_import_batches(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_ji_rows_batch ON _atlas.journal_import_rows(batch_id);
CREATE INDEX IF NOT EXISTS idx_ji_rows_status ON _atlas.journal_import_rows(batch_id, status);
