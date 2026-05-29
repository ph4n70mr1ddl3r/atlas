-- Migration 167: Ledger Sets
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Manage Ledger Sets
--
-- Ledger sets allow grouping multiple ledgers that share the same Chart of Accounts
-- and Accounting Calendar. This enables cross-ledger reporting and processing.

BEGIN;

-- ============================================================================
-- Ledger Sets
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_ledger_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    chart_of_accounts_id VARCHAR(100) NOT NULL,
    accounting_calendar VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, name)
);

CREATE INDEX idx_ledger_sets_org ON _atlas.fin_ledger_sets(organization_id);

-- ============================================================================
-- Ledger Set Assignments
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_ledger_set_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ledger_set_id UUID NOT NULL REFERENCES _atlas.fin_ledger_sets(id) ON DELETE CASCADE,
    ledger_id UUID NOT NULL REFERENCES _atlas.accounting_books(id) ON DELETE CASCADE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(ledger_set_id, ledger_id)
);

CREATE INDEX idx_ledger_set_assignments_set ON _atlas.fin_ledger_set_assignments(ledger_set_id);
CREATE INDEX idx_ledger_set_assignments_ledger ON _atlas.fin_ledger_set_assignments(ledger_id);

COMMENT ON TABLE _atlas.fin_ledger_sets IS 'Groups of ledgers sharing same COA and Calendar';
COMMENT ON TABLE _atlas.fin_ledger_set_assignments IS 'Links individual ledgers to a ledger set';

COMMIT;
