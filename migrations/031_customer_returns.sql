-- Customer Returns Management / Return Material Authorization (RMA)
-- Oracle Fusion Cloud ERP: Order Management > Returns
-- Migration 031

-- Return reason codes
CREATE TABLE IF NOT EXISTS _atlas.return_reasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    return_type VARCHAR(30) NOT NULL DEFAULT 'standard_return',
    default_disposition VARCHAR(30),
    requires_approval BOOLEAN DEFAULT false,
    credit_issued_automatically BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Return Material Authorizations (RMA headers)
CREATE TABLE IF NOT EXISTS _atlas.return_authorizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rma_number VARCHAR(50) NOT NULL,
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    return_type VARCHAR(30) NOT NULL DEFAULT 'standard_return',
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    reason_code VARCHAR(50),
    reason_name VARCHAR(200),
    original_order_number VARCHAR(50),
    original_order_id UUID,
    customer_contact VARCHAR(200),
    customer_email VARCHAR(200),
    customer_phone VARCHAR(50),
    return_date DATE NOT NULL DEFAULT CURRENT_DATE,
    expected_receipt_date DATE,
    total_quantity NUMERIC(18,2) DEFAULT 0,
    total_amount NUMERIC(18,2) DEFAULT 0,
    total_credit_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    notes TEXT,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    credit_memo_id UUID,
    credit_memo_number VARCHAR(50),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, rma_number)
);

-- RMA line items
CREATE TABLE IF NOT EXISTS _atlas.return_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rma_id UUID NOT NULL REFERENCES _atlas.return_authorizations(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    item_id UUID,
    item_code VARCHAR(50),
    item_description TEXT,
    original_line_id UUID,
    original_quantity NUMERIC(18,2) DEFAULT 0,
    return_quantity NUMERIC(18,2) NOT NULL,
    unit_price NUMERIC(18,2) DEFAULT 0,
    return_amount NUMERIC(18,2) DEFAULT 0,
    credit_amount NUMERIC(18,2) DEFAULT 0,
    reason_code VARCHAR(50),
    disposition VARCHAR(30),
    lot_number VARCHAR(50),
    serial_number VARCHAR(100),
    condition VARCHAR(30),
    received_quantity NUMERIC(18,2) DEFAULT 0,
    received_date DATE,
    inspection_status VARCHAR(30) DEFAULT 'pending',
    inspection_notes TEXT,
    credit_status VARCHAR(20) DEFAULT 'pending',
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Credit memos
CREATE TABLE IF NOT EXISTS _atlas.credit_memos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    credit_memo_number VARCHAR(50) NOT NULL,
    rma_id UUID REFERENCES _atlas.return_authorizations(id),
    rma_number VARCHAR(50),
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    applied_amount NUMERIC(18,2) DEFAULT 0,
    remaining_amount NUMERIC(18,2) DEFAULT 0,
    issue_date DATE,
    applied_to_invoice_id UUID,
    applied_to_invoice_number VARCHAR(50),
    gl_account_code VARCHAR(50),
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, credit_memo_number)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_return_reasons_org ON _atlas.return_reasons(organization_id);
CREATE INDEX IF NOT EXISTS idx_return_reasons_type ON _atlas.return_reasons(organization_id, return_type);
CREATE INDEX IF NOT EXISTS idx_rma_org ON _atlas.return_authorizations(organization_id);
CREATE INDEX IF NOT EXISTS idx_rma_customer ON _atlas.return_authorizations(customer_id);
CREATE INDEX IF NOT EXISTS idx_rma_status ON _atlas.return_authorizations(status);
CREATE INDEX IF NOT EXISTS idx_return_lines_rma ON _atlas.return_lines(rma_id);
CREATE INDEX IF NOT EXISTS idx_credit_memos_org ON _atlas.credit_memos(organization_id);
CREATE INDEX IF NOT EXISTS idx_credit_memos_customer ON _atlas.credit_memos(customer_id);
CREATE INDEX IF NOT EXISTS idx_credit_memos_status ON _atlas.credit_memos(status);
CREATE INDEX IF NOT EXISTS idx_credit_memos_rma ON _atlas.credit_memos(rma_id);
