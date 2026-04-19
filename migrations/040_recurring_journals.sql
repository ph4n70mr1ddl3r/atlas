-- Recurring Journals (Oracle Fusion GL > Recurring Journals)
-- Define recurring journal templates that automatically generate journal entries on a schedule.
-- Supports three types: Standard (fixed amounts), Skeleton (amounts filled at generation),
-- and Incremental (amounts increase by a percentage each period).

-- Recurring journal schedule definitions
CREATE TABLE IF NOT EXISTS _atlas.recurring_journal_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    recurrence_type VARCHAR(30) NOT NULL DEFAULT 'monthly', -- daily, weekly, monthly, quarterly, semi_annual, annual
    journal_type VARCHAR(30) NOT NULL DEFAULT 'standard', -- standard, skeleton, incremental
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, active, inactive
    effective_from DATE,
    effective_to DATE,
    last_generation_date DATE,
    next_generation_date DATE,
    total_generations INT NOT NULL DEFAULT 0,
    incremental_percent NUMERIC(10,4) DEFAULT 0,
    auto_post BOOLEAN NOT NULL DEFAULT false,
    reversal_method VARCHAR(30), -- none, auto, manual
    ledger_id UUID,
    journal_category VARCHAR(100),
    reference_template VARCHAR(200),
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_by UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, schedule_number)
);

-- Recurring journal template lines
CREATE TABLE IF NOT EXISTS _atlas.recurring_journal_schedule_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_id UUID NOT NULL REFERENCES _atlas.recurring_journal_schedules(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    line_type VARCHAR(20) NOT NULL DEFAULT 'debit', -- debit, credit
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(200),
    description TEXT,
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    tax_code VARCHAR(50),
    cost_center VARCHAR(100),
    department_id UUID,
    project_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Recurring journal generation history
CREATE TABLE IF NOT EXISTS _atlas.recurring_journal_generations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_id UUID NOT NULL REFERENCES _atlas.recurring_journal_schedules(id) ON DELETE CASCADE,
    generation_number INT NOT NULL,
    journal_entry_id UUID,
    journal_entry_number VARCHAR(50),
    generation_date DATE NOT NULL,
    period_name VARCHAR(20),
    total_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    line_count INT NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'generated', -- generated, posted, reversed, cancelled
    reversal_entry_id UUID,
    reversed_at TIMESTAMPTZ,
    posted_at TIMESTAMPTZ,
    generated_by UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Generated journal lines from recurring schedules
CREATE TABLE IF NOT EXISTS _atlas.recurring_journal_generation_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    generation_id UUID NOT NULL REFERENCES _atlas.recurring_journal_generations(id) ON DELETE CASCADE,
    schedule_line_id UUID,
    line_number INT NOT NULL,
    line_type VARCHAR(20) NOT NULL DEFAULT 'debit',
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(200),
    description TEXT,
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    tax_code VARCHAR(50),
    cost_center VARCHAR(100),
    department_id UUID,
    project_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_rjs_org ON _atlas.recurring_journal_schedules (organization_id);
CREATE INDEX IF NOT EXISTS idx_rjs_status ON _atlas.recurring_journal_schedules (organization_id, status);
CREATE INDEX IF NOT EXISTS idx_rjs_next_gen ON _atlas.recurring_journal_schedules (next_generation_date) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_rjsl_schedule ON _atlas.recurring_journal_schedule_lines (schedule_id);
CREATE INDEX IF NOT EXISTS idx_rjg_schedule ON _atlas.recurring_journal_generations (schedule_id);
CREATE INDEX IF NOT EXISTS idx_rjg_status ON _atlas.recurring_journal_generations (organization_id, status);
CREATE INDEX IF NOT EXISTS idx_rjgl_generation ON _atlas.recurring_journal_generation_lines (generation_id);
