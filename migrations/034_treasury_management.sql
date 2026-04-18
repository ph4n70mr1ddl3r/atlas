-- Treasury Management (Oracle Fusion Treasury)
-- Manages counterparties, treasury deals, settlements, and interest calculations.

-- Counterparties (banks and financial institutions)
CREATE TABLE IF NOT EXISTS _atlas.treasury_counterparties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    counterparty_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    counterparty_type VARCHAR(50) NOT NULL DEFAULT 'bank', -- bank, financial_institution, internal
    country_code VARCHAR(3),
    credit_rating VARCHAR(10),
    credit_limit NUMERIC(18, 2),
    settlement_currency VARCHAR(3),
    contact_name VARCHAR(200),
    contact_email VARCHAR(200),
    contact_phone VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, counterparty_code)
);

-- Treasury deals (investments, borrowings, FX)
CREATE TABLE IF NOT EXISTS _atlas.treasury_deals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    deal_number VARCHAR(50) NOT NULL UNIQUE,
    deal_type VARCHAR(50) NOT NULL, -- investment, borrowing, fx_spot, fx_forward
    description TEXT,
    counterparty_id UUID NOT NULL REFERENCES _atlas.treasury_counterparties(id),
    counterparty_name VARCHAR(200),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    principal_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    interest_rate NUMERIC(10, 6),
    interest_basis VARCHAR(20) DEFAULT 'actual_360', -- actual_360, actual_365, 30_360
    start_date DATE NOT NULL,
    maturity_date DATE NOT NULL,
    term_days INT NOT NULL DEFAULT 0,
    -- FX-specific fields
    fx_buy_currency VARCHAR(3),
    fx_buy_amount NUMERIC(18, 2),
    fx_sell_currency VARCHAR(3),
    fx_sell_amount NUMERIC(18, 2),
    fx_rate NUMERIC(18, 8),
    -- Calculated / running values
    accrued_interest NUMERIC(18, 2) NOT NULL DEFAULT 0,
    settlement_amount NUMERIC(18, 2),
    -- GL integration
    gl_account_code VARCHAR(50),
    -- Status / lifecycle
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, authorized, settled, matured, cancelled
    authorized_by UUID,
    authorized_at TIMESTAMPTZ,
    settled_at TIMESTAMPTZ,
    matured_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_treasury_deals_org ON _atlas.treasury_deals(organization_id);
CREATE INDEX idx_treasury_deals_type ON _atlas.treasury_deals(organization_id, deal_type);
CREATE INDEX idx_treasury_deals_status ON _atlas.treasury_deals(organization_id, status);
CREATE INDEX idx_treasury_deals_counterparty ON _atlas.treasury_deals(counterparty_id);
CREATE INDEX idx_treasury_deals_maturity ON _atlas.treasury_deals(maturity_date) WHERE status IN ('authorized', 'settled');

-- Deal settlements
CREATE TABLE IF NOT EXISTS _atlas.treasury_settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    deal_id UUID NOT NULL REFERENCES _atlas.treasury_deals(id),
    settlement_number VARCHAR(50) NOT NULL,
    settlement_type VARCHAR(30) NOT NULL DEFAULT 'full', -- full, partial, early
    settlement_date DATE NOT NULL,
    principal_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    interest_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    payment_reference VARCHAR(100),
    journal_entry_id UUID,
    status VARCHAR(30) NOT NULL DEFAULT 'pending', -- pending, completed, reversed
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(deal_id, settlement_number)
);

CREATE INDEX idx_treasury_settlements_deal ON _atlas.treasury_settlements(deal_id);
