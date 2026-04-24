-- ═══════════════════════════════════════════════════════════════════════════════
-- Payroll Management (Oracle Fusion Cloud HCM Global Payroll)
-- ═══════════════════════════════════════════════════════════════════════════════
-- Provides payroll definitions, elements (earnings & deductions),
-- employee element entries, payroll run lifecycle, and pay slip generation.

-- Payroll Definitions (pay groups)
CREATE TABLE IF NOT EXISTS _atlas.payroll_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    pay_frequency VARCHAR(20) NOT NULL DEFAULT 'monthly',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    salary_expense_account VARCHAR(50),
    liability_account VARCHAR(50),
    employer_tax_account VARCHAR(50),
    payment_account VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_payroll_defs_org ON _atlas.payroll_definitions (organization_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_payroll_defs_org_name ON _atlas.payroll_definitions (organization_id, name) WHERE is_active = true;

-- Payroll Elements (earning & deduction types)
CREATE TABLE IF NOT EXISTS _atlas.payroll_elements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    element_type VARCHAR(20) NOT NULL, -- "earning" or "deduction"
    category VARCHAR(50) NOT NULL,     -- salary, hourly, overtime, bonus, commission, benefit, tax, retirement, garnishment, other
    calculation_method VARCHAR(20) NOT NULL DEFAULT 'flat', -- flat, percentage, hourly_rate, formula
    default_value NUMERIC,
    is_recurring BOOLEAN NOT NULL DEFAULT true,
    has_employer_contribution BOOLEAN NOT NULL DEFAULT false,
    employer_contribution_rate NUMERIC,
    gl_account_code VARCHAR(50),
    is_pretax BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_payroll_elems_org ON _atlas.payroll_elements (organization_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_payroll_elems_org_code ON _atlas.payroll_elements (organization_id, code) WHERE is_active = true;

-- Element Entries (employee assignments)
CREATE TABLE IF NOT EXISTS _atlas.payroll_element_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    employee_id UUID NOT NULL,
    element_id UUID NOT NULL REFERENCES _atlas.payroll_elements(id),
    element_code VARCHAR(50) NOT NULL,
    element_name VARCHAR(200) NOT NULL,
    element_type VARCHAR(20) NOT NULL,
    entry_value NUMERIC NOT NULL DEFAULT 0,
    remaining_periods INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_payroll_entries_emp ON _atlas.payroll_element_entries (employee_id);
CREATE INDEX IF NOT EXISTS idx_payroll_entries_elem ON _atlas.payroll_element_entries (element_id);

-- Payroll Runs
CREATE TABLE IF NOT EXISTS _atlas.payroll_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payroll_id UUID NOT NULL REFERENCES _atlas.payroll_definitions(id),
    run_number VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'open', -- open, calculated, confirmed, paid, reversed
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    pay_date DATE NOT NULL,
    total_gross NUMERIC NOT NULL DEFAULT 0,
    total_deductions NUMERIC NOT NULL DEFAULT 0,
    total_net NUMERIC NOT NULL DEFAULT 0,
    total_employer_cost NUMERIC NOT NULL DEFAULT 0,
    employee_count INTEGER NOT NULL DEFAULT 0,
    confirmed_by UUID,
    confirmed_at TIMESTAMPTZ,
    paid_by UUID,
    paid_at TIMESTAMPTZ,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_payroll_runs_org ON _atlas.payroll_runs (organization_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_payroll_runs_number ON _atlas.payroll_runs (organization_id, run_number);
CREATE INDEX IF NOT EXISTS idx_payroll_runs_status ON _atlas.payroll_runs (organization_id, status);

-- Pay Slips (per-employee results within a run)
CREATE TABLE IF NOT EXISTS _atlas.pay_slips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payroll_run_id UUID NOT NULL REFERENCES _atlas.payroll_runs(id),
    employee_id UUID NOT NULL,
    employee_name VARCHAR(200),
    gross_earnings NUMERIC NOT NULL DEFAULT 0,
    total_deductions NUMERIC NOT NULL DEFAULT 0,
    net_pay NUMERIC NOT NULL DEFAULT 0,
    employer_cost NUMERIC NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    payment_method VARCHAR(50),
    bank_account_last4 VARCHAR(4),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_pay_slips_run ON _atlas.pay_slips (payroll_run_id);
CREATE INDEX IF NOT EXISTS idx_pay_slips_emp ON _atlas.pay_slips (employee_id);

-- Pay Slip Lines (individual earnings & deductions)
CREATE TABLE IF NOT EXISTS _atlas.pay_slip_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pay_slip_id UUID NOT NULL REFERENCES _atlas.pay_slips(id) ON DELETE CASCADE,
    element_code VARCHAR(50) NOT NULL,
    element_name VARCHAR(200) NOT NULL,
    element_type VARCHAR(20) NOT NULL, -- earning or deduction
    category VARCHAR(50) NOT NULL,
    hours_or_units NUMERIC,
    rate NUMERIC,
    amount NUMERIC NOT NULL DEFAULT 0,
    is_pretax BOOLEAN NOT NULL DEFAULT false,
    is_employer BOOLEAN NOT NULL DEFAULT false,
    gl_account_code VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pay_slip_lines_slip ON _atlas.pay_slip_lines (pay_slip_id);
