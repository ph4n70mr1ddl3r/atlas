-- ═══════════════════════════════════════════════════════════════════════════════
-- Time and Labor Management (Oracle Fusion Cloud HCM Time and Labor)
-- ═══════════════════════════════════════════════════════════════════════════════
--
-- Oracle Fusion equivalent: HCM > Time and Labor > Time Cards, Time Entries,
--   Work Schedules, Overtime Rules, Labor Distribution
--
-- Supports:
--  - Time card periods (weekly/bi-weekly/semi-monthly/monthly)
--  - Time entries with start/stop times, project/department attribution
--  - Overtime rules (daily/weekly thresholds, multipliers)
--  - Work schedules (standard hours, shift definitions)
--  - Labor distribution (cost center / project allocation)
--  - Approval workflows for time cards

-- Work Schedules
CREATE TABLE IF NOT EXISTS _atlas.work_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    schedule_type VARCHAR(50) NOT NULL DEFAULT 'fixed',  -- fixed, flexible, rotating, shift
    standard_hours_per_day TEXT NOT NULL DEFAULT '8.00',
    standard_hours_per_week TEXT NOT NULL DEFAULT '40.00',
    work_days_per_week INT NOT NULL DEFAULT 5,
    start_time TIME DEFAULT '09:00',
    end_time TIME DEFAULT '17:00',
    break_duration_minutes INT DEFAULT 60,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Overtime Rules
CREATE TABLE IF NOT EXISTS _atlas.overtime_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    threshold_type VARCHAR(50) NOT NULL DEFAULT 'daily',  -- daily, weekly, both
    daily_threshold_hours TEXT NOT NULL DEFAULT '8.00',
    weekly_threshold_hours TEXT NOT NULL DEFAULT '40.00',
    overtime_multiplier TEXT NOT NULL DEFAULT '1.5000',
    double_time_threshold_hours TEXT,  -- hours after which double-time kicks in
    double_time_multiplier TEXT NOT NULL DEFAULT '2.0000',
    include_holidays BOOLEAN DEFAULT false,
    include_weekends BOOLEAN DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Time Cards (one per employee per period)
CREATE TABLE IF NOT EXISTS _atlas.time_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    employee_id UUID NOT NULL,
    employee_name VARCHAR(200),
    card_number VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',  -- draft, submitted, approved, rejected, cancelled
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    total_regular_hours TEXT NOT NULL DEFAULT '0.0000',
    total_overtime_hours TEXT NOT NULL DEFAULT '0.0000',
    total_double_time_hours TEXT NOT NULL DEFAULT '0.0000',
    total_hours TEXT NOT NULL DEFAULT '0.0000',
    schedule_id UUID REFERENCES _atlas.work_schedules(id),
    overtime_rule_id UUID REFERENCES _atlas.overtime_rules(id),
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    comments TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, employee_id, period_start, period_end)
);

-- Time Entries (individual time punches within a time card)
CREATE TABLE IF NOT EXISTS _atlas.time_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    time_card_id UUID NOT NULL REFERENCES _atlas.time_cards(id) ON DELETE CASCADE,
    entry_date DATE NOT NULL,
    entry_type VARCHAR(50) NOT NULL DEFAULT 'regular',  -- regular, overtime, double_time, holiday, sick, vacation, break
    start_time TIME,
    end_time TIME,
    duration_hours TEXT NOT NULL DEFAULT '0.0000',
    project_id UUID,
    project_name VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    task_name VARCHAR(200),
    location VARCHAR(200),
    cost_center VARCHAR(100),
    labor_category VARCHAR(100),
    comments TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_time_entries_card ON _atlas.time_entries(time_card_id);
CREATE INDEX IF NOT EXISTS idx_time_entries_date ON _atlas.time_entries(entry_date);
CREATE INDEX IF NOT EXISTS idx_time_entries_project ON _atlas.time_entries(project_id);

-- Time Card History (audit trail for status changes)
CREATE TABLE IF NOT EXISTS _atlas.time_card_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    time_card_id UUID NOT NULL REFERENCES _atlas.time_cards(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,  -- create, submit, approve, reject, cancel, reopen
    from_status VARCHAR(50),
    to_status VARCHAR(50),
    performed_by UUID,
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_time_card_history_card ON _atlas.time_card_history(time_card_id);

-- Labor Distribution (cost allocation for time entries)
CREATE TABLE IF NOT EXISTS _atlas.labor_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    time_entry_id UUID NOT NULL REFERENCES _atlas.time_entries(id) ON DELETE CASCADE,
    distribution_percent TEXT NOT NULL DEFAULT '100.00',
    cost_center VARCHAR(100),
    project_id UUID,
    project_name VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    gl_account_code VARCHAR(100),
    allocated_hours TEXT NOT NULL DEFAULT '0.0000',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_labor_distributions_entry ON _atlas.labor_distributions(time_entry_id);
