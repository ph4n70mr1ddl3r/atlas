-- 156_intercompany_balancing_rules.sql
-- Oracle Fusion Financial Feature: Intercompany Balancing Rules
-- Automatically generates Due To / Due From lines to balance journals across different primary balancing segments.

CREATE TABLE IF NOT EXISTS financials.intercompany_balancing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_name VARCHAR(200) NOT NULL,
    -- E.g., 'Primary Balancing Segment', 'Legal Entity', 'Ledger'
    balancing_level VARCHAR(50) NOT NULL DEFAULT 'PRIMARY_BALANCING_SEGMENT',
    from_segment_value VARCHAR(50) NOT NULL,
    to_segment_value VARCHAR(50) NOT NULL,
    -- Account to use for Due From (Receivable)
    receivable_account_ccid UUID NOT NULL,
    -- Account to use for Due To (Payable)
    payable_account_ccid UUID NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    end_date DATE,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, from_segment_value, to_segment_value)
);

COMMENT ON TABLE financials.intercompany_balancing_rules IS 'Intercompany Balancing Rules - configuration for automatic generation of Due To / Due From lines';
