-- Atlas ERP - Period Close Management
-- Oracle Fusion Cloud ERP: General Ledger Period Close
--
-- Controls accounting periods: open/close, subledger tracking,
-- period close checklist, and posting validation.
--
-- This is a standard Oracle Fusion Cloud ERP feature that prevents
-- posting transactions to closed accounting periods and tracks the
-- financial close process across subledgers.

-- ============================================================================
-- Accounting Calendars
-- Oracle Fusion: Defines the fiscal year structure and period breakdown
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.accounting_calendars (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Calendar type: 'monthly', 'quarterly', '445', '454', '544', 'weekly'
    calendar_type VARCHAR(20) NOT NULL DEFAULT 'monthly',
    -- Fiscal year start month (1=January, 4=April, 7=July, 10=October)
    fiscal_year_start_month INT NOT NULL DEFAULT 1,
    -- Number of periods in a year
    periods_per_year INT NOT NULL DEFAULT 12,
    -- Adjusting period(s) at year end
    has_adjusting_period BOOLEAN DEFAULT false,
    -- Year specification
    current_fiscal_year INT,
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Meta
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    UNIQUE(organization_id, name)
);

CREATE INDEX idx_calendars_org ON _atlas.accounting_calendars(organization_id);
CREATE INDEX idx_calendars_active ON _atlas.accounting_calendars(organization_id, is_active);

-- ============================================================================
-- Accounting Periods
-- Oracle Fusion: Individual periods within a fiscal year with open/close status
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.accounting_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    calendar_id UUID NOT NULL REFERENCES _atlas.accounting_calendars(id) ON DELETE CASCADE,
    -- Period identification
    period_name VARCHAR(100) NOT NULL, -- e.g., "Jan-2026", "Q1-2026", "Adj-2026"
    period_number INT NOT NULL, -- 1-12 for monthly, 1-4 for quarterly, 13 for adjusting
    fiscal_year INT NOT NULL,
    quarter INT, -- 1-4
    -- Date range
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    -- Period status: 'future', 'not_opened', 'open', 'pending_close', 'closed', 'permanently_closed'
    status VARCHAR(30) NOT NULL DEFAULT 'not_opened',
    -- Who changed the status
    status_changed_by UUID REFERENCES _atlas.users(id),
    status_changed_at TIMESTAMPTZ,
    -- Closing info
    closed_by UUID REFERENCES _atlas.users(id),
    closed_at TIMESTAMPTZ,
    -- Period type: 'regular', 'adjusting', 'opening'
    period_type VARCHAR(20) DEFAULT 'regular',
    -- Subledger close tracking
    gl_status VARCHAR(30) DEFAULT 'not_opened', -- General Ledger
    ap_status VARCHAR(30) DEFAULT 'not_opened', -- Accounts Payable
    ar_status VARCHAR(30) DEFAULT 'not_opened', -- Accounts Receivable
    fa_status VARCHAR(30) DEFAULT 'not_opened', -- Fixed Assets
    po_status VARCHAR(30) DEFAULT 'not_opened', -- Purchasing
    -- Aggregated balances for the period (updated on close)
    total_debits NUMERIC(18,2) DEFAULT 0,
    total_credits NUMERIC(18,2) DEFAULT 0,
    net_activity NUMERIC(18,2) DEFAULT 0,
    beginning_balance NUMERIC(18,2) DEFAULT 0,
    ending_balance NUMERIC(18,2) DEFAULT 0,
    -- Number of journal entries in this period
    journal_entry_count INT DEFAULT 0,
    -- Meta
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(calendar_id, period_name)
);

CREATE INDEX idx_periods_calendar ON _atlas.accounting_periods(calendar_id, fiscal_year, period_number);
CREATE INDEX idx_periods_org ON _atlas.accounting_periods(organization_id);
CREATE INDEX idx_periods_status ON _atlas.accounting_periods(organization_id, status);
CREATE INDEX idx_periods_dates ON _atlas.accounting_periods(start_date, end_date);
CREATE INDEX idx_periods_fiscal_year ON _atlas.accounting_periods(organization_id, fiscal_year);

-- ============================================================================
-- Period Close Checklist Items
-- Oracle Fusion: Track the close process with a checklist of tasks
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.period_close_checklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    period_id UUID NOT NULL REFERENCES _atlas.accounting_periods(id) ON DELETE CASCADE,
    -- Task definition
    task_name VARCHAR(200) NOT NULL,
    task_description TEXT,
    task_order INT NOT NULL DEFAULT 0,
    -- Task category
    category VARCHAR(100), -- 'reconciliation', 'accrual', 'review', 'report', 'other'
    subledger VARCHAR(50), -- 'gl', 'ap', 'ar', 'fa', 'po', null for general
    -- Status
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'in_progress', 'completed', 'skipped'
    -- Assignment
    assigned_to UUID REFERENCES _atlas.users(id),
    due_date DATE,
    -- Completion tracking
    completed_by UUID REFERENCES _atlas.users(id),
    completed_at TIMESTAMPTZ,
    -- Dependencies
    depends_on UUID REFERENCES _atlas.period_close_checklist(id),
    -- Notes
    notes TEXT,
    -- Meta
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_checklist_period ON _atlas.period_close_checklist(period_id, task_order);
CREATE INDEX idx_checklist_org ON _atlas.period_close_checklist(organization_id);
CREATE INDEX idx_checklist_status ON _atlas.period_close_checklist(period_id, status);
CREATE INDEX idx_checklist_assigned ON _atlas.period_close_checklist(assigned_to);

-- ============================================================================
-- Period Close Lock Exceptions
-- Oracle Fusion: Allow specific users to post to locked/closed periods
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.period_close_exceptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    period_id UUID NOT NULL REFERENCES _atlas.accounting_periods(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES _atlas.users(id),
    -- What the exception allows
    allowed_actions JSONB DEFAULT '["post"]', -- ["post", "adjust", "reverse"]
    -- Reason
    reason TEXT,
    granted_by UUID REFERENCES _atlas.users(id),
    -- Time-limited exception
    valid_until TIMESTAMPTZ,
    -- Meta
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(period_id, user_id)
);

CREATE INDEX idx_period_exceptions_period ON _atlas.period_close_exceptions(period_id);
CREATE INDEX idx_period_exceptions_user ON _atlas.period_close_exceptions(user_id);
