-- 136_invoice_matching.sql
-- Oracle Fusion Financial Feature: Invoice Matching (2-way, 3-way, 4-way)
-- Matches AP invoices against purchase orders, receipts, and inspections.

-- ============================================================================
-- Invoice Match Header
-- Oracle Fusion: Financials > Payables > Invoice Matching
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.invoice_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    match_number VARCHAR(50) NOT NULL,
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(50),
    purchase_order_id UUID NOT NULL,
    po_number VARCHAR(50),
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(300) NOT NULL,
    match_type VARCHAR(20) NOT NULL DEFAULT 'two_way',
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    invoice_amount NUMERIC(18,2) NOT NULL,
    po_amount NUMERIC(18,2) NOT NULL,
    receipt_amount NUMERIC(18,2),
    inspection_amount NUMERIC(18,2),
    price_tolerance_pct NUMERIC(8,4) NOT NULL DEFAULT 2.0000,
    quantity_tolerance_pct NUMERIC(8,4) NOT NULL DEFAULT 5.0000,
    amount_tolerance NUMERIC(18,2) NOT NULL DEFAULT 100.00,
    price_variance NUMERIC(18,4),
    quantity_variance NUMERIC(18,4),
    amount_variance NUMERIC(18,2),
    variance_reason TEXT,
    receipt_id UUID,
    receipt_number VARCHAR(50),
    inspection_id UUID,
    inspection_status VARCHAR(30),
    hold_reason TEXT,
    hold_at TIMESTAMPTZ,
    hold_by UUID,
    override_reason TEXT,
    overridden_at TIMESTAMPTZ,
    overridden_by UUID,
    matched_at TIMESTAMPTZ,
    matched_by UUID,
    cancelled_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, match_number)
);

CREATE INDEX idx_invoice_matches_org ON financials.invoice_matches(organization_id);
CREATE INDEX idx_invoice_matches_invoice ON financials.invoice_matches(invoice_id);
CREATE INDEX idx_invoice_matches_po ON financials.invoice_matches(purchase_order_id);
CREATE INDEX idx_invoice_matches_supplier ON financials.invoice_matches(supplier_id);
CREATE INDEX idx_invoice_matches_status ON financials.invoice_matches(status);
CREATE INDEX idx_invoice_matches_type ON financials.invoice_matches(match_type);

-- ============================================================================
-- Invoice Match Lines (line-level matching detail)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.invoice_match_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    match_id UUID NOT NULL REFERENCES financials.invoice_matches(id) ON DELETE CASCADE,
    invoice_line_id UUID,
    po_line_id UUID,
    receipt_line_id UUID,
    inspection_line_id UUID,
    line_number INT NOT NULL,
    item_description TEXT,
    invoice_quantity NUMERIC(18,4) NOT NULL,
    po_quantity NUMERIC(18,4) NOT NULL,
    receipt_quantity NUMERIC(18,4),
    inspected_quantity NUMERIC(18,4),
    invoice_unit_price NUMERIC(18,4) NOT NULL,
    po_unit_price NUMERIC(18,4) NOT NULL,
    invoice_line_amount NUMERIC(18,2) NOT NULL,
    po_line_amount NUMERIC(18,2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'unmatched',
    price_variance NUMERIC(18,4),
    quantity_variance NUMERIC(18,4),
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_invoice_match_lines_match ON financials.invoice_match_lines(match_id);
CREATE INDEX idx_invoice_match_lines_status ON financials.invoice_match_lines(status);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.invoice_matching_dashboard AS
SELECT
    organization_id,
    COUNT(*) AS total_matches,
    COUNT(*) FILTER (WHERE status = 'matched') AS matched_count,
    COUNT(*) FILTER (WHERE status = 'pending') AS pending_count,
    COUNT(*) FILTER (WHERE status = 'exception') AS exception_count,
    COUNT(*) FILTER (WHERE status = 'overridden') AS overridden_count,
    COALESCE(SUM(invoice_amount), 0) AS total_invoice_amount,
    COALESCE(SUM(ABS(COALESCE(amount_variance, 0))), 0) AS total_variance_amount
FROM financials.invoice_matches
GROUP BY organization_id;

COMMENT ON TABLE financials.invoice_matches IS 'Invoice matching records - 2-way, 3-way, and 4-way matching of AP invoices to POs/receipts/inspections';
COMMENT ON TABLE financials.invoice_match_lines IS 'Line-level detail for invoice matching';
COMMENT ON VIEW financials.invoice_matching_dashboard IS 'Dashboard aggregation for invoice matching statistics';
