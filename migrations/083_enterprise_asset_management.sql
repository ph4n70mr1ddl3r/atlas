-- Enterprise Asset Management (eAM)
-- Oracle Fusion Cloud: Maintenance Management / Enterprise Asset Management
-- Provides: Physical asset definitions, work orders, preventive maintenance
--   schedules, material & labor tracking, maintenance KPIs.

-- ============================================================================
-- Asset Locations: where assets are physically located
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.asset_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    parent_location_id UUID REFERENCES _atlas.asset_locations(id),
    location_type VARCHAR(50) NOT NULL DEFAULT 'building',
    address TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_asset_locations_org ON _atlas.asset_locations(organization_id);

-- ============================================================================
-- Asset Definitions: physical assets (distinct from financial fixed assets)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.asset_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    asset_number VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    -- Asset grouping
    asset_group VARCHAR(100) NOT NULL DEFAULT 'general',
    -- Criticality: low, medium, high, critical
    asset_criticality VARCHAR(50) NOT NULL DEFAULT 'medium',
    -- Status: active, inactive, disposed, in_repair
    asset_status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- Location
    location_id UUID REFERENCES _atlas.asset_locations(id),
    location_name VARCHAR(300),
    -- Hierarchy
    parent_asset_id UUID REFERENCES _atlas.asset_definitions(id),
    -- Identification
    serial_number VARCHAR(200),
    manufacturer VARCHAR(300),
    model VARCHAR(300),
    -- Dates
    install_date DATE,
    warranty_expiry DATE,
    last_maintenance_date DATE,
    next_maintenance_date DATE,
    -- Meter readings: {type, value, unit, last_read_date}
    meter_reading JSONB DEFAULT '{}'::jsonb,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, asset_number)
);

CREATE INDEX IF NOT EXISTS idx_asset_definitions_org ON _atlas.asset_definitions(organization_id);
CREATE INDEX IF NOT EXISTS idx_asset_definitions_status ON _atlas.asset_definitions(asset_status);
CREATE INDEX IF NOT EXISTS idx_asset_definitions_group ON _atlas.asset_definitions(asset_group);
CREATE INDEX IF NOT EXISTS idx_asset_definitions_criticality ON _atlas.asset_definitions(asset_criticality);
CREATE INDEX IF NOT EXISTS idx_asset_definitions_location ON _atlas.asset_definitions(location_id);

-- ============================================================================
-- Work Orders: maintenance work orders
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.work_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    work_order_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    -- Type: corrective, preventive, emergency, inspection, project
    work_order_type VARCHAR(50) NOT NULL DEFAULT 'corrective',
    -- Priority: low, normal, high, urgent
    priority VARCHAR(50) NOT NULL DEFAULT 'normal',
    -- Status: draft, approved, in_progress, completed, closed, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- Asset reference
    asset_id UUID NOT NULL REFERENCES _atlas.asset_definitions(id),
    asset_number VARCHAR(100),
    asset_name VARCHAR(500),
    location_name VARCHAR(300),
    -- Assignment
    assigned_to UUID,
    assigned_to_name VARCHAR(300),
    -- Scheduling
    scheduled_start DATE,
    scheduled_end DATE,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    -- Estimates
    estimated_hours JSONB DEFAULT '{}'::jsonb,
    actual_hours JSONB DEFAULT '{}'::jsonb,
    estimated_cost VARCHAR(50) DEFAULT '0.00',
    actual_cost VARCHAR(50) DEFAULT '0.00',
    -- Downtime
    downtime_hours NUMERIC(10,2) DEFAULT 0,
    -- Failure analysis codes
    failure_code VARCHAR(100),
    cause_code VARCHAR(100),
    resolution_code VARCHAR(100),
    -- Materials: [{item, quantity, unit_cost}]
    materials JSONB DEFAULT '[]'::jsonb,
    -- Labor: [{person_id, name, hours, rate}]
    labor JSONB DEFAULT '[]'::jsonb,
    -- Completion
    completion_notes TEXT,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, work_order_number)
);

CREATE INDEX IF NOT EXISTS idx_work_orders_org ON _atlas.work_orders(organization_id);
CREATE INDEX IF NOT EXISTS idx_work_orders_status ON _atlas.work_orders(status);
CREATE INDEX IF NOT EXISTS idx_work_orders_type ON _atlas.work_orders(work_order_type);
CREATE INDEX IF NOT EXISTS idx_work_orders_priority ON _atlas.work_orders(priority);
CREATE INDEX IF NOT EXISTS idx_work_orders_asset ON _atlas.work_orders(asset_id);
CREATE INDEX IF NOT EXISTS idx_work_orders_assigned ON _atlas.work_orders(assigned_to);
CREATE INDEX IF NOT EXISTS idx_work_orders_scheduled ON _atlas.work_orders(scheduled_start);

-- ============================================================================
-- Preventive Maintenance Schedules
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.preventive_maintenance_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_number VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    -- Asset reference
    asset_id UUID NOT NULL REFERENCES _atlas.asset_definitions(id),
    asset_number VARCHAR(100),
    asset_name VARCHAR(500),
    -- Schedule type: time_based, meter_based, condition_based
    schedule_type VARCHAR(50) NOT NULL DEFAULT 'time_based',
    -- Frequency: daily, weekly, monthly, quarterly, semi_annual, annual
    frequency VARCHAR(50) NOT NULL DEFAULT 'monthly',
    -- Interval: every N units
    interval_value INT NOT NULL DEFAULT 1,
    interval_unit VARCHAR(50) NOT NULL DEFAULT 'months',
    -- Meter-based fields
    meter_type VARCHAR(50),
    meter_threshold JSONB,
    -- Work order template: {title, type, description_template, estimated_hours, materials, labor}
    work_order_template JSONB,
    -- Estimates
    estimated_duration_hours NUMERIC(10,2) DEFAULT 0,
    estimated_cost VARCHAR(50) DEFAULT '0.00',
    -- Tracking
    next_due_date DATE,
    last_completed_date DATE,
    last_completed_wo VARCHAR(100),
    -- Auto-generation
    auto_generate BOOLEAN NOT NULL DEFAULT false,
    lead_time_days INT NOT NULL DEFAULT 7,
    -- Status: active, inactive, completed
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- Effectivity
    effective_start DATE,
    effective_end DATE,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, schedule_number)
);

CREATE INDEX IF NOT EXISTS idx_pm_schedules_org ON _atlas.preventive_maintenance_schedules(organization_id);
CREATE INDEX IF NOT EXISTS idx_pm_schedules_asset ON _atlas.preventive_maintenance_schedules(asset_id);
CREATE INDEX IF NOT EXISTS idx_pm_schedules_status ON _atlas.preventive_maintenance_schedules(status);
CREATE INDEX IF NOT EXISTS idx_pm_schedules_next_due ON _atlas.preventive_maintenance_schedules(next_due_date);

-- ============================================================================
-- Maintenance Dashboard Summary
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.maintenance_dashboard (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    total_assets INT DEFAULT 0,
    active_assets INT DEFAULT 0,
    assets_in_repair INT DEFAULT 0,
    critical_assets INT DEFAULT 0,
    total_work_orders INT DEFAULT 0,
    open_work_orders INT DEFAULT 0,
    in_progress_work_orders INT DEFAULT 0,
    completed_work_orders INT DEFAULT 0,
    overdue_work_orders INT DEFAULT 0,
    emergency_work_orders INT DEFAULT 0,
    preventive_work_orders INT DEFAULT 0,
    corrective_work_orders INT DEFAULT 0,
    total_schedules INT DEFAULT 0,
    active_schedules INT DEFAULT 0,
    overdue_schedules INT DEFAULT 0,
    avg_completion_days NUMERIC(10,2) DEFAULT 0,
    total_maintenance_cost VARCHAR(50) DEFAULT '0.00',
    total_downtime_hours NUMERIC(10,2) DEFAULT 0,
    mtbf_hours NUMERIC(10,2) DEFAULT 0,
    mttr_hours NUMERIC(10,2) DEFAULT 0,
    work_orders_by_priority JSONB DEFAULT '{}'::jsonb,
    work_orders_by_type JSONB DEFAULT '{}'::jsonb,
    assets_by_criticality JSONB DEFAULT '{}'::jsonb,
    costs_by_month JSONB DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id)
);
