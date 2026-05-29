-- 162_ar_receipt_reversal.sql
-- Oracle Fusion Financial Feature: AR Receipt Reversal
-- Manages the reversal of cash receipts (e.g., NSF, Stop Payment) and tracking of unapplied invoices.

CREATE TABLE IF NOT EXISTS financials.ar_receipt_reversals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    receipt_id UUID NOT NULL,
    
    -- Oracle Fusion Reversal Categories
    reversal_category VARCHAR(50) NOT NULL, -- e.g., 'NSF', 'STOP_PAYMENT', 'REVERSE_PAYMENT', 'UNAPPLIED'
    reversal_reason_code VARCHAR(100) NOT NULL, 
    
    reversal_date DATE NOT NULL DEFAULT CURRENT_DATE,
    reversal_comments TEXT,
    
    -- Audit fields
    reversed_by UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'COMPLETED',
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(receipt_id)
);

COMMENT ON TABLE financials.ar_receipt_reversals IS 'AR Receipt Reversals - tracks the reversal of customer payments and the reasons for bouncing or reversing';
