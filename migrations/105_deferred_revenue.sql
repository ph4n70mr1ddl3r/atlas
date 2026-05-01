-- 105_deferred_revenue.sql
-- Deferred Revenue/Cost Management tables
-- Oracle Fusion equivalent: Financials > Revenue Management > Deferral Schedules

CREATE TABLE IF NOT EXISTS _atlas.deferral_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    deferral_type VARCHAR(20) NOT NULL DEFAULT 'revenue', -- revenue, cost
    recognition_method VARCHAR(30) NOT NULL DEFAULT 'straight_line', -- straight_line, daily_rate, front_loaded, back_loaded, fixed_schedule
    deferral_account_code VARCHAR(50) NOT NULL,
    recognition_account_code VARCHAR(50) NOT NULL,
    contra_account_code VARCHAR(50),
    default_periods INT NOT NULL DEFAULT 12,
    period_type VARCHAR(20) NOT NULL DEFAULT 'monthly', -- monthly, daily, quarterly, yearly
    start_date_basis VARCHAR(30) NOT NULL DEFAULT 'transaction_date',
    end_date_basis VARCHAR(30) NOT NULL DEFAULT 'fixed_periods',
    prorate_partial_periods BOOLEAN NOT NULL DEFAULT true,
    auto_generate_schedule BOOLEAN NOT NULL DEFAULT true,
    auto_post BOOLEAN NOT NULL DEFAULT false,
    rounding_threshold DOUBLE PRECISION DEFAULT 0.01,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.deferral_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_number VARCHAR(50) NOT NULL,
    template_id UUID NOT NULL,
    template_code VARCHAR(100),
    deferral_type VARCHAR(20) NOT NULL,
    source_type VARCHAR(50) NOT NULL, -- invoice, po, expense, manual
    source_id UUID,
    source_number VARCHAR(100),
    source_line_id UUID,
    description TEXT,
    total_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    recognized_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    remaining_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    deferral_account_code VARCHAR(50) NOT NULL,
    recognition_account_code VARCHAR(50) NOT NULL,
    contra_account_code VARCHAR(50),
    recognition_method VARCHAR(30) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    total_periods INT NOT NULL DEFAULT 0,
    completed_periods INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- draft, active, on_hold, completed, cancelled
    hold_reason TEXT,
    original_journal_entry_id UUID,
    last_recognition_date DATE,
    completion_date DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, schedule_number)
);

CREATE TABLE IF NOT EXISTS _atlas.deferral_schedule_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_id UUID NOT NULL REFERENCES _atlas.deferral_schedules(id),
    line_number INT NOT NULL,
    period_name VARCHAR(50),
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    days_in_period INT NOT NULL DEFAULT 0,
    amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    recognized_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, recognized, reversed, on_hold
    recognition_date DATE,
    journal_entry_id UUID,
    reversal_journal_entry_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_deferral_templates_org ON _atlas.deferral_templates(organization_id);
CREATE INDEX IF NOT EXISTS idx_deferral_schedules_org ON _atlas.deferral_schedules(organization_id);
CREATE INDEX IF NOT EXISTS idx_deferral_schedules_status ON _atlas.deferral_schedules(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_deferral_schedule_lines_schedule ON _atlas.deferral_schedule_lines(schedule_id);
CREATE INDEX IF NOT EXISTS idx_deferral_schedule_lines_pending ON _atlas.deferral_schedule_lines(status) WHERE status = 'pending';
