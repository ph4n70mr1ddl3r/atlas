-- Transportation Management
-- Oracle Fusion Cloud SCM > Transportation Management
-- Provides: carrier management, freight routes/lanes, shipments, load planning,
--           freight cost rating, delivery tracking, and transportation analytics

-- Carriers (shipping partners: FedEx, UPS, DHL, etc.)
CREATE TABLE IF NOT EXISTS _atlas.carriers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    carrier_code VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    carrier_type VARCHAR(30) NOT NULL DEFAULT 'parcel', -- parcel, ltl, ftl, air, ocean, rail, multimodal
    status VARCHAR(30) NOT NULL DEFAULT 'active', -- active, inactive, suspended, blacklisted
    scac_code VARCHAR(10), -- Standard Carrier Alpha Code
    dot_number VARCHAR(20), -- Department of Transportation number
    mc_number VARCHAR(20), -- Motor Carrier number
    tax_id VARCHAR(50),
    contact_name VARCHAR(200),
    contact_email VARCHAR(200),
    contact_phone VARCHAR(50),
    address_line1 VARCHAR(300),
    address_line2 VARCHAR(300),
    city VARCHAR(100),
    state VARCHAR(100),
    postal_code VARCHAR(20),
    country VARCHAR(3) DEFAULT 'USA',
    currency_code VARCHAR(3) DEFAULT 'USD',
    payment_terms VARCHAR(50) DEFAULT 'net_30',
    insurance_policy_number VARCHAR(100),
    insurance_expiry_date DATE,
    performance_rating NUMERIC(3,2) DEFAULT 0.00, -- 0.00 to 5.00
    on_time_delivery_pct NUMERIC(5,2) DEFAULT 0.00,
    claims_ratio NUMERIC(5,4) DEFAULT 0.0000,
    default_service_level VARCHAR(30) DEFAULT 'standard', -- standard, express, economy, premium
    capabilities JSONB DEFAULT '[]', -- ["hazardous", "temperature_controlled", "oversized", "international"]
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, carrier_code)
);

-- Carrier Services (specific service levels offered by each carrier)
CREATE TABLE IF NOT EXISTS _atlas.carrier_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    carrier_id UUID NOT NULL REFERENCES _atlas.carriers(id) ON DELETE CASCADE,
    service_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    service_level VARCHAR(30) NOT NULL DEFAULT 'standard', -- standard, express, economy, premium, same_day
    transit_days_min INT DEFAULT 1,
    transit_days_max INT DEFAULT 5,
    max_weight_kg NUMERIC(10,2),
    max_dimensions JSONB, -- {"length": 0, "width": 0, "height": 0, "unit": "cm"}
    cutoff_time TIME, -- daily cutoff for same-day pickup
    operates_on_weekends BOOLEAN DEFAULT false,
    is_international BOOLEAN DEFAULT false,
    rate_per_kg NUMERIC(10,4) DEFAULT 0,
    minimum_charge NUMERIC(12,2) DEFAULT 0,
    fuel_surcharge_pct NUMERIC(5,2) DEFAULT 0.00,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(carrier_id, service_code)
);

-- Transportation Lanes (origin-destination routes)
CREATE TABLE IF NOT EXISTS _atlas.transport_lanes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lane_code VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    origin_location_id UUID,
    origin_location_name VARCHAR(300),
    origin_city VARCHAR(100),
    origin_state VARCHAR(100),
    origin_country VARCHAR(3) DEFAULT 'USA',
    origin_postal_code VARCHAR(20),
    destination_location_id UUID,
    destination_location_name VARCHAR(300),
    destination_city VARCHAR(100),
    destination_state VARCHAR(100),
    destination_country VARCHAR(3) DEFAULT 'USA',
    destination_postal_code VARCHAR(20),
    distance_km NUMERIC(10,2),
    estimated_transit_hours NUMERIC(10,2),
    lane_type VARCHAR(30) NOT NULL DEFAULT 'domestic', -- domestic, international, intercompany
    preferred_carrier_id UUID,
    preferred_service_id UUID,
    status VARCHAR(30) NOT NULL DEFAULT 'active', -- active, inactive, seasonal
    effective_from DATE,
    effective_to DATE,
    restrictions JSONB DEFAULT '[]', -- hazmat restrictions, weight limits, etc.
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, lane_code)
);

-- Transportation Shipments (master shipments)
CREATE TABLE IF NOT EXISTS _atlas.shipments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    shipment_number VARCHAR(50) NOT NULL,
    name VARCHAR(300),
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, booked, picked_up, in_transit, at_destination, delivered, cancelled, exception
    shipment_type VARCHAR(30) NOT NULL DEFAULT 'outbound', -- outbound, inbound, transfer, returns
    priority VARCHAR(20) DEFAULT 'normal', -- low, normal, high, critical
    carrier_id UUID,
    carrier_code VARCHAR(50),
    carrier_name VARCHAR(300),
    carrier_service_id UUID,
    carrier_service_code VARCHAR(50),
    lane_id UUID,
    lane_code VARCHAR(50),
    origin_location_id UUID,
    origin_location_name VARCHAR(300),
    origin_address JSONB DEFAULT '{}',
    destination_location_id UUID,
    destination_location_name VARCHAR(300),
    destination_address JSONB DEFAULT '{}',
    planned_ship_date DATE,
    actual_ship_date DATE,
    planned_delivery_date DATE,
    actual_delivery_date DATE,
    pickup_window_start TIMESTAMPTZ,
    pickup_window_end TIMESTAMPTZ,
    delivery_window_start TIMESTAMPTZ,
    delivery_window_end TIMESTAMPTZ,
    total_weight_kg NUMERIC(10,2) DEFAULT 0,
    total_volume_cbm NUMERIC(10,2) DEFAULT 0,
    total_pieces INT DEFAULT 0,
    freight_cost NUMERIC(14,2) DEFAULT 0,
    fuel_surcharge NUMERIC(14,2) DEFAULT 0,
    accessorial_charges NUMERIC(14,2) DEFAULT 0,
    total_cost NUMERIC(14,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    tracking_number VARCHAR(100),
    tracking_url VARCHAR(500),
    pro_number VARCHAR(100), -- carrier's PRO/reference number
    bill_of_lading VARCHAR(100),
    sales_order_id UUID,
    sales_order_number VARCHAR(50),
    purchase_order_id UUID,
    purchase_order_number VARCHAR(50),
    transfer_order_id UUID,
    special_instructions TEXT,
    declared_value NUMERIC(14,2),
    insurance_required BOOLEAN DEFAULT false,
    signature_required BOOLEAN DEFAULT false,
    temperature_requirements JSONB, -- {"min_celsius": null, "max_celsius": null}
    hazmat_info JSONB,
    driver_name VARCHAR(200),
    vehicle_id VARCHAR(50),
    metadata JSONB DEFAULT '{}',
    booked_by UUID,
    shipped_by UUID,
    received_by UUID,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, shipment_number)
);

-- Shipment Stops (pickup/delivery stops for multi-stop routes)
CREATE TABLE IF NOT EXISTS _atlas.shipment_stops (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    shipment_id UUID NOT NULL REFERENCES _atlas.shipments(id) ON DELETE CASCADE,
    stop_number INT NOT NULL,
    stop_type VARCHAR(20) NOT NULL, -- pickup, delivery, transfer
    location_id UUID,
    location_name VARCHAR(300),
    address JSONB DEFAULT '{}',
    planned_arrival TIMESTAMPTZ,
    actual_arrival TIMESTAMPTZ,
    planned_departure TIMESTAMPTZ,
    actual_departure TIMESTAMPTZ,
    status VARCHAR(30) DEFAULT 'pending', -- pending, arrived, departed, skipped, failed
    contact_name VARCHAR(200),
    contact_phone VARCHAR(50),
    special_instructions TEXT,
    pieces INT DEFAULT 0,
    weight_kg NUMERIC(10,2) DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(shipment_id, stop_number)
);

-- Shipment Lines (items within a shipment)
CREATE TABLE IF NOT EXISTS _atlas.shipment_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    shipment_id UUID NOT NULL REFERENCES _atlas.shipments(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_id UUID,
    item_number VARCHAR(50),
    item_description VARCHAR(500),
    quantity INT NOT NULL DEFAULT 0,
    quantity_shipped INT DEFAULT 0,
    quantity_received INT DEFAULT 0,
    unit_of_measure VARCHAR(20) DEFAULT 'EA',
    weight_kg NUMERIC(10,2) DEFAULT 0,
    volume_cbm NUMERIC(10,2) DEFAULT 0,
    lot_number VARCHAR(100),
    serial_numbers JSONB DEFAULT '[]',
    source_line_id UUID, -- reference to order/requisition line
    source_line_type VARCHAR(30), -- sales_order, purchase_order, transfer_order
    stop_id UUID, -- which stop this line is for
    freight_class VARCHAR(10),
    nmfc_code VARCHAR(20),
    hazmat_class VARCHAR(20),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(shipment_id, line_number)
);

-- Shipment Tracking Events (status updates during transit)
CREATE TABLE IF NOT EXISTS _atlas.shipment_tracking_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    shipment_id UUID NOT NULL REFERENCES _atlas.shipments(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL, -- picked_up, in_transit, out_for_delivery, delivered, exception, delayed, customs_clearance, at_hub
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    location_description VARCHAR(300),
    city VARCHAR(100),
    state VARCHAR(100),
    country VARCHAR(3),
    latitude NUMERIC(10,7),
    longitude NUMERIC(10,7),
    description TEXT,
    carrier_event_code VARCHAR(50),
    carrier_event_description TEXT,
    updated_by VARCHAR(100), -- system, carrier_api, manual
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Freight Rate Records (carrier rate agreements)
CREATE TABLE IF NOT EXISTS _atlas.freight_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rate_code VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    carrier_id UUID NOT NULL REFERENCES _atlas.carriers(id),
    carrier_service_id UUID REFERENCES _atlas.carrier_services(id),
    lane_id UUID REFERENCES _atlas.transport_lanes(id),
    rate_type VARCHAR(30) NOT NULL DEFAULT 'per_kg', -- per_kg, per_unit, flat, per_mile, per_pallet, zone_based
    rate_amount NUMERIC(12,4) NOT NULL DEFAULT 0,
    minimum_charge NUMERIC(12,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    fuel_surcharge_pct NUMERIC(5,2) DEFAULT 0.00,
    accessorial_rates JSONB DEFAULT '{}',
    effective_from DATE NOT NULL,
    effective_to DATE,
    status VARCHAR(30) DEFAULT 'active', -- active, expired, pending, superseded
    is_contract_rate BOOLEAN DEFAULT false,
    contract_number VARCHAR(50),
    volume_threshold_min NUMERIC(12,2), -- minimum volume for rate to apply
    volume_threshold_max NUMERIC(12,2), -- maximum volume for rate to apply
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, rate_code)
);

-- Create indexes
CREATE INDEX idx_carriers_org ON _atlas.carriers(organization_id);
CREATE INDEX idx_carriers_status ON _atlas.carriers(status);
CREATE INDEX idx_carriers_type ON _atlas.carriers(carrier_type);
CREATE INDEX idx_carrier_services_carrier ON _atlas.carrier_services(carrier_id);
CREATE INDEX idx_carrier_services_active ON _atlas.carrier_services(is_active);
CREATE INDEX idx_transport_lanes_org ON _atlas.transport_lanes(organization_id);
CREATE INDEX idx_transport_lanes_status ON _atlas.transport_lanes(status);
CREATE INDEX idx_transport_lanes_route ON _atlas.transport_lanes(origin_country, destination_country);
CREATE INDEX idx_shipments_org ON _atlas.shipments(organization_id);
CREATE INDEX idx_shipments_status ON _atlas.shipments(status);
CREATE INDEX idx_shipments_carrier ON _atlas.shipments(carrier_id);
CREATE INDEX idx_shipments_dates ON _atlas.shipments(planned_ship_date, planned_delivery_date);
CREATE INDEX idx_shipments_tracking ON _atlas.shipments(tracking_number);
CREATE INDEX idx_shipment_stops_shipment ON _atlas.shipment_stops(shipment_id);
CREATE INDEX idx_shipment_lines_shipment ON _atlas.shipment_lines(shipment_id);
CREATE INDEX idx_tracking_events_shipment ON _atlas.shipment_tracking_events(shipment_id);
CREATE INDEX idx_tracking_events_timestamp ON _atlas.shipment_tracking_events(event_timestamp);
CREATE INDEX idx_freight_rates_org ON _atlas.freight_rates(organization_id);
CREATE INDEX idx_freight_rates_carrier ON _atlas.freight_rates(carrier_id);
CREATE INDEX idx_freight_rates_status ON _atlas.freight_rates(status);
CREATE INDEX idx_freight_rates_effective ON _atlas.freight_rates(effective_from, effective_to);
