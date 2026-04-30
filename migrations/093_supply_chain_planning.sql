-- 093_supply_chain_planning.sql
-- Oracle Fusion Supply Chain Planning / Material Requirements Planning (MRP)
--
-- Provides planning scenarios, planned orders, supply/demand matching,
-- planning parameters, and exception management.
--
-- Oracle Fusion equivalent: Supply Chain Management > Supply Chain Planning

BEGIN;
CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Planning Scenarios: Define planning run contexts
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.planning_scenarios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    scenario_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    scenario_type VARCHAR(30) NOT NULL DEFAULT 'mrp',
        -- mrp, distribution_planning, demand_planning, production_planning
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
        -- draft, running, completed, error, cancelled
    planning_horizon_days INT NOT NULL DEFAULT 90,
    planning_start_date DATE,
    planning_end_date DATE,
    include_existing_supply BOOLEAN NOT NULL DEFAULT true,
    include_on_hand BOOLEAN NOT NULL DEFAULT true,
    include_work_in_progress BOOLEAN NOT NULL DEFAULT true,
    auto_firm BOOLEAN NOT NULL DEFAULT false,
    auto_firm_days INT,
    net_shortages_only BOOLEAN NOT NULL DEFAULT false,
    total_planned_orders INT NOT NULL DEFAULT 0,
    total_exceptions INT NOT NULL DEFAULT 0,
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, scenario_number)
);

-- ============================================================================
-- Planning Parameters: Item-level planning attributes
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.planning_parameters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    item_id UUID NOT NULL,
    item_name VARCHAR(200),
    item_number VARCHAR(100),
    planner_code VARCHAR(50),
    planning_method VARCHAR(30) NOT NULL DEFAULT 'mrp',
        -- mrp, min_max, reorder_point, kanban, not_planned
    make_buy VARCHAR(10) NOT NULL DEFAULT 'buy',
        -- make, buy
    lead_time_days INT NOT NULL DEFAULT 0,
    safety_stock_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    min_order_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_order_quantity DOUBLE PRECISION,
    fixed_order_quantity DOUBLE PRECISION,
    fixed_lot_multiplier DOUBLE PRECISION NOT NULL DEFAULT 1,
    order_multiple DOUBLE PRECISION NOT NULL DEFAULT 1,
    planning_time_fence_days INT NOT NULL DEFAULT 0,
    release_time_fence_days INT NOT NULL DEFAULT 0,
    shrinkage_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    lot_size_policy VARCHAR(30) NOT NULL DEFAULT 'fixed_quantity',
        -- fixed_quantity, lot_for_lot, period_order_quantity, min_max
    period_order_quantity_days INT,
    source_type VARCHAR(30),
        -- internal, supplier, transfer
    source_id UUID,
    source_name VARCHAR(200),
    default_supplier_id UUID,
    default_supplier_name VARCHAR(200),
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, item_id)
);

-- ============================================================================
-- Supply/Demand Records: Netting inputs for planning
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.supply_demand_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    scenario_id UUID REFERENCES _atlas.planning_scenarios(id),
    item_id UUID NOT NULL,
    item_name VARCHAR(200),
    item_number VARCHAR(100),
    entry_type VARCHAR(10) NOT NULL,
        -- supply, demand
    source_type VARCHAR(50) NOT NULL,
        -- on_hand, purchase_order, work_order, transfer_order, sales_order, forecast, safety_stock
    source_id UUID,
    source_number VARCHAR(100),
    quantity DOUBLE PRECISION NOT NULL,
    quantity_remaining DOUBLE PRECISION NOT NULL DEFAULT 0,
    due_date DATE NOT NULL,
    priority INT NOT NULL DEFAULT 5,
    status VARCHAR(20) NOT NULL DEFAULT 'open',
        -- open, reserved, cancelled, closed
    pegged_to_id UUID,
        -- references another supply_demand_entries.id (pegging)
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sde_scenario_item ON _atlas.supply_demand_entries(scenario_id, item_id);
CREATE INDEX IF NOT EXISTS idx_sde_type ON _atlas.supply_demand_entries(entry_type);

-- ============================================================================
-- Planned Orders: Output of MRP planning run
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.planned_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    scenario_id UUID REFERENCES _atlas.planning_scenarios(id),
    item_id UUID NOT NULL,
    item_name VARCHAR(200),
    item_number VARCHAR(100),
    order_number VARCHAR(50) NOT NULL,
    order_type VARCHAR(30) NOT NULL DEFAULT 'buy',
        -- buy (purchase), make (manufacture), transfer
    status VARCHAR(20) NOT NULL DEFAULT 'unfirm',
        -- unfirm, firmed, released, cancelled, completed
    quantity DOUBLE PRECISION NOT NULL,
    quantity_firmed DOUBLE PRECISION NOT NULL DEFAULT 0,
    due_date DATE NOT NULL,
    start_date DATE,
    need_date DATE,
    planner_notes TEXT,
    planning_priority INT NOT NULL DEFAULT 5,
    order_action VARCHAR(20) NOT NULL DEFAULT 'new',
        -- new, reschedule_in, reschedule_out, cancel, expedite
    suggested_supplier_id UUID,
    suggested_supplier_name VARCHAR(200),
    suggested_source_type VARCHAR(30),
    suggested_source_id UUID,
    firm_deadline DATE,
    pegging_demand_id UUID,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_po_scenario ON _atlas.planned_orders(scenario_id);
CREATE INDEX IF NOT EXISTS idx_po_status ON _atlas.planned_orders(status);

-- ============================================================================
-- Planning Exceptions: Issues flagged during planning run
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.planning_exceptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    scenario_id UUID REFERENCES _atlas.planning_scenarios(id),
    item_id UUID NOT NULL,
    item_name VARCHAR(200),
    item_number VARCHAR(100),
    exception_type VARCHAR(50) NOT NULL,
        -- late_order, early_order, excess_supply, shortage,
        -- past_due_demand, order_past_due, over_planned, under_planned,
        -- cancel_suggestion, reschedule_suggestion
    severity VARCHAR(10) NOT NULL DEFAULT 'warning',
        -- info, warning, error, critical
    message TEXT NOT NULL,
    source_type VARCHAR(50),
    source_id UUID,
    source_number VARCHAR(100),
    affected_quantity DOUBLE PRECISION,
    affected_date DATE,
    resolution_status VARCHAR(20) NOT NULL DEFAULT 'open',
        -- open, acknowledged, resolved, dismissed
    resolution_notes TEXT,
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pe_scenario ON _atlas.planning_exceptions(scenario_id);
CREATE INDEX IF NOT EXISTS idx_pe_severity ON _atlas.planning_exceptions(severity);
CREATE INDEX IF NOT EXISTS idx_pe_resolution ON _atlas.planning_exceptions(resolution_status);

COMMIT;
