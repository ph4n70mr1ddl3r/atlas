-- 142_statistical_accounting.sql
-- Oracle Fusion Financial Feature: Statistical Accounting
-- Tracks non-monetary statistical data alongside financial data for
-- reporting and allocation purposes (e.g., headcount, square footage,
-- units produced, machine hours).
-- Oracle Fusion: Financials > General Ledger > Statistical Accounting

-- ============================================================================
-- Statistical Units (the unit of measurement for statistical entries)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.statistical_units (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    stat_type VARCHAR(30) NOT NULL DEFAULT 'custom',
    unit_of_measure VARCHAR(30) NOT NULL DEFAULT 'each',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_su_org ON financials.statistical_units(organization_id);
CREATE INDEX IF NOT EXISTS idx_su_type ON financials.statistical_units(stat_type);
CREATE INDEX IF NOT EXISTS idx_su_active ON financials.statistical_units(is_active);

-- ============================================================================
-- Statistical Entries (actual statistical data records)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.statistical_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entry_number VARCHAR(50) NOT NULL,
    statistical_unit_id UUID NOT NULL REFERENCES financials.statistical_units(id),
    statistical_unit_code VARCHAR(50),
    account_code VARCHAR(500),
    dimension1 VARCHAR(200),
    dimension2 VARCHAR(200),
    dimension3 VARCHAR(200),
    fiscal_year INT NOT NULL,
    period_number INT NOT NULL,
    quantity NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_cost NUMERIC(18,4),
    extended_amount NUMERIC(18,2),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    source_type VARCHAR(50),
    source_id UUID,
    source_number VARCHAR(100),
    description TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_se_org ON financials.statistical_entries(organization_id);
CREATE INDEX IF NOT EXISTS idx_se_unit ON financials.statistical_entries(statistical_unit_id);
CREATE INDEX IF NOT EXISTS idx_se_status ON financials.statistical_entries(status);
CREATE INDEX IF NOT EXISTS idx_se_fiscal ON financials.statistical_entries(fiscal_year, period_number);
CREATE INDEX IF NOT EXISTS idx_se_number ON financials.statistical_entries(entry_number);
CREATE INDEX IF NOT EXISTS idx_se_unit_period ON financials.statistical_entries(statistical_unit_id, fiscal_year, period_number);

-- ============================================================================
-- Statistical Balances (materialized balances per unit/period)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.statistical_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statistical_unit_id UUID NOT NULL REFERENCES financials.statistical_units(id),
    statistical_unit_code VARCHAR(50),
    fiscal_year INT NOT NULL,
    period_number INT NOT NULL,
    beginning_balance NUMERIC(18,4) NOT NULL DEFAULT 0,
    period_activity NUMERIC(18,4) NOT NULL DEFAULT 0,
    ending_balance NUMERIC(18,4) NOT NULL DEFAULT 0,
    last_entry_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(statistical_unit_id, fiscal_year, period_number)
);

CREATE INDEX IF NOT EXISTS idx_sb_org ON financials.statistical_balances(organization_id);
CREATE INDEX IF NOT EXISTS idx_sb_unit ON financials.statistical_balances(statistical_unit_id);
CREATE INDEX IF NOT EXISTS idx_sb_fiscal ON financials.statistical_balances(fiscal_year, period_number);

-- ============================================================================
-- Statistical Entry Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.statistical_entry_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entry_id UUID,
    unit_id UUID,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_sea_entry ON financials.statistical_entry_audit(entry_id);
CREATE INDEX IF NOT EXISTS idx_sea_action ON financials.statistical_entry_audit(action);
CREATE INDEX IF NOT EXISTS idx_sea_performed ON financials.statistical_entry_audit(performed_at);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.statistical_dashboard AS
SELECT
    su.organization_id,
    COUNT(DISTINCT su.id) AS total_units,
    COUNT(DISTINCT su.id) FILTER (WHERE su.is_active) AS active_units,
    COUNT(DISTINCT se.id) AS total_entries,
    COUNT(DISTINCT se.id) FILTER (WHERE se.status = 'posted') AS posted_entries,
    COUNT(DISTINCT se.id) FILTER (WHERE se.status = 'draft') AS draft_entries,
    COUNT(DISTINCT se.id) FILTER (WHERE se.status = 'reversed') AS reversed_entries,
    COALESCE(SUM(se.quantity) FILTER (WHERE se.status = 'posted'), 0) AS total_posted_quantity,
    COALESCE(SUM(se.extended_amount) FILTER (WHERE se.status = 'posted'), 0) AS total_posted_extended,
    (
        SELECT json_agg(jsonb_build_object(
            'stat_type', t.stat_type,
            'unit_count', t.unit_count,
            'entry_count', t.entry_count,
            'total_quantity', t.total_quantity
        )) FROM (
            SELECT
                su2.stat_type,
                COUNT(DISTINCT su2.id) AS unit_count,
                COUNT(se2.id) AS entry_count,
                COALESCE(SUM(se2.quantity) FILTER (WHERE se2.status = 'posted'), 0) AS total_quantity
            FROM financials.statistical_units su2
            LEFT JOIN financials.statistical_entries se2 ON se2.statistical_unit_id = su2.id AND se2.status = 'posted'
            WHERE su2.organization_id = su.organization_id
            GROUP BY su2.stat_type
        ) t
    ) AS by_type
FROM financials.statistical_units su
LEFT JOIN financials.statistical_entries se ON se.statistical_unit_id = su.id
GROUP BY su.organization_id;

COMMENT ON TABLE financials.statistical_units IS 'Statistical units defining non-monetary measurement types for statistical accounting';
COMMENT ON TABLE financials.statistical_entries IS 'Statistical journal entries tracking non-monetary quantities by fiscal period';
COMMENT ON TABLE financials.statistical_balances IS 'Materialized period balances for statistical units';
COMMENT ON TABLE financials.statistical_entry_audit IS 'Audit trail for statistical accounting actions';
COMMENT ON VIEW financials.statistical_dashboard IS 'Dashboard aggregation for statistical accounting overview';
