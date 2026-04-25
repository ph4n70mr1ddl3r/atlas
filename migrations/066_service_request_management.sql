-- Service Request Management (Oracle Fusion CX Service)
-- Tables for service categories, service requests, updates, and assignments.

-- Service categories
CREATE TABLE IF NOT EXISTS _atlas.service_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    parent_category_id UUID REFERENCES _atlas.service_categories(id),
    default_priority VARCHAR(20),
    default_sla_hours INT,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Service requests
CREATE TABLE IF NOT EXISTS _atlas.service_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    request_number VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    category_id UUID REFERENCES _atlas.service_categories(id),
    category_name VARCHAR(200),
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    request_type VARCHAR(30) NOT NULL DEFAULT 'incident',
    channel VARCHAR(30) NOT NULL DEFAULT 'web',
    customer_id UUID,
    customer_name VARCHAR(200),
    contact_id UUID,
    contact_name VARCHAR(200),
    assigned_to UUID,
    assigned_to_name VARCHAR(200),
    assigned_group VARCHAR(200),
    product_id UUID,
    product_name VARCHAR(200),
    serial_number VARCHAR(100),
    resolution TEXT,
    resolution_code VARCHAR(30),
    sla_due_date DATE,
    sla_breached BOOLEAN DEFAULT false,
    parent_request_id UUID REFERENCES _atlas.service_requests(id),
    related_object_type VARCHAR(100),
    related_object_id UUID,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sr_org_status ON _atlas.service_requests(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_sr_org_number ON _atlas.service_requests(organization_id, request_number);
CREATE INDEX IF NOT EXISTS idx_sr_customer ON _atlas.service_requests(organization_id, customer_id);
CREATE INDEX IF NOT EXISTS idx_sr_assigned ON _atlas.service_requests(assigned_to);
CREATE INDEX IF NOT EXISTS idx_sr_priority ON _atlas.service_requests(organization_id, priority);

-- Service request updates / communications
CREATE TABLE IF NOT EXISTS _atlas.service_request_updates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    request_id UUID NOT NULL REFERENCES _atlas.service_requests(id) ON DELETE CASCADE,
    update_type VARCHAR(30) NOT NULL DEFAULT 'comment',
    author_id UUID,
    author_name VARCHAR(200),
    subject VARCHAR(500),
    body TEXT NOT NULL,
    is_internal BOOLEAN DEFAULT false,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sru_request ON _atlas.service_request_updates(request_id);

-- Service request assignments
CREATE TABLE IF NOT EXISTS _atlas.service_request_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    request_id UUID NOT NULL REFERENCES _atlas.service_requests(id) ON DELETE CASCADE,
    assigned_to UUID,
    assigned_to_name VARCHAR(200),
    assigned_group VARCHAR(200),
    assigned_by UUID,
    assigned_by_name VARCHAR(200),
    assignment_type VARCHAR(30) NOT NULL DEFAULT 'initial',
    status VARCHAR(20) DEFAULT 'active',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sra_request ON _atlas.service_request_assignments(request_id);
