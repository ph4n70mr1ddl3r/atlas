-- Inventory Management (Oracle Fusion SCM > Inventory Management)
-- Tables for inventory organizations, items, subinventories, locators,
-- on-hand quantities, and inventory transactions.

-- Inventory Organizations (warehouses, stores, distribution centers)
CREATE TABLE IF NOT EXISTS _atlas.inventory_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    org_type VARCHAR(50) NOT NULL DEFAULT 'warehouse',
    location_code VARCHAR(200),
    address JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    default_subinventory_code VARCHAR(100),
    default_currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    requires_approval_for_issues BOOLEAN NOT NULL DEFAULT false,
    requires_approval_for_transfers BOOLEAN NOT NULL DEFAULT true,
    enable_lot_control BOOLEAN NOT NULL DEFAULT false,
    enable_serial_control BOOLEAN NOT NULL DEFAULT false,
    enable_revision_control BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Item Categories
CREATE TABLE IF NOT EXISTS _atlas.item_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    parent_category_id UUID REFERENCES _atlas.item_categories(id),
    track_as_asset BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Items (products, materials, supplies)
CREATE TABLE IF NOT EXISTS _atlas.items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    item_code VARCHAR(200) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    long_description TEXT,
    category_id UUID REFERENCES _atlas.item_categories(id),
    category_code VARCHAR(100),
    item_type VARCHAR(50) NOT NULL DEFAULT 'inventory',
    uom VARCHAR(50) NOT NULL DEFAULT 'EA',
    secondary_uom VARCHAR(50),
    weight NUMERIC(18,4),
    weight_uom VARCHAR(50),
    volume NUMERIC(18,4),
    volume_uom VARCHAR(50),
    list_price NUMERIC(18,4) DEFAULT 0,
    standard_cost NUMERIC(18,4) DEFAULT 0,
    min_order_quantity NUMERIC(18,4),
    max_order_quantity NUMERIC(18,4),
    lead_time_days INT DEFAULT 0,
    shelf_life_days INT,
    is_lot_controlled BOOLEAN NOT NULL DEFAULT false,
    is_serial_controlled BOOLEAN NOT NULL DEFAULT false,
    is_revision_controlled BOOLEAN NOT NULL DEFAULT false,
    is_perishable BOOLEAN NOT NULL DEFAULT false,
    is_hazardous BOOLEAN NOT NULL DEFAULT false,
    is_purchasable BOOLEAN NOT NULL DEFAULT true,
    is_sellable BOOLEAN NOT NULL DEFAULT true,
    is_stockable BOOLEAN NOT NULL DEFAULT true,
    inventory_asset_account_code VARCHAR(100),
    expense_account_code VARCHAR(100),
    cost_of_goods_sold_account VARCHAR(100),
    revenue_account_code VARCHAR(100),
    image_url TEXT,
    barcode VARCHAR(200),
    supplier_item_codes JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, item_code)
);

-- Subinventories (logical storage areas within an org)
CREATE TABLE IF NOT EXISTS _atlas.subinventories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    inventory_org_id UUID NOT NULL REFERENCES _atlas.inventory_organizations(id),
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    subinventory_type VARCHAR(50) NOT NULL DEFAULT 'storage',
    asset_subinventory BOOLEAN NOT NULL DEFAULT true,
    quantity_tracked BOOLEAN NOT NULL DEFAULT true,
    location_code VARCHAR(200),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, inventory_org_id, code)
);

-- Locators (specific bin/shelf/row within a subinventory)
CREATE TABLE IF NOT EXISTS _atlas.locators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    subinventory_id UUID NOT NULL REFERENCES _atlas.subinventories(id),
    code VARCHAR(100) NOT NULL,
    description TEXT,
    picker_order INT DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(subinventory_id, code)
);

-- On-hand Balances (current stock quantities)
CREATE TABLE IF NOT EXISTS _atlas.on_hand_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    inventory_org_id UUID NOT NULL REFERENCES _atlas.inventory_organizations(id),
    item_id UUID NOT NULL REFERENCES _atlas.items(id),
    subinventory_id UUID NOT NULL REFERENCES _atlas.subinventories(id),
    locator_id UUID REFERENCES _atlas.locators(id),
    lot_number VARCHAR(200),
    serial_number VARCHAR(200),
    revision VARCHAR(100),
    quantity NUMERIC(18,4) NOT NULL DEFAULT 0,
    reserved_quantity NUMERIC(18,4) NOT NULL DEFAULT 0,
    available_quantity NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    total_value NUMERIC(18,2) NOT NULL DEFAULT 0,
    last_transaction_date TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, inventory_org_id, item_id, subinventory_id, locator_id, lot_number, serial_number, revision)
);

-- Inventory Transaction Types
CREATE TABLE IF NOT EXISTS _atlas.inventory_transaction_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    transaction_action VARCHAR(50) NOT NULL,
    source_type VARCHAR(50) NOT NULL DEFAULT 'manual',
    is_system BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Inventory Transactions (all material movements)
CREATE TABLE IF NOT EXISTS _atlas.inventory_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    transaction_number VARCHAR(50) NOT NULL,
    transaction_type_id UUID REFERENCES _atlas.inventory_transaction_types(id),
    transaction_type_code VARCHAR(100),
    transaction_action VARCHAR(50) NOT NULL,
    source_type VARCHAR(50) NOT NULL DEFAULT 'manual',
    source_id UUID,
    source_number VARCHAR(200),
    source_line_id UUID,
    item_id UUID NOT NULL REFERENCES _atlas.items(id),
    item_code VARCHAR(200),
    item_description VARCHAR(500),
    -- From location
    from_inventory_org_id UUID REFERENCES _atlas.inventory_organizations(id),
    from_subinventory_id UUID REFERENCES _atlas.subinventories(id),
    from_locator_id UUID REFERENCES _atlas.locators(id),
    -- To location
    to_inventory_org_id UUID REFERENCES _atlas.inventory_organizations(id),
    to_subinventory_id UUID REFERENCES _atlas.subinventories(id),
    to_locator_id UUID REFERENCES _atlas.locators(id),
    -- Quantities and costing
    quantity NUMERIC(18,4) NOT NULL,
    uom VARCHAR(50) NOT NULL DEFAULT 'EA',
    unit_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    total_cost NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Lot/Serial/Revision
    lot_number VARCHAR(200),
    serial_number VARCHAR(200),
    revision VARCHAR(100),
    -- Dates
    transaction_date TIMESTAMPTZ NOT NULL DEFAULT now(),
    accounting_date DATE,
    -- Reference info
    reason_id UUID,
    reason_name VARCHAR(200),
    reference VARCHAR(200),
    reference_type VARCHAR(100),
    notes TEXT,
    -- GL posting
    is_posted BOOLEAN NOT NULL DEFAULT false,
    posted_at TIMESTAMPTZ,
    journal_entry_id UUID,
    -- Workflow
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    -- Audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Cycle Count Headers
CREATE TABLE IF NOT EXISTS _atlas.cycle_count_headers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    count_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    inventory_org_id UUID NOT NULL REFERENCES _atlas.inventory_organizations(id),
    subinventory_id UUID REFERENCES _atlas.subinventories(id),
    count_date DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- ABC analysis settings
    count_method VARCHAR(50) NOT NULL DEFAULT 'full',
    tolerance_percent NUMERIC(8,4) DEFAULT 0,
    -- Summary
    total_items INT NOT NULL DEFAULT 0,
    counted_items INT NOT NULL DEFAULT 0,
    matched_items INT NOT NULL DEFAULT 0,
    mismatched_items INT NOT NULL DEFAULT 0,
    -- Approval
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    -- Audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, count_number)
);

-- Cycle Count Lines
CREATE TABLE IF NOT EXISTS _atlas.cycle_count_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    cycle_count_id UUID NOT NULL REFERENCES _atlas.cycle_count_headers(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_id UUID NOT NULL REFERENCES _atlas.items(id),
    item_code VARCHAR(200),
    item_description VARCHAR(500),
    subinventory_id UUID REFERENCES _atlas.subinventories(id),
    locator_id UUID REFERENCES _atlas.locators(id),
    lot_number VARCHAR(200),
    revision VARCHAR(100),
    -- System quantity
    system_quantity NUMERIC(18,4) NOT NULL DEFAULT 0,
    -- Counted quantities (multiple counts allowed)
    count_quantity_1 NUMERIC(18,4),
    count_quantity_2 NUMERIC(18,4),
    count_quantity_3 NUMERIC(18,4),
    count_date_1 TIMESTAMPTZ,
    count_date_2 TIMESTAMPTZ,
    count_date_3 TIMESTAMPTZ,
    counted_by_1 UUID,
    counted_by_2 UUID,
    counted_by_3 UUID,
    -- Final
    approved_quantity NUMERIC(18,4),
    variance_quantity NUMERIC(18,4),
    variance_percent NUMERIC(8,4),
    is_matched BOOLEAN NOT NULL DEFAULT false,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    adjustment_transaction_id UUID,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Transaction Reasons
CREATE TABLE IF NOT EXISTS _atlas.transaction_reasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    applicable_actions JSONB DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_inv_orgs_org ON _atlas.inventory_organizations(organization_id);
CREATE INDEX IF NOT EXISTS idx_item_categories_org ON _atlas.item_categories(organization_id);
CREATE INDEX IF NOT EXISTS idx_items_org ON _atlas.items(organization_id);
CREATE INDEX IF NOT EXISTS idx_items_category ON _atlas.items(category_id);
CREATE INDEX IF NOT EXISTS idx_items_code ON _atlas.items(item_code);
CREATE INDEX IF NOT EXISTS idx_subinventories_org ON _atlas.subinventories(inventory_org_id);
CREATE INDEX IF NOT EXISTS idx_locators_subinv ON _atlas.locators(subinventory_id);
CREATE INDEX IF NOT EXISTS idx_on_hand_item ON _atlas.on_hand_balances(item_id);
CREATE INDEX IF NOT EXISTS idx_on_hand_org ON _atlas.on_hand_balances(inventory_org_id);
CREATE INDEX IF NOT EXISTS idx_on_hand_subinv ON _atlas.on_hand_balances(subinventory_id);
CREATE INDEX IF NOT EXISTS idx_inv_txns_org ON _atlas.inventory_transactions(organization_id);
CREATE INDEX IF NOT EXISTS idx_inv_txns_item ON _atlas.inventory_transactions(item_id);
CREATE INDEX IF NOT EXISTS idx_inv_txns_date ON _atlas.inventory_transactions(transaction_date);
CREATE INDEX IF NOT EXISTS idx_inv_txns_type ON _atlas.inventory_transactions(transaction_type_code);
CREATE INDEX IF NOT EXISTS idx_cycle_count_hdr_org ON _atlas.cycle_count_headers(organization_id);
CREATE INDEX IF NOT EXISTS idx_cycle_count_lines_cc ON _atlas.cycle_count_lines(cycle_count_id);
CREATE INDEX IF NOT EXISTS idx_txn_reasons_org ON _atlas.transaction_reasons(organization_id);
