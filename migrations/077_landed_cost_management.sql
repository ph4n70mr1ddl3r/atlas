-- Landed Cost Management (Oracle Fusion SCM > Landed Cost Management)
-- Provides: Cost templates, cost components, charge capture, cost allocation,
-- landed cost simulation, and variance analysis.

-- ============================================================================
-- Landed Cost Templates: groups of cost components for different sourcing scenarios
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.landed_cost_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'active',                   -- active, inactive
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Landed Cost Components: individual cost elements (freight, insurance, customs)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.landed_cost_components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID REFERENCES _atlas.landed_cost_templates(id),
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    cost_type VARCHAR(50) NOT NULL DEFAULT 'other',        -- freight, insurance, customs_duty, handling, brokerage, storage, other
    allocation_basis VARCHAR(50) NOT NULL DEFAULT 'quantity', -- quantity, weight, volume, value, equal
    default_rate NUMERIC(18,6),
    rate_uom VARCHAR(50),                                   -- per_unit, percentage, flat
    expense_account VARCHAR(100),
    is_taxable BOOLEAN DEFAULT false,
    status VARCHAR(50) DEFAULT 'active',                   -- active, inactive
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_lcc_template ON _atlas.landed_cost_components(template_id);

-- ============================================================================
-- Landed Cost Charges: actual or estimated charges from suppliers/carriers
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.landed_cost_charges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    charge_number VARCHAR(100) NOT NULL,
    template_id UUID REFERENCES _atlas.landed_cost_templates(id),
    receipt_id UUID,                                        -- link to receiving
    purchase_order_id UUID,
    supplier_id UUID,
    supplier_name VARCHAR(300),
    charge_type VARCHAR(50) NOT NULL DEFAULT 'actual',     -- estimated, actual, adjustment
    charge_date DATE,
    total_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    currency VARCHAR(10) DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'draft',           -- draft, submitted, allocated, posted, cancelled
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, charge_number)
);

CREATE INDEX IF NOT EXISTS idx_lchg_receipt ON _atlas.landed_cost_charges(receipt_id);
CREATE INDEX IF NOT EXISTS idx_lchg_po ON _atlas.landed_cost_charges(purchase_order_id);
CREATE INDEX IF NOT EXISTS idx_lchg_supplier ON _atlas.landed_cost_charges(supplier_id);
CREATE INDEX IF NOT EXISTS idx_lchg_status ON _atlas.landed_cost_charges(status);

-- ============================================================================
-- Landed Cost Charge Lines: charge details per component/line
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.landed_cost_charge_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    charge_id UUID NOT NULL REFERENCES _atlas.landed_cost_charges(id) ON DELETE CASCADE,
    component_id UUID REFERENCES _atlas.landed_cost_components(id),
    line_number INT NOT NULL,
    receipt_line_id UUID,
    item_id UUID,
    item_code VARCHAR(100),
    item_description VARCHAR(500),
    charge_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    allocated_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    allocation_basis VARCHAR(50) DEFAULT 'quantity',
    allocation_qty NUMERIC(18,6),
    allocation_value NUMERIC(18,6),
    expense_account VARCHAR(100),
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_lcl_charge ON _atlas.landed_cost_charge_lines(charge_id);
CREATE INDEX IF NOT EXISTS idx_lcl_item ON _atlas.landed_cost_charge_lines(item_id);

-- ============================================================================
-- Landed Cost Allocations: final allocated cost per receipt line / item
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.landed_cost_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    charge_id UUID NOT NULL REFERENCES _atlas.landed_cost_charges(id),
    charge_line_id UUID NOT NULL REFERENCES _atlas.landed_cost_charge_lines(id),
    receipt_id UUID,
    receipt_line_id UUID,
    item_id UUID,
    item_code VARCHAR(100),
    allocated_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    allocation_basis VARCHAR(50) NOT NULL DEFAULT 'quantity',
    allocation_basis_value NUMERIC(18,6),
    total_basis_value NUMERIC(18,6),
    allocation_pct NUMERIC(8,4),
    unit_landed_cost NUMERIC(18,6),
    original_unit_cost NUMERIC(18,6),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_lca_charge ON _atlas.landed_cost_allocations(charge_id);
CREATE INDEX IF NOT EXISTS idx_lca_receipt ON _atlas.landed_cost_allocations(receipt_id);
CREATE INDEX IF NOT EXISTS idx_lca_item ON _atlas.landed_cost_allocations(item_id);

-- ============================================================================
-- Landed Cost Simulations: what-if analysis for estimated landed costs
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.landed_cost_simulations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    simulation_number VARCHAR(100) NOT NULL,
    template_id UUID REFERENCES _atlas.landed_cost_templates(id),
    purchase_order_id UUID,
    item_id UUID,
    item_code VARCHAR(100),
    item_description VARCHAR(500),
    estimated_quantity NUMERIC(18,6) NOT NULL DEFAULT 0,
    unit_price NUMERIC(18,6) NOT NULL DEFAULT 0,
    currency VARCHAR(10) DEFAULT 'USD',
    estimated_charges JSONB DEFAULT '[]'::jsonb,
    estimated_landed_cost NUMERIC(18,6) NOT NULL DEFAULT 0,
    estimated_landed_cost_per_unit NUMERIC(18,6) NOT NULL DEFAULT 0,
    variance_vs_actual NUMERIC(18,6),
    status VARCHAR(50) NOT NULL DEFAULT 'draft',           -- draft, completed, archived
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, simulation_number)
);

CREATE INDEX IF NOT EXISTS idx_lcs_template ON _atlas.landed_cost_simulations(template_id);
CREATE INDEX IF NOT EXISTS idx_lcs_status ON _atlas.landed_cost_simulations(status);
