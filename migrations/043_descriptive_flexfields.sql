-- 043: Descriptive Flexfields (Oracle Fusion Cloud ERP DFF)
--
-- Allows administrators to add custom configurable fields to any entity at runtime.
-- Oracle Fusion equivalent: Application Extensions > Flexfields > Descriptive
--
-- Tables:
--   dff_value_sets         - Value set definitions (validation rules for segments)
--   dff_value_set_entries  - Individual values within independent/dependent value sets
--   dff_flexfields         - Flexfield definitions attached to entities
--   dff_contexts           - Contexts within a flexfield (global + context-sensitive)
--   dff_segments           - Segments (custom fields) within a context
--   dff_data               - Flexfield data stored per entity record

-- ═══════════════════════════════════════════════════════════════════════════════
-- Value Sets
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.dff_value_sets (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID NOT NULL,
    code                VARCHAR(100) NOT NULL,
    name                VARCHAR(200) NOT NULL,
    description         TEXT,
    validation_type     VARCHAR(20) NOT NULL DEFAULT 'none',  -- none, independent, dependent, table, format_only
    data_type           VARCHAR(20) NOT NULL DEFAULT 'string', -- string, number, date, datetime
    max_length          INT NOT NULL DEFAULT 240,
    min_length          INT NOT NULL DEFAULT 0,
    format_mask         VARCHAR(200),
    table_validation    JSONB,             -- {table, value_column, meaning_column, where_clause}
    independent_values  JSONB,             -- deprecated: use dff_value_set_entries instead
    parent_value_set_code VARCHAR(100),    -- for dependent value sets
    is_active           BOOLEAN NOT NULL DEFAULT true,
    created_by          UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.dff_value_set_entries (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID NOT NULL,
    value_set_id        UUID NOT NULL REFERENCES _atlas.dff_value_sets(id) ON DELETE CASCADE,
    value               VARCHAR(500) NOT NULL,
    meaning             VARCHAR(500),
    description         TEXT,
    parent_value        VARCHAR(500),      -- for dependent value sets
    is_enabled          BOOLEAN NOT NULL DEFAULT true,
    effective_from      DATE,
    effective_to        DATE,
    sort_order          INT NOT NULL DEFAULT 0,
    created_by          UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_dff_vs_entries_vs ON _atlas.dff_value_set_entries(value_set_id);
CREATE INDEX IF NOT EXISTS idx_dff_vs_entries_parent ON _atlas.dff_value_set_entries(value_set_id, parent_value);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Flexfields
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.dff_flexfields (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id         UUID NOT NULL,
    code                    VARCHAR(100) NOT NULL,
    name                    VARCHAR(200) NOT NULL,
    description             TEXT,
    entity_name             VARCHAR(200) NOT NULL,    -- the entity/table this DFF is attached to
    context_column          VARCHAR(100) NOT NULL DEFAULT 'dff_context',
    default_context_code    VARCHAR(100),
    is_active               BOOLEAN NOT NULL DEFAULT true,
    created_by              UUID,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_dff_flexfields_entity ON _atlas.dff_flexfields(organization_id, entity_name) WHERE is_active = true;

-- ═══════════════════════════════════════════════════════════════════════════════
-- Contexts
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.dff_contexts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID NOT NULL,
    flexfield_id        UUID NOT NULL REFERENCES _atlas.dff_flexfields(id) ON DELETE CASCADE,
    code                VARCHAR(100) NOT NULL,
    name                VARCHAR(200) NOT NULL,
    description         TEXT,
    is_global           BOOLEAN NOT NULL DEFAULT false,  -- global context applies to all records
    is_enabled          BOOLEAN NOT NULL DEFAULT true,
    created_by          UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(flexfield_id, code)
);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Segments
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.dff_segments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID NOT NULL,
    flexfield_id        UUID NOT NULL REFERENCES _atlas.dff_flexfields(id) ON DELETE CASCADE,
    context_id          UUID NOT NULL REFERENCES _atlas.dff_contexts(id) ON DELETE CASCADE,
    segment_code        VARCHAR(100) NOT NULL,
    name                VARCHAR(200) NOT NULL,
    description         TEXT,
    display_order       INT NOT NULL DEFAULT 1,
    column_name         VARCHAR(100) NOT NULL,   -- e.g., attribute1, attribute2
    data_type           VARCHAR(20) NOT NULL DEFAULT 'string',
    is_required         BOOLEAN NOT NULL DEFAULT false,
    is_read_only        BOOLEAN NOT NULL DEFAULT false,
    is_visible          BOOLEAN NOT NULL DEFAULT true,
    default_value       VARCHAR(500),
    value_set_id        UUID REFERENCES _atlas.dff_value_sets(id),
    value_set_code      VARCHAR(100),  -- denormalized for quick lookup
    help_text           TEXT,
    created_by          UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(context_id, segment_code)
);

CREATE INDEX IF NOT EXISTS idx_dff_segments_context ON _atlas.dff_segments(context_id);
CREATE INDEX IF NOT EXISTS idx_dff_segments_flexfield ON _atlas.dff_segments(flexfield_id);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Flexfield Data (per-record custom field values)
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.dff_data (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID NOT NULL,
    flexfield_id        UUID NOT NULL REFERENCES _atlas.dff_flexfields(id) ON DELETE CASCADE,
    entity_name         VARCHAR(200) NOT NULL,
    entity_id           UUID NOT NULL,
    context_code        VARCHAR(100) NOT NULL,
    segment_values      JSONB NOT NULL DEFAULT '{}',  -- {segment_code: value, ...}
    created_by          UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(entity_name, entity_id, context_code)
);

CREATE INDEX IF NOT EXISTS idx_dff_data_entity ON _atlas.dff_data(entity_name, entity_id);
