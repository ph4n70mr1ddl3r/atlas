-- 141_average_balance_processing.sql
-- Oracle Fusion Financial Feature: Average Balance Processing
-- Provides daily average balance calculations for GL accounts, weighted averages
-- over configurable periods, average-to-date and period-end averages.
-- Used primarily by financial institutions for regulatory reporting (call reports).
-- Oracle Fusion: Financials > General Ledger > Average Balances

-- ============================================================================
-- Average Balance Book (defines the averaging context)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.average_balance_books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    book_code VARCHAR(50) NOT NULL,
    book_name VARCHAR(200) NOT NULL,
    description TEXT,
    -- 'daily', 'monthly', 'quarterly', 'yearly'
    period_type VARCHAR(20) NOT NULL DEFAULT 'daily',
    -- Number of days in the averaging window
    averaging_window_days INT NOT NULL DEFAULT 30,
    -- Whether this is the primary book for the org
    is_primary BOOLEAN NOT NULL DEFAULT false,
    -- 'active', 'inactive'
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, book_code)
);

CREATE INDEX IF NOT EXISTS idx_abb_org ON financials.average_balance_books(organization_id);
CREATE INDEX IF NOT EXISTS idx_abb_status ON financials.average_balance_books(status);

-- ============================================================================
-- Average Balance Book Accounts (GL accounts tracked in each book)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.average_balance_book_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    book_id UUID NOT NULL REFERENCES financials.average_balance_books(id) ON DELETE CASCADE,
    gl_account VARCHAR(500) NOT NULL,
    gl_account_name VARCHAR(200),
    -- 'asset', 'liability', 'equity', 'revenue', 'expense'
    account_type VARCHAR(30),
    -- Whether negative balances are tracked separately
    track_negative BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(book_id, gl_account)
);

CREATE INDEX IF NOT EXISTS idx_abba_book ON financials.average_balance_book_accounts(book_id);

-- ============================================================================
-- Daily Balance Entries (the raw daily balances for each account)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.daily_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    book_id UUID NOT NULL REFERENCES financials.average_balance_books(id),
    account_id UUID NOT NULL REFERENCES financials.average_balance_book_accounts(id),
    balance_date DATE NOT NULL,
    -- The actual closing balance for the day
    closing_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- The opening balance for the day
    opening_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Total debits for the day
    total_debits NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Total credits for the day
    total_credits NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Number of transactions for the day
    transaction_count INT NOT NULL DEFAULT 0,
    -- For negative balance tracking
    negative_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(book_id, account_id, balance_date)
);

CREATE INDEX IF NOT EXISTS idx_db_org ON financials.daily_balances(organization_id);
CREATE INDEX IF NOT EXISTS idx_db_book ON financials.daily_balances(book_id);
CREATE INDEX IF NOT EXISTS idx_db_account ON financials.daily_balances(account_id);
CREATE INDEX IF NOT EXISTS idx_db_date ON financials.daily_balances(balance_date);
CREATE INDEX IF NOT EXISTS idx_db_book_date ON financials.daily_balances(book_id, balance_date);

-- ============================================================================
-- Average Balance Calculations (computed averages per account per period)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.average_balance_calculations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    book_id UUID NOT NULL REFERENCES financials.average_balance_books(id),
    account_id UUID NOT NULL REFERENCES financials.average_balance_book_accounts(id),
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    -- Number of days in the calculation
    days_in_period INT NOT NULL,
    -- The calculated average balance
    average_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Weighted average (balance * days) / days_in_period
    weighted_average_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Period total debits
    period_total_debits NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Period total credits
    period_total_credits NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Average-to-date (from start of fiscal year)
    average_to_date NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Period-end closing balance
    period_end_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Peak (maximum) balance in the period
    peak_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Trough (minimum) balance in the period
    trough_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- 'daily', 'monthly', 'quarterly', 'yearly'
    calculation_type VARCHAR(20) NOT NULL DEFAULT 'daily',
    -- 'pending', 'calculated', 'approved', 'posted'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    calculated_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(book_id, account_id, period_start_date, calculation_type)
);

CREATE INDEX IF NOT EXISTS idx_abc_org ON financials.average_balance_calculations(organization_id);
CREATE INDEX IF NOT EXISTS idx_abc_book ON financials.average_balance_calculations(book_id);
CREATE INDEX IF NOT EXISTS idx_abc_account ON financials.average_balance_calculations(account_id);
CREATE INDEX IF NOT EXISTS idx_abc_period ON financials.average_balance_calculations(period_start_date, period_end_date);
CREATE INDEX IF NOT EXISTS idx_abc_status ON financials.average_balance_calculations(status);
CREATE INDEX IF NOT EXISTS idx_abc_type ON financials.average_balance_calculations(calculation_type);

-- ============================================================================
-- Average Balance Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.average_balance_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    book_id UUID,
    account_id UUID,
    calculation_id UUID,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_abau_book ON financials.average_balance_audit(book_id);
CREATE INDEX IF NOT EXISTS idx_abau_action ON financials.average_balance_audit(action);
CREATE INDEX IF NOT EXISTS idx_abau_performed ON financials.average_balance_audit(performed_at);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.average_balance_dashboard AS
SELECT
    books.organization_id,
    COUNT(DISTINCT books.id) AS total_books,
    COUNT(DISTINCT books.id) FILTER (WHERE books.status = 'active') AS active_books,
    COUNT(DISTINCT bacc.id) AS tracked_accounts,
    COUNT(DISTINCT dbals.id) AS daily_balance_entries,
    COALESCE(SUM(calcs.average_balance), 0) AS total_average_balance,
    COALESCE(SUM(calcs.peak_balance), 0) AS total_peak_balance,
    COALESCE(SUM(calcs.period_total_debits), 0) AS total_period_debits,
    COALESCE(SUM(calcs.period_total_credits), 0) AS total_period_credits,
    (
        SELECT json_agg(jsonb_build_object(
            'book_id', b.id,
            'book_code', b.book_code,
            'book_name', b.book_name,
            'period_type', b.period_type,
            'tracked_accounts', (
                SELECT COUNT(*) FROM financials.average_balance_book_accounts ba WHERE ba.book_id = b.id
            ),
            'latest_calculation', (
                SELECT MAX(calculated_at) FROM financials.average_balance_calculations c WHERE c.book_id = b.id
            )
        )) FROM financials.average_balance_books b WHERE b.organization_id = books.organization_id
    ) AS books_summary
FROM financials.average_balance_books books
LEFT JOIN financials.average_balance_book_accounts bacc ON bacc.book_id = books.id
LEFT JOIN LATERAL (
    SELECT id, account_id, average_balance, peak_balance, period_total_debits, period_total_credits
    FROM financials.average_balance_calculations
    WHERE book_id = books.id
      AND status = 'calculated'
    ORDER BY period_end_date DESC
    LIMIT 1
) calcs ON true
LEFT JOIN LATERAL (
    SELECT COUNT(id) AS id FROM financials.daily_balances
    WHERE book_id = books.id AND balance_date >= CURRENT_DATE - INTERVAL '30 days'
) dbals ON true
GROUP BY books.organization_id;

COMMENT ON TABLE financials.average_balance_books IS 'Average balance books - define averaging contexts and parameters for GL account average balance calculations';
COMMENT ON TABLE financials.average_balance_book_accounts IS 'GL accounts tracked for average balance processing within a book';
COMMENT ON TABLE financials.daily_balances IS 'Daily balance entries - raw daily closing/opening balances for each tracked GL account';
COMMENT ON TABLE financials.average_balance_calculations IS 'Computed average balances per account per period with weighted averages and period statistics';
COMMENT ON TABLE financials.average_balance_audit IS 'Audit trail for average balance processing actions';
COMMENT ON VIEW financials.average_balance_dashboard IS 'Dashboard aggregation for average balance processing statistics';
