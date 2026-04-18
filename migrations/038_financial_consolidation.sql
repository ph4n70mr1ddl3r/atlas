-- ============================================================================
-- Financial Consolidation
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Financial Consolidation
-- Migration 038
--
-- Manages consolidation of financial statements across multiple legal entities
-- and business units, including:
--   - Consolidation scenarios (periodic consolidation runs)
--   - Currency translation (temporal & current-rate methods)
--   - Intercompany elimination entries (auto-generated)
--   - Minority interest calculations
--   - Consolidation adjustments and reclassifications
--   - Equity elimination (investment in subsidiary ↔ subsidiary equity)
--   - Consolidated trial balance and financial statements
-- ============================================================================

-- ============================================================================
-- Consolidation Ledgers
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.consolidation_ledgers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    base_currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    translation_method VARCHAR(30) NOT NULL DEFAULT 'current_rate',  -- current_rate, temporal, weighted_average
    equity_elimination_method VARCHAR(30) NOT NULL DEFAULT 'full',   -- full, proportional, equity_method
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_conso_ledgers_org ON _atlas.consolidation_ledgers(organization_id);

-- ============================================================================
-- Consolidation Entities (subsidiaries / business units participating)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.consolidation_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ledger_id UUID NOT NULL REFERENCES _atlas.consolidation_ledgers(id),
    entity_id UUID NOT NULL,                    -- references the legal entity / BU
    entity_name VARCHAR(200) NOT NULL,
    entity_code VARCHAR(50) NOT NULL,
    local_currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    ownership_percentage NUMERIC(5,2) NOT NULL DEFAULT 100.00,
    consolidation_method VARCHAR(30) NOT NULL DEFAULT 'full',  -- full, proportional, equity_method
    is_active BOOLEAN NOT NULL DEFAULT true,
    include_in_consolidation BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(ledger_id, entity_code)
);

CREATE INDEX idx_conso_entities_ledger ON _atlas.consolidation_entities(ledger_id);
CREATE INDEX idx_conso_entities_entity ON _atlas.consolidation_entities(entity_id);

-- ============================================================================
-- Consolidation Scenarios (periodic consolidation runs)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.consolidation_scenarios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ledger_id UUID NOT NULL REFERENCES _atlas.consolidation_ledgers(id),
    scenario_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    fiscal_year INT NOT NULL,
    period_name VARCHAR(50) NOT NULL,
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',  -- draft, in_progress, pending_review, approved, posted, reversed
    translation_date DATE,
    translation_rate_type VARCHAR(30) DEFAULT 'period_end',
    total_entities INT NOT NULL DEFAULT 0,
    total_eliminations INT NOT NULL DEFAULT 0,
    total_adjustments INT NOT NULL DEFAULT 0,
    total_debits NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credits NUMERIC(18,2) NOT NULL DEFAULT 0,
    is_balanced BOOLEAN NOT NULL DEFAULT false,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, scenario_number)
);

CREATE INDEX idx_conso_scenarios_ledger ON _atlas.consolidation_scenarios(ledger_id);
CREATE INDEX idx_conso_scenarios_status ON _atlas.consolidation_scenarios(organization_id, status);
CREATE INDEX idx_conso_scenarios_period ON _atlas.consolidation_scenarios(fiscal_year, period_name);

-- ============================================================================
-- Consolidation Trial Balance Lines (per entity per scenario)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.consolidation_trial_balance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    scenario_id UUID NOT NULL REFERENCES _atlas.consolidation_scenarios(id),
    entity_id UUID,                              -- NULL for consolidated totals
    entity_code VARCHAR(50),
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(300),
    account_type VARCHAR(50),                    -- asset, liability, equity, revenue, expense
    financial_statement VARCHAR(50),             -- balance_sheet, income_statement, cash_flow
    local_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    local_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    local_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    exchange_rate NUMERIC(18,10),
    translated_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    translated_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    translated_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    elimination_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    elimination_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    elimination_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    minority_interest_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    minority_interest_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    minority_interest_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    consolidated_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    consolidated_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    consolidated_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    is_elimination_entry BOOLEAN NOT NULL DEFAULT false,
    line_type VARCHAR(30) NOT NULL DEFAULT 'entity',  -- entity, elimination, adjustment, minority, consolidated
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_conso_tb_scenario ON _atlas.consolidation_trial_balance(scenario_id);
CREATE INDEX idx_conso_tb_entity ON _atlas.consolidation_trial_balance(scenario_id, entity_id);
CREATE INDEX idx_conso_tb_account ON _atlas.consolidation_trial_balance(scenario_id, account_code);
CREATE INDEX idx_conso_tb_line_type ON _atlas.consolidation_trial_balance(scenario_id, line_type);

-- ============================================================================
-- Intercompany Elimination Rules
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.consolidation_elimination_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ledger_id UUID NOT NULL REFERENCES _atlas.consolidation_ledgers(id),
    rule_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    elimination_type VARCHAR(50) NOT NULL,       -- intercompany_receivable_payable, intercompany_revenue_expense,
                                                  -- investment_equity, intercompany_inventory_profit, other
    from_entity_id UUID,                         -- NULL = all entities
    to_entity_id UUID,                           -- NULL = all entities
    from_account_pattern VARCHAR(200),            -- account code pattern or prefix
    to_account_pattern VARCHAR(200),
    offset_account_code VARCHAR(100) NOT NULL,
    priority INT NOT NULL DEFAULT 100,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(ledger_id, rule_code)
);

CREATE INDEX idx_conso_elim_rules_ledger ON _atlas.consolidation_elimination_rules(ledger_id);

-- ============================================================================
-- Consolidation Adjustments (manual journal adjustments)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.consolidation_adjustments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    scenario_id UUID NOT NULL REFERENCES _atlas.consolidation_scenarios(id),
    adjustment_number VARCHAR(50) NOT NULL,
    description TEXT,
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(300),
    entity_id UUID,
    entity_code VARCHAR(50),
    debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    adjustment_type VARCHAR(50) NOT NULL DEFAULT 'manual',  -- manual, reclassification, correction
    reference VARCHAR(100),
    status VARCHAR(30) NOT NULL DEFAULT 'draft',           -- draft, approved, posted
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_conso_adj_scenario ON _atlas.consolidation_adjustments(scenario_id);
CREATE INDEX idx_conso_adj_status ON _atlas.consolidation_adjustments(status);

-- ============================================================================
-- Consolidation Currency Translation Rates
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.consolidation_translation_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    scenario_id UUID NOT NULL REFERENCES _atlas.consolidation_scenarios(id),
    entity_id UUID NOT NULL,
    from_currency VARCHAR(3) NOT NULL,
    to_currency VARCHAR(3) NOT NULL,
    rate_type VARCHAR(30) NOT NULL DEFAULT 'period_end',  -- period_end, average, historical, spot
    exchange_rate NUMERIC(18,10) NOT NULL,
    effective_date DATE NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(scenario_id, entity_id, rate_type)
);

CREATE INDEX idx_conso_rates_scenario ON _atlas.consolidation_translation_rates(scenario_id);
