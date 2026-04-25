-- 068: Demand Planning / Demand Management
-- Oracle Fusion SCM: Demand Management
-- Manages demand schedules (forecasts), schedule lines, historical demand,
-- forecast consumption, and accuracy measurement.

BEGIN;

-- ============================================================================
-- Demand forecast methods
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.demand_forecast_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    method_type VARCHAR(50) NOT NULL,  -- 'moving_average', 'exponential_smoothing', 'weighted_average', 'regression', 'manual'
    parameters JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Demand schedules (forecasts)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.demand_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    method_id UUID REFERENCES _atlas.demand_forecast_methods(id),
    method_name VARCHAR(200),
    schedule_type VARCHAR(50) NOT NULL DEFAULT 'monthly', -- 'daily', 'weekly', 'monthly', 'quarterly'
    status VARCHAR(50) NOT NULL DEFAULT 'draft',  -- 'draft', 'submitted', 'approved', 'active', 'closed', 'cancelled'
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    currency_code VARCHAR(3) DEFAULT 'USD',
    total_forecast_quantity DOUBLE PRECISION DEFAULT 0,
    total_forecast_value DOUBLE PRECISION DEFAULT 0,
    confidence_level VARCHAR(20) DEFAULT 'medium', -- 'low', 'medium', 'high'
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    owner_id UUID,
    owner_name VARCHAR(200),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, schedule_number)
);

-- ============================================================================
-- Demand schedule lines (forecast items per period)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.demand_schedule_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_id UUID NOT NULL REFERENCES _atlas.demand_schedules(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_code VARCHAR(100) NOT NULL,
    item_name VARCHAR(200),
    item_category VARCHAR(200),
    warehouse_code VARCHAR(100),
    region VARCHAR(100),
    customer_group VARCHAR(200),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    forecast_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    forecast_value DOUBLE PRECISION DEFAULT 0,
    unit_price DOUBLE PRECISION DEFAULT 0,
    consumed_quantity DOUBLE PRECISION DEFAULT 0,
    remaining_quantity DOUBLE PRECISION DEFAULT 0,
    confidence_pct DOUBLE PRECISION DEFAULT 0,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_demand_schedule_lines_schedule ON _atlas.demand_schedule_lines(schedule_id);
CREATE INDEX IF NOT EXISTS idx_demand_schedule_lines_item ON _atlas.demand_schedule_lines(organization_id, item_code);
CREATE INDEX IF NOT EXISTS idx_demand_schedule_lines_period ON _atlas.demand_schedule_lines(period_start, period_end);

-- ============================================================================
-- Historical demand data (actuals)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.demand_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    item_code VARCHAR(100) NOT NULL,
    item_name VARCHAR(200),
    warehouse_code VARCHAR(100),
    region VARCHAR(100),
    customer_group VARCHAR(200),
    actual_date DATE NOT NULL,
    actual_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    actual_value DOUBLE PRECISION DEFAULT 0,
    source_type VARCHAR(50) DEFAULT 'sales_order', -- 'sales_order', 'manual', 'shipment', 'import'
    source_id UUID,
    source_line_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_demand_history_item ON _atlas.demand_history(organization_id, item_code);
CREATE INDEX IF NOT EXISTS idx_demand_history_date ON _atlas.demand_history(actual_date);

-- ============================================================================
-- Forecast consumption (linking actuals to forecast lines)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.demand_consumption (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_line_id UUID NOT NULL REFERENCES _atlas.demand_schedule_lines(id) ON DELETE CASCADE,
    history_id UUID REFERENCES _atlas.demand_history(id),
    consumed_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    consumed_date DATE NOT NULL,
    source_type VARCHAR(50) DEFAULT 'auto', -- 'auto', 'manual'
    notes TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_demand_consumption_line ON _atlas.demand_consumption(schedule_line_id);

-- ============================================================================
-- Forecast accuracy measurements
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.demand_accuracy (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_id UUID NOT NULL REFERENCES _atlas.demand_schedules(id) ON DELETE CASCADE,
    schedule_line_id UUID REFERENCES _atlas.demand_schedule_lines(id) ON DELETE CASCADE,
    item_code VARCHAR(100) NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    forecast_quantity DOUBLE PRECISION NOT NULL,
    actual_quantity DOUBLE PRECISION NOT NULL,
    absolute_error DOUBLE PRECISION NOT NULL,
    absolute_pct_error DOUBLE PRECISION NOT NULL,
    bias DOUBLE PRECISION NOT NULL,
    measurement_date DATE NOT NULL DEFAULT CURRENT_DATE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_demand_accuracy_schedule ON _atlas.demand_accuracy(schedule_id);
CREATE INDEX IF NOT EXISTS idx_demand_accuracy_item ON _atlas.demand_accuracy(organization_id, item_code);

COMMIT;
