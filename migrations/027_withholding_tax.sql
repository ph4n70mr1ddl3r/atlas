-- ============================================================================
-- Withholding Tax Management (Oracle Fusion Payables > Withholding Tax)
-- Migration 027
-- ============================================================================

-- Withholding Tax Codes
-- Defines individual withholding tax types with rates and thresholds
CREATE TABLE IF NOT EXISTS _atlas.withholding_tax_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Tax type: income_tax, vat, service_tax, contract_tax, royalty, dividend, interest, other
    tax_type VARCHAR(50) NOT NULL DEFAULT 'income_tax',
    -- Withholding rate percentage (e.g., 10.00 means 10%)
    rate_percentage NUMERIC(12,4) NOT NULL DEFAULT 0,
    -- Minimum amount below which no withholding applies
    threshold_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    -- Whether threshold is cumulative (year-to-date) or per-invoice
    threshold_is_cumulative BOOLEAN NOT NULL DEFAULT false,
    -- GL account codes
    withholding_account_code VARCHAR(50),
    expense_account_code VARCHAR(50),
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- Effective dates
    effective_from DATE,
    effective_to DATE,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_wht_codes_org ON _atlas.withholding_tax_codes(organization_id);
CREATE INDEX idx_wht_codes_type ON _atlas.withholding_tax_codes(organization_id, tax_type);
COMMENT ON TABLE _atlas.withholding_tax_codes IS 'Withholding tax code definitions with rates and thresholds';

-- Withholding Tax Groups
-- Groups multiple tax codes into reusable sets assignable to suppliers
CREATE TABLE IF NOT EXISTS _atlas.withholding_tax_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_wht_groups_org ON _atlas.withholding_tax_groups(organization_id);
COMMENT ON TABLE _atlas.withholding_tax_groups IS 'Withholding tax groups - reusable collections of tax codes';

-- Withholding Tax Group Members
-- Links tax codes to groups with optional rate overrides
CREATE TABLE IF NOT EXISTS _atlas.withholding_tax_group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES _atlas.withholding_tax_groups(id) ON DELETE CASCADE,
    tax_code_id UUID NOT NULL REFERENCES _atlas.withholding_tax_codes(id),
    -- Optional rate override (overrides the tax code default rate)
    rate_override NUMERIC(12,4),
    display_order INT NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(group_id, tax_code_id)
);

CREATE INDEX idx_wht_group_members_group ON _atlas.withholding_tax_group_members(group_id);
COMMENT ON TABLE _atlas.withholding_tax_group_members IS 'Tax codes within a withholding tax group';

-- Supplier Withholding Tax Assignments
-- Links suppliers to withholding tax groups with exemption support
CREATE TABLE IF NOT EXISTS _atlas.supplier_withholding_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    supplier_id UUID NOT NULL,
    supplier_number VARCHAR(50),
    supplier_name VARCHAR(200),
    tax_group_id UUID NOT NULL REFERENCES _atlas.withholding_tax_groups(id),
    -- Exemption handling
    is_exempt BOOLEAN NOT NULL DEFAULT false,
    exemption_reason TEXT,
    exemption_certificate VARCHAR(100),
    exemption_valid_until DATE,
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, supplier_id)
);

CREATE INDEX idx_wht_supplier_assign_org ON _atlas.supplier_withholding_assignments(organization_id);
CREATE INDEX idx_wht_supplier_assign_supplier ON _atlas.supplier_withholding_assignments(supplier_id);
COMMENT ON TABLE _atlas.supplier_withholding_assignments IS 'Supplier assignments to withholding tax groups';

-- Withholding Tax Lines
-- Records actual tax withheld from payments
CREATE TABLE IF NOT EXISTS _atlas.withholding_tax_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Payment reference
    payment_id UUID NOT NULL,
    payment_number VARCHAR(50),
    -- Invoice reference
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(50),
    -- Supplier
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(200),
    -- Tax code applied
    tax_code_id UUID NOT NULL REFERENCES _atlas.withholding_tax_codes(id),
    tax_code VARCHAR(50) NOT NULL,
    tax_code_name VARCHAR(200),
    tax_type VARCHAR(50) NOT NULL,
    rate_percentage NUMERIC(12,4) NOT NULL,
    -- Amounts
    taxable_amount NUMERIC(18,4) NOT NULL,
    withheld_amount NUMERIC(18,4) NOT NULL,
    -- GL account
    withholding_account_code VARCHAR(50),
    -- Status: pending, withheld, remitted, refunded
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Remittance tracking
    remittance_date DATE,
    remittance_reference VARCHAR(100),
    -- Metadata
    metadata JSONB NOT NULL DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_wht_lines_org ON _atlas.withholding_tax_lines(organization_id);
CREATE INDEX idx_wht_lines_payment ON _atlas.withholding_tax_lines(payment_id);
CREATE INDEX idx_wht_lines_supplier ON _atlas.withholding_tax_lines(organization_id, supplier_id);
CREATE INDEX idx_wht_lines_status ON _atlas.withholding_tax_lines(status);
CREATE INDEX idx_wht_lines_date ON _atlas.withholding_tax_lines(created_at);
COMMENT ON TABLE _atlas.withholding_tax_lines IS 'Withholding tax lines recorded from supplier payments';

-- Withholding Tax Certificates
-- Certificates issued to suppliers for tax withheld
CREATE TABLE IF NOT EXISTS _atlas.withholding_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    certificate_number VARCHAR(100) NOT NULL,
    -- Supplier
    supplier_id UUID NOT NULL,
    supplier_number VARCHAR(50),
    supplier_name VARCHAR(200),
    -- Tax type and code
    tax_type VARCHAR(50) NOT NULL,
    tax_code_id UUID NOT NULL REFERENCES _atlas.withholding_tax_codes(id),
    tax_code VARCHAR(50) NOT NULL,
    -- Period covered
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    -- Amounts
    total_invoice_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    total_withheld_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    rate_percentage NUMERIC(12,4) NOT NULL,
    -- Payment references covered by this certificate
    payment_ids JSONB NOT NULL DEFAULT '[]',
    -- Status: draft, issued, acknowledged, cancelled
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    issued_at TIMESTAMPTZ,
    acknowledged_at TIMESTAMPTZ,
    -- Notes
    notes TEXT,
    -- Metadata
    metadata JSONB NOT NULL DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, certificate_number)
);

CREATE INDEX idx_wht_certs_org ON _atlas.withholding_certificates(organization_id);
CREATE INDEX idx_wht_certs_supplier ON _atlas.withholding_certificates(organization_id, supplier_id);
CREATE INDEX idx_wht_certs_status ON _atlas.withholding_certificates(status);
COMMENT ON TABLE _atlas.withholding_certificates IS 'Withholding tax certificates issued to suppliers';
