-- Cash Concentration / Pooling (Oracle Fusion: Treasury > Cash Pooling)
-- Supports physical and notional cash pools, sweep rules, sweep runs,
-- and pool participants for automated cash concentration.

CREATE TABLE IF NOT EXISTS _atlas.cash_pools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    pool_code VARCHAR(50) NOT NULL,
    pool_name VARCHAR(200) NOT NULL,
    pool_type VARCHAR(30) NOT NULL DEFAULT 'physical',  -- physical, notional
    concentration_account_id UUID,                        -- the master/target account
    concentration_account_name VARCHAR(200),
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(30) NOT NULL DEFAULT 'draft',         -- draft, active, suspended, closed
    effective_date DATE,
    termination_date DATE,
    sweep_frequency VARCHAR(30),                          -- daily, weekly, monthly, on_demand
    sweep_time VARCHAR(10),                               -- HH:MM in 24h format
    minimum_transfer_amount DOUBLE PRECISION DEFAULT 0,
    maximum_transfer_amount DOUBLE PRECISION,
    target_balance DOUBLE PRECISION,                         -- for target-balance sweeps
    interest_allocation_method VARCHAR(30),               -- proportional, flat_rate, none
    interest_rate NUMERIC(10,6),                          -- for notional pooling
    description TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    UNIQUE(organization_id, pool_code)
);

CREATE TABLE IF NOT EXISTS _atlas.cash_pool_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    pool_id UUID NOT NULL REFERENCES _atlas.cash_pools(id) ON DELETE CASCADE,
    participant_code VARCHAR(50) NOT NULL,
    bank_account_id UUID,
    bank_account_name VARCHAR(200),
    bank_name VARCHAR(200),
    account_number VARCHAR(50),
    participant_type VARCHAR(30) NOT NULL DEFAULT 'source', -- source, concentration, both
    sweep_direction VARCHAR(30) NOT NULL DEFAULT 'to_concentration', -- to_concentration, from_concentration, two_way
    priority INT DEFAULT 0,
    minimum_balance DOUBLE PRECISION,              -- floor balance after sweep
    maximum_balance DOUBLE PRECISION,                         -- ceiling balance (excess swept)
    threshold_amount DOUBLE PRECISION,              -- only sweep above this amount
    current_balance DOUBLE PRECISION,
    status VARCHAR(30) NOT NULL DEFAULT 'active',          -- active, suspended, removed
    effective_date DATE,
    termination_date DATE,
    entity_id UUID,                                        -- subsidiary / BU
    entity_name VARCHAR(200),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    UNIQUE(pool_id, participant_code)
);

CREATE TABLE IF NOT EXISTS _atlas.cash_pool_sweep_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    pool_id UUID NOT NULL REFERENCES _atlas.cash_pools(id) ON DELETE CASCADE,
    rule_code VARCHAR(50) NOT NULL,
    rule_name VARCHAR(200) NOT NULL,
    sweep_type VARCHAR(30) NOT NULL DEFAULT 'zero_balance', -- zero_balance, target_balance, threshold, excess_balance
    participant_id UUID REFERENCES _atlas.cash_pool_participants(id) ON DELETE CASCADE,
    direction VARCHAR(30) NOT NULL DEFAULT 'to_concentration', -- to_concentration, from_concentration, two_way
    trigger_condition VARCHAR(100),                        -- e.g. balance > 50000
    threshold_amount DOUBLE PRECISION DEFAULT 0,
    target_balance DOUBLE PRECISION DEFAULT 0,
    minimum_transfer DOUBLE PRECISION DEFAULT 0,
    maximum_transfer DOUBLE PRECISION,
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    effective_date DATE,
    termination_date DATE,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    UNIQUE(pool_id, rule_code)
);

CREATE TABLE IF NOT EXISTS _atlas.cash_pool_sweep_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    pool_id UUID NOT NULL REFERENCES _atlas.cash_pools(id),
    run_number VARCHAR(50) NOT NULL,
    run_date DATE NOT NULL,
    run_type VARCHAR(30) NOT NULL DEFAULT 'scheduled',     -- scheduled, manual, automatic
    status VARCHAR(30) NOT NULL DEFAULT 'pending',         -- pending, in_progress, completed, partially_completed, failed, cancelled
    total_swept_amount DOUBLE PRECISION DEFAULT 0,
    total_transactions INT DEFAULT 0,
    successful_transactions INT DEFAULT 0,
    failed_transactions INT DEFAULT 0,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    initiated_by UUID,
    notes TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, run_number)
);

CREATE TABLE IF NOT EXISTS _atlas.cash_pool_sweep_run_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    sweep_run_id UUID NOT NULL REFERENCES _atlas.cash_pool_sweep_runs(id) ON DELETE CASCADE,
    pool_id UUID NOT NULL,
    participant_id UUID NOT NULL REFERENCES _atlas.cash_pool_participants(id),
    participant_code VARCHAR(50),
    bank_account_name VARCHAR(200),
    sweep_rule_id UUID,
    direction VARCHAR(30) NOT NULL,                        -- debit, credit
    pre_sweep_balance DOUBLE PRECISION DEFAULT 0,
    sweep_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    post_sweep_balance DOUBLE PRECISION DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',         -- pending, completed, failed, skipped
    reference_number VARCHAR(100),
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cash_pools_org ON _atlas.cash_pools(organization_id);
CREATE INDEX IF NOT EXISTS idx_cash_pools_status ON _atlas.cash_pools(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_cash_pool_participants_pool ON _atlas.cash_pool_participants(pool_id);
CREATE INDEX IF NOT EXISTS idx_cash_pool_sweep_rules_pool ON _atlas.cash_pool_sweep_rules(pool_id);
CREATE INDEX IF NOT EXISTS idx_cash_pool_sweep_runs_pool ON _atlas.cash_pool_sweep_runs(pool_id);
CREATE INDEX IF NOT EXISTS idx_cash_pool_sweep_run_lines_run ON _atlas.cash_pool_sweep_run_lines(sweep_run_id);
