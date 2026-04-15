-- Atlas ERP - Tables for Procure-to-Pay and Order-to-Cash workflows

-- ============================================================================
-- Purchase Order Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS scm_purchase_order_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    purchase_order_id UUID NOT NULL REFERENCES scm_purchase_orders(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    product_id UUID REFERENCES scm_products(id),
    description TEXT,
    quantity NUMERIC(12,2) NOT NULL DEFAULT 0,
    unit_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    unit_of_measure TEXT,
    tax_rate NUMERIC(5,2) DEFAULT 0,
    tax_amount NUMERIC(18,2) DEFAULT 0,
    line_total NUMERIC(18,2) NOT NULL DEFAULT 0,
    received_quantity NUMERIC(12,2) DEFAULT 0,
    invoiced_quantity NUMERIC(12,2) DEFAULT 0,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_po_lines_po ON scm_purchase_order_lines(purchase_order_id);

-- ============================================================================
-- Sales Order Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS scm_sales_order_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    sales_order_id UUID NOT NULL REFERENCES scm_sales_orders(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    product_id UUID REFERENCES scm_products(id),
    description TEXT,
    quantity NUMERIC(12,2) NOT NULL DEFAULT 0,
    unit_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    discount_percent NUMERIC(5,2) DEFAULT 0,
    tax_rate NUMERIC(5,2) DEFAULT 0,
    tax_amount NUMERIC(18,2) DEFAULT 0,
    line_total NUMERIC(18,2) NOT NULL DEFAULT 0,
    shipped_quantity NUMERIC(12,2) DEFAULT 0,
    invoiced_quantity NUMERIC(12,2) DEFAULT 0,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_so_lines_so ON scm_sales_order_lines(sales_order_id);

-- ============================================================================
-- Goods Receipts (Receiving)
-- ============================================================================

CREATE TABLE IF NOT EXISTS scm_goods_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'draft',
    receipt_number TEXT NOT NULL,
    purchase_order_id UUID REFERENCES scm_purchase_orders(id),
    supplier_id UUID REFERENCES scm_suppliers(id),
    warehouse_id UUID REFERENCES scm_warehouses(id),
    receipt_date DATE,
    total_quantity NUMERIC(12,2) DEFAULT 0,
    status VARCHAR(50) DEFAULT 'draft',
    notes TEXT
);

CREATE TABLE IF NOT EXISTS scm_goods_receipt_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    goods_receipt_id UUID NOT NULL REFERENCES scm_goods_receipts(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    purchase_order_line_id UUID REFERENCES scm_purchase_order_lines(id),
    product_id UUID REFERENCES scm_products(id),
    warehouse_id UUID REFERENCES scm_warehouses(id),
    quantity_received NUMERIC(12,2) NOT NULL DEFAULT 0,
    quantity_accepted NUMERIC(12,2) DEFAULT 0,
    quantity_rejected NUMERIC(12,2) DEFAULT 0,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_gr_receipt ON scm_goods_receipts(purchase_order_id);
CREATE INDEX IF NOT EXISTS idx_gr_lines_gr ON scm_goods_receipt_lines(goods_receipt_id);

-- ============================================================================
-- Invoice Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_invoice_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    invoice_id UUID NOT NULL REFERENCES fin_invoices(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    product_id UUID REFERENCES scm_products(id),
    description TEXT,
    quantity NUMERIC(12,2) NOT NULL DEFAULT 0,
    unit_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_rate NUMERIC(5,2) DEFAULT 0,
    tax_amount NUMERIC(18,2) DEFAULT 0,
    line_total NUMERIC(18,2) NOT NULL DEFAULT 0,
    reference_type VARCHAR(50),
    reference_id UUID
);

CREATE INDEX IF NOT EXISTS idx_inv_lines_inv ON fin_invoice_lines(invoice_id);

-- ============================================================================
-- Payments
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'draft',
    payment_number TEXT NOT NULL,
    payment_type VARCHAR(50) NOT NULL DEFAULT 'disbursement',
    payment_method VARCHAR(50) DEFAULT 'bank_transfer',
    payer_id UUID,
    payee_id UUID,
    invoice_id UUID REFERENCES fin_invoices(id),
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) DEFAULT 'USD',
    payment_date DATE,
    reference_number TEXT,
    bank_account TEXT,
    status VARCHAR(50) DEFAULT 'draft',
    notes TEXT,
    reconciled BOOLEAN DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_payments_number ON fin_payments(payment_number);
CREATE INDEX IF NOT EXISTS idx_payments_invoice ON fin_payments(invoice_id);

-- ============================================================================
-- Shipments (Order Fulfillment)
-- ============================================================================

CREATE TABLE IF NOT EXISTS scm_shipments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'pending',
    shipment_number TEXT NOT NULL,
    sales_order_id UUID REFERENCES scm_sales_orders(id),
    customer_id UUID,
    warehouse_id UUID REFERENCES scm_warehouses(id),
    shipment_date DATE,
    estimated_delivery DATE,
    actual_delivery DATE,
    carrier TEXT,
    tracking_number TEXT,
    shipping_address JSONB,
    status VARCHAR(50) DEFAULT 'pending',
    notes TEXT
);

CREATE TABLE IF NOT EXISTS scm_shipment_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    shipment_id UUID NOT NULL REFERENCES scm_shipments(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    sales_order_line_id UUID REFERENCES scm_sales_order_lines(id),
    product_id UUID REFERENCES scm_products(id),
    quantity_shipped NUMERIC(12,2) NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_shipments_so ON scm_shipments(sales_order_id);
CREATE INDEX IF NOT EXISTS idx_shipment_lines_ship ON scm_shipment_lines(shipment_id);

-- ============================================================================
-- Journal Entry Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_journal_entry_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    journal_entry_id UUID NOT NULL REFERENCES fin_journal_entries(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    account_id UUID REFERENCES fin_chart_of_accounts(id),
    description TEXT,
    debit_amount NUMERIC(18,2) DEFAULT 0,
    credit_amount NUMERIC(18,2) DEFAULT 0,
    reference_type VARCHAR(50),
    reference_id UUID
);

CREATE INDEX IF NOT EXISTS idx_je_lines_je ON fin_journal_entry_lines(journal_entry_id);
