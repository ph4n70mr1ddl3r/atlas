-- Purchase Requisitions (Oracle Fusion Self-Service Procurement > Requisitions)
-- Implements requisition creation, approval workflow, and AutoCreate conversion to purchase orders

-- Requisition priorities
CREATE TYPE _atlas.requisition_priority AS ENUM ('low', 'medium', 'high', 'urgent');

-- Requisition statuses
CREATE TYPE _atlas.requisition_status AS ENUM ('draft', 'submitted', 'approved', 'rejected', 'cancelled', 'closed', 'in_review');

-- Requisition line statuses
CREATE TYPE _atlas.requisition_line_status AS ENUM ('draft', 'submitted', 'approved', 'rejected', 'cancelled', 'partially_ordered', 'ordered', 'closed');

-- Purchase requisitions
CREATE TABLE IF NOT EXISTS _atlas.purchase_requisitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    requisition_number VARCHAR(50) NOT NULL,
    description TEXT,
    urgency_code _atlas.requisition_priority NOT NULL DEFAULT 'medium',
    status _atlas.requisition_status NOT NULL DEFAULT 'draft',
    requester_id UUID,
    requester_name VARCHAR(200),
    department VARCHAR(200),
    justification TEXT,
    budget_code VARCHAR(100),
    amount_limit NUMERIC(15,2),
    total_amount NUMERIC(15,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    charge_account_code VARCHAR(100),
    delivery_address TEXT,
    requested_delivery_date DATE,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    submitted_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, requisition_number)
);

-- Purchase requisition lines
CREATE TABLE IF NOT EXISTS _atlas.requisition_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    requisition_id UUID NOT NULL REFERENCES _atlas.purchase_requisitions(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_code VARCHAR(100),
    item_description TEXT NOT NULL,
    category VARCHAR(200),
    quantity NUMERIC(15,4) NOT NULL DEFAULT 1,
    unit_of_measure VARCHAR(50) DEFAULT 'EACH',
    unit_price NUMERIC(15,4) NOT NULL DEFAULT 0,
    line_amount NUMERIC(15,4) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    charge_account_code VARCHAR(100),
    requested_delivery_date DATE,
    supplier_id UUID,
    supplier_name VARCHAR(200),
    status _atlas.requisition_line_status NOT NULL DEFAULT 'draft',
    source_type VARCHAR(50) DEFAULT 'manual',  -- manual, catalog, punchout
    source_reference VARCHAR(200),
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Requisition distributions (accounting)
CREATE TABLE IF NOT EXISTS _atlas.requisition_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    requisition_id UUID NOT NULL REFERENCES _atlas.purchase_requisitions(id) ON DELETE CASCADE,
    line_id UUID NOT NULL REFERENCES _atlas.requisition_lines(id) ON DELETE CASCADE,
    distribution_number INT NOT NULL,
    charge_account_code VARCHAR(100) NOT NULL,
    allocation_percentage NUMERIC(7,4) NOT NULL DEFAULT 100.0000,
    amount NUMERIC(15,4) NOT NULL DEFAULT 0,
    project_code VARCHAR(100),
    cost_center VARCHAR(100),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Approval history for requisitions
CREATE TABLE IF NOT EXISTS _atlas.requisition_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    requisition_id UUID NOT NULL REFERENCES _atlas.purchase_requisitions(id) ON DELETE CASCADE,
    approver_id UUID NOT NULL,
    approver_name VARCHAR(200),
    action VARCHAR(20) NOT NULL,  -- approved, rejected, delegated, returned
    comments TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- AutoCreate links (tracks requisition lines converted to purchase orders)
CREATE TABLE IF NOT EXISTS _atlas.autocreate_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    requisition_id UUID NOT NULL,
    requisition_line_id UUID NOT NULL REFERENCES _atlas.requisition_lines(id) ON DELETE CASCADE,
    purchase_order_id UUID,
    purchase_order_number VARCHAR(50),
    purchase_order_line_id UUID,
    purchase_order_line_number INT,
    quantity_ordered NUMERIC(15,4) DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, ordered, partial, completed, cancelled
    autocreate_date TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_requisitions_org ON _atlas.purchase_requisitions(organization_id);
CREATE INDEX IF NOT EXISTS idx_requisitions_status ON _atlas.purchase_requisitions(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_requisitions_requester ON _atlas.purchase_requisitions(organization_id, requester_id);
CREATE INDEX IF NOT EXISTS idx_requisition_lines_req ON _atlas.requisition_lines(requisition_id);
CREATE INDEX IF NOT EXISTS idx_requisition_lines_status ON _atlas.requisition_lines(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_req_distributions_line ON _atlas.requisition_distributions(line_id);
CREATE INDEX IF NOT EXISTS idx_req_approvals_req ON _atlas.requisition_approvals(requisition_id);
CREATE INDEX IF NOT EXISTS idx_autocreate_links_req ON _atlas.autocreate_links(requisition_id);
CREATE INDEX IF NOT EXISTS idx_autocreate_links_status ON _atlas.autocreate_links(organization_id, status);