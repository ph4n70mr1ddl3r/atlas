-- 092_cost_accounting.sql
-- Oracle Fusion Cost Accounting / Cost Management
--
-- Provides product costing with standard, average, FIFO, and LIFO methods.
-- Includes cost books, cost elements, cost profiles, standard costs,
-- cost adjustments, and variance analysis.

BEGIN;
CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Cost Books: Define costing methods per organization
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.cost_books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    costing_method VARCHAR(30) NOT NULL DEFAULT 'standard',
        -- standard, average, fifo, lifo
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    is_active BOOLEAN NOT NULL DEFAULT true,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
        -- active, inactive
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Cost Elements: Material, Labor, Overhead, Subcontracting, Expense
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.cost_elements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    element_type VARCHAR(30) NOT NULL,
        -- material, labor, overhead, subcontracting, expense
    cost_book_id UUID REFERENCES _atlas.cost_books(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    default_rate NUMERIC(18,6) DEFAULT 0,
    rate_uom VARCHAR(10),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Cost Profiles: Item-level costing configuration
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.cost_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    cost_book_id UUID NOT NULL REFERENCES _atlas.cost_books(id),
    item_id UUID,
    item_name VARCHAR(200),
    cost_type VARCHAR(30) NOT NULL DEFAULT 'standard',
        -- standard, average, fifo, lifo
    lot_level_costing BOOLEAN DEFAULT false,
    include_landed_costs BOOLEAN DEFAULT true,
    overhead_absorption_method VARCHAR(30) DEFAULT 'rate',
        -- rate, amount, percentage
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Standard Costs: Standard cost per item per cost book per element
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.standard_costs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    cost_book_id UUID NOT NULL REFERENCES _atlas.cost_books(id),
    cost_profile_id UUID REFERENCES _atlas.cost_profiles(id),
    cost_element_id UUID NOT NULL REFERENCES _atlas.cost_elements(id),
    item_id UUID NOT NULL,
    item_name VARCHAR(200),
    standard_cost NUMERIC(18,6) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    effective_date DATE NOT NULL DEFAULT CURRENT_DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
        -- active, pending, superseded
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_std_costs_item ON _atlas.standard_costs(item_id);
CREATE INDEX IF NOT EXISTS idx_std_costs_book ON _atlas.standard_books_cost ON _atlas.standard_costs(cost_book_id);

-- ============================================================================
-- Cost Adjustments: Cost corrections and updates
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.cost_adjustments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    adjustment_number VARCHAR(50) NOT NULL,
    cost_book_id UUID NOT NULL REFERENCES _atlas.cost_books(id),
    adjustment_type VARCHAR(30) NOT NULL,
        -- standard_cost_update, cost_correction, revaluation, overhead_adjustment
    description TEXT,
    reason TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
        -- draft, submitted, approved, rejected, posted
    total_adjustment_amount NUMERIC(18,6) DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    effective_date DATE,
    posted_at TIMESTAMPTZ,
    posted_by UUID,
    approved_by UUID,
    rejected_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, adjustment_number)
);

-- Cost adjustment lines
CREATE TABLE IF NOT EXISTS _atlas.cost_adjustment_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    adjustment_id UUID NOT NULL REFERENCES _atlas.cost_adjustments(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_id UUID NOT NULL,
    item_name VARCHAR(200),
    cost_element_id UUID REFERENCES _atlas.cost_elements(id),
    old_cost NUMERIC(18,6) NOT NULL DEFAULT 0,
    new_cost NUMERIC(18,6) NOT NULL DEFAULT 0,
    adjustment_amount NUMERIC(18,6) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    effective_date DATE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================================
-- Variance Analysis: Track cost variances (PPV, routing, overhead, rate)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.cost_variances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    cost_book_id UUID NOT NULL REFERENCES _atlas.cost_books(id),
    variance_type VARCHAR(30) NOT NULL,
        -- purchase_price, routing, overhead, rate, usage, mix
    variance_date DATE NOT NULL DEFAULT CURRENT_DATE,
    item_id UUID NOT NULL,
    item_name VARCHAR(200),
    cost_element_id UUID REFERENCES _atlas.cost_elements(id),
    source_type VARCHAR(30),
        -- purchase_order, work_order, transfer_order
    source_id UUID,
    source_number VARCHAR(50),
    standard_cost NUMERIC(18,6) NOT NULL DEFAULT 0,
    actual_cost NUMERIC(18,6) NOT NULL DEFAULT 0,
    variance_amount NUMERIC(18,6) NOT NULL DEFAULT 0,
    variance_percent NUMERIC(8,4) DEFAULT 0,
    quantity NUMERIC(18,6) DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    accounting_period VARCHAR(20),
    is_analyzed BOOLEAN DEFAULT false,
    analysis_notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_cost_variances_item ON _atlas.cost_variances(item_id);
CREATE INDEX IF NOT EXISTS idx_cost_variances_type ON _atlas.cost_variances(variance_type);
CREATE INDEX IF NOT EXISTS idx_cost_variances_date ON _atlas.cost_variances(variance_date);

COMMIT;
