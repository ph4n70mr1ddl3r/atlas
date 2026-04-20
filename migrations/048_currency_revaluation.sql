-- ==============================================================================
-- Currency Revaluation
-- Oracle Fusion Cloud ERP: General Ledger > Currency Revaluation
--
-- Supports defining revaluation rules, executing revaluation runs,
-- generating unrealized gain/loss journal entries, and reversing revaluations.
-- ==============================================================================

-- Revaluation definitions (rules for which accounts to revalue and how)
CREATE TABLE IF NOT EXISTS _atlas.currency_revaluation_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    revaluation_type VARCHAR(50) NOT NULL DEFAULT 'period_end',
    -- 'period_end' for standard period-end revaluation,
    -- 'balance_sheet' for balance sheet accounts only,
    -- 'income_statement' for income statement accounts only
    currency_code VARCHAR(10) NOT NULL,
    rate_type VARCHAR(50) NOT NULL DEFAULT 'period_end',
    gain_account_code VARCHAR(100) NOT NULL,
    loss_account_code VARCHAR(100) NOT NULL,
    unrealized_gain_account_code VARCHAR(100),
    unrealized_loss_account_code VARCHAR(100),
    -- Account range filter (which accounts to revalue)
    account_range_from VARCHAR(100),
    account_range_to VARCHAR(100),
    -- Whether to include subledger entries
    include_subledger BOOLEAN NOT NULL DEFAULT false,
    -- Whether to auto-reverse in next period
    auto_reverse BOOLEAN NOT NULL DEFAULT true,
    -- Reversal period offset (1 = next period)
    reversal_period_offset INT NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Revaluation definition account inclusions (specific accounts or ranges to revalue)
CREATE TABLE IF NOT EXISTS _atlas.currency_revaluation_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_id UUID NOT NULL REFERENCES _atlas.currency_revaluation_definitions(id) ON DELETE CASCADE,
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(200),
    account_type VARCHAR(50) NOT NULL DEFAULT 'asset',
    -- 'asset', 'liability', 'equity', 'revenue', 'expense'
    is_included BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Revaluation runs (batch execution of revaluation)
CREATE TABLE IF NOT EXISTS _atlas.currency_revaluation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(100) NOT NULL,
    definition_id UUID NOT NULL REFERENCES _atlas.currency_revaluation_definitions(id),
    definition_code VARCHAR(100) NOT NULL,
    definition_name VARCHAR(200) NOT NULL,
    -- Period being revalued
    period_name VARCHAR(50) NOT NULL,
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    -- Revaluation date (usually period_end_date)
    revaluation_date DATE NOT NULL,
    currency_code VARCHAR(10) NOT NULL,
    rate_type VARCHAR(50) NOT NULL DEFAULT 'period_end',
    -- Totals
    total_revalued_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_gain_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_loss_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_entries INT NOT NULL DEFAULT 0,
    -- Status: draft, posted, reversed, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- Reversal tracking
    reversal_run_id UUID,
    original_run_id UUID,
    reversed_at TIMESTAMPTZ,
    -- Posting
    posted_at TIMESTAMPTZ,
    posted_by UUID,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Revaluation run lines (individual gain/loss entries per account)
CREATE TABLE IF NOT EXISTS _atlas.currency_revaluation_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.currency_revaluation_runs(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    -- Account being revalued
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(200),
    account_type VARCHAR(50) NOT NULL DEFAULT 'asset',
    -- Original amounts
    original_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    original_currency VARCHAR(10) NOT NULL,
    original_exchange_rate NUMERIC(18, 10) NOT NULL DEFAULT 1,
    original_base_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    -- Revalued amounts
    revalued_exchange_rate NUMERIC(18, 10) NOT NULL DEFAULT 1,
    revalued_base_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    -- Gain/Loss
    gain_loss_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    gain_loss_type VARCHAR(20) NOT NULL DEFAULT 'none',
    -- 'gain', 'loss', 'none'
    -- Offset account for journal entry
    gain_loss_account_code VARCHAR(100) NOT NULL,
    -- Reversal tracking
    reversal_line_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_reval_defs_org ON _atlas.currency_revaluation_definitions(organization_id);
CREATE INDEX IF NOT EXISTS idx_reval_defs_code ON _atlas.currency_revaluation_definitions(organization_id, code);
CREATE INDEX IF NOT EXISTS idx_reval_accounts_def ON _atlas.currency_revaluation_accounts(definition_id);
CREATE INDEX IF NOT EXISTS idx_reval_runs_org ON _atlas.currency_revaluation_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_reval_runs_def ON _atlas.currency_revaluation_runs(definition_id);
CREATE INDEX IF NOT EXISTS idx_reval_runs_status ON _atlas.currency_revaluation_runs(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_reval_lines_run ON _atlas.currency_revaluation_lines(run_id);