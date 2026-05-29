-- 159_autoaccounting_rules.sql
-- Oracle Fusion Financial Feature: Receivables AutoAccounting Rules
-- Dynamically derives GL account segments for AR transactions based on predefined rules.

CREATE TABLE IF NOT EXISTS financials.ar_autoaccounting_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Account class: e.g., 'REVENUE', 'RECEIVABLE', 'FREIGHT', 'TAX'
    account_class VARCHAR(50) NOT NULL,
    -- The specific GL segment being derived, e.g., 'COMPANY', 'COST_CENTER', 'ACCOUNT'
    segment_name VARCHAR(50) NOT NULL,
    
    -- The source to derive the segment from: 
    -- 'CONSTANT', 'TRANSACTION_TYPE', 'SALESPERSON', 'STANDARD_LINE', 'TAXES'
    source_type VARCHAR(50) NOT NULL,
    
    -- If source_type is 'CONSTANT', this holds the value
    constant_value VARCHAR(50),
    
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, account_class, segment_name)
);

COMMENT ON TABLE financials.ar_autoaccounting_rules IS 'AutoAccounting Rules - configuration for dynamically generating GL combinations for AR transactions';
