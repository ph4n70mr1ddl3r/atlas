-- Hedge Management
-- Oracle Fusion: Treasury > Hedge Management
-- IFRS 9 / ASC 815 Hedge Accounting
-- Manages derivative instruments, hedge relationships, effectiveness testing,
-- and hedge documentation for hedge accounting compliance.
-- Uses VARCHAR for monetary amounts to avoid SQLx NUMERIC decoding issues.

-- Derivative Instruments table
CREATE TABLE IF NOT EXISTS _atlas.derivative_instruments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    instrument_number VARCHAR(50) NOT NULL,
    instrument_type VARCHAR(30) NOT NULL,
    underlying_type VARCHAR(30) NOT NULL,
    underlying_description TEXT,
    currency_code VARCHAR(3) DEFAULT 'USD',
    counter_currency_code VARCHAR(3),
    notional_amount VARCHAR(50) NOT NULL,
    strike_rate VARCHAR(30),
    forward_rate VARCHAR(30),
    spot_rate VARCHAR(30),
    option_type VARCHAR(20) DEFAULT 'none',
    premium_amount VARCHAR(50) DEFAULT '0',
    trade_date DATE,
    effective_date DATE,
    maturity_date DATE,
    settlement_date DATE,
    settlement_type VARCHAR(20) DEFAULT 'cash',
    counterparty_name VARCHAR(300),
    counterparty_reference VARCHAR(100),
    portfolio_code VARCHAR(50),
    trading_book VARCHAR(100),
    accounting_treatment VARCHAR(30) DEFAULT 'trading',
    fair_value VARCHAR(50) DEFAULT '0',
    unrealized_gain_loss VARCHAR(50) DEFAULT '0',
    realized_gain_loss VARCHAR(50) DEFAULT '0',
    valuation_method VARCHAR(100),
    last_valuation_date DATE,
    risk_factor VARCHAR(200),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    UNIQUE(organization_id, instrument_number)
);

-- Hedge Relationships table
CREATE TABLE IF NOT EXISTS _atlas.hedge_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    hedge_id VARCHAR(50) NOT NULL,
    hedge_type VARCHAR(30) NOT NULL,
    derivative_id UUID REFERENCES _atlas.derivative_instruments(id),
    derivative_number VARCHAR(50),
    hedged_item_description TEXT,
    hedged_item_id UUID,
    hedged_risk VARCHAR(30) NOT NULL,
    hedge_strategy VARCHAR(50),
    hedged_item_reference VARCHAR(200),
    hedged_item_currency VARCHAR(3),
    hedged_amount VARCHAR(50) NOT NULL,
    hedge_ratio VARCHAR(20),
    designated_start_date DATE,
    designated_end_date DATE,
    effectiveness_method VARCHAR(30) NOT NULL DEFAULT 'dollar_offset',
    critical_terms_match VARCHAR(200),
    prospective_effective BOOLEAN DEFAULT false,
    retrospective_effective BOOLEAN DEFAULT false,
    hedge_documentation_ref VARCHAR(200),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    last_effectiveness_test_date DATE,
    last_effectiveness_result VARCHAR(30),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    UNIQUE(organization_id, hedge_id)
);

-- Hedge Effectiveness Tests table
CREATE TABLE IF NOT EXISTS _atlas.hedge_effectiveness_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    hedge_relationship_id UUID NOT NULL REFERENCES _atlas.hedge_relationships(id),
    hedge_id VARCHAR(50),
    test_type VARCHAR(20) NOT NULL,
    effectiveness_method VARCHAR(30) NOT NULL,
    test_date DATE NOT NULL,
    test_period_start DATE,
    test_period_end DATE,
    derivative_fair_value_change VARCHAR(50),
    hedged_item_fair_value_change VARCHAR(50),
    hedge_ratio_result VARCHAR(30),
    ratio_lower_bound VARCHAR(20) DEFAULT '0.80',
    ratio_upper_bound VARCHAR(20) DEFAULT '1.25',
    effectiveness_result VARCHAR(20) NOT NULL,
    ineffective_amount VARCHAR(50) DEFAULT '0',
    cumulative_gain_loss VARCHAR(50) DEFAULT '0',
    regression_r_squared VARCHAR(30),
    notes TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    updated_by UUID
);

-- Hedge Documentation table
CREATE TABLE IF NOT EXISTS _atlas.hedge_documentation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    hedge_relationship_id UUID REFERENCES _atlas.hedge_relationships(id),
    hedge_id VARCHAR(50),
    document_number VARCHAR(50) NOT NULL,
    hedge_type VARCHAR(30) NOT NULL,
    risk_management_objective TEXT,
    hedging_strategy_description TEXT,
    hedged_item_description TEXT,
    hedged_risk_description TEXT,
    derivative_description TEXT,
    effectiveness_method_description TEXT,
    assessment_frequency VARCHAR(50),
    designation_date DATE,
    documentation_date DATE,
    approval_date DATE,
    approved_by UUID,
    prepared_by VARCHAR(300),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    UNIQUE(organization_id, document_number)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_deriv_org ON _atlas.derivative_instruments(organization_id);
CREATE INDEX IF NOT EXISTS idx_deriv_type ON _atlas.derivative_instruments(instrument_type);
CREATE INDEX IF NOT EXISTS idx_deriv_status ON _atlas.derivative_instruments(status);
CREATE INDEX IF NOT EXISTS idx_deriv_underlying ON _atlas.derivative_instruments(underlying_type);
CREATE INDEX IF NOT EXISTS idx_deriv_maturity ON _atlas.derivative_instruments(maturity_date);

CREATE INDEX IF NOT EXISTS idx_hedge_rel_org ON _atlas.hedge_relationships(organization_id);
CREATE INDEX IF NOT EXISTS idx_hedge_rel_type ON _atlas.hedge_relationships(hedge_type);
CREATE INDEX IF NOT EXISTS idx_hedge_rel_status ON _atlas.hedge_relationships(status);
CREATE INDEX IF NOT EXISTS idx_hedge_rel_derivative ON _atlas.hedge_relationships(derivative_id);

CREATE INDEX IF NOT EXISTS idx_hedge_test_org ON _atlas.hedge_effectiveness_tests(organization_id);
CREATE INDEX IF NOT EXISTS idx_hedge_test_rel ON _atlas.hedge_effectiveness_tests(hedge_relationship_id);
CREATE INDEX IF NOT EXISTS idx_hedge_test_result ON _atlas.hedge_effectiveness_tests(effectiveness_result);
CREATE INDEX IF NOT EXISTS idx_hedge_test_date ON _atlas.hedge_effectiveness_tests(test_date);

CREATE INDEX IF NOT EXISTS idx_hedge_doc_org ON _atlas.hedge_documentation(organization_id);
CREATE INDEX IF NOT EXISTS idx_hedge_doc_rel ON _atlas.hedge_documentation(hedge_relationship_id);
CREATE INDEX IF NOT EXISTS idx_hedge_doc_status ON _atlas.hedge_documentation(status);
