-- Order Management (Oracle Fusion SCM > Order Management)
-- Sales Orders, Order Lines, Holds, Fulfillment Shipments

-- Sales Order Headers
CREATE TABLE IF NOT EXISTS _atlas.sales_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    order_number VARCHAR(50) NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(200),
    customer_po_number VARCHAR(50),
    order_date DATE NOT NULL,
    requested_ship_date DATE,
    actual_ship_date DATE,
    requested_delivery_date DATE,
    actual_delivery_date DATE,
    ship_to_address TEXT,
    bill_to_address TEXT,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    subtotal_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    shipping_charges NUMERIC(18,4) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    payment_terms VARCHAR(100),
    shipping_method VARCHAR(100),
    sales_channel VARCHAR(50),
    salesperson_id UUID,
    salesperson_name VARCHAR(200),
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    fulfillment_status VARCHAR(30) NOT NULL DEFAULT 'not_started',
    submitted_at TIMESTAMPTZ,
    confirmed_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    created_by UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, order_number)
);

-- Sales Order Lines
CREATE TABLE IF NOT EXISTS _atlas.sales_order_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    order_id UUID NOT NULL REFERENCES _atlas.sales_orders(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_id UUID,
    item_code VARCHAR(50),
    item_description TEXT,
    quantity_ordered NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_shipped NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_cancelled NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_backordered NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_selling_price NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_list_price NUMERIC(18,4),
    line_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    discount_percent NUMERIC(5,2),
    discount_amount NUMERIC(18,4),
    tax_code VARCHAR(30),
    tax_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    requested_ship_date DATE,
    actual_ship_date DATE,
    promised_delivery_date DATE,
    ship_from_warehouse VARCHAR(100),
    fulfillment_status VARCHAR(30) NOT NULL DEFAULT 'not_started',
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    cancellation_reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(order_id, line_number)
);

-- Order Holds
CREATE TABLE IF NOT EXISTS _atlas.order_holds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    order_id UUID NOT NULL REFERENCES _atlas.sales_orders(id),
    order_line_id UUID REFERENCES _atlas.sales_order_lines(id),
    hold_type VARCHAR(50) NOT NULL,
    hold_reason TEXT NOT NULL,
    applied_by UUID,
    applied_by_name VARCHAR(200),
    released_by UUID,
    released_by_name VARCHAR(200),
    released_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Fulfillment Shipments
CREATE TABLE IF NOT EXISTS _atlas.fulfillment_shipments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    shipment_number VARCHAR(50) NOT NULL,
    order_id UUID NOT NULL REFERENCES _atlas.sales_orders(id),
    order_line_ids JSONB NOT NULL DEFAULT '[]',
    warehouse VARCHAR(100),
    carrier VARCHAR(100),
    tracking_number VARCHAR(100),
    shipping_method VARCHAR(100),
    ship_date DATE,
    estimated_delivery_date DATE,
    actual_delivery_date DATE,
    delivery_confirmation TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'planned',
    shipped_by UUID,
    shipped_by_name VARCHAR(200),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, shipment_number)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_sales_orders_org ON _atlas.sales_orders(organization_id);
CREATE INDEX IF NOT EXISTS idx_sales_orders_status ON _atlas.sales_orders(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_sales_orders_customer ON _atlas.sales_orders(organization_id, customer_id);
CREATE INDEX IF NOT EXISTS idx_sales_orders_date ON _atlas.sales_orders(organization_id, order_date DESC);
CREATE INDEX IF NOT EXISTS idx_sales_order_lines_order ON _atlas.sales_order_lines(order_id);
CREATE INDEX IF NOT EXISTS idx_order_holds_order ON _atlas.order_holds(order_id);
CREATE INDEX IF NOT EXISTS idx_order_holds_active ON _atlas.order_holds(order_id, is_active);
CREATE INDEX IF NOT EXISTS idx_fulfillment_shipments_order ON _atlas.fulfillment_shipments(order_id);
CREATE INDEX IF NOT EXISTS idx_fulfillment_shipments_status ON _atlas.fulfillment_shipments(organization_id, status);
