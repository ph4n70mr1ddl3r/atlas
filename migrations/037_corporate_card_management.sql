-- ============================================================================
-- Corporate Card Management
-- Oracle Fusion Cloud ERP: Financials > Expenses > Corporate Cards
-- Migration 037
--
-- Manages corporate credit card programmes, card issuance to employees,
-- statement imports from card issuers, transaction-to-expense matching,
-- spending limit overrides, and dispute handling.
-- ============================================================================

-- ============================================================================
-- Card Programs
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.corporate_card_programs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    issuer_bank VARCHAR(200) NOT NULL,
    card_network VARCHAR(50) NOT NULL,           -- e.g. Visa, Mastercard, Amex
    card_type VARCHAR(50) NOT NULL DEFAULT 'corporate',  -- corporate, purchasing, travel
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    default_single_purchase_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    default_monthly_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    default_cash_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    default_atm_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    allow_cash_withdrawal BOOLEAN NOT NULL DEFAULT false,
    allow_international BOOLEAN NOT NULL DEFAULT true,
    auto_deactivate_on_termination BOOLEAN NOT NULL DEFAULT true,
    expense_matching_method VARCHAR(50) NOT NULL DEFAULT 'auto', -- auto, manual, semi
    billing_cycle_day INT NOT NULL DEFAULT 1,     -- day of month for statement generation
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, program_code)
);

CREATE INDEX idx_cc_programs_org ON _atlas.corporate_card_programs(organization_id);
CREATE INDEX idx_cc_programs_active ON _atlas.corporate_card_programs(organization_id, is_active);

-- ============================================================================
-- Corporate Cards
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.corporate_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_id UUID NOT NULL REFERENCES _atlas.corporate_card_programs(id),
    card_number_masked VARCHAR(30) NOT NULL,      -- e.g. ****-****-****-1234
    cardholder_name VARCHAR(200) NOT NULL,
    cardholder_id UUID NOT NULL,                  -- employee / user
    cardholder_email VARCHAR(300),
    department_id UUID,
    department_name VARCHAR(200),
    status VARCHAR(30) NOT NULL DEFAULT 'active', -- active, suspended, cancelled, expired, lost, stolen
    issue_date DATE NOT NULL,
    expiry_date DATE NOT NULL,
    single_purchase_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    monthly_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    cash_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    atm_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    current_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_spend_current_cycle NUMERIC(18,2) NOT NULL DEFAULT 0,
    last_statement_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    last_statement_date DATE,
    gl_liability_account VARCHAR(100),
    gl_expense_account VARCHAR(100),
    cost_center VARCHAR(100),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cc_cards_org ON _atlas.corporate_cards(organization_id);
CREATE INDEX idx_cc_cards_program ON _atlas.corporate_cards(program_id);
CREATE INDEX idx_cc_cards_cardholder ON _atlas.corporate_cards(cardholder_id);
CREATE INDEX idx_cc_cards_status ON _atlas.corporate_cards(organization_id, status);

-- ============================================================================
-- Card Transactions
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.corporate_card_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    card_id UUID NOT NULL REFERENCES _atlas.corporate_cards(id),
    program_id UUID NOT NULL REFERENCES _atlas.corporate_card_programs(id),
    transaction_reference VARCHAR(100) NOT NULL,
    posting_date DATE NOT NULL,
    transaction_date DATE NOT NULL,
    merchant_name VARCHAR(500) NOT NULL,
    merchant_category VARCHAR(200),
    merchant_category_code VARCHAR(20),
    amount NUMERIC(18,2) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    original_amount NUMERIC(18,2),
    original_currency VARCHAR(3),
    exchange_rate NUMERIC(18,6),
    transaction_type VARCHAR(50) NOT NULL DEFAULT 'charge',  -- charge, credit, payment, cash_withdrawal, fee, interest
    status VARCHAR(30) NOT NULL DEFAULT 'unmatched',          -- unmatched, matched, disputed, approved, rejected
    expense_report_id UUID,
    expense_line_id UUID,
    matched_at TIMESTAMPTZ,
    matched_by UUID,
    match_confidence VARCHAR(20),
    dispute_reason TEXT,
    dispute_date DATE,
    dispute_resolution TEXT,
    gl_posted BOOLEAN NOT NULL DEFAULT false,
    gl_journal_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, transaction_reference)
);

CREATE INDEX idx_cc_txns_org ON _atlas.corporate_card_transactions(organization_id);
CREATE INDEX idx_cc_txns_card ON _atlas.corporate_card_transactions(card_id);
CREATE INDEX idx_cc_txns_status ON _atlas.corporate_card_transactions(organization_id, status);
CREATE INDEX idx_cc_txns_date ON _atlas.corporate_card_transactions(transaction_date);
CREATE INDEX idx_cc_txns_expense ON _atlas.corporate_card_transactions(expense_report_id)
    WHERE expense_report_id IS NOT NULL;

-- ============================================================================
-- Card Statements
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.corporate_card_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_id UUID NOT NULL REFERENCES _atlas.corporate_card_programs(id),
    statement_number VARCHAR(50) NOT NULL,
    statement_date DATE NOT NULL,
    billing_period_start DATE NOT NULL,
    billing_period_end DATE NOT NULL,
    opening_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    closing_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_charges NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credits NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_payments NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_fees NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_interest NUMERIC(18,2) NOT NULL DEFAULT 0,
    payment_due_date DATE,
    minimum_payment NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_transaction_count INT NOT NULL DEFAULT 0,
    matched_transaction_count INT NOT NULL DEFAULT 0,
    unmatched_transaction_count INT NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'imported',  -- imported, processing, matched, reconciled, paid
    payment_reference VARCHAR(100),
    paid_at TIMESTAMPTZ,
    gl_payment_journal_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    imported_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, statement_number)
);

CREATE INDEX idx_cc_stmts_org ON _atlas.corporate_card_statements(organization_id);
CREATE INDEX idx_cc_stmts_program ON _atlas.corporate_card_statements(program_id);
CREATE INDEX idx_cc_stmts_status ON _atlas.corporate_card_statements(status);

-- ============================================================================
-- Spending Limit Overrides
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.corporate_card_limit_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    card_id UUID NOT NULL REFERENCES _atlas.corporate_cards(id),
    override_type VARCHAR(50) NOT NULL,            -- single_purchase, monthly, cash, atm
    original_value NUMERIC(18,2) NOT NULL,
    new_value NUMERIC(18,2) NOT NULL,
    reason TEXT NOT NULL,
    effective_from DATE NOT NULL,
    effective_to DATE,
    status VARCHAR(30) NOT NULL DEFAULT 'pending', -- pending, approved, rejected, expired
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cc_limits_card ON _atlas.corporate_card_limit_overrides(card_id);
CREATE INDEX idx_cc_limits_status ON _atlas.corporate_card_limit_overrides(organization_id, status);
