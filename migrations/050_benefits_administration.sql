-- Benefits Administration (Oracle Fusion HCM > Benefits)
-- Manages employee benefits plans, coverage tiers, enrollments,
-- qualifying life events, and payroll deductions.
--
-- Oracle Fusion equivalent: Benefits > Benefits Plans, Enrollments, Coverage

-- Benefits plans
CREATE TABLE IF NOT EXISTS _atlas.benefits_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    plan_type VARCHAR(50) NOT NULL,  -- medical, dental, vision, life_insurance, disability, retirement, hsa, fsa
    coverage_tiers JSONB NOT NULL DEFAULT '[]',
    provider_name VARCHAR(200),
    provider_plan_id VARCHAR(100),
    plan_year_start DATE,
    plan_year_end DATE,
    open_enrollment_start DATE,
    open_enrollment_end DATE,
    allow_life_event_changes BOOLEAN NOT NULL DEFAULT true,
    requires_eoi BOOLEAN NOT NULL DEFAULT false,
    waiting_period_days INT NOT NULL DEFAULT 0,
    max_dependents INT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Benefits enrollments
CREATE TABLE IF NOT EXISTS _atlas.benefits_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    employee_id UUID NOT NULL,
    employee_name VARCHAR(200),
    plan_id UUID NOT NULL REFERENCES _atlas.benefits_plans(id),
    plan_code VARCHAR(50),
    plan_name VARCHAR(200),
    plan_type VARCHAR(50),
    coverage_tier VARCHAR(50) NOT NULL,  -- employee_only, employee_spouse, employee_child, family
    enrollment_type VARCHAR(30) NOT NULL,  -- open_enrollment, new_hire, life_event, manual
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, active, waived, cancelled, suspended
    effective_start_date DATE NOT NULL,
    effective_end_date DATE,
    employee_cost TEXT NOT NULL DEFAULT '0',
    employer_cost TEXT NOT NULL DEFAULT '0',
    total_cost TEXT NOT NULL DEFAULT '0',
    deduction_frequency VARCHAR(30) NOT NULL DEFAULT 'per_pay_period',  -- per_pay_period, monthly, semi_monthly
    deduction_account_code VARCHAR(100),
    employer_contribution_account_code VARCHAR(100),
    dependents JSONB NOT NULL DEFAULT '[]',
    life_event_reason VARCHAR(200),
    life_event_date DATE,
    processed_by UUID,
    processed_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    cancelled_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Benefits deductions (per pay period)
CREATE TABLE IF NOT EXISTS _atlas.benefits_deductions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    enrollment_id UUID NOT NULL REFERENCES _atlas.benefits_enrollments(id),
    employee_id UUID NOT NULL,
    plan_id UUID NOT NULL REFERENCES _atlas.benefits_plans(id),
    plan_code VARCHAR(50),
    plan_name VARCHAR(200),
    employee_amount TEXT NOT NULL DEFAULT '0',
    employer_amount TEXT NOT NULL DEFAULT '0',
    total_amount TEXT NOT NULL DEFAULT '0',
    pay_period_start DATE NOT NULL,
    pay_period_end DATE NOT NULL,
    deduction_account_code VARCHAR(100),
    is_processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_benefits_plans_org ON _atlas.benefits_plans(organization_id);
CREATE INDEX IF NOT EXISTS idx_benefits_plans_type ON _atlas.benefits_plans(organization_id, plan_type) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_benefits_enrollments_org ON _atlas.benefits_enrollments(organization_id);
CREATE INDEX IF NOT EXISTS idx_benefits_enrollments_employee ON _atlas.benefits_enrollments(organization_id, employee_id);
CREATE INDEX IF NOT EXISTS idx_benefits_enrollments_plan ON _atlas.benefits_enrollments(organization_id, plan_id);
CREATE INDEX IF NOT EXISTS idx_benefits_enrollments_status ON _atlas.benefits_enrollments(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_benefits_deductions_org ON _atlas.benefits_deductions(organization_id);
CREATE INDEX IF NOT EXISTS idx_benefits_deductions_employee ON _atlas.benefits_deductions(organization_id, employee_id);
CREATE INDEX IF NOT EXISTS idx_benefits_deductions_enrollment ON _atlas.benefits_deductions(enrollment_id);
CREATE INDEX IF NOT EXISTS idx_benefits_deductions_period ON _atlas.benefits_deductions(pay_period_start, pay_period_end);
