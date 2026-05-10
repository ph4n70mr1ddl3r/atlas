-- 150_multi_period_accounting.sql
-- Oracle Fusion Financial Feature: Multi-Period Accounting (MPA)
-- Allows journal entry amounts to be distributed across multiple accounting periods.
-- Example: A $12,000 annual insurance premium amortized $1,000/month over 12 months.
-- Oracle Fusion: Financials > General Ledger > Multi-Period Accounting

-- ============================================================================
-- MPA Templates (reusable distribution definitions)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.mpa_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_name VARCHAR(100) NOT NULL,
    description TEXT,
    -- 'equal' = spread evenly, 'custom' = user-defined percentages, 'days' = by days in period
    distribution_method VARCHAR(20) NOT NULL DEFAULT 'equal',
    -- Number of periods to spread across
    number_of_periods INT NOT NULL DEFAULT 1,
    -- Period type: 'month', 'quarter', 'year'
    period_type VARCHAR(20) NOT NULL DEFAULT 'month',
    -- GL account to credit/debit for the deferred amount
    deferred_account_code VARCHAR(50),
    expense_account_code VARCHAR(50),
    -- 'draft', 'active', 'inactive'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, template_name)
);

CREATE INDEX IF NOT EXISTS idx_mpa_tpl_org ON financials.mpa_templates(organization_id);
CREATE INDEX IF NOT EXISTS idx_mpa_tpl_status ON financials.mpa_templates(status);

-- ============================================================================
-- MPA Template Lines (custom distribution percentages)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.mpa_template_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES financials.mpa_templates(id) ON DELETE CASCADE,
    period_sequence INT NOT NULL,
    -- Percentage for this period (0-100, must sum to 100 across all lines)
    percentage VARCHAR(50) NOT NULL DEFAULT '0',
    -- Offset days from the start date for this period
    offset_days INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_mpa_tpl_line_tpl ON financials.mpa_template_lines(template_id);

-- ============================================================================
-- MPA Schedules (actual allocations applied to journal entries)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.mpa_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_number VARCHAR(50) NOT NULL,
    description TEXT,
    template_id UUID REFERENCES financials.mpa_templates(id),
    -- Source journal entry being allocated
    source_journal_entry_id UUID,
    source_journal_line_id UUID,
    -- Total amount being distributed
    total_amount VARCHAR(50) NOT NULL,
    -- Amount already recognized
    recognized_amount VARCHAR(50) NOT NULL DEFAULT '0',
    -- Remaining amount to recognize
    remaining_amount VARCHAR(50) NOT NULL DEFAULT '0',
    -- Start date of the allocation
    start_date DATE NOT NULL,
    -- End date of the allocation
    end_date DATE,
    -- 'draft', 'active', 'completed', 'cancelled', 'on_hold'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Accounting flexfield segments
    company_code VARCHAR(20),
    cost_center VARCHAR(20),
    account_segment VARCHAR(50),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, schedule_number)
);

CREATE INDEX IF NOT EXISTS idx_mpa_sched_org ON financials.mpa_schedules(organization_id);
CREATE INDEX IF NOT EXISTS idx_mpa_sched_status ON financials.mpa_schedules(status);
CREATE INDEX IF NOT EXISTS idx_mpa_sched_source ON financials.mpa_schedules(source_journal_entry_id);
CREATE INDEX IF NOT EXISTS idx_mpa_sched_dates ON financials.mpa_schedules(start_date, end_date);

-- ============================================================================
-- MPA Schedule Lines (individual period allocations)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.mpa_schedule_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID NOT NULL REFERENCES financials.mpa_schedules(id) ON DELETE CASCADE,
    period_sequence INT NOT NULL,
    -- The accounting period this line belongs to
    period_name VARCHAR(20),
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    -- Amount to recognize in this period
    amount VARCHAR(50) NOT NULL DEFAULT '0',
    -- Percentage of total
    percentage VARCHAR(50) NOT NULL DEFAULT '0',
    -- 'pending', 'recognized', 'reversed'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Generated journal entry for this period
    journal_entry_id UUID,
    recognized_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_mpa_sl_sched ON financials.mpa_schedule_lines(schedule_id);
CREATE INDEX IF NOT EXISTS idx_mpa_sl_status ON financials.mpa_schedule_lines(status);
CREATE INDEX IF NOT EXISTS idx_mpa_sl_period ON financials.mpa_schedule_lines(period_start_date, period_end_date);

-- ============================================================================
-- Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.mpa_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_id UUID,
    template_id UUID,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_mpa_audit_sched ON financials.mpa_audit(schedule_id);
CREATE INDEX IF NOT EXISTS idx_mpa_audit_action ON financials.mpa_audit(action);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.mpa_dashboard AS
SELECT
    s.organization_id,
    COUNT(DISTINCT t.id) AS total_templates,
    COUNT(DISTINCT t.id) FILTER (WHERE t.status = 'active') AS active_templates,
    COUNT(DISTINCT s.id) AS total_schedules,
    COUNT(DISTINCT s.id) FILTER (WHERE s.status = 'draft') AS draft_schedules,
    COUNT(DISTINCT s.id) FILTER (WHERE s.status = 'active') AS active_schedules,
    COUNT(DISTINCT s.id) FILTER (WHERE s.status = 'completed') AS completed_schedules,
    COUNT(DISTINCT s.id) FILTER (WHERE s.status = 'cancelled') AS cancelled_schedules,
    COUNT(DISTINCT s.id) FILTER (WHERE s.status = 'on_hold') AS on_hold_schedules,
    COALESCE(SUM(s.total_amount::NUMERIC), 0) AS total_scheduled_amount,
    COALESCE(SUM(s.recognized_amount::NUMERIC), 0) AS total_recognized_amount,
    COALESCE(SUM(s.remaining_amount::NUMERIC), 0) AS total_remaining_amount,
    COUNT(DISTINCT sl.id) FILTER (WHERE sl.status = 'pending') AS pending_lines,
    COUNT(DISTINCT sl.id) FILTER (WHERE sl.status = 'recognized') AS recognized_lines,
    COUNT(DISTINCT sl.id) FILTER (WHERE sl.status = 'reversed') AS reversed_lines
FROM financials.mpa_schedules s
LEFT JOIN financials.mpa_schedule_lines sl ON sl.schedule_id = s.id
LEFT JOIN financials.mpa_templates t ON t.organization_id = s.organization_id
GROUP BY s.organization_id;

COMMENT ON TABLE financials.mpa_templates IS 'Multi-Period Accounting Templates - reusable distribution definitions for spreading journal amounts across periods';
COMMENT ON TABLE financials.mpa_template_lines IS 'MPA Template Lines - custom distribution percentages per period';
COMMENT ON TABLE financials.mpa_schedules IS 'MPA Schedules - actual allocations applied to journal entries';
COMMENT ON TABLE financials.mpa_schedule_lines IS 'MPA Schedule Lines - individual period allocations with recognition status';
COMMENT ON TABLE financials.mpa_audit IS 'Audit trail for multi-period accounting actions';
COMMENT ON VIEW financials.mpa_dashboard IS 'Dashboard aggregation for multi-period accounting statistics';
