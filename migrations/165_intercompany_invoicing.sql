-- 165_intercompany_invoicing.sql
-- Oracle Fusion Financial Feature: Intercompany Invoicing
-- Generates matching AR and AP invoices for intercompany (cross-charge) transactions.

CREATE TABLE IF NOT EXISTS financials.intercompany_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- The provider (seller) organization
    provider_org_id UUID NOT NULL,
    
    -- The receiver (buyer) organization
    receiver_org_id UUID NOT NULL,
    
    transaction_date DATE NOT NULL DEFAULT CURRENT_DATE,
    transaction_amount DECIMAL(15,2) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    
    description TEXT,
    
    -- Status of the intercompany transaction
    status VARCHAR(20) NOT NULL DEFAULT 'NEW', -- 'NEW', 'INVOICED', 'FAILED'
    
    -- References to the auto-generated invoices
    ar_invoice_id UUID, -- Receivables invoice in the provider org
    ap_invoice_id UUID, -- Payables invoice in the receiver org
    
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

COMMENT ON TABLE financials.intercompany_transactions IS 'Intercompany Transactions - records source cross-charges that require corresponding AR/AP invoices';
