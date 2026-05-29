-- 158_cash_receipt_automatch_rules.sql
-- Oracle Fusion Financial Feature: Cash Receipt AutoMatch Rules
-- Manages rules for automatically applying incoming cash receipts to open Receivables invoices.

CREATE TABLE IF NOT EXISTS financials.ar_automatch_rule_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_set_name VARCHAR(200) NOT NULL,
    
    -- Matching Criteria Thresholds
    -- Minimum percentage of the receipt amount that must be matched before auto-applying
    min_match_percentage DECIMAL(5,2) NOT NULL DEFAULT 100.00,
    
    -- Matching stringency
    match_by_invoice_number BOOLEAN NOT NULL DEFAULT true,
    match_by_purchase_order BOOLEAN NOT NULL DEFAULT false,
    match_by_sales_order BOOLEAN NOT NULL DEFAULT false,
    
    -- Actions if exact match isn't met but is within tolerance
    unapplied_handling VARCHAR(50) NOT NULL DEFAULT 'LEAVE_UNAPPLIED', -- 'LEAVE_UNAPPLIED', 'ON_ACCOUNT', 'REFUND'
    
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, rule_set_name)
);

CREATE TABLE IF NOT EXISTS financials.ar_automatch_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    receipt_id UUID NOT NULL,
    rule_set_applied_id UUID REFERENCES financials.ar_automatch_rule_sets(id),
    matched_invoice_id UUID,
    applied_amount DECIMAL(15,2) NOT NULL,
    status VARCHAR(50) NOT NULL, -- 'APPLIED', 'PARTIAL', 'UNAPPLIED'
    created_at TIMESTAMPTZ DEFAULT now()
);

COMMENT ON TABLE financials.ar_automatch_rule_sets IS 'Receivables AutoMatch Rule Sets - defines criteria for auto-applying cash receipts';
COMMENT ON TABLE financials.ar_automatch_history IS 'Audit trail of AutoMatch attempts and outcomes for cash receipts';
