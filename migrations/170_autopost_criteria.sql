-- Migration 170: AutoPost Criteria Sets
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Manage AutoPost Criteria Sets
--
-- This migration adds support for centralized, rule-based automatic journal posting.
-- Criteria sets define which journals (by Ledger, Source, Category) should be posted
-- automatically, and within what date range relative to today.

BEGIN;

-- ============================================================================
-- AutoPost Criteria Sets
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_autopost_criteria_sets (
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

CREATE INDEX IF NOT EXISTS idx_autopost_sets_org ON _atlas.fin_autopost_criteria_sets(organization_id);

-- ============================================================================
-- AutoPost Criteria
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_autopost_criteria (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    criteria_set_id UUID NOT NULL REFERENCES _atlas.fin_autopost_criteria_sets(id) ON DELETE CASCADE,
    ledger_id UUID REFERENCES _atlas.accounting_books(id),
    journal_source_id UUID REFERENCES _atlas.fin_journal_sources(id),
    journal_category_id UUID REFERENCES _atlas.fin_journal_categories(id),
    num_days_before INTEGER NOT NULL DEFAULT 0,
    num_days_after INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_autopost_criteria_set ON _atlas.fin_autopost_criteria(criteria_set_id);

COMMENT ON TABLE _atlas.fin_autopost_criteria_sets IS 'Sets of criteria for automatic journal posting';
COMMENT ON TABLE _atlas.fin_autopost_criteria IS 'Individual rule lines within an AutoPost criteria set';

COMMIT;
