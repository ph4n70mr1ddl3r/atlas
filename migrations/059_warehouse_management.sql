-- ═══════════════════════════════════════════════════════════════════════════════
-- Warehouse Management (Oracle Fusion Cloud Warehouse Management)
-- ═══════════════════════════════════════════════════════════════════════════════

-- Warehouses
CREATE TABLE IF NOT EXISTS _atlas.warehouses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    location_code VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Warehouse Zones
CREATE TABLE IF NOT EXISTS _atlas.warehouse_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    warehouse_id UUID NOT NULL REFERENCES _atlas.warehouses(id) ON DELETE CASCADE,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    zone_type VARCHAR(50) NOT NULL DEFAULT 'storage',
    description TEXT,
    aisle_count INTEGER,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(warehouse_id, code)
);

-- Put-Away Rules
CREATE TABLE IF NOT EXISTS _atlas.put_away_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    warehouse_id UUID NOT NULL REFERENCES _atlas.warehouses(id) ON DELETE CASCADE,
    rule_name VARCHAR(200) NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 10,
    item_category VARCHAR(200),
    target_zone_type VARCHAR(50) NOT NULL DEFAULT 'storage',
    strategy VARCHAR(50) NOT NULL DEFAULT 'closest',
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Pick Waves (created before warehouse_tasks due to FK reference)
CREATE TABLE IF NOT EXISTS _atlas.pick_waves (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    warehouse_id UUID NOT NULL REFERENCES _atlas.warehouses(id),
    wave_number VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    cut_off_date DATE,
    shipping_method VARCHAR(100),
    total_tasks INTEGER NOT NULL DEFAULT 0,
    completed_tasks INTEGER NOT NULL DEFAULT 0,
    released_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, wave_number)
);

-- Warehouse Tasks
CREATE TABLE IF NOT EXISTS _atlas.warehouse_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    warehouse_id UUID NOT NULL REFERENCES _atlas.warehouses(id),
    task_number VARCHAR(100) NOT NULL,
    task_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    source_document VARCHAR(100),
    source_document_id UUID,
    source_line_id UUID,
    item_id UUID,
    item_description TEXT,
    from_zone_id UUID REFERENCES _atlas.warehouse_zones(id),
    to_zone_id UUID REFERENCES _atlas.warehouse_zones(id),
    from_location VARCHAR(200),
    to_location VARCHAR(200),
    quantity TEXT,
    uom VARCHAR(20),
    assigned_to UUID,
    wave_id UUID REFERENCES _atlas.pick_waves(id),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, task_number)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_warehouse_tasks_status ON _atlas.warehouse_tasks(status);
CREATE INDEX IF NOT EXISTS idx_warehouse_tasks_warehouse ON _atlas.warehouse_tasks(warehouse_id);
CREATE INDEX IF NOT EXISTS idx_warehouse_tasks_type ON _atlas.warehouse_tasks(task_type);
CREATE INDEX IF NOT EXISTS idx_warehouse_tasks_wave ON _atlas.warehouse_tasks(wave_id);
CREATE INDEX IF NOT EXISTS idx_pick_waves_status ON _atlas.pick_waves(status);
CREATE INDEX IF NOT EXISTS idx_warehouse_zones_warehouse ON _atlas.warehouse_zones(warehouse_id);
CREATE INDEX IF NOT EXISTS idx_put_away_rules_warehouse ON _atlas.put_away_rules(warehouse_id);
