-- ═══════════════════════════════════════════════════════════════════════════════
-- Absence Management (Oracle Fusion Cloud HCM Absence Management)
-- ═══════════════════════════════════════════════════════════════════════════════
--
-- Oracle Fusion equivalent: HCM > Absence Management > Absence Types, Plans, Entries
--
-- Supports:
--  - Absence types (vacation, sick, parental, bereavement, etc.)
--  - Absence plans with accrual rules (e.g. 15 days/year)
--  - Absence entries (recording employee absences)
--  - Balance tracking per employee per plan
--  - Approval workflows for absence requests

-- Absence Types
CREATE TABLE IF NOT EXISTS _atlas.absence_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL DEFAULT 'general',  -- general, sick, vacation, parental, bereavement, jury_duty, personal
    plan_type VARCHAR(50) NOT NULL DEFAULT 'no_entitlement',  -- accrual, qualification, no_entitlement
    requires_approval BOOLEAN NOT NULL DEFAULT true,
    requires_documentation BOOLEAN NOT NULL DEFAULT false,
    auto_approve_below_days TEXT NOT NULL DEFAULT '0.00',
    allow_negative_balance BOOLEAN NOT NULL DEFAULT false,
    allow_half_day BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Absence Plans (accrual-based leave plans)
CREATE TABLE IF NOT EXISTS _atlas.absence_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    absence_type_id UUID NOT NULL REFERENCES _atlas.absence_types(id) ON DELETE CASCADE,
    accrual_frequency VARCHAR(50) NOT NULL DEFAULT 'yearly',  -- yearly, monthly, semi_monthly, weekly
    accrual_rate TEXT NOT NULL DEFAULT '0',  -- number of days/units accrued per period
    accrual_unit VARCHAR(20) NOT NULL DEFAULT 'days',  -- days, hours
    carry_over_max TEXT DEFAULT '0',
    carry_over_expiry_months INT DEFAULT 0,
    max_balance TEXT,
    probation_period_days INT DEFAULT 0,
    prorate_first_year BOOLEAN DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Employee Absence Balances (accrued vs. taken)
CREATE TABLE IF NOT EXISTS _atlas.absence_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    employee_id UUID NOT NULL,
    plan_id UUID NOT NULL REFERENCES _atlas.absence_plans(id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    accrued TEXT NOT NULL DEFAULT '0',
    taken TEXT NOT NULL DEFAULT '0',
    adjusted TEXT NOT NULL DEFAULT '0',
    carried_over TEXT NOT NULL DEFAULT '0',
    remaining TEXT NOT NULL DEFAULT '0',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(employee_id, plan_id, period_start, period_end)
);

-- Absence Entries (individual absence records)
CREATE TABLE IF NOT EXISTS _atlas.absence_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    employee_id UUID NOT NULL,
    employee_name VARCHAR(200),
    absence_type_id UUID NOT NULL REFERENCES _atlas.absence_types(id) ON DELETE CASCADE,
    plan_id UUID REFERENCES _atlas.absence_plans(id) ON DELETE SET NULL,
    entry_number VARCHAR(50) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',  -- draft, submitted, approved, rejected, cancelled
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    duration_days TEXT NOT NULL DEFAULT '0',
    duration_hours TEXT,
    is_half_day BOOLEAN DEFAULT false,
    half_day_period VARCHAR(20),  -- first_half, second_half
    reason TEXT,
    comments TEXT,
    documentation_provided BOOLEAN DEFAULT false,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    cancelled_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Absence Entry Audit Trail
CREATE TABLE IF NOT EXISTS _atlas.absence_entry_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id UUID NOT NULL REFERENCES _atlas.absence_entries(id) ON DELETE CASCADE,
    action VARCHAR(30) NOT NULL,
    from_status VARCHAR(30),
    to_status VARCHAR(30),
    performed_by UUID,
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_absence_types_org ON _atlas.absence_types(organization_id);
CREATE INDEX IF NOT EXISTS idx_absence_plans_org ON _atlas.absence_plans(organization_id);
CREATE INDEX IF NOT EXISTS idx_absence_plans_type ON _atlas.absence_plans(absence_type_id);
CREATE INDEX IF NOT EXISTS idx_absence_balances_emp ON _atlas.absence_balances(employee_id);
CREATE INDEX IF NOT EXISTS idx_absence_balances_plan ON _atlas.absence_balances(plan_id);
CREATE INDEX IF NOT EXISTS idx_absence_entries_emp ON _atlas.absence_entries(employee_id);
CREATE INDEX IF NOT EXISTS idx_absence_entries_type ON _atlas.absence_entries(absence_type_id);
CREATE INDEX IF NOT EXISTS idx_absence_entries_status ON _atlas.absence_entries(status);
CREATE INDEX IF NOT EXISTS idx_absence_entries_dates ON _atlas.absence_entries(start_date, end_date);
CREATE INDEX IF NOT EXISTS idx_absence_entry_history_entry ON _atlas.absence_entry_history(entry_id);
