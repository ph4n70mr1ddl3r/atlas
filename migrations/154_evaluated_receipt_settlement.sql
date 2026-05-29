-- 154_evaluated_receipt_settlement.sql
-- Oracle Fusion Financial Feature: Evaluated Receipt Settlement (ERS)
-- Also known as "Pay on Receipt". Automatically generates AP invoices from PO receipts.

CREATE TABLE IF NOT EXISTS financials.ers_processing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    supplier_id UUID NOT NULL,
    supplier_site_id UUID NOT NULL,
    -- 'RECEIPT' means generate invoice upon receiving. 'USE_MATCHING' means wait for further matching.
    aging_period_days INT NOT NULL DEFAULT 0,
    ers_enabled BOOLEAN NOT NULL DEFAULT true,
    invoice_currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    payment_terms_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, supplier_id, supplier_site_id)
);

CREATE TABLE IF NOT EXISTS financials.ers_invoice_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    receipt_transaction_id UUID NOT NULL,
    purchase_order_id UUID NOT NULL,
    supplier_id UUID NOT NULL,
    generated_invoice_id UUID,
    invoice_amount DECIMAL(15,2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING', -- 'PENDING', 'PROCESSED', 'FAILED'
    error_reason TEXT,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);

COMMENT ON TABLE financials.ers_processing_rules IS 'ERS Processing Rules - configuration for which suppliers have ERS/Pay On Receipt enabled';
COMMENT ON TABLE financials.ers_invoice_history IS 'ERS Invoice History - audit trail of AP invoices automatically generated from PO receipts';
