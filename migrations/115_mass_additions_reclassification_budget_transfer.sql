-- Migration: Mass Additions, Asset Reclassification, GL Budget Transfer,
-- Payment Format, Financial Dimension Set, Receipt Write-Off, Prepayment Application
-- Oracle Fusion Cloud ERP Financial Features

-- Mass Additions (Oracle Fusion: Fixed Assets > Mass Additions)
CREATE TABLE IF NOT EXISTS _atlas.fin_mass_additions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    mass_addition_number VARCHAR(50) NOT NULL,
    invoice_id UUID,
    invoice_number VARCHAR(50),
    invoice_line_id UUID,
    invoice_line_number INTEGER,
    description TEXT,
    asset_key VARCHAR(100),
    category_id UUID,
    category_code VARCHAR(50),
    book_id UUID,
    book_code VARCHAR(50),
    asset_type VARCHAR(30),
    depreciation_method VARCHAR(50),
    useful_life_months INTEGER,
    cost NUMERIC(18,2) DEFAULT 0,
    salvage_value NUMERIC(18,2) DEFAULT 0,
    salvage_value_percent VARCHAR(20),
    asset_account_code VARCHAR(100),
    depr_expense_account_code VARCHAR(100),
    location VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    supplier_id UUID,
    supplier_number VARCHAR(50),
    supplier_name VARCHAR(200),
    po_number VARCHAR(50),
    invoice_date DATE,
    date_placed_in_service DATE,
    merge_to_id UUID,
    merge_to_number VARCHAR(50),
    reject_reason TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'posted',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_mass_additions_org ON _atlas.fin_mass_additions(organization_id);
CREATE INDEX idx_mass_additions_status ON _atlas.fin_mass_additions(status);
CREATE INDEX idx_mass_additions_invoice ON _atlas.fin_mass_additions(invoice_id);
CREATE UNIQUE INDEX idx_mass_additions_number ON _atlas.fin_mass_additions(organization_id, mass_addition_number);

-- Asset Reclassification (Oracle Fusion: Fixed Assets > Asset Reclassification)
CREATE TABLE IF NOT EXISTS _atlas.fin_asset_reclassifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    reclassification_number VARCHAR(50) NOT NULL,
    asset_id UUID NOT NULL,
    asset_number VARCHAR(50),
    asset_name VARCHAR(200),
    reclassification_type VARCHAR(50) NOT NULL,
    reason TEXT,
    from_category_id UUID,
    from_category_code VARCHAR(50),
    from_asset_type VARCHAR(30),
    from_depreciation_method VARCHAR(50),
    from_useful_life_months INTEGER,
    from_asset_account_code VARCHAR(100),
    from_depr_expense_account_code VARCHAR(100),
    to_category_id UUID,
    to_category_code VARCHAR(50),
    to_asset_type VARCHAR(30),
    to_depreciation_method VARCHAR(50),
    to_useful_life_months INTEGER,
    to_asset_account_code VARCHAR(100),
    to_depr_expense_account_code VARCHAR(100),
    effective_date DATE NOT NULL,
    amortization_adjustment VARCHAR(50),
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    approved_by UUID,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_asset_reclass_org ON _atlas.fin_asset_reclassifications(organization_id);
CREATE INDEX idx_asset_reclass_status ON _atlas.fin_asset_reclassifications(status);
CREATE INDEX idx_asset_reclass_asset ON _atlas.fin_asset_reclassifications(asset_id);
CREATE UNIQUE INDEX idx_asset_reclass_number ON _atlas.fin_asset_reclassifications(organization_id, reclassification_number);

-- GL Budget Transfer (Oracle Fusion: General Ledger > Budget Transfers)
CREATE TABLE IF NOT EXISTS _atlas.fin_gl_budget_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    transfer_number VARCHAR(50) NOT NULL,
    description TEXT,
    transfer_date DATE NOT NULL,
    effective_date DATE NOT NULL,
    budget_name VARCHAR(200),
    transfer_type VARCHAR(50) NOT NULL,
    from_account_combination VARCHAR(500),
    from_department VARCHAR(200),
    from_period VARCHAR(50),
    to_account_combination VARCHAR(500),
    to_department VARCHAR(200),
    to_period VARCHAR(50),
    transfer_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    reason TEXT,
    approved_by UUID,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_gl_budget_transfer_org ON _atlas.fin_gl_budget_transfers(organization_id);
CREATE INDEX idx_gl_budget_transfer_status ON _atlas.fin_gl_budget_transfers(status);
CREATE UNIQUE INDEX idx_gl_budget_transfer_number ON _atlas.fin_gl_budget_transfers(organization_id, transfer_number);

-- Payment Format (Oracle Fusion: Payables > Payment Formats)
CREATE TABLE IF NOT EXISTS _atlas.fin_payment_formats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    format_type VARCHAR(30) NOT NULL,
    payment_method VARCHAR(30) NOT NULL,
    file_template VARCHAR(500),
    requires_bank_details BOOLEAN DEFAULT false,
    supports_remittance BOOLEAN DEFAULT true,
    supports_void BOOLEAN DEFAULT true,
    max_payments_per_file INTEGER,
    currency_code VARCHAR(3) DEFAULT 'USD',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    metadata JSONB DEFAULT '{}'
);

CREATE UNIQUE INDEX idx_payment_format_code ON _atlas.fin_payment_formats(organization_id, code);

-- Financial Dimension Set (Oracle Fusion: GL > Financial Dimension Sets)
CREATE TABLE IF NOT EXISTS _atlas.fin_financial_dimension_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    dimension_members JSONB DEFAULT '[]',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    metadata JSONB DEFAULT '{}'
);

CREATE UNIQUE INDEX idx_fin_dim_set_code ON _atlas.fin_financial_dimension_sets(organization_id, code);

-- Financial Dimension Set Members
CREATE TABLE IF NOT EXISTS _atlas.fin_financial_dimension_set_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    dimension_set_id UUID NOT NULL REFERENCES _atlas.fin_financial_dimension_sets(id),
    dimension_id UUID NOT NULL,
    dimension_code VARCHAR(50),
    dimension_value_id UUID NOT NULL,
    dimension_value_code VARCHAR(100),
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_fin_dim_set_member_set ON _atlas.fin_financial_dimension_set_members(dimension_set_id);

-- Receipt Write-Off (Oracle Fusion: Receivables > Receipt Write-Off)
CREATE TABLE IF NOT EXISTS _atlas.fin_receipt_write_offs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    write_off_number VARCHAR(50) NOT NULL,
    receipt_id UUID NOT NULL,
    receipt_number VARCHAR(50),
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    write_off_type VARCHAR(30) NOT NULL,
    write_off_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    write_off_account_code VARCHAR(100),
    receivable_account_code VARCHAR(100),
    write_off_date DATE NOT NULL,
    gl_date DATE NOT NULL,
    reason_code VARCHAR(50),
    reason_description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    approved_by UUID,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_receipt_write_off_org ON _atlas.fin_receipt_write_offs(organization_id);
CREATE INDEX idx_receipt_write_off_status ON _atlas.fin_receipt_write_offs(status);
CREATE INDEX idx_receipt_write_off_receipt ON _atlas.fin_receipt_write_offs(receipt_id);
CREATE UNIQUE INDEX idx_receipt_write_off_number ON _atlas.fin_receipt_write_offs(organization_id, write_off_number);

-- Prepayment Application (Oracle Fusion: Payables > Prepayment Application)
CREATE TABLE IF NOT EXISTS _atlas.fin_prepayment_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    application_number VARCHAR(50) NOT NULL,
    prepayment_invoice_id UUID NOT NULL,
    prepayment_invoice_number VARCHAR(50),
    standard_invoice_id UUID NOT NULL,
    standard_invoice_number VARCHAR(50),
    supplier_id UUID NOT NULL,
    supplier_number VARCHAR(50),
    applied_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    remaining_prepayment_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    application_date DATE NOT NULL,
    gl_date DATE NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    reason TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_prepayment_app_org ON _atlas.fin_prepayment_applications(organization_id);
CREATE INDEX idx_prepayment_app_status ON _atlas.fin_prepayment_applications(status);
CREATE INDEX idx_prepayment_app_supplier ON _atlas.fin_prepayment_applications(supplier_id);
CREATE UNIQUE INDEX idx_prepayment_app_number ON _atlas.fin_prepayment_applications(organization_id, application_number);
