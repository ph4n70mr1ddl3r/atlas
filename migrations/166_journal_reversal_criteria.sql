-- Migration 166: Journal Reversal Criteria Management
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Manage Journal Reversal Criteria Sets
--
-- This migration adds support for automated journal reversal based on category-level rules.
-- Criteria sets are assigned to accounting books (ledgers) to control reversal behavior.

BEGIN;

-- ============================================================================
-- Journal Reversal Criteria Sets
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_journal_reversal_criteria_sets (
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

CREATE INDEX idx_reversal_criteria_org ON _atlas.fin_journal_reversal_criteria_sets(organization_id);

-- ============================================================================
-- Journal Reversal Criteria Rules
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_journal_reversal_criteria_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    criteria_set_id UUID NOT NULL REFERENCES _atlas.fin_journal_reversal_criteria_sets(id) ON DELETE CASCADE,
    journal_category VARCHAR(100) NOT NULL,
    reversal_period VARCHAR(30) NOT NULL, -- next_period, next_day, same_period, same_day
    reversal_method VARCHAR(30) NOT NULL, -- switch_dr_cr, sign_reverse
    is_automatic_reversal BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(criteria_set_id, journal_category)
);

CREATE INDEX idx_reversal_rules_set ON _atlas.fin_journal_reversal_criteria_rules(criteria_set_id);

-- ============================================================================
-- Add reference to Accounting Books
-- ============================================================================
ALTER TABLE _atlas.accounting_books 
ADD COLUMN IF NOT EXISTS reversal_criteria_set_id UUID REFERENCES _atlas.fin_journal_reversal_criteria_sets(id);

COMMENT ON TABLE _atlas.fin_journal_reversal_criteria_sets IS 'Sets of criteria for automatic journal reversal';
COMMENT ON TABLE _atlas.fin_journal_reversal_criteria_rules IS 'Individual category-level rules for journal reversal';
COMMENT ON COLUMN _atlas.accounting_books.reversal_criteria_set_id IS 'Assigned journal reversal criteria set for this book';

COMMIT;
