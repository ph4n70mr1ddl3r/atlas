-- Procurement Contracts (Oracle Fusion SCM > Procurement > Contracts)
-- Tables for contract types, procurement contracts, contract lines,
-- milestones, renewals, and spend tracking.

-- Contract Type Definitions
CREATE TABLE IF NOT EXISTS _atlas.procurement_contract_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    contract_classification VARCHAR(50) NOT NULL DEFAULT 'blanket',
    requires_approval BOOLEAN NOT NULL DEFAULT true,
    default_duration_days INT,
    allow_amount_commitment BOOLEAN NOT NULL DEFAULT true,
    allow_quantity_commitment BOOLEAN NOT NULL DEFAULT true,
    allow_line_additions BOOLEAN NOT NULL DEFAULT true,
    allow_price_adjustment BOOLEAN NOT NULL DEFAULT false,
    allow_renewal BOOLEAN NOT NULL DEFAULT true,
    allow_termination BOOLEAN NOT NULL DEFAULT true,
    max_renewals INT,
    default_payment_terms_code VARCHAR(50),
    default_currency_code VARCHAR(3),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Procurement Contracts
CREATE TABLE IF NOT EXISTS _atlas.procurement_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_number VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    contract_type_code VARCHAR(100),
    contract_classification VARCHAR(50) NOT NULL DEFAULT 'blanket',
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    supplier_id UUID NOT NULL,
    supplier_number VARCHAR(100),
    supplier_name VARCHAR(500),
    supplier_contact VARCHAR(500),
    buyer_id UUID,
    buyer_name VARCHAR(500),
    start_date DATE,
    end_date DATE,
    total_committed_amount NUMERIC(18,2) DEFAULT 0,
    total_released_amount NUMERIC(18,2) DEFAULT 0,
    total_invoiced_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    payment_terms_code VARCHAR(50),
    price_type VARCHAR(20) NOT NULL DEFAULT 'fixed',
    renewal_count INT NOT NULL DEFAULT 0,
    max_renewals INT,
    line_count INT NOT NULL DEFAULT 0,
    milestone_count INT NOT NULL DEFAULT 0,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,
    terminated_by UUID,
    termination_reason TEXT,
    terminated_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, contract_number)
);

-- Contract Lines
CREATE TABLE IF NOT EXISTS _atlas.procurement_contract_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.procurement_contracts(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_description VARCHAR(1000) NOT NULL,
    item_code VARCHAR(200),
    category VARCHAR(200),
    uom VARCHAR(50),
    quantity_committed NUMERIC(18,4),
    quantity_released NUMERIC(18,4) DEFAULT 0,
    unit_price NUMERIC(18,4) NOT NULL DEFAULT 0,
    line_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_released NUMERIC(18,2) DEFAULT 0,
    delivery_date DATE,
    supplier_part_number VARCHAR(200),
    account_code VARCHAR(100),
    cost_center VARCHAR(100),
    project_id UUID,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Contract Milestones / Deliverables
CREATE TABLE IF NOT EXISTS _atlas.procurement_contract_milestones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.procurement_contracts(id) ON DELETE CASCADE,
    contract_line_id UUID REFERENCES _atlas.procurement_contract_lines(id) ON DELETE SET NULL,
    milestone_number INT NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    milestone_type VARCHAR(50) NOT NULL DEFAULT 'delivery',
    target_date DATE NOT NULL,
    actual_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    amount NUMERIC(18,2) DEFAULT 0,
    percent_of_total NUMERIC(8,4) DEFAULT 0,
    deliverable TEXT,
    is_billable BOOLEAN NOT NULL DEFAULT false,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Contract Renewals
CREATE TABLE IF NOT EXISTS _atlas.procurement_contract_renewals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.procurement_contracts(id) ON DELETE CASCADE,
    renewal_number INT NOT NULL,
    previous_end_date DATE NOT NULL,
    new_end_date DATE NOT NULL,
    renewal_type VARCHAR(50) NOT NULL DEFAULT 'manual',
    terms_changed TEXT,
    renewed_by UUID,
    renewed_at TIMESTAMPTZ DEFAULT now(),
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Contract Spend Entries
CREATE TABLE IF NOT EXISTS _atlas.procurement_contract_spend (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.procurement_contracts(id) ON DELETE CASCADE,
    contract_line_id UUID REFERENCES _atlas.procurement_contract_lines(id) ON DELETE SET NULL,
    source_type VARCHAR(100) NOT NULL,
    source_id UUID,
    source_number VARCHAR(200),
    transaction_date DATE NOT NULL,
    amount NUMERIC(18,2) NOT NULL,
    quantity NUMERIC(18,4),
    description TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_pc_types_org ON _atlas.procurement_contract_types(organization_id);
CREATE INDEX IF NOT EXISTS idx_pc_contracts_org ON _atlas.procurement_contracts(organization_id);
CREATE INDEX IF NOT EXISTS idx_pc_contracts_status ON _atlas.procurement_contracts(status);
CREATE INDEX IF NOT EXISTS idx_pc_contracts_supplier ON _atlas.procurement_contracts(supplier_id);
CREATE INDEX IF NOT EXISTS idx_pcl_contract ON _atlas.procurement_contract_lines(contract_id);
CREATE INDEX IF NOT EXISTS idx_pcm_contract ON _atlas.procurement_contract_milestones(contract_id);
CREATE INDEX IF NOT EXISTS idx_pcr_contract ON _atlas.procurement_contract_renewals(contract_id);
CREATE INDEX IF NOT EXISTS idx_pcs_contract ON _atlas.procurement_contract_spend(contract_id);
CREATE INDEX IF NOT EXISTS idx_pcs_date ON _atlas.procurement_contract_spend(transaction_date);
