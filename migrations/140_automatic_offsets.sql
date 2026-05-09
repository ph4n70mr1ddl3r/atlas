-- ============================================================================
-- Automatic Offsets (Intercompany Balancing)
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Automatic Offsets
-- Migration 140
--
-- Manages the automatic generation of intercompany offset (due-to / due-from)
-- entries when a single journal entry's lines span multiple balancing segments
-- (legal entities). Ensures each balancing segment is self-balancing.
--
-- Key concepts:
--   - Offset Templates: Define the offset accounts per balancing segment pair
--   - Offset Rules: Define when and how offsets are generated
--   - Offset Generations: Audit trail of generated offset entries
--   - Offset Lines: Individual due-to / due-from lines generated
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Offset Templates
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.auto_offset_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_code VARCHAR(50) NOT NULL,
    template_name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Balancing segment configuration
    balancing_segment VARCHAR(30) NOT NULL DEFAULT 'entity'
        CHECK (balancing_segment IN ('entity', 'department', 'cost_center', 'location', 'intercompany')),
    intercompany_segment VARCHAR(30),

    -- Generation method
    generation_method VARCHAR(30) NOT NULL DEFAULT 'single_entry'
        CHECK (generation_method IN ('single_entry', 'multi_entry', 'net_zero')),

    -- Default offset account (used when no specific template line matches)
    default_offset_account VARCHAR(100) NOT NULL,
    default_offset_account_description VARCHAR(300),

    -- Intra-entity balancing
    enable_intra_entity BOOLEAN NOT NULL DEFAULT false,
    intra_entity_account VARCHAR(100),

    -- Control
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(organization_id, template_code)
);

CREATE INDEX IF NOT EXISTS idx_auto_offset_templates_org
    ON _atlas.auto_offset_templates(organization_id);
CREATE INDEX IF NOT EXISTS idx_auto_offset_templates_active
    ON _atlas.auto_offset_templates(organization_id, is_active);

-- ============================================================================
-- Offset Template Lines (per balancing segment value / entity)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.auto_offset_template_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.auto_offset_templates(id) ON DELETE CASCADE,
    line_number INT NOT NULL,

    -- The balancing segment value this line applies to
    balancing_segment_value VARCHAR(100) NOT NULL,

    -- Offset (due-to / due-from) accounts
    due_to_account VARCHAR(100) NOT NULL,
    due_to_account_description VARCHAR(300),
    due_from_account VARCHAR(100) NOT NULL,
    due_from_account_description VARCHAR(300),

    -- Clearing account (optional, for net-zero method)
    clearing_account VARCHAR(100),

    -- Priority (higher = more specific match wins)
    priority INT NOT NULL DEFAULT 100,

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(template_id, line_number)
);

CREATE INDEX IF NOT EXISTS idx_auto_offset_template_lines_template
    ON _atlas.auto_offset_template_lines(template_id);
CREATE INDEX IF NOT EXISTS idx_auto_offset_template_lines_seg
    ON _atlas.auto_offset_template_lines(template_id, balancing_segment_value);

-- ============================================================================
-- Offset Generations (audit trail of offset entry generation)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.auto_offset_generations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    generation_number VARCHAR(50) NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.auto_offset_templates(id),

    -- Source reference (what triggered this generation)
    source_type VARCHAR(50) NOT NULL
        CHECK (source_type IN ('journal_entry', 'invoice', 'payment', 'receipt', 'manual')),
    source_id UUID,
    source_number VARCHAR(100),

    -- Generation details
    fiscal_year INT NOT NULL,
    period_name VARCHAR(50) NOT NULL,
    generation_date DATE NOT NULL,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',

    -- Statistics
    total_source_lines INT NOT NULL DEFAULT 0,
    balancing_segments_affected INT NOT NULL DEFAULT 0,
    total_offset_lines INT NOT NULL DEFAULT 0,
    total_debit_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_credit_amount DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'generated'
        CHECK (status IN ('generated', 'posted', 'reversed', 'cancelled')),

    -- GL posting
    gl_batch_id UUID,
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    reversed_by UUID,
    reversed_at TIMESTAMPTZ,

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(organization_id, generation_number)
);

CREATE INDEX IF NOT EXISTS idx_auto_offset_gen_org
    ON _atlas.auto_offset_generations(organization_id);
CREATE INDEX IF NOT EXISTS idx_auto_offset_gen_template
    ON _atlas.auto_offset_generations(template_id);
CREATE INDEX IF NOT EXISTS idx_auto_offset_gen_status
    ON _atlas.auto_offset_generations(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_auto_offset_gen_source
    ON _atlas.auto_offset_generations(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_auto_offset_gen_period
    ON _atlas.auto_offset_generations(fiscal_year, period_name);

-- ============================================================================
-- Offset Generation Lines (individual due-to / due-from entries)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.auto_offset_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    generation_id UUID NOT NULL REFERENCES _atlas.auto_offset_generations(id) ON DELETE CASCADE,
    line_number INT NOT NULL,

    -- Balancing segment references
    from_segment_value VARCHAR(100) NOT NULL,
    to_segment_value VARCHAR(100) NOT NULL,

    -- Offset entry details
    offset_type VARCHAR(20) NOT NULL
        CHECK (offset_type IN ('due_to', 'due_from', 'clearing')),
    account_code VARCHAR(100) NOT NULL,
    account_description VARCHAR(300),

    -- Amounts
    amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',

    -- Line status
    status VARCHAR(20) NOT NULL DEFAULT 'generated'
        CHECK (status IN ('generated', 'posted', 'reversed')),

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_auto_offset_lines_gen
    ON _atlas.auto_offset_lines(generation_id);
CREATE INDEX IF NOT EXISTS idx_auto_offset_lines_seg
    ON _atlas.auto_offset_lines(from_segment_value, to_segment_value);

-- ============================================================================
-- Offset Generation Activities (audit trail)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.auto_offset_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    generation_id UUID NOT NULL,
    line_id UUID,

    activity_type VARCHAR(50) NOT NULL,
    description TEXT,
    old_status VARCHAR(20),
    new_status VARCHAR(20),
    performed_by UUID,
    performed_by_name VARCHAR(200),
    details JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_auto_offset_activities_gen
    ON _atlas.auto_offset_activities(generation_id);
CREATE INDEX IF NOT EXISTS idx_auto_offset_activities_org
    ON _atlas.auto_offset_activities(organization_id);
