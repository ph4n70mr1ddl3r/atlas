-- ═══════════════════════════════════════════════════════════════════════════════
-- Compensation Management (Oracle Fusion Cloud HCM Compensation Workbench)
-- ═══════════════════════════════════════════════════════════════════════════════
-- Provides compensation plans, cycles (annual review), budget pools,
-- manager worksheets with per-employee allocation lines, and employee
-- compensation statements.

-- Compensation Plans
CREATE TABLE IF NOT EXISTS _atlas.compensation_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    plan_code VARCHAR(50) NOT NULL,
    plan_name VARCHAR(200) NOT NULL,
    description TEXT,
    plan_type VARCHAR(30) NOT NULL DEFAULT 'salary',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    effective_start_date DATE,
    effective_end_date DATE,
    eligibility_criteria JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_comp_plans_org ON _atlas.compensation_plans (organization_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_comp_plans_org_code ON _atlas.compensation_plans (organization_id, plan_code) WHERE is_active = true;

-- Compensation Plan Components
CREATE TABLE IF NOT EXISTS _atlas.compensation_components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    plan_id UUID NOT NULL REFERENCES _atlas.compensation_plans(id) ON DELETE CASCADE,
    component_name VARCHAR(100) NOT NULL,
    component_type VARCHAR(30) NOT NULL DEFAULT 'salary',
    description TEXT,
    is_recurring BOOLEAN NOT NULL DEFAULT true,
    frequency VARCHAR(20),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_comp_components_plan ON _atlas.compensation_components (plan_id);

-- Compensation Cycles
CREATE TABLE IF NOT EXISTS _atlas.compensation_cycles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    cycle_name VARCHAR(200) NOT NULL,
    description TEXT,
    cycle_type VARCHAR(20) NOT NULL DEFAULT 'annual',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    allocation_start_date DATE,
    allocation_end_date DATE,
    review_start_date DATE,
    review_end_date DATE,
    total_budget DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_allocated DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_approved DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_employees INT NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_comp_cycles_org ON _atlas.compensation_cycles (organization_id);

-- Compensation Budget Pools
CREATE TABLE IF NOT EXISTS _atlas.compensation_budget_pools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    cycle_id UUID NOT NULL REFERENCES _atlas.compensation_cycles(id) ON DELETE CASCADE,
    pool_name VARCHAR(200) NOT NULL,
    pool_type VARCHAR(20) NOT NULL DEFAULT 'merit',
    manager_id UUID,
    manager_name VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    total_budget DOUBLE PRECISION NOT NULL DEFAULT 0,
    allocated_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    approved_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    remaining_budget DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_comp_pools_cycle ON _atlas.compensation_budget_pools (cycle_id);
CREATE INDEX IF NOT EXISTS idx_comp_pools_manager ON _atlas.compensation_budget_pools (manager_id);

-- Compensation Worksheets
CREATE TABLE IF NOT EXISTS _atlas.compensation_worksheets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    cycle_id UUID NOT NULL REFERENCES _atlas.compensation_cycles(id) ON DELETE CASCADE,
    pool_id UUID REFERENCES _atlas.compensation_budget_pools(id),
    manager_id UUID NOT NULL,
    manager_name VARCHAR(200),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    total_employees INT NOT NULL DEFAULT 0,
    total_current_salary DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_proposed_salary DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_merit DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_bonus DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_equity DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_compensation_change DOUBLE PRECISION NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_comp_ws_cycle ON _atlas.compensation_worksheets (cycle_id);
CREATE INDEX IF NOT EXISTS idx_comp_ws_manager ON _atlas.compensation_worksheets (manager_id);

-- Compensation Worksheet Lines (per-employee)
CREATE TABLE IF NOT EXISTS _atlas.compensation_worksheet_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    worksheet_id UUID NOT NULL REFERENCES _atlas.compensation_worksheets(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL,
    employee_name VARCHAR(200),
    job_title VARCHAR(200),
    department_name VARCHAR(200),
    current_base_salary DOUBLE PRECISION NOT NULL DEFAULT 0,
    proposed_base_salary DOUBLE PRECISION NOT NULL DEFAULT 0,
    salary_change_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    salary_change_percent DOUBLE PRECISION NOT NULL DEFAULT 0,
    merit_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    bonus_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    equity_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_compensation DOUBLE PRECISION NOT NULL DEFAULT 0,
    performance_rating VARCHAR(10),
    compa_ratio DOUBLE PRECISION NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    manager_comments TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_comp_wsl_ws ON _atlas.compensation_worksheet_lines (worksheet_id);
CREATE INDEX IF NOT EXISTS idx_comp_wsl_employee ON _atlas.compensation_worksheet_lines (employee_id);

-- Compensation Statements (employee view)
CREATE TABLE IF NOT EXISTS _atlas.compensation_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    cycle_id UUID NOT NULL REFERENCES _atlas.compensation_cycles(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL,
    employee_name VARCHAR(200),
    statement_date DATE NOT NULL,
    base_salary DOUBLE PRECISION NOT NULL DEFAULT 0,
    merit_increase DOUBLE PRECISION NOT NULL DEFAULT 0,
    bonus DOUBLE PRECISION NOT NULL DEFAULT 0,
    equity DOUBLE PRECISION NOT NULL DEFAULT 0,
    benefits_value DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_compensation DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_direct_compensation DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_indirect_compensation DOUBLE PRECISION NOT NULL DEFAULT 0,
    change_from_previous DOUBLE PRECISION NOT NULL DEFAULT 0,
    change_percent DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    components JSONB NOT NULL DEFAULT '[]',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_comp_stmt_cycle ON _atlas.compensation_statements (cycle_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_comp_stmt_cycle_emp ON _atlas.compensation_statements (cycle_id, employee_id);
