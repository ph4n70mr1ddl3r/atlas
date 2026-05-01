-- Impairment Management (IAS 36 / ASC 360)
-- Oracle Fusion equivalent: Financials > Fixed Assets > Impairment Management
-- Migration 102

-- Impairment indicators define triggers that may indicate impairment
CREATE TABLE IF NOT EXISTS _atlas.impairment_indicators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    indicator_type VARCHAR(50) NOT NULL DEFAULT 'external', -- external, internal, market
    severity VARCHAR(20) NOT NULL DEFAULT 'medium', -- low, medium, high, critical
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Impairment tests (cash-generating unit or individual asset)
CREATE TABLE IF NOT EXISTS _atlas.impairment_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    test_number VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    test_type VARCHAR(50) NOT NULL DEFAULT 'individual', -- individual, cash_generating_unit
    test_method VARCHAR(50) NOT NULL DEFAULT 'value_in_use', -- value_in_use, fair_value_less_costs
    test_date DATE NOT NULL,
    reporting_period VARCHAR(20),
    indicator_id UUID REFERENCES _atlas.impairment_indicators(id),
    carrying_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    recoverable_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    impairment_loss DECIMAL(18,2) NOT NULL DEFAULT 0,
    reversal_amount DECIMAL(18,2) DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, submitted, approved, completed, reversed
    impairment_account VARCHAR(100),
    reversal_account VARCHAR(100),
    asset_id UUID,
    cgu_id UUID,
    discount_rate DECIMAL(8,4),
    growth_rate DECIMAL(8,4),
    terminal_value DECIMAL(18,2),
    metadata JSONB DEFAULT '{}',
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, test_number)
);

-- Impairment test cash flow projections (for value-in-use calculation)
CREATE TABLE IF NOT EXISTS _atlas.impairment_cash_flows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    test_id UUID NOT NULL REFERENCES _atlas.impairment_tests(id) ON DELETE CASCADE,
    period_year INT NOT NULL,
    period_number INT NOT NULL DEFAULT 1,
    description TEXT,
    cash_inflow DECIMAL(18,2) NOT NULL DEFAULT 0,
    cash_outflow DECIMAL(18,2) NOT NULL DEFAULT 0,
    net_cash_flow DECIMAL(18,2) NOT NULL DEFAULT 0,
    discount_factor DECIMAL(10,6) DEFAULT 1.0,
    present_value DECIMAL(18,2) DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Impairment test assets (links test to specific assets)
CREATE TABLE IF NOT EXISTS _atlas.impairment_test_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    test_id UUID NOT NULL REFERENCES _atlas.impairment_tests(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL,
    asset_number VARCHAR(100),
    asset_name VARCHAR(200),
    asset_category VARCHAR(100),
    carrying_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    recoverable_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    impairment_loss DECIMAL(18,2) NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, impaired, not_impaired, reversed
    impairment_date DATE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_impairment_indicators_org ON _atlas.impairment_indicators(organization_id);
CREATE INDEX IF NOT EXISTS idx_impairment_tests_org ON _atlas.impairment_tests(organization_id);
CREATE INDEX IF NOT EXISTS idx_impairment_tests_status ON _atlas.impairment_tests(status);
CREATE INDEX IF NOT EXISTS idx_impairment_cash_flows_test ON _atlas.impairment_cash_flows(test_id);
CREATE INDEX IF NOT EXISTS idx_impairment_test_assets_test ON _atlas.impairment_test_assets(test_id);
