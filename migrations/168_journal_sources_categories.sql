-- Migration 168: Journal Sources and Categories
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Manage Journal Sources / Categories
--
-- This migration adds managed entities for Journal Sources and Journal Categories,
-- allowing for fine-grained control over journal behavior (approval, freezing, etc.)

BEGIN;

-- ============================================================================
-- Journal Sources
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_journal_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    import_journal_references BOOLEAN NOT NULL DEFAULT false,
    freeze_journals BOOLEAN NOT NULL DEFAULT false,
    require_journal_approval BOOLEAN NOT NULL DEFAULT false,
    action_if_unbalanced VARCHAR(30) NOT NULL DEFAULT 'error', -- error, warning, post_to_suspense
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, name)
);

CREATE INDEX idx_journal_sources_org ON _atlas.fin_journal_sources(organization_id);

-- ============================================================================
-- Journal Categories
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_journal_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, name)
);

CREATE INDEX idx_journal_categories_org ON _atlas.fin_journal_categories(organization_id);

COMMENT ON TABLE _atlas.fin_journal_sources IS 'Definitions of journal entry sources and their processing rules';
COMMENT ON TABLE _atlas.fin_journal_categories IS 'Definitions of journal entry categories';

-- Seed default sources and categories (optional, but helpful)
-- INSERT INTO _atlas.fin_journal_sources (organization_id, name, description) 
-- SELECT id, 'Manual', 'Manual journal entries' FROM _atlas.organizations;

COMMIT;
