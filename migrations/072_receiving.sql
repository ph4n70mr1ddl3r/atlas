-- Receiving Management (Oracle Fusion SCM > Receiving)
-- Receipt headers, lines, inspections, deliveries, and returns
-- Using DOUBLE PRECISION instead of NUMERIC for sqlx 0.7 compatibility

-- Receiving Locations
CREATE TABLE IF NOT EXISTS _atlas.receiving_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    location_type VARCHAR(50) DEFAULT 'warehouse',
    address TEXT,
    city VARCHAR(200),
    state VARCHAR(200),
    country VARCHAR(200),
    postal_code VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Receipt Headers
CREATE TABLE IF NOT EXISTS _atlas.receipt_headers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    receipt_number VARCHAR(100) NOT NULL,
    receipt_type VARCHAR(50) DEFAULT 'standard',
    receipt_source VARCHAR(50) DEFAULT 'purchase_order',
    supplier_id UUID,
    supplier_name VARCHAR(500),
    supplier_number VARCHAR(100),
    purchase_order_id UUID,
    purchase_order_number VARCHAR(100),
    receiving_location_id UUID REFERENCES _atlas.receiving_locations(id),
    receiving_location_code VARCHAR(100),
    receiving_date DATE NOT NULL,
    packing_slip_number VARCHAR(100),
    bill_of_lading VARCHAR(100),
    carrier VARCHAR(200),
    tracking_number VARCHAR(200),
    waybill_number VARCHAR(100),
    notes TEXT,
    status VARCHAR(50) DEFAULT 'draft',
    total_received_qty INT DEFAULT 0,
    total_inspected_qty INT DEFAULT 0,
    total_accepted_qty INT DEFAULT 0,
    total_rejected_qty INT DEFAULT 0,
    total_delivered_qty INT DEFAULT 0,
    received_by UUID,
    received_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, receipt_number)
);

-- Receipt Lines
CREATE TABLE IF NOT EXISTS _atlas.receipt_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    receipt_id UUID NOT NULL REFERENCES _atlas.receipt_headers(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    purchase_order_line_id UUID,
    item_id UUID,
    item_code VARCHAR(100),
    item_description TEXT,
    ordered_qty DOUBLE PRECISION DEFAULT 0,
    ordered_uom VARCHAR(50),
    received_qty DOUBLE PRECISION DEFAULT 0,
    received_uom VARCHAR(50),
    accepted_qty DOUBLE PRECISION DEFAULT 0,
    rejected_qty DOUBLE PRECISION DEFAULT 0,
    inspection_status VARCHAR(50) DEFAULT 'pending',
    delivery_status VARCHAR(50) DEFAULT 'pending',
    lot_number VARCHAR(100),
    serial_numbers JSONB DEFAULT '[]',
    expiration_date DATE,
    manufacture_date DATE,
    unit_price DOUBLE PRECISION,
    currency VARCHAR(3) DEFAULT 'USD',
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Inspections
CREATE TABLE IF NOT EXISTS _atlas.receipt_inspections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    receipt_id UUID NOT NULL REFERENCES _atlas.receipt_headers(id),
    receipt_line_id UUID NOT NULL REFERENCES _atlas.receipt_lines(id),
    inspection_number VARCHAR(100) NOT NULL,
    inspection_template VARCHAR(100),
    inspector_id UUID,
    inspector_name VARCHAR(500),
    inspection_date DATE NOT NULL,
    sample_size DOUBLE PRECISION,
    quantity_inspected DOUBLE PRECISION DEFAULT 0,
    quantity_accepted DOUBLE PRECISION DEFAULT 0,
    quantity_rejected DOUBLE PRECISION DEFAULT 0,
    disposition VARCHAR(50) DEFAULT 'accept',
    rejection_reason VARCHAR(500),
    quality_score DOUBLE PRECISION,
    notes TEXT,
    status VARCHAR(50) DEFAULT 'pending',
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, inspection_number)
);

-- Inspection Details (individual quality checks)
CREATE TABLE IF NOT EXISTS _atlas.inspection_details (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    inspection_id UUID NOT NULL REFERENCES _atlas.receipt_inspections(id) ON DELETE CASCADE,
    check_number INT NOT NULL,
    check_name VARCHAR(500) NOT NULL,
    check_type VARCHAR(50) DEFAULT 'visual',
    specification TEXT,
    result VARCHAR(50) DEFAULT 'pass',
    measured_value VARCHAR(200),
    expected_value VARCHAR(200),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Deliveries (putaway to subinventory)
CREATE TABLE IF NOT EXISTS _atlas.receipt_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    receipt_id UUID NOT NULL REFERENCES _atlas.receipt_headers(id),
    receipt_line_id UUID NOT NULL REFERENCES _atlas.receipt_lines(id),
    delivery_number VARCHAR(100) NOT NULL,
    subinventory VARCHAR(100),
    locator VARCHAR(200),
    quantity_delivered DOUBLE PRECISION DEFAULT 0,
    uom VARCHAR(50),
    lot_number VARCHAR(100),
    serial_number VARCHAR(200),
    delivered_by UUID,
    delivered_by_name VARCHAR(500),
    delivery_date TIMESTAMPTZ DEFAULT now(),
    destination_type VARCHAR(50) DEFAULT 'inventory',
    account_code VARCHAR(100),
    notes TEXT,
    status VARCHAR(50) DEFAULT 'pending',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, delivery_number)
);

-- Return to Supplier (RTV)
CREATE TABLE IF NOT EXISTS _atlas.receipt_returns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    return_number VARCHAR(100) NOT NULL,
    receipt_id UUID REFERENCES _atlas.receipt_headers(id),
    receipt_line_id UUID REFERENCES _atlas.receipt_lines(id),
    supplier_id UUID,
    supplier_name VARCHAR(500),
    return_type VARCHAR(50) DEFAULT 'reject',
    item_id UUID,
    item_code VARCHAR(100),
    item_description TEXT,
    quantity_returned DOUBLE PRECISION DEFAULT 0,
    uom VARCHAR(50),
    unit_price DOUBLE PRECISION,
    currency VARCHAR(3) DEFAULT 'USD',
    return_reason TEXT,
    return_date DATE NOT NULL,
    carrier VARCHAR(200),
    tracking_number VARCHAR(200),
    credit_expected BOOLEAN DEFAULT true,
    credit_memo_number VARCHAR(100),
    status VARCHAR(50) DEFAULT 'draft',
    shipped_at TIMESTAMPTZ,
    credited_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, return_number)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_receiving_locations_org ON _atlas.receiving_locations(organization_id);
CREATE INDEX IF NOT EXISTS idx_receipt_headers_org ON _atlas.receipt_headers(organization_id);
CREATE INDEX IF NOT EXISTS idx_receipt_headers_status ON _atlas.receipt_headers(status);
CREATE INDEX IF NOT EXISTS idx_receipt_headers_supplier ON _atlas.receipt_headers(supplier_id);
CREATE INDEX IF NOT EXISTS idx_receipt_headers_po ON _atlas.receipt_headers(purchase_order_id);
CREATE INDEX IF NOT EXISTS idx_receipt_lines_receipt ON _atlas.receipt_lines(receipt_id);
CREATE INDEX IF NOT EXISTS idx_receipt_inspections_receipt ON _atlas.receipt_inspections(receipt_id);
CREATE INDEX IF NOT EXISTS idx_receipt_inspections_line ON _atlas.receipt_inspections(receipt_line_id);
CREATE INDEX IF NOT EXISTS idx_inspection_details_inspection ON _atlas.inspection_details(inspection_id);
CREATE INDEX IF NOT EXISTS idx_receipt_deliveries_receipt ON _atlas.receipt_deliveries(receipt_id);
CREATE INDEX IF NOT EXISTS idx_receipt_deliveries_line ON _atlas.receipt_deliveries(receipt_line_id);
CREATE INDEX IF NOT EXISTS idx_receipt_returns_receipt ON _atlas.receipt_returns(receipt_id);
CREATE INDEX IF NOT EXISTS idx_receipt_returns_status ON _atlas.receipt_returns(status);
