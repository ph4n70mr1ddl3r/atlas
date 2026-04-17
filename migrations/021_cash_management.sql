-- 021_cash_management.sql
-- Cash Position & Cash Forecasting (Oracle Fusion Treasury Management)
-- 
-- Provides:
-- - Cash Positions: Real-time cash balances across bank accounts
-- - Cash Forecast Templates: Define forecast structure, time buckets
-- - Cash Forecast Sources: Configure data sources (AP, AR, Payroll, etc.)
-- - Cash Forecasts: Generated forecast runs with projected cash flows
-- - Cash Forecast Lines: Individual period/source amounts

-- Cash Positions
CREATE TABLE IF NOT EXISTS _atlas.cash_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    bank_account_id UUID NOT NULL,
    account_number VARCHAR(50) NOT NULL,
    account_name VARCHAR(200) NOT NULL,
    currency_code VARCHAR(3) NOT NULL,
    book_balance NUMERIC(20, 2) NOT NULL DEFAULT 0,
    available_balance NUMERIC(20, 2) NOT NULL DEFAULT 0,
    float_amount NUMERIC(20, 2) NOT NULL DEFAULT 0,
    one_day_float NUMERIC(20, 2) NOT NULL DEFAULT 0,
    two_day_float NUMERIC(20, 2) NOT NULL DEFAULT 0,
    position_date DATE NOT NULL,
    average_balance NUMERIC(20, 2),
    prior_day_balance NUMERIC(20, 2),
    projected_inflows NUMERIC(20, 2) NOT NULL DEFAULT 0,
    projected_outflows NUMERIC(20, 2) NOT NULL DEFAULT 0,
    projected_net NUMERIC(20, 2) NOT NULL DEFAULT 0,
    is_reconciled BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, bank_account_id, position_date)
);

CREATE INDEX IF NOT EXISTS idx_cash_positions_org_date ON _atlas.cash_positions(organization_id, position_date);
CREATE INDEX IF NOT EXISTS idx_cash_positions_bank ON _atlas.cash_positions(bank_account_id);

-- Cash Forecast Templates
CREATE TABLE IF NOT EXISTS _atlas.cash_forecast_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    bucket_type VARCHAR(20) NOT NULL DEFAULT 'monthly',  -- daily, weekly, monthly
    number_of_periods INT NOT NULL DEFAULT 12,
    start_offset_days INT NOT NULL DEFAULT 0,
    is_default BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    columns JSONB NOT NULL DEFAULT '[]',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_cash_forecast_templates_org ON _atlas.cash_forecast_templates(organization_id);

-- Cash Forecast Sources
CREATE TABLE IF NOT EXISTS _atlas.cash_forecast_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.cash_forecast_templates(id) ON DELETE CASCADE,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    source_type VARCHAR(50) NOT NULL,  -- accounts_payable, accounts_receivable, payroll, etc.
    cash_flow_direction VARCHAR(10) NOT NULL DEFAULT 'both',  -- inflow, outflow, both
    is_actual BOOLEAN NOT NULL DEFAULT false,
    display_order INT NOT NULL DEFAULT 10,
    is_active BOOLEAN NOT NULL DEFAULT true,
    lead_time_days INT NOT NULL DEFAULT 0,
    payment_terms_reference VARCHAR(200),
    account_code_filter VARCHAR(100),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(template_id, code)
);

CREATE INDEX IF NOT EXISTS idx_cash_forecast_sources_template ON _atlas.cash_forecast_sources(template_id);

-- Cash Forecasts
CREATE TABLE IF NOT EXISTS _atlas.cash_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    forecast_number VARCHAR(50) NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.cash_forecast_templates(id),
    template_name VARCHAR(200),
    name VARCHAR(200) NOT NULL,
    description TEXT,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    opening_balance NUMERIC(20, 2) NOT NULL DEFAULT 0,
    total_inflows NUMERIC(20, 2) NOT NULL DEFAULT 0,
    total_outflows NUMERIC(20, 2) NOT NULL DEFAULT 0,
    net_cash_flow NUMERIC(20, 2) NOT NULL DEFAULT 0,
    closing_balance NUMERIC(20, 2) NOT NULL DEFAULT 0,
    minimum_balance NUMERIC(20, 2) NOT NULL DEFAULT 0,
    maximum_balance NUMERIC(20, 2) NOT NULL DEFAULT 0,
    deficit_count INT NOT NULL DEFAULT 0,
    surplus_count INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft, generated, approved, superseded
    is_latest BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, forecast_number)
);

CREATE INDEX IF NOT EXISTS idx_cash_forecasts_org ON _atlas.cash_forecasts(organization_id);
CREATE INDEX IF NOT EXISTS idx_cash_forecasts_template ON _atlas.cash_forecasts(template_id);
CREATE INDEX IF NOT EXISTS idx_cash_forecasts_status ON _atlas.cash_forecasts(status);

-- Cash Forecast Lines
CREATE TABLE IF NOT EXISTS _atlas.cash_forecast_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    forecast_id UUID NOT NULL REFERENCES _atlas.cash_forecasts(id) ON DELETE CASCADE,
    source_id UUID NOT NULL REFERENCES _atlas.cash_forecast_sources(id),
    source_name VARCHAR(200) NOT NULL,
    source_type VARCHAR(50) NOT NULL,
    cash_flow_direction VARCHAR(10) NOT NULL,
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    period_label VARCHAR(100) NOT NULL,
    period_sequence INT NOT NULL,
    amount NUMERIC(20, 2) NOT NULL DEFAULT 0,
    cumulative_amount NUMERIC(20, 2) NOT NULL DEFAULT 0,
    is_actual BOOLEAN NOT NULL DEFAULT false,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    transaction_count INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cash_forecast_lines_forecast ON _atlas.cash_forecast_lines(forecast_id);
CREATE INDEX IF NOT EXISTS idx_cash_forecast_lines_period ON _atlas.cash_forecast_lines(forecast_id, period_start_date, period_end_date);
