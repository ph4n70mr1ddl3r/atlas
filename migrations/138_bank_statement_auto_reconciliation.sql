-- ============================================================================
-- Migration 138: Bank Statement Auto-Reconciliation Engine
-- Oracle Fusion: Cash Management > Bank Statements > Auto-Reconciliation
--
-- Provides bank statement import, auto-matching to system transactions,
-- reconciliation matching rules, and exception management.
-- ============================================================================

-- Bank statement header (represents a single bank statement from the bank)
CREATE TABLE IF NOT EXISTS fin_bank_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_number VARCHAR(50) NOT NULL,
    bank_account_id UUID NOT NULL,
    bank_account_number VARCHAR(50) NOT NULL,
    bank_name VARCHAR(200),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    statement_date DATE NOT NULL,
    opening_balance DECIMAL(18,2) NOT NULL DEFAULT 0,
    closing_balance DECIMAL(18,2) NOT NULL DEFAULT 0,
    statement_total_credits DECIMAL(18,2) NOT NULL DEFAULT 0,
    statement_total_debits DECIMAL(18,2) NOT NULL DEFAULT 0,
    number_of_lines INTEGER NOT NULL DEFAULT 0,
    -- Status: imported, validating, validated, reconciling, reconciled, exception, cancelled
    status VARCHAR(30) NOT NULL DEFAULT 'imported',
    -- Import source: mt940, bai2, ofx, csv, manual, api
    import_source VARCHAR(20) NOT NULL DEFAULT 'manual',
    original_file_name VARCHAR(500),
    -- Reconciliation summary
    matched_lines INTEGER NOT NULL DEFAULT 0,
    unmatched_lines INTEGER NOT NULL DEFAULT 0,
    partially_matched_lines INTEGER NOT NULL DEFAULT 0,
    reconciliation_difference DECIMAL(18,2),
    reconciled_at TIMESTAMPTZ,
    reconciled_by UUID,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, bank_account_id, statement_number, statement_date)
);

-- Bank statement lines (individual transactions on the statement)
CREATE TABLE IF NOT EXISTS fin_bank_statement_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_id UUID NOT NULL REFERENCES fin_bank_statements(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    -- Transaction details from the bank
    transaction_date DATE NOT NULL,
    value_date DATE,
    amount DECIMAL(18,2) NOT NULL,
    -- Type: credit (money in), debit (money out)
    transaction_type VARCHAR(10) NOT NULL,
    -- Bank reference information
    bank_reference VARCHAR(100),
    customer_reference VARCHAR(100),
    -- Description/memo from the bank
    description TEXT,
    -- Additional identification
    cheque_number VARCHAR(50),
    transaction_code VARCHAR(20),
    -- Matching status: unmatched, matched, partially_matched, exception, manually_matched, excluded
    match_status VARCHAR(30) NOT NULL DEFAULT 'unmatched',
    -- Matched system transaction reference
    matched_transaction_type VARCHAR(30), -- receipt, payment, journal_entry, bank_charge, other
    matched_transaction_id UUID,
    match_rule_used VARCHAR(100),
    match_confidence DECIMAL(5,2),
    match_date TIMESTAMPTZ,
    matched_by UUID,
    -- Audit
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(statement_id, line_number)
);

-- Reconciliation matching rules (configurable rules for auto-matching)
CREATE TABLE IF NOT EXISTS fin_reconciliation_matching_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_name VARCHAR(100) NOT NULL,
    rule_description TEXT,
    -- Priority (lower = higher priority)
    priority INTEGER NOT NULL DEFAULT 100,
    -- Matching strategy: exact_amount, amount_tolerance, reference_match,
    -- date_range, combined_amount_reference, combined_amount_date, fuzzy_match
    match_strategy VARCHAR(50) NOT NULL,
    -- Matching criteria (JSONB for flexible configuration)
    -- Example: {"amount_tolerance_pct": 0.5, "date_tolerance_days": 3,
    --           "reference_field": "customer_reference", "match_case": false}
    match_criteria JSONB NOT NULL DEFAULT '{}',
    -- Target transaction types to match against
    -- Array of: receipt, payment, journal_entry, bank_charge, all
    target_transaction_types JSONB NOT NULL DEFAULT '["all"]',
    -- Whether the rule is active
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- Oracle Fusion: Cash Management > Bank Statements > Matching Rules
    -- Auto-match mode: auto (apply automatically), suggest (suggest for manual review)
    auto_apply BOOLEAN NOT NULL DEFAULT true,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, rule_name)
);

-- Reconciliation exceptions (items that could not be auto-matched)
CREATE TABLE IF NOT EXISTS fin_reconciliation_exceptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_id UUID NOT NULL REFERENCES fin_bank_statements(id),
    statement_line_id UUID NOT NULL REFERENCES fin_bank_statement_lines(id),
    bank_account_id UUID NOT NULL,
    -- Exception details
    exception_type VARCHAR(50) NOT NULL, -- unmatched, multiple_match, amount_mismatch, date_out_of_range
    exception_reason TEXT,
    -- Potential matches found (JSONB array of match candidates)
    potential_matches JSONB,
    -- Resolution: open, resolved_matched, resolved_write_off, resolved_excluded, resolved_adjustment
    resolution_status VARCHAR(30) NOT NULL DEFAULT 'open',
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    -- Assigned reviewer
    assigned_to UUID,
    -- Audit
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Reconciliation audit trail
CREATE TABLE IF NOT EXISTS fin_reconciliation_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_id UUID NOT NULL REFERENCES fin_bank_statements(id),
    -- Action: imported, validated, auto_matched, manually_matched, exception_created,
    --         exception_resolved, reconciliation_completed, reconciliation_reversed
    action VARCHAR(50) NOT NULL,
    action_details JSONB,
    -- Records affected
    lines_affected INTEGER DEFAULT 0,
    -- Who performed the action
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_bank_stmt_org ON fin_bank_statements(organization_id);
CREATE INDEX IF NOT EXISTS idx_bank_stmt_account_date ON fin_bank_statements(bank_account_id, statement_date);
CREATE INDEX IF NOT EXISTS idx_bank_stmt_status ON fin_bank_statements(status);
CREATE INDEX IF NOT EXISTS idx_bank_stmt_line_statement ON fin_bank_statement_lines(statement_id);
CREATE INDEX IF NOT EXISTS idx_bank_stmt_line_match ON fin_bank_statement_lines(match_status);
CREATE INDEX IF NOT EXISTS idx_bank_stmt_line_date ON fin_bank_statement_lines(transaction_date);
CREATE INDEX IF NOT EXISTS idx_recon_rule_org ON fin_reconciliation_matching_rules(organization_id, priority);
CREATE INDEX IF NOT EXISTS idx_recon_rule_active ON fin_reconciliation_matching_rules(is_active);
CREATE INDEX IF NOT EXISTS idx_recon_exception_org ON fin_reconciliation_exceptions(organization_id);
CREATE INDEX IF NOT EXISTS idx_recon_exception_status ON fin_reconciliation_exceptions(resolution_status);
CREATE INDEX IF NOT EXISTS idx_recon_audit_stmt ON fin_reconciliation_audit(statement_id);
