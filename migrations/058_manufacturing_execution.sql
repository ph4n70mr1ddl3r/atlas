-- Manufacturing Execution (Oracle Fusion SCM > Manufacturing)
-- Work definitions (BOMs + routings), work orders, operations,
-- material requirements, and production completion tracking.

-- ======================================================================
-- Work Definitions (BOM + Routing templates)
-- ======================================================================
CREATE TABLE IF NOT EXISTS _atlas.work_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_number VARCHAR(50) NOT NULL,
    description TEXT,
    item_id UUID,                          -- the finished good being manufactured
    item_code VARCHAR(100),
    item_description TEXT,
    version INT NOT NULL DEFAULT 1,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',  -- draft, active, inactive, obsolete
    -- Manufacturing details
    production_type VARCHAR(30) NOT NULL DEFAULT 'discrete',  -- discrete, process, repetitive
    planning_type VARCHAR(30) NOT NULL DEFAULT 'make_to_order',  -- make_to_order, make_to_stock, engineer_to_order
    standard_lot_size NUMERIC(18,4) DEFAULT 1,
    unit_of_measure VARCHAR(30) DEFAULT 'EA',
    lead_time_days INT DEFAULT 0,
    -- Accounting
    cost_type VARCHAR(30) DEFAULT 'standard',  -- standard, actual, average
    standard_cost NUMERIC(18,4) DEFAULT 0,
    -- Tracking
    effective_from DATE,
    effective_to DATE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, definition_number)
);

-- ======================================================================
-- Work Definition Components (Bill of Materials)
-- ======================================================================
CREATE TABLE IF NOT EXISTS _atlas.work_definition_components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    work_definition_id UUID NOT NULL REFERENCES _atlas.work_definitions(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    -- Component item
    component_item_id UUID,
    component_item_code VARCHAR(100) NOT NULL,
    component_item_description TEXT,
    -- Quantity
    quantity_required NUMERIC(18,4) NOT NULL,
    unit_of_measure VARCHAR(30) DEFAULT 'EA',
    -- BOM details
    component_type VARCHAR(30) NOT NULL DEFAULT 'material',  -- material, phantom, by_product, co_product
    scrap_percent NUMERIC(5,2) DEFAULT 0,
    yield_percent NUMERIC(5,2) DEFAULT 100,
    -- Supply
    supply_type VARCHAR(30) DEFAULT 'push',  -- push, pull, bulk
    supply_subinventory VARCHAR(100),
    wip_supply_type VARCHAR(30) DEFAULT 'component_issue',  -- component_issue, assembly_pull, operation_pull, bulk
    -- Effectivity
    operation_sequence INT,  -- which operation consumes this component
    effective_from DATE,
    effective_to DATE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ======================================================================
-- Work Definition Operations (Routing)
-- ======================================================================
CREATE TABLE IF NOT EXISTS _atlas.work_definition_operations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    work_definition_id UUID NOT NULL REFERENCES _atlas.work_definitions(id) ON DELETE CASCADE,
    operation_sequence INT NOT NULL,
    operation_name VARCHAR(200) NOT NULL,
    operation_description TEXT,
    -- Work center
    work_center_code VARCHAR(100),
    work_center_name VARCHAR(200),
    department_code VARCHAR(100),
    -- Timing
    setup_hours NUMERIC(10,2) DEFAULT 0,
    run_time_hours NUMERIC(10,2) DEFAULT 0,
    run_time_unit VARCHAR(30) DEFAULT 'hour',  -- hour, minute
    units_per_run NUMERIC(18,4) DEFAULT 1,
    -- Resources
    resource_code VARCHAR(100),
    resource_type VARCHAR(30) DEFAULT 'machine',  -- machine, labor, both
    resource_count INT DEFAULT 1,
    -- Costing
    standard_labor_cost NUMERIC(18,4) DEFAULT 0,
    standard_overhead_cost NUMERIC(18,4) DEFAULT 0,
    standard_machine_cost NUMERIC(18,4) DEFAULT 0,
    -- Operation details
    operation_type VARCHAR(30) DEFAULT 'standard',  -- standard, inspection, rework
    backflush_enabled BOOLEAN DEFAULT false,
    count_point_type VARCHAR(30) DEFAULT 'manual',  -- manual, automatic
    -- Yield & Scrap
    yield_percent NUMERIC(5,2) DEFAULT 100,
    scrap_percent NUMERIC(5,2) DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ======================================================================
-- Work Orders (Production Orders)
-- ======================================================================
CREATE TABLE IF NOT EXISTS _atlas.work_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    work_order_number VARCHAR(50) NOT NULL,
    description TEXT,
    -- Reference to work definition
    work_definition_id UUID REFERENCES _atlas.work_definitions(id),
    -- Product
    item_id UUID,
    item_code VARCHAR(100),
    item_description TEXT,
    -- Quantities
    quantity_ordered NUMERIC(18,4) NOT NULL,
    quantity_completed NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_scrapped NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_in_queue NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_running NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_rejected NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_of_measure VARCHAR(30) DEFAULT 'EA',
    -- Dates
    scheduled_start_date DATE,
    scheduled_completion_date DATE,
    actual_start_date DATE,
    actual_completion_date DATE,
    due_date DATE,
    -- Status lifecycle: draft → released → started → completed → closed (or cancelled)
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    -- Priority
    priority VARCHAR(30) NOT NULL DEFAULT 'normal',  -- low, normal, high, urgent
    -- Manufacturing
    production_line VARCHAR(100),
    work_center_code VARCHAR(100),
    warehouse_code VARCHAR(100),
    -- Accounting
    cost_type VARCHAR(30) DEFAULT 'standard',
    estimated_material_cost NUMERIC(18,4) DEFAULT 0,
    estimated_labor_cost NUMERIC(18,4) DEFAULT 0,
    estimated_overhead_cost NUMERIC(18,4) DEFAULT 0,
    estimated_total_cost NUMERIC(18,4) DEFAULT 0,
    actual_material_cost NUMERIC(18,4) DEFAULT 0,
    actual_labor_cost NUMERIC(18,4) DEFAULT 0,
    actual_overhead_cost NUMERIC(18,4) DEFAULT 0,
    actual_total_cost NUMERIC(18,4) DEFAULT 0,
    -- Source
    source_type VARCHAR(30),           -- manual, sales_order, mrp, min_max
    source_document_number VARCHAR(100),
    source_document_line_id UUID,
    -- Tracking
    firm_planned BOOLEAN DEFAULT false,
    company_id UUID,
    plant_code VARCHAR(100),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    submitted_at TIMESTAMPTZ,
    released_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, work_order_number)
);

-- ======================================================================
-- Work Order Operations (tracking each operation on a work order)
-- ======================================================================
CREATE TABLE IF NOT EXISTS _atlas.work_order_operations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    work_order_id UUID NOT NULL REFERENCES _atlas.work_orders(id) ON DELETE CASCADE,
    operation_sequence INT NOT NULL,
    operation_name VARCHAR(200) NOT NULL,
    -- Work center
    work_center_code VARCHAR(100),
    work_center_name VARCHAR(200),
    department_code VARCHAR(100),
    -- Quantities
    quantity_in_queue NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_running NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_completed NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_rejected NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_scrapped NUMERIC(18,4) NOT NULL DEFAULT 0,
    -- Timing
    scheduled_start_date DATE,
    scheduled_completion_date DATE,
    actual_start_date DATE,
    actual_completion_date DATE,
    -- Actual time tracking
    actual_setup_hours NUMERIC(10,2) DEFAULT 0,
    actual_run_hours NUMERIC(10,2) DEFAULT 0,
    -- Resources
    resource_code VARCHAR(100),
    resource_type VARCHAR(30) DEFAULT 'machine',
    -- Status: pending, in_queue, running, completed, skipped, error
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    -- Costing
    actual_labor_cost NUMERIC(18,4) DEFAULT 0,
    actual_overhead_cost NUMERIC(18,4) DEFAULT 0,
    actual_machine_cost NUMERIC(18,4) DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ======================================================================
-- Work Order Material Requirements
-- ======================================================================
CREATE TABLE IF NOT EXISTS _atlas.work_order_materials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    work_order_id UUID NOT NULL REFERENCES _atlas.work_orders(id) ON DELETE CASCADE,
    operation_sequence INT,             -- which operation consumes this material
    -- Component
    component_item_id UUID,
    component_item_code VARCHAR(100) NOT NULL,
    component_item_description TEXT,
    -- Quantities
    quantity_required NUMERIC(18,4) NOT NULL,
    quantity_issued NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_returned NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_scrapped NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_of_measure VARCHAR(30) DEFAULT 'EA',
    -- Supply
    supply_type VARCHAR(30) DEFAULT 'push',
    supply_subinventory VARCHAR(100),
    wip_supply_type VARCHAR(30) DEFAULT 'component_issue',
    -- Status: pending, partially_issued, fully_issued, returned, short
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_work_definitions_org ON _atlas.work_definitions(organization_id);
CREATE INDEX IF NOT EXISTS idx_work_definitions_item ON _atlas.work_definitions(item_id);
CREATE INDEX IF NOT EXISTS idx_work_definitions_status ON _atlas.work_definitions(status);
CREATE INDEX IF NOT EXISTS idx_work_def_components_def ON _atlas.work_definition_components(work_definition_id);
CREATE INDEX IF NOT EXISTS idx_work_def_operations_def ON _atlas.work_definition_operations(work_definition_id);
CREATE INDEX IF NOT EXISTS idx_work_orders_org ON _atlas.work_orders(organization_id);
CREATE INDEX IF NOT EXISTS idx_work_orders_status ON _atlas.work_orders(status);
CREATE INDEX IF NOT EXISTS idx_work_orders_definition ON _atlas.work_orders(work_definition_id);
CREATE INDEX IF NOT EXISTS idx_work_order_operations_wo ON _atlas.work_order_operations(work_order_id);
CREATE INDEX IF NOT EXISTS idx_work_order_materials_wo ON _atlas.work_order_materials(work_order_id);
