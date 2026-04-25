-- 069: Shipping Execution
-- Oracle Fusion SCM: Shipping Execution
-- Manages carriers, shipping methods, shipments, ship confirmations,
-- and shipping documentation.

BEGIN;

-- ============================================================================
-- Carriers
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.shipping_carriers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    carrier_type VARCHAR(50) DEFAULT 'external',  -- 'external', 'internal', 'third_party'
    tracking_url_template TEXT,
    contact_name VARCHAR(200),
    contact_phone VARCHAR(50),
    contact_email VARCHAR(200),
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Shipping Methods
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.shipping_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    carrier_id UUID REFERENCES _atlas.shipping_carriers(id),
    transit_time_days INT DEFAULT 1,
    is_express BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Shipments (shipping headers)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.shipments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    shipment_number VARCHAR(50) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',  -- 'draft', 'confirmed', 'picked', 'packed', 'shipped', 'delivered', 'cancelled'
    carrier_id UUID REFERENCES _atlas.shipping_carriers(id),
    carrier_name VARCHAR(200),
    shipping_method_id UUID REFERENCES _atlas.shipping_methods(id),
    shipping_method_name VARCHAR(200),
    order_id UUID,
    order_number VARCHAR(50),
    customer_id UUID,
    customer_name VARCHAR(200),
    ship_from_warehouse VARCHAR(100),
    ship_to_name VARCHAR(200),
    ship_to_address TEXT,
    ship_to_city VARCHAR(100),
    ship_to_state VARCHAR(100),
    ship_to_postal_code VARCHAR(20),
    ship_to_country VARCHAR(3),
    tracking_number VARCHAR(200),
    total_weight DOUBLE PRECISION DEFAULT 0,
    weight_unit VARCHAR(10) DEFAULT 'kg',
    total_volume DOUBLE PRECISION DEFAULT 0,
    volume_unit VARCHAR(10) DEFAULT 'm3',
    total_packages INT DEFAULT 0,
    shipped_date TIMESTAMPTZ,
    estimated_delivery DATE,
    actual_delivery TIMESTAMPTZ,
    confirmed_by UUID,
    confirmed_at TIMESTAMPTZ,
    shipped_by UUID,
    delivered_by UUID,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, shipment_number)
);

-- ============================================================================
-- Shipment Lines
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.shipment_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    shipment_id UUID NOT NULL REFERENCES _atlas.shipments(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    order_line_id UUID,
    item_code VARCHAR(100) NOT NULL,
    item_name VARCHAR(200),
    item_description TEXT,
    requested_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    shipped_quantity DOUBLE PRECISION DEFAULT 0,
    backordered_quantity DOUBLE PRECISION DEFAULT 0,
    unit_of_measure VARCHAR(20) DEFAULT 'EA',
    weight DOUBLE PRECISION DEFAULT 0,
    weight_unit VARCHAR(10) DEFAULT 'kg',
    lot_number VARCHAR(100),
    serial_number VARCHAR(200),
    is_fragile BOOLEAN DEFAULT false,
    is_hazardous BOOLEAN DEFAULT false,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_shipment_lines_shipment ON _atlas.shipment_lines(shipment_id);
CREATE INDEX IF NOT EXISTS idx_shipment_lines_item ON _atlas.shipment_lines(organization_id, item_code);

-- ============================================================================
-- Packing Slips
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.packing_slips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    shipment_id UUID NOT NULL REFERENCES _atlas.shipments(id) ON DELETE CASCADE,
    packing_slip_number VARCHAR(50) NOT NULL,
    package_number INT NOT NULL DEFAULT 1,
    package_type VARCHAR(50) DEFAULT 'box',  -- 'box', 'pallet', 'envelope', 'crate'
    weight DOUBLE PRECISION DEFAULT 0,
    weight_unit VARCHAR(10) DEFAULT 'kg',
    dimensions_length DOUBLE PRECISION DEFAULT 0,
    dimensions_width DOUBLE PRECISION DEFAULT 0,
    dimensions_height DOUBLE PRECISION DEFAULT 0,
    dimensions_unit VARCHAR(10) DEFAULT 'cm',
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, packing_slip_number)
);

-- ============================================================================
-- Packing Slip Lines
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.packing_slip_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    packing_slip_id UUID NOT NULL REFERENCES _atlas.packing_slips(id) ON DELETE CASCADE,
    shipment_line_id UUID NOT NULL REFERENCES _atlas.shipment_lines(id),
    line_number INT NOT NULL,
    item_code VARCHAR(100) NOT NULL,
    item_name VARCHAR(200),
    packed_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_packing_slip_lines_slip ON _atlas.packing_slip_lines(packing_slip_id);

COMMIT;
