-- Atlas ERP - Tax Management
-- Oracle Fusion Cloud ERP: Tax > Tax Configuration and Calculation
--
-- Manages tax regimes, tax rates by jurisdiction, tax determination
-- rules, and transaction-level tax calculation. Supports inclusive/
-- exclusive tax handling, tax recovery, and tax reporting.
--
-- This is a standard Oracle Fusion Cloud ERP feature that every
-- implementation requires for sales tax, VAT, GST compliance.

-- ============================================================================
-- Tax Regimes
-- Oracle Fusion: Define tax regimes (e.g., US Sales Tax, EU VAT, India GST)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.tax_regimes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Regime code (e.g., 'US_SALES_TAX', 'EU_VAT', 'IN_GST')
    code VARCHAR(50) NOT NULL,
    -- Display name
    name VARCHAR(200) NOT NULL,
    -- Description
    description TEXT,
    -- Tax type: 'sales_tax', 'vat', 'gst', 'withholding', 'excise', 'customs'
    tax_type VARCHAR(30) NOT NULL DEFAULT 'vat',
    -- Whether tax is included in the line amount (inclusive) or added on top (exclusive)
    default_inclusive BOOLEAN DEFAULT false,
    -- Whether this regime allows tax recovery (input tax credit)
    allows_recovery BOOLEAN DEFAULT false,
    -- Rounding rule: 'nearest', 'up', 'down', 'none'
    rounding_rule VARCHAR(10) NOT NULL DEFAULT 'nearest',
    -- Rounding precision (number of decimal places)
    rounding_precision INT NOT NULL DEFAULT 2,
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Effective dates
    effective_from DATE,
    effective_to DATE,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_tax_regimes_org ON _atlas.tax_regimes(organization_id);
CREATE INDEX idx_tax_regimes_active ON _atlas.tax_regimes(organization_id, is_active) WHERE is_active = true;

-- ============================================================================
-- Tax Jurisdictions
-- Oracle Fusion: Geographic or authority-based tax jurisdictions
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.tax_jurisdictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    regime_id UUID NOT NULL REFERENCES _atlas.tax_regimes(id),
    -- Jurisdiction code (e.g., 'CA', 'NY', 'DE', 'UK')
    code VARCHAR(50) NOT NULL,
    -- Display name (e.g., 'California', 'Germany')
    name VARCHAR(200) NOT NULL,
    -- Geographic scope: 'country', 'state', 'county', 'city', 'region'
    geographic_level VARCHAR(20) NOT NULL DEFAULT 'country',
    -- Country code (ISO 3166-1 alpha-2)
    country_code VARCHAR(2),
    -- State/Province/Region
    state_code VARCHAR(10),
    -- County
    county VARCHAR(100),
    -- City
    city VARCHAR(100),
    -- Postal code pattern (supports LIKE patterns, e.g., '94%')
    postal_code_pattern VARCHAR(20),
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, regime_id, code)
);

CREATE INDEX idx_tax_jurisdictions_org ON _atlas.tax_jurisdictions(organization_id);
CREATE INDEX idx_tax_jurisdictions_regime ON _atlas.tax_jurisdictions(regime_id);

-- ============================================================================
-- Tax Rates
-- Oracle Fusion: Define tax rates per jurisdiction within a regime
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.tax_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    regime_id UUID NOT NULL REFERENCES _atlas.tax_regimes(id),
    jurisdiction_id UUID REFERENCES _atlas.tax_jurisdictions(id),
    -- Rate code (e.g., 'CA_STANDARD', 'EU_REDUCED', 'GST_18')
    code VARCHAR(50) NOT NULL,
    -- Display name (e.g., 'California Standard Rate', 'EU Reduced Rate')
    name VARCHAR(200) NOT NULL,
    -- Rate percentage (e.g., 8.25 for 8.25%)
    rate_percentage NUMERIC(8,4) NOT NULL,
    -- Rate type: 'standard', 'reduced', 'zero', 'exempt'
    rate_type VARCHAR(20) NOT NULL DEFAULT 'standard',
    -- Tax account code for posting
    tax_account_code VARCHAR(50),
    -- Whether tax can be recovered (input tax credit)
    recoverable BOOLEAN DEFAULT false,
    -- Recovery percentage if recoverable (0-100)
    recovery_percentage NUMERIC(5,2) DEFAULT 0,
    -- Effective dates
    effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_to DATE,
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, regime_id, code)
);

CREATE INDEX idx_tax_rates_org ON _atlas.tax_regimes(organization_id);
CREATE INDEX idx_tax_rates_regime ON _atlas.tax_rates(regime_id);
CREATE INDEX idx_tax_rates_jurisdiction ON _atlas.tax_rates(jurisdiction_id);
CREATE INDEX idx_tax_rates_effective ON _atlas.tax_rates(effective_from, effective_to);
CREATE INDEX idx_tax_rates_active ON _atlas.tax_rates(organization_id, is_active) WHERE is_active = true;

-- ============================================================================
-- Tax Determination Rules
-- Oracle Fusion: Rules that determine which taxes apply to a transaction
-- based on product classification, ship-from/to location, etc.
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.tax_determination_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    regime_id UUID NOT NULL REFERENCES _atlas.tax_regimes(id),
    -- Rule name
    name VARCHAR(200) NOT NULL,
    -- Description
    description TEXT,
    -- Priority (lower = evaluated first)
    priority INT NOT NULL DEFAULT 100,
    -- Condition: JSON expression for matching
    -- e.g., {"product_category": "goods", "ship_to_country": "US"}
    condition JSONB NOT NULL DEFAULT '{}',
    -- Action: which tax rate(s) to apply
    -- e.g., {"tax_rate_codes": ["CA_STANDARD"]}
    action JSONB NOT NULL DEFAULT '{}',
    -- Whether to stop evaluating further rules after this match
    stop_on_match BOOLEAN DEFAULT true,
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Effective dates
    effective_from DATE,
    effective_to DATE,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_tax_det_rules_org ON _atlas.tax_determination_rules(organization_id);
CREATE INDEX idx_tax_det_rules_regime ON _atlas.tax_determination_rules(regime_id);
CREATE INDEX idx_tax_det_rules_priority ON _atlas.tax_determination_rules(priority);

-- ============================================================================
-- Tax Lines (calculated taxes on transactions)
-- Oracle Fusion: Tax lines attached to invoice/purchase order lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.tax_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Source transaction reference
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    -- Line reference (links to a specific line on the transaction)
    line_id UUID,
    -- Tax regime and rate applied
    regime_id UUID REFERENCES _atlas.tax_regimes(id),
    jurisdiction_id UUID REFERENCES _atlas.tax_jurisdictions(id),
    tax_rate_id UUID NOT NULL REFERENCES _atlas.tax_rates(id),
    -- Tax calculation details
    taxable_amount NUMERIC(18,2) NOT NULL,
    tax_rate_percentage NUMERIC(8,4) NOT NULL,
    tax_amount NUMERIC(18,2) NOT NULL,
    -- Inclusive tax handling
    is_inclusive BOOLEAN DEFAULT false,
    -- Original amount (before tax extraction, for inclusive)
    original_amount NUMERIC(18,2),
    -- Recovery
    recoverable_amount NUMERIC(18,2),
    non_recoverable_amount NUMERIC(18,2),
    -- Tax account for posting
    tax_account_code VARCHAR(50),
    -- Determination rule that matched (if any)
    determination_rule_id UUID REFERENCES _atlas.tax_determination_rules(id),
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_tax_lines_entity ON _atlas.tax_lines(entity_type, entity_id);
CREATE INDEX idx_tax_lines_org ON _atlas.tax_lines(organization_id);
CREATE INDEX idx_tax_lines_regime ON _atlas.tax_lines(regime_id);
CREATE INDEX idx_tax_lines_rate ON _atlas.tax_rates(id);
CREATE INDEX idx_tax_lines_date ON _atlas.tax_lines(created_at);

-- ============================================================================
-- Tax Reporting Summary
-- Oracle Fusion: Aggregated tax data for filing and reporting
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.tax_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    regime_id UUID NOT NULL REFERENCES _atlas.tax_regimes(id),
    jurisdiction_id UUID REFERENCES _atlas.tax_jurisdictions(id),
    -- Reporting period
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    -- Tax type summary
    total_taxable_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_tax_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_recoverable_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_non_recoverable_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    transaction_count INT NOT NULL DEFAULT 0,
    -- Report status
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- 'draft', 'filed', 'approved'
    filed_by UUID,
    filed_at TIMESTAMPTZ,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, regime_id, jurisdiction_id, period_start, period_end)
);

CREATE INDEX idx_tax_reports_org ON _atlas.tax_reports(organization_id);
CREATE INDEX idx_tax_reports_period ON _atlas.tax_reports(period_start, period_end);
