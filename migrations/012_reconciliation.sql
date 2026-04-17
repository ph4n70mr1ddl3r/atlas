-- Atlas ERP - Bank Reconciliation
-- Oracle Fusion Cloud ERP: Cash Management > Bank Statements and Reconciliation
--
-- Manages bank accounts, bank statement imports, system transaction
-- matching (auto and manual), and reconciliation tracking.
--
-- This is a standard Oracle Fusion Cloud ERP feature required for:
-- - Matching bank statement lines to AP payments, AR receipts, and GL entries
-- - Auto-matching by amount, date, and reference
-- - Tracking reconciliation status per bank account / period
-- - Generating reconciliation reports and summaries

-- ============================================================================
-- Bank Accounts
-- Oracle Fusion: Cash Management > Bank Accounts
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.bank_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Account identification
    account_number VARCHAR(50) NOT NULL,
    account_name VARCHAR(200) NOT NULL,
    -- Bank details
    bank_name VARCHAR(200) NOT NULL,
    bank_code VARCHAR(50),
    branch_name VARCHAR(200),
    branch_code VARCHAR(50),
    -- GL Account linkage
    gl_account_code VARCHAR(50),
    -- Currency
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Account type: 'checking', 'savings', 'payroll', 'escrow'
    account_type VARCHAR(30) NOT NULL DEFAULT 'checking',
    -- Current balance (from last reconciled statement)
    last_statement_balance NUMERIC(18,2) DEFAULT 0,
    last_statement_date DATE,
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(organization_id, account_number)
);

CREATE INDEX idx_bank_accounts_org ON _atlas.bank_accounts(organization_id);
CREATE INDEX idx_bank_accounts_active ON _atlas.bank_accounts(organization_id, is_active) WHERE is_active = true;

-- ============================================================================
-- Bank Statements
-- Oracle Fusion: Cash Management > Bank Statements
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.bank_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    bank_account_id UUID NOT NULL REFERENCES _atlas.bank_accounts(id),
    -- Statement identification
    statement_number VARCHAR(100) NOT NULL,
    -- Statement date range
    statement_date DATE NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    -- Balance information
    opening_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    closing_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Computed totals
    total_deposits NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_withdrawals NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_interest NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_charges NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Line counts
    total_lines INT NOT NULL DEFAULT 0,
    matched_lines INT NOT NULL DEFAULT 0,
    unmatched_lines INT NOT NULL DEFAULT 0,
    -- Status: 'draft', 'imported', 'in_review', 'reconciled', 'error'
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    -- Reconciliation progress
    reconciliation_percent NUMERIC(5,2) DEFAULT 0,
    -- Audit
    imported_by UUID,
    reviewed_by UUID,
    reconciled_by UUID,
    reconciled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(organization_id, bank_account_id, statement_number)
);

CREATE INDEX idx_bank_statements_org ON _atlas.bank_statements(organization_id);
CREATE INDEX idx_bank_statements_account ON _atlas.bank_statements(bank_account_id);
CREATE INDEX idx_bank_statements_status ON _atlas.bank_statements(status);
CREATE INDEX idx_bank_statements_date ON _atlas.bank_statements(statement_date);

-- ============================================================================
-- Bank Statement Lines
-- Oracle Fusion: Individual line items within a bank statement
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.bank_statement_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_id UUID NOT NULL REFERENCES _atlas.bank_statements(id) ON DELETE CASCADE,
    -- Line identification
    line_number INT NOT NULL,
    -- Transaction details
    transaction_date DATE NOT NULL,
    -- Type: 'deposit', 'withdrawal', 'interest', 'charge', 'transfer_in', 'transfer_out', 'adjustment'
    transaction_type VARCHAR(30) NOT NULL,
    -- Amount (positive for deposits, negative for withdrawals)
    amount NUMERIC(18,2) NOT NULL,
    -- Description / memo from the bank
    description TEXT,
    -- Reference number from bank (check number, ACH reference, etc.)
    reference_number VARCHAR(100),
    -- Check number (for checks)
    check_number VARCHAR(50),
    -- Counterparty information
    counterparty_name VARCHAR(200),
    counterparty_account VARCHAR(50),
    -- Matching status: 'unmatched', 'matched', 'partially_matched', 'manually_matched', 'excluded'
    match_status VARCHAR(30) NOT NULL DEFAULT 'unmatched',
    -- Match metadata
    matched_by UUID,
    matched_at TIMESTAMPTZ,
    match_method VARCHAR(30), -- 'auto_amount_date', 'auto_reference', 'auto_check', 'manual'
    -- Audit
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(statement_id, line_number)
);

CREATE INDEX idx_stmt_lines_statement ON _atlas.bank_statement_lines(statement_id);
CREATE INDEX idx_stmt_lines_status ON _atlas.bank_statement_lines(match_status);
CREATE INDEX idx_stmt_lines_date ON _atlas.bank_statement_lines(transaction_date);
CREATE INDEX idx_stmt_lines_amount ON _atlas.bank_statement_lines(amount);

-- ============================================================================
-- System Transactions
-- Oracle Fusion: AP payments, AR receipts, GL journal entries available
-- for reconciliation matching
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.system_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    bank_account_id UUID NOT NULL REFERENCES _atlas.bank_accounts(id),
    -- Source reference
    source_type VARCHAR(50) NOT NULL, -- 'ap_payment', 'ar_receipt', 'gl_journal', 'cash_transfer'
    source_id UUID NOT NULL,
    source_number VARCHAR(100),  -- payment number, receipt number, journal entry number
    -- Transaction details
    transaction_date DATE NOT NULL,
    amount NUMERIC(18,2) NOT NULL,
    -- Type: matches bank statement line types
    transaction_type VARCHAR(30) NOT NULL,
    -- Description for matching
    description TEXT,
    -- Reference / check number for matching
    reference_number VARCHAR(100),
    check_number VARCHAR(50),
    -- Counterparty
    counterparty_name VARCHAR(200),
    -- Reconciliation status: 'unreconciled', 'reconciled', 'voided'
    status VARCHAR(30) NOT NULL DEFAULT 'unreconciled',
    -- GL posting date
    gl_posting_date DATE,
    -- Currency (for multi-currency)
    currency_code VARCHAR(3) DEFAULT 'USD',
    exchange_rate NUMERIC(18,6),
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_sys_trans_org ON _atlas.system_transactions(organization_id);
CREATE INDEX idx_sys_trans_account ON _atlas.system_transactions(bank_account_id);
CREATE INDEX idx_sys_trans_status ON _atlas.system_transactions(status);
CREATE INDEX idx_sys_trans_date ON _atlas.system_transactions(transaction_date);
CREATE INDEX idx_sys_trans_amount ON _atlas.system_transactions(amount);
CREATE INDEX idx_sys_trans_source ON _atlas.system_transactions(source_type, source_id);

-- ============================================================================
-- Reconciliation Matches (linking statement lines to system transactions)
-- Oracle Fusion: The reconciliation matching records
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.reconciliation_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_id UUID NOT NULL REFERENCES _atlas.bank_statements(id),
    -- The bank statement line
    statement_line_id UUID NOT NULL REFERENCES _atlas.bank_statement_lines(id),
    -- The system transaction
    system_transaction_id UUID NOT NULL REFERENCES _atlas.system_transactions(id),
    -- Match details
    match_method VARCHAR(30) NOT NULL, -- 'auto_amount_date', 'auto_reference', 'auto_check', 'manual'
    match_confidence NUMERIC(5,2), -- 0-100 confidence score for auto matches
    -- Who matched / unmatched
    matched_by UUID,
    matched_at TIMESTAMPTZ DEFAULT now(),
    unmatched_by UUID,
    unmatched_at TIMESTAMPTZ,
    -- Status: 'active', 'unmatched'
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    -- Notes
    notes TEXT,
    -- Audit
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}',
    -- Prevent duplicate active matches on same statement line + system transaction
    UNIQUE(statement_line_id, system_transaction_id)
);

CREATE INDEX idx_recon_matches_statement ON _atlas.reconciliation_matches(statement_id);
CREATE INDEX idx_recon_matches_line ON _atlas.reconciliation_matches(statement_line_id);
CREATE INDEX idx_recon_matches_trans ON _atlas.reconciliation_matches(system_transaction_id);
CREATE INDEX idx_recon_matches_status ON _atlas.reconciliation_matches(status);
CREATE INDEX idx_recon_matches_org ON _atlas.reconciliation_matches(organization_id);

-- ============================================================================
-- Reconciliation Summary (per account per period)
-- Oracle Fusion: Reconciliation Dashboard
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.reconciliation_summaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    bank_account_id UUID NOT NULL REFERENCES _atlas.bank_accounts(id),
    -- Period
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    -- Statement details
    statement_id UUID REFERENCES _atlas.bank_statements(id),
    statement_balance NUMERIC(18,2) DEFAULT 0,
    -- Book balance (GL balance at period end)
    book_balance NUMERIC(18,2) DEFAULT 0,
    -- Adjustments
    deposits_in_transit NUMERIC(18,2) DEFAULT 0,
    outstanding_checks NUMERIC(18,2) DEFAULT 0,
    bank_charges NUMERIC(18,2) DEFAULT 0,
    bank_interest NUMERIC(18,2) DEFAULT 0,
    errors_and_omissions NUMERIC(18,2) DEFAULT 0,
    -- Computed
    adjusted_book_balance NUMERIC(18,2) DEFAULT 0,
    adjusted_bank_balance NUMERIC(18,2) DEFAULT 0,
    difference NUMERIC(18,2) DEFAULT 0,
    is_balanced BOOLEAN DEFAULT false,
    -- Status: 'in_progress', 'balanced', 'reviewed', 'approved'
    status VARCHAR(30) NOT NULL DEFAULT 'in_progress',
    -- Audit
    reviewed_by UUID,
    reviewed_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(organization_id, bank_account_id, period_start, period_end)
);

CREATE INDEX idx_recon_summary_org ON _atlas.reconciliation_summaries(organization_id);
CREATE INDEX idx_recon_summary_account ON _atlas.reconciliation_summaries(bank_account_id);
CREATE INDEX idx_recon_summary_status ON _atlas.reconciliation_summaries(status);
CREATE INDEX idx_recon_summary_period ON _atlas.reconciliation_summaries(period_start, period_end);

-- ============================================================================
-- Auto-Matching Rules
-- Oracle Fusion: User-defined reconciliation matching rules
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.reconciliation_matching_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    bank_account_id UUID, -- NULL = applies to all accounts
    -- Rule definition
    name VARCHAR(200) NOT NULL,
    description TEXT,
    priority INT NOT NULL DEFAULT 100,
    -- Match criteria (JSON)
    -- e.g., {"match_by": "amount_and_date", "amount_tolerance": 0.01, "date_tolerance_days": 3}
    -- e.g., {"match_by": "reference_exact", "field": "reference_number"}
    -- e.g., {"match_by": "check_number_exact"}
    -- e.g., {"match_by": "amount_and_counterparty", "amount_tolerance": 0.01}
    criteria JSONB NOT NULL,
    -- Whether to stop evaluating further rules after this match
    stop_on_match BOOLEAN DEFAULT true,
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_recon_rules_org ON _atlas.reconciliation_matching_rules(organization_id);
CREATE INDEX idx_recon_rules_account ON _atlas.reconciliation_matching_rules(bank_account_id);
CREATE INDEX idx_recon_rules_priority ON _atlas.reconciliation_matching_rules(priority);
