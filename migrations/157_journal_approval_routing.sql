-- 157_journal_approval_routing.sql
-- Oracle Fusion Financial Feature: Journal Approval Routing
-- Manages routing rules and thresholds for General Ledger Journal Approvals.

CREATE TABLE IF NOT EXISTS financials.gl_journal_approval_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_name VARCHAR(200) NOT NULL,
    ledger_id UUID, -- If null, applies to all ledgers for the org
    source_name VARCHAR(100), -- E.g., 'Manual', 'Spreadsheet', 'Payables'
    
    -- Thresholds
    min_amount DECIMAL(15,2) DEFAULT 0.00,
    max_amount DECIMAL(15,2), -- If null, unbounded
    
    -- Routing
    approver_user_id UUID, -- Specific user
    approver_role VARCHAR(100), -- Or role based

    is_auto_approved BOOLEAN NOT NULL DEFAULT false,
    
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, rule_name)
);

CREATE TABLE IF NOT EXISTS financials.gl_journal_approval_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    journal_batch_id UUID NOT NULL,
    rule_applied_id UUID REFERENCES financials.gl_journal_approval_rules(id),
    action VARCHAR(50) NOT NULL, -- 'SUBMITTED', 'APPROVED', 'REJECTED', 'AUTO_APPROVED'
    action_by UUID, -- User who performed action
    comments TEXT,
    action_date TIMESTAMPTZ DEFAULT now()
);

COMMENT ON TABLE financials.gl_journal_approval_rules IS 'General Ledger Journal Approval Rules - routing and threshold definitions';
COMMENT ON TABLE financials.gl_journal_approval_history IS 'Audit trail for GL Journal Approval actions';
