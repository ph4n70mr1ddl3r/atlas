-- Atlas ERP - Multi-Currency Management
-- Oracle Fusion Cloud ERP: General Ledger > Currency Rates Manager
--
-- Manages currency definitions, daily exchange rates, and currency
-- conversion with triangulation support. Tracks unrealized gains/losses
-- on foreign-currency-denominated balances.
--
-- This is a standard Oracle Fusion Cloud ERP feature that enables
-- multi-currency transactions, automatic revaluation, and gain/loss
-- calculation across all financial modules.

-- ============================================================================
-- Currency Definitions
-- Oracle Fusion: Define currencies used across the enterprise
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.currencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- ISO 4217 currency code (USD, EUR, GBP, JPY, etc.)
    code VARCHAR(3) NOT NULL,
    -- Full currency name
    name VARCHAR(200) NOT NULL,
    -- Display symbol ($, €, £, ¥, etc.)
    symbol VARCHAR(10),
    -- Decimal precision (2 for most currencies, 0 for JPY, 3 for BHD)
    precision INT NOT NULL DEFAULT 2,
    -- Whether this is the organization's base (functional) currency
    is_base_currency BOOLEAN DEFAULT false,
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Meta
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_currencies_org ON _atlas.currencies(organization_id);
CREATE INDEX idx_currencies_base ON _atlas.currencies(organization_id, is_base_currency) WHERE is_base_currency = true;

-- ============================================================================
-- Exchange Rates (Daily Rates)
-- Oracle Fusion: Daily Rates table for currency conversion
--
-- Supports multiple rate types: daily, spot, corporate, period average/end
-- Stores both forward and inverse rates for efficient lookup
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.exchange_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Currency pair
    from_currency VARCHAR(3) NOT NULL,
    to_currency VARCHAR(3) NOT NULL,
    -- Rate type: 'daily', 'spot', 'corporate', 'period_average', 'period_end', 'user', 'fixed'
    rate_type VARCHAR(20) NOT NULL DEFAULT 'daily',
    -- The exchange rate (to_amount = from_amount * rate)
    rate NUMERIC(18,10) NOT NULL,
    -- Effective date for this rate
    effective_date DATE NOT NULL,
    -- Auto-computed inverse rate (1 / rate)
    inverse_rate NUMERIC(18,10),
    -- Source of the rate (e.g., 'ECB', 'Fed', 'manual')
    source VARCHAR(100),
    -- Audit
    created_by UUID REFERENCES _atlas.users(id),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    -- One rate per currency pair per date per type
    UNIQUE(organization_id, from_currency, to_currency, rate_type, effective_date)
);

CREATE INDEX idx_exchange_rates_lookup ON _atlas.exchange_rates(
    organization_id, from_currency, to_currency, rate_type, effective_date DESC
);
CREATE INDEX idx_exchange_rates_inverse ON _atlas.exchange_rates(
    organization_id, to_currency, from_currency, rate_type, effective_date DESC
);
CREATE INDEX idx_exchange_rates_date ON _atlas.exchange_rates(effective_date DESC);
CREATE INDEX idx_exchange_rates_org ON _atlas.exchange_rates(organization_id);

-- ============================================================================
-- Currency Conversion History
-- Oracle Fusion: Audit trail of all currency conversions performed
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.currency_conversions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Source transaction reference
    entity_type VARCHAR(100),
    entity_id UUID,
    -- Conversion details
    from_currency VARCHAR(3) NOT NULL,
    to_currency VARCHAR(3) NOT NULL,
    from_amount NUMERIC(18,2) NOT NULL,
    to_amount NUMERIC(18,2) NOT NULL,
    exchange_rate NUMERIC(18,10) NOT NULL,
    rate_type VARCHAR(20) NOT NULL DEFAULT 'daily',
    effective_date DATE NOT NULL,
    -- Gain/loss tracking
    gain_loss_amount NUMERIC(18,2),
    gain_loss_type VARCHAR(10), -- 'gain', 'loss', 'none'
    -- Triangulation info (for indirect conversions through a pivot currency)
    triangulation_currency VARCHAR(3),
    triangulation_rate1 NUMERIC(18,10),
    triangulation_rate2 NUMERIC(18,10),
    -- Audit
    created_by UUID REFERENCES _atlas.users(id),
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_currency_conv_entity ON _atlas.currency_conversions(entity_type, entity_id);
CREATE INDEX idx_currency_conv_org ON _atlas.currency_conversions(organization_id);
CREATE INDEX idx_currency_conv_currencies ON _atlas.currency_conversions(from_currency, to_currency);
CREATE INDEX idx_currency_conv_date ON _atlas.currency_conversions(effective_date);
