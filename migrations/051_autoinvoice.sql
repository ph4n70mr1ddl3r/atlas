-- 051: AutoInvoice (Oracle Fusion Receivables AutoInvoice)
-- Supports automated invoice creation from external transaction sources
-- with validation rules, grouping rules, and line ordering.

-- AutoInvoice grouping rules
CREATE TABLE IF NOT EXISTS _atlas.autoinvoice_grouping_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    transaction_types JSONB DEFAULT '["invoice", "credit_memo"]'::jsonb,
    group_by_fields JSONB DEFAULT '["bill_to_customer_id", "currency_code", "transaction_type"]'::jsonb,
    line_order_by JSONB DEFAULT '["line_number"]'::jsonb,
    is_default BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    priority INT DEFAULT 10,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, name)
);

-- AutoInvoice validation rules
CREATE TABLE IF NOT EXISTS _atlas.autoinvoice_validation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    field_name VARCHAR(100) NOT NULL,
    validation_type VARCHAR(50) NOT NULL, -- 'required', 'format', 'reference', 'range', 'custom'
    validation_expression TEXT,
    error_message TEXT NOT NULL,
    is_fatal BOOLEAN DEFAULT true,
    transaction_types JSONB DEFAULT '["invoice", "credit_memo", "debit_memo"]'::jsonb,
    is_active BOOLEAN DEFAULT true,
    priority INT DEFAULT 10,
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, name)
);

-- AutoInvoice import batches
CREATE TABLE IF NOT EXISTS _atlas.autoinvoice_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(100) NOT NULL,
    batch_source VARCHAR(100) NOT NULL,
    description TEXT,
    status VARCHAR(30) DEFAULT 'pending',
    total_lines INT DEFAULT 0,
    valid_lines INT DEFAULT 0,
    invalid_lines INT DEFAULT 0,
    invoices_created INT DEFAULT 0,
    invoices_total_amount NUMERIC(18,2) DEFAULT 0,
    grouping_rule_id UUID REFERENCES _atlas.autoinvoice_grouping_rules(id),
    validation_errors JSONB DEFAULT '[]'::jsonb,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

-- AutoInvoice interface lines (raw imported lines)
CREATE TABLE IF NOT EXISTS _atlas.autoinvoice_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL REFERENCES _atlas.autoinvoice_batches(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    source_line_id VARCHAR(200),
    transaction_type VARCHAR(50) DEFAULT 'invoice',
    customer_id UUID,
    customer_number VARCHAR(100),
    customer_name VARCHAR(300),
    bill_to_customer_id UUID,
    bill_to_site_id UUID,
    ship_to_customer_id UUID,
    ship_to_site_id UUID,
    item_code VARCHAR(100),
    item_description TEXT,
    quantity NUMERIC(18,6),
    unit_of_measure VARCHAR(30),
    unit_price NUMERIC(18,4) DEFAULT 0,
    line_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(10) DEFAULT 'USD',
    exchange_rate NUMERIC(18,8),
    transaction_date DATE NOT NULL,
    gl_date DATE NOT NULL,
    due_date DATE,
    revenue_account_code VARCHAR(100),
    receivable_account_code VARCHAR(100),
    tax_code VARCHAR(50),
    tax_amount NUMERIC(18,2),
    sales_rep_id UUID,
    sales_rep_name VARCHAR(200),
    memo_line VARCHAR(500),
    reference_number VARCHAR(200),
    sales_order_number VARCHAR(200),
    sales_order_line VARCHAR(50),
    status VARCHAR(30) DEFAULT 'pending',
    validation_errors JSONB DEFAULT '[]'::jsonb,
    invoice_id UUID,
    invoice_line_number INT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- AutoInvoice result invoices (generated AR invoices)
CREATE TABLE IF NOT EXISTS _atlas.autoinvoice_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_id UUID NOT NULL REFERENCES _atlas.autoinvoice_batches(id),
    invoice_number VARCHAR(100) NOT NULL,
    transaction_type VARCHAR(50) DEFAULT 'invoice',
    customer_id UUID,
    bill_to_customer_id UUID,
    bill_to_site_id UUID,
    ship_to_customer_id UUID,
    ship_to_site_id UUID,
    currency_code VARCHAR(10) DEFAULT 'USD',
    exchange_rate NUMERIC(18,8),
    transaction_date DATE NOT NULL,
    gl_date DATE NOT NULL,
    due_date DATE,
    subtotal NUMERIC(18,2) DEFAULT 0,
    tax_amount NUMERIC(18,2) DEFAULT 0,
    total_amount NUMERIC(18,2) DEFAULT 0,
    line_count INT DEFAULT 0,
    receivable_account_code VARCHAR(100),
    sales_rep_id UUID,
    sales_order_number VARCHAR(200),
    reference_number VARCHAR(200),
    status VARCHAR(30) DEFAULT 'draft',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, invoice_number)
);

-- AutoInvoice result lines
CREATE TABLE IF NOT EXISTS _atlas.autoinvoice_result_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    invoice_id UUID NOT NULL REFERENCES _atlas.autoinvoice_results(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    source_line_id VARCHAR(200),
    item_code VARCHAR(100),
    item_description TEXT,
    quantity NUMERIC(18,6),
    unit_of_measure VARCHAR(30),
    unit_price NUMERIC(18,4) DEFAULT 0,
    line_amount NUMERIC(18,2) DEFAULT 0,
    tax_code VARCHAR(50),
    tax_amount NUMERIC(18,2),
    revenue_account_code VARCHAR(100),
    sales_order_number VARCHAR(200),
    sales_order_line VARCHAR(50),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_ai_lines_batch ON _atlas.autoinvoice_lines(batch_id);
CREATE INDEX IF NOT EXISTS idx_ai_lines_status ON _atlas.autoinvoice_lines(status);
CREATE INDEX IF NOT EXISTS idx_ai_lines_customer ON _atlas.autoinvoice_lines(bill_to_customer_id);
CREATE INDEX IF NOT EXISTS idx_ai_batches_status ON _atlas.autoinvoice_batches(status);
CREATE INDEX IF NOT EXISTS idx_ai_batches_org ON _atlas.autoinvoice_batches(organization_id);
CREATE INDEX IF NOT EXISTS idx_ai_results_batch ON _atlas.autoinvoice_results(batch_id);
CREATE INDEX IF NOT EXISTS idx_ai_results_invoice ON _atlas.autoinvoice_results(invoice_number);
CREATE INDEX IF NOT EXISTS idx_ai_result_lines_invoice ON _atlas.autoinvoice_result_lines(invoice_id);
