-- Product Configurator (Configure-to-Order)
-- Oracle Fusion Cloud SCM > Product Management > Configurator
-- Provides: configurable product models, features, options, rules, and configuration instances

-- Configuration Models (top-level configurable product definitions)
CREATE TABLE IF NOT EXISTS _atlas.config_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    model_number VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    base_product_id UUID,
    base_product_number VARCHAR(50),
    base_product_name VARCHAR(300),
    model_type VARCHAR(30) NOT NULL DEFAULT 'standard', -- standard, kit, bundle
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, active, inactive, obsolete
    version INT NOT NULL DEFAULT 1,
    effective_from DATE,
    effective_to DATE,
    default_config JSONB DEFAULT '{}',
    validation_mode VARCHAR(20) DEFAULT 'strict', -- strict, relaxed, none
    ui_layout JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, model_number)
);

-- Configuration Features (groups of choices, e.g. "Color", "Size", "Engine Type")
CREATE TABLE IF NOT EXISTS _atlas.config_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    model_id UUID NOT NULL REFERENCES _atlas.config_models(id) ON DELETE CASCADE,
    feature_code VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    feature_type VARCHAR(30) NOT NULL DEFAULT 'single_select', -- single_select, multi_select, numeric, text, boolean
    is_required BOOLEAN NOT NULL DEFAULT false,
    display_order INT NOT NULL DEFAULT 0,
    ui_hints JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(model_id, feature_code)
);

-- Configuration Options (choices within features, e.g. "Red", "Blue" for Color)
CREATE TABLE IF NOT EXISTS _atlas.config_options (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    feature_id UUID NOT NULL REFERENCES _atlas.config_features(id) ON DELETE CASCADE,
    option_code VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    option_type VARCHAR(30) DEFAULT 'standard', -- standard, default, recommended
    price_adjustment NUMERIC(18,2) DEFAULT 0,
    cost_adjustment NUMERIC(18,2) DEFAULT 0,
    lead_time_days INT DEFAULT 0,
    is_default BOOLEAN DEFAULT false,
    is_available BOOLEAN DEFAULT true,
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(feature_id, option_code)
);

-- Configuration Rules (constraints between options)
CREATE TABLE IF NOT EXISTS _atlas.config_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    model_id UUID NOT NULL REFERENCES _atlas.config_models(id) ON DELETE CASCADE,
    rule_code VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    rule_type VARCHAR(30) NOT NULL, -- compatibility, incompatibility, default, requirement, exclusion
    source_feature_id UUID,
    source_option_id UUID,
    target_feature_id UUID,
    target_option_id UUID,
    condition_expression TEXT,
    severity VARCHAR(20) DEFAULT 'error', -- error, warning, info
    is_active BOOLEAN DEFAULT true,
    priority INT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(model_id, rule_code)
);

-- Configuration Instances (actual configurations created from models)
CREATE TABLE IF NOT EXISTS _atlas.config_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    instance_number VARCHAR(50) NOT NULL,
    model_id UUID NOT NULL REFERENCES _atlas.config_models(id),
    model_number VARCHAR(50),
    name VARCHAR(300),
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, valid, invalid, submitted, approved, ordered, cancelled
    selections JSONB NOT NULL DEFAULT '{}',
    validation_errors JSONB DEFAULT '[]',
    validation_warnings JSONB DEFAULT '[]',
    base_price NUMERIC(18,2) DEFAULT 0,
    total_price NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    config_hash VARCHAR(64),
    effective_date DATE,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    sales_order_id UUID,
    sales_order_number VARCHAR(50),
    sales_order_line INT,
    configured_by UUID,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, instance_number)
);

-- Create indexes
CREATE INDEX idx_config_models_org ON _atlas.config_models(organization_id);
CREATE INDEX idx_config_models_status ON _atlas.config_models(status);
CREATE INDEX idx_config_features_model ON _atlas.config_features(model_id);
CREATE INDEX idx_config_options_feature ON _atlas.config_options(feature_id);
CREATE INDEX idx_config_rules_model ON _atlas.config_rules(model_id);
CREATE INDEX idx_config_rules_type ON _atlas.config_rules(rule_type);
CREATE INDEX idx_config_instances_org ON _atlas.config_instances(organization_id);
CREATE INDEX idx_config_instances_model ON _atlas.config_instances(model_id);
CREATE INDEX idx_config_instances_status ON _atlas.config_instances(status);
CREATE INDEX idx_config_instances_order ON _atlas.config_instances(sales_order_id);
