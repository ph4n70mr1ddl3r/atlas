-- Inflation Adjustment (IAS 29 Hyperinflationary Economy Accounting)
-- Oracle Fusion equivalent: Financials > General Ledger > Inflation Adjustment
-- Migration 101

-- Inflation indices track consumer price indices or similar metrics for hyperinflationary economies
CREATE TABLE IF NOT EXISTS _atlas.inflation_indices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    country_code VARCHAR(3) NOT NULL,
    currency_code VARCHAR(10) NOT NULL,
    index_type VARCHAR(50) NOT NULL DEFAULT 'cpi',  -- cpi, gdp_deflator, custom
    is_hyperinflationary BOOLEAN NOT NULL DEFAULT false,
    hyperinflationary_start_date DATE,
    effective_from DATE,
    effective_to DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Inflation index rates (monthly/periodic rates)
CREATE TABLE IF NOT EXISTS _atlas.inflation_index_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    index_id UUID NOT NULL REFERENCES _atlas.inflation_indices(id),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    index_value DECIMAL(18,6) NOT NULL,
    cumulative_factor DECIMAL(18,6) DEFAULT 1.0,
    period_factor DECIMAL(18,6) DEFAULT 1.0,
    source VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(index_id, period_start)
);

-- Inflation adjustment runs
CREATE TABLE IF NOT EXISTS _atlas.inflation_adjustment_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(100) NOT NULL,
    name VARCHAR(200),
    description TEXT,
    index_id UUID NOT NULL REFERENCES _atlas.inflation_indices(id),
    ledger_id UUID,
    from_period DATE NOT NULL,
    to_period DATE NOT NULL,
    adjustment_method VARCHAR(50) NOT NULL DEFAULT 'historical', -- historical, current
    total_debit_adjustment DECIMAL(18,2) DEFAULT 0,
    total_credit_adjustment DECIMAL(18,2) DEFAULT 0,
    total_monetary_gain_loss DECIMAL(18,2) DEFAULT 0,
    account_count INT DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    journal_entry_id UUID,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, run_number)
);

-- Inflation adjustment lines (per account)
CREATE TABLE IF NOT EXISTS _atlas.inflation_adjustment_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.inflation_adjustment_runs(id),
    line_number INT NOT NULL,
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(200),
    account_type VARCHAR(50),  -- monetary, non_monetary
    balance_type VARCHAR(20),  -- debit, credit
    original_balance DECIMAL(18,2) NOT NULL DEFAULT 0,
    restated_balance DECIMAL(18,2) NOT NULL DEFAULT 0,
    adjustment_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    inflation_factor DECIMAL(18,6) NOT NULL DEFAULT 1.0,
    acquisition_date DATE,
    gain_loss_amount DECIMAL(18,2) DEFAULT 0,
    gain_loss_account VARCHAR(100),
    currency_code VARCHAR(10),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Index on frequently queried columns
CREATE INDEX IF NOT EXISTS idx_inflation_indices_org ON _atlas.inflation_indices(organization_id);
CREATE INDEX IF NOT EXISTS idx_inflation_rates_index ON _atlas.inflation_index_rates(index_id);
CREATE INDEX IF NOT EXISTS idx_inflation_runs_org ON _atlas.inflation_adjustment_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_inflation_lines_run ON _atlas.inflation_adjustment_lines(run_id);
