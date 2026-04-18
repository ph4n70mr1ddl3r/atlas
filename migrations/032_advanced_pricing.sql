-- Advanced Pricing Management
-- Oracle Fusion Cloud ERP: Order Management > Pricing
-- Migration 032

-- Price Lists
CREATE TABLE IF NOT EXISTS _atlas.price_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    list_type VARCHAR(30) NOT NULL DEFAULT 'sale',
    pricing_basis VARCHAR(30) NOT NULL DEFAULT 'fixed',
    effective_from DATE,
    effective_to DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Price List Lines (item-level pricing)
CREATE TABLE IF NOT EXISTS _atlas.price_list_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    price_list_id UUID NOT NULL REFERENCES _atlas.price_lists(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_id UUID,
    item_code VARCHAR(100),
    item_description TEXT,
    pricing_unit_of_measure VARCHAR(30) DEFAULT 'Ea',
    list_price NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_price NUMERIC(18,4) NOT NULL DEFAULT 0,
    cost_price NUMERIC(18,4) DEFAULT 0,
    margin_percent NUMERIC(8,4) DEFAULT 0,
    minimum_quantity NUMERIC(18,4) DEFAULT 1,
    maximum_quantity NUMERIC(18,4),
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Tiered Pricing (quantity breaks)
CREATE TABLE IF NOT EXISTS _atlas.price_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    price_list_line_id UUID NOT NULL REFERENCES _atlas.price_list_lines(id) ON DELETE CASCADE,
    tier_number INT NOT NULL,
    from_quantity NUMERIC(18,4) NOT NULL DEFAULT 0,
    to_quantity NUMERIC(18,4),
    price NUMERIC(18,4) NOT NULL DEFAULT 0,
    discount_percent NUMERIC(8,4) DEFAULT 0,
    price_type VARCHAR(20) NOT NULL DEFAULT 'fixed',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Discount Rules
CREATE TABLE IF NOT EXISTS _atlas.discount_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    discount_type VARCHAR(30) NOT NULL DEFAULT 'percentage',
    discount_value NUMERIC(18,4) NOT NULL DEFAULT 0,
    discount uom VARCHAR(30),
    application_method VARCHAR(30) NOT NULL DEFAULT 'line',
    stacking_rule VARCHAR(20) NOT NULL DEFAULT 'exclusive',
    priority INT DEFAULT 10,
    condition JSONB DEFAULT '{}',
    effective_from DATE,
    effective_to DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    is_active BOOLEAN DEFAULT true,
    usage_count INT DEFAULT 0,
    max_usage INT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Charge Definitions (shipping, handling, surcharges)
CREATE TABLE IF NOT EXISTS _atlas.charge_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    charge_type VARCHAR(30) NOT NULL DEFAULT 'surcharge',
    charge_category VARCHAR(30) NOT NULL DEFAULT 'handling',
    calculation_method VARCHAR(30) NOT NULL DEFAULT 'fixed',
    charge_amount NUMERIC(18,4) DEFAULT 0,
    charge_percent NUMERIC(8,4) DEFAULT 0,
    minimum_charge NUMERIC(18,4) DEFAULT 0,
    maximum_charge NUMERIC(18,4),
    taxable BOOLEAN DEFAULT false,
    condition JSONB DEFAULT '{}',
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Pricing Strategies (rule-based price determination)
CREATE TABLE IF NOT EXISTS _atlas.pricing_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    strategy_type VARCHAR(30) NOT NULL DEFAULT 'price_list',
    priority INT DEFAULT 10,
    condition JSONB DEFAULT '{}',
    price_list_id UUID REFERENCES _atlas.price_lists(id),
    markup_percent NUMERIC(8,4) DEFAULT 0,
    markdown_percent NUMERIC(8,4) DEFAULT 0,
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Price Calculation Logs (audit trail of pricing decisions)
CREATE TABLE IF NOT EXISTS _atlas.price_calculation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    calculation_date TIMESTAMPTZ DEFAULT now(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    line_id UUID,
    item_id UUID,
    item_code VARCHAR(100),
    requested_quantity NUMERIC(18,4),
    unit_list_price NUMERIC(18,4),
    unit_selling_price NUMERIC(18,4),
    discount_amount NUMERIC(18,4) DEFAULT 0,
    discount_rule_id UUID,
    charge_amount NUMERIC(18,4) DEFAULT 0,
    charge_definition_id UUID,
    strategy_id UUID,
    price_list_id UUID,
    calculation_steps JSONB DEFAULT '[]',
    currency_code VARCHAR(3) DEFAULT 'USD',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_price_lists_org ON _atlas.price_lists(organization_id);
CREATE INDEX IF NOT EXISTS idx_price_lists_status ON _atlas.price_lists(status);
CREATE INDEX IF NOT EXISTS idx_price_lists_type ON _atlas.price_lists(list_type);
CREATE INDEX IF NOT EXISTS idx_price_list_lines_list ON _atlas.price_list_lines(price_list_id);
CREATE INDEX IF NOT EXISTS idx_price_list_lines_item ON _atlas.price_list_lines(item_code);
CREATE INDEX IF NOT EXISTS idx_price_tiers_line ON _atlas.price_tiers(price_list_line_id);
CREATE INDEX IF NOT EXISTS idx_discount_rules_org ON _atlas.discount_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_discount_rules_status ON _atlas.discount_rules(status);
CREATE INDEX IF NOT EXISTS idx_charge_definitions_org ON _atlas.charge_definitions(organization_id);
CREATE INDEX IF NOT EXISTS idx_charge_definitions_type ON _atlas.charge_definitions(charge_type);
CREATE INDEX IF NOT EXISTS idx_pricing_strategies_org ON _atlas.pricing_strategies(organization_id);
CREATE INDEX IF NOT EXISTS idx_price_calc_logs_entity ON _atlas.price_calculation_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_price_calc_logs_date ON _atlas.price_calculation_logs(calculation_date);
