-- Encumbrance Management Tables
-- Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Encumbrance Management
-- Tracks financial commitments before actual expenditure for budgetary control.

-- Encumbrance Types
-- Defines the types of commitments an organization tracks.
CREATE TABLE IF NOT EXISTS _atlas.encumbrance_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL DEFAULT 'commitment',
    is_enabled BOOLEAN DEFAULT true,
    allow_manual_entry BOOLEAN DEFAULT true,
    default_encumbrance_account_code VARCHAR(50),
    allow_carry_forward BOOLEAN DEFAULT true,
    priority INT DEFAULT 10,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_enc_types_org ON _atlas.encumbrance_types(organization_id);

-- Encumbrance Entries
-- Represents a commitment transaction (e.g., a purchase order creates an encumbrance).
CREATE TABLE IF NOT EXISTS _atlas.encumbrance_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entry_number VARCHAR(50) NOT NULL,
    encumbrance_type_id UUID NOT NULL REFERENCES _atlas.encumbrance_types(id),
    encumbrance_type_code VARCHAR(100) NOT NULL,
    source_type VARCHAR(100),
    source_id UUID,
    source_number VARCHAR(100),
    description TEXT,
    encumbrance_date DATE NOT NULL,
    original_amount NUMERIC(18,2) DEFAULT 0,
    current_amount NUMERIC(18,2) DEFAULT 0,
    liquidated_amount NUMERIC(18,2) DEFAULT 0,
    adjusted_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    fiscal_year INT,
    period_name VARCHAR(100),
    is_carry_forward BOOLEAN DEFAULT false,
    carried_forward_from_id UUID,
    expiry_date DATE,
    budget_line_id UUID,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    approved_by UUID,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, entry_number)
);

CREATE INDEX IF NOT EXISTS idx_enc_entries_org ON _atlas.encumbrance_entries(organization_id);
CREATE INDEX IF NOT EXISTS idx_enc_entries_status ON _atlas.encumbrance_entries(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_enc_entries_type ON _atlas.encumbrance_entries(organization_id, encumbrance_type_code);
CREATE INDEX IF NOT EXISTS idx_enc_entries_source ON _atlas.encumbrance_entries(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_enc_entries_fiscal ON _atlas.encumbrance_entries(organization_id, fiscal_year);

-- Encumbrance Lines
-- Individual line within an encumbrance entry.
CREATE TABLE IF NOT EXISTS _atlas.encumbrance_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entry_id UUID NOT NULL REFERENCES _atlas.encumbrance_entries(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    account_code VARCHAR(50) NOT NULL,
    account_description VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    project_id UUID,
    project_name VARCHAR(200),
    cost_center VARCHAR(100),
    original_amount NUMERIC(18,2) DEFAULT 0,
    current_amount NUMERIC(18,2) DEFAULT 0,
    liquidated_amount NUMERIC(18,2) DEFAULT 0,
    encumbrance_account_code VARCHAR(50),
    source_line_id UUID,
    attribute_category VARCHAR(100),
    attribute1 VARCHAR(200),
    attribute2 VARCHAR(200),
    attribute3 VARCHAR(200),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_enc_lines_entry ON _atlas.encumbrance_lines(entry_id);
CREATE INDEX IF NOT EXISTS idx_enc_lines_account ON _atlas.encumbrance_lines(organization_id, account_code);

-- Encumbrance Liquidations
-- Records the reduction of an encumbrance when actual expenditure occurs.
CREATE TABLE IF NOT EXISTS _atlas.encumbrance_liquidations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    liquidation_number VARCHAR(50) NOT NULL,
    encumbrance_entry_id UUID NOT NULL REFERENCES _atlas.encumbrance_entries(id),
    encumbrance_line_id UUID REFERENCES _atlas.encumbrance_lines(id),
    liquidation_type VARCHAR(50) NOT NULL DEFAULT 'partial',
    liquidation_amount NUMERIC(18,2) DEFAULT 0,
    source_type VARCHAR(100),
    source_id UUID,
    source_number VARCHAR(100),
    description TEXT,
    liquidation_date DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    reversed_by_id UUID,
    reversal_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, liquidation_number)
);

CREATE INDEX IF NOT EXISTS idx_enc_liq_entry ON _atlas.encumbrance_liquidations(encumbrance_entry_id);
CREATE INDEX IF NOT EXISTS idx_enc_liq_status ON _atlas.encumbrance_liquidations(organization_id, status);

-- Encumbrance Carry-Forward
-- Tracks year-end carry-forward of open encumbrances.
CREATE TABLE IF NOT EXISTS _atlas.encumbrance_carry_forwards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL,
    from_fiscal_year INT NOT NULL,
    to_fiscal_year INT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    entry_count INT DEFAULT 0,
    total_amount NUMERIC(18,2) DEFAULT 0,
    description TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    processed_by UUID,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

CREATE INDEX IF NOT EXISTS idx_enc_cf_org ON _atlas.encumbrance_carry_forwards(organization_id);
