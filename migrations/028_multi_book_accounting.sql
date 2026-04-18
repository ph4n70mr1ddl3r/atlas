-- 028_multi_book_accounting.sql
-- Multi-Book Accounting (Secondary Ledgers)
-- Oracle Fusion equivalent: General Ledger > Multi-Book Accounting
--
-- Supports multiple accounting books (primary + secondary), each with its own
-- chart of accounts, calendar, and currency. Includes account mapping rules
-- and automatic journal propagation between books.

BEGIN;

-- ============================================================================
-- Accounting Books
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.accounting_books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    book_type VARCHAR(20) NOT NULL DEFAULT 'secondary',
    chart_of_accounts_code VARCHAR(50) NOT NULL,
    calendar_code VARCHAR(50) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    auto_propagation_enabled BOOLEAN NOT NULL DEFAULT false,
    mapping_level VARCHAR(20) NOT NULL DEFAULT 'journal',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_accounting_books_org ON _atlas.accounting_books(organization_id);
CREATE INDEX IF NOT EXISTS idx_accounting_books_type ON _atlas.accounting_books(organization_id, book_type);

COMMENT ON TABLE _atlas.accounting_books IS 'Accounting books (primary and secondary) for multi-book accounting';
COMMENT ON COLUMN _atlas.accounting_books.book_type IS 'primary or secondary book';
COMMENT ON COLUMN _atlas.accounting_books.mapping_level IS 'journal or subledger level mapping';
COMMENT ON COLUMN _atlas.accounting_books.auto_propagation_enabled IS 'Whether entries auto-propagate to this secondary book';

-- ============================================================================
-- Account Mapping Rules
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.account_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    source_book_id UUID NOT NULL REFERENCES _atlas.accounting_books(id),
    target_book_id UUID NOT NULL REFERENCES _atlas.accounting_books(id),
    source_account_code VARCHAR(100) NOT NULL,
    target_account_code VARCHAR(100) NOT NULL,
    segment_mappings JSONB NOT NULL DEFAULT '{}',
    priority INT NOT NULL DEFAULT 10,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_account_mappings_org ON _atlas.account_mappings(organization_id);
CREATE INDEX IF NOT EXISTS idx_account_mappings_source ON _atlas.account_mappings(source_book_id, target_book_id);
CREATE INDEX IF NOT EXISTS idx_account_mappings_account ON _atlas.account_mappings(source_book_id, target_book_id, source_account_code);

COMMENT ON TABLE _atlas.account_mappings IS 'Maps accounts between source and target accounting books';

-- ============================================================================
-- Book Journal Entries
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.book_journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    book_id UUID NOT NULL REFERENCES _atlas.accounting_books(id),
    entry_number VARCHAR(100) NOT NULL,
    header_description TEXT,
    source_book_id UUID REFERENCES _atlas.accounting_books(id),
    source_entry_id UUID REFERENCES _atlas.book_journal_entries(id),
    external_reference VARCHAR(200),
    accounting_date DATE NOT NULL,
    period_name VARCHAR(50),
    total_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    is_auto_propagated BOOLEAN NOT NULL DEFAULT false,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    conversion_rate NUMERIC(18,6),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(book_id, entry_number)
);

CREATE INDEX IF NOT EXISTS idx_book_journal_entries_org ON _atlas.book_journal_entries(organization_id);
CREATE INDEX IF NOT EXISTS idx_book_journal_entries_book ON _atlas.book_journal_entries(book_id);
CREATE INDEX IF NOT EXISTS idx_book_journal_entries_status ON _atlas.book_journal_entries(book_id, status);
CREATE INDEX IF NOT EXISTS idx_book_journal_entries_source ON _atlas.book_journal_entries(source_entry_id);

COMMENT ON TABLE _atlas.book_journal_entries IS 'Journal entries in a specific accounting book';

-- ============================================================================
-- Book Journal Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.book_journal_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entry_id UUID NOT NULL REFERENCES _atlas.book_journal_entries(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(200),
    debit_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    description TEXT,
    tax_code VARCHAR(50),
    source_line_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_book_journal_lines_entry ON _atlas.book_journal_lines(entry_id);
CREATE INDEX IF NOT EXISTS idx_book_journal_lines_account ON _atlas.book_journal_lines(account_code);

COMMENT ON TABLE _atlas.book_journal_lines IS 'Individual debit/credit lines within a book journal entry';

-- ============================================================================
-- Propagation Logs
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.propagation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    source_book_id UUID NOT NULL REFERENCES _atlas.accounting_books(id),
    target_book_id UUID NOT NULL REFERENCES _atlas.accounting_books(id),
    source_entry_id UUID NOT NULL REFERENCES _atlas.book_journal_entries(id),
    target_entry_id UUID REFERENCES _atlas.book_journal_entries(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    lines_propagated INT NOT NULL DEFAULT 0,
    lines_unmapped INT NOT NULL DEFAULT 0,
    error_message TEXT,
    propagated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_propagation_logs_org ON _atlas.propagation_logs(organization_id);
CREATE INDEX IF NOT EXISTS idx_propagation_logs_source ON _atlas.propagation_logs(source_book_id, target_book_id);
CREATE INDEX IF NOT EXISTS idx_propagation_logs_entry ON _atlas.propagation_logs(source_entry_id);

COMMENT ON TABLE _atlas.propagation_logs IS 'Tracks propagation of journal entries between accounting books';

COMMIT;
