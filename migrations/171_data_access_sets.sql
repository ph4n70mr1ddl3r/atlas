-- Migration 171: Data Access Sets
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Manage Data Access Sets
--
-- This migration adds core security infrastructure for General Ledger.
-- Data Access Sets provide granular control over which ledgers and segment values
-- a user can access, and whether they have Read Only or Read/Write permissions.

BEGIN;

-- ============================================================================
-- Data Access Sets
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_data_access_sets (
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

CREATE INDEX IF NOT EXISTS idx_data_access_sets_org ON _atlas.fin_data_access_sets(organization_id);

-- ============================================================================
-- Data Access Set Details
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_data_access_set_details (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    data_access_set_id UUID NOT NULL REFERENCES _atlas.fin_data_access_sets(id) ON DELETE CASCADE,
    ledger_id UUID REFERENCES _atlas.accounting_books(id),
    ledger_set_id UUID REFERENCES _atlas.fin_ledger_sets(id),
    access_level VARCHAR(30) NOT NULL DEFAULT 'read_only', -- read_only, read_write
    all_segment_values BOOLEAN NOT NULL DEFAULT true,
    specific_segment_value VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (ledger_id IS NOT NULL OR ledger_set_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_data_access_details_set ON _atlas.fin_data_access_set_details(data_access_set_id);

COMMENT ON TABLE _atlas.fin_data_access_sets IS 'Security sets for ledger and segment value access';
COMMENT ON TABLE _atlas.fin_data_access_set_details IS 'Individual access rules within a data access set';

COMMIT;
