-- 024: Project Costing Tables (Oracle Fusion Cloud ERP: Project Management > Project Costing)
--
-- Provides cost transaction tracking, burden schedules, cost adjustments,
-- and GL cost distributions for projects.

-- Project cost transactions
CREATE TABLE IF NOT EXISTS _atlas.project_cost_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    transaction_number VARCHAR(50) NOT NULL,
    project_id UUID NOT NULL,
    project_number VARCHAR(50),
    task_id UUID,
    task_number VARCHAR(50),
    cost_type VARCHAR(30) NOT NULL,          -- labor, material, expense, equipment, other
    raw_cost_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    burdened_cost_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    burden_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    transaction_date DATE NOT NULL,
    gl_date DATE,
    description TEXT,
    supplier_id UUID,
    supplier_name VARCHAR(200),
    employee_id UUID,
    employee_name VARCHAR(200),
    expenditure_category VARCHAR(100),
    quantity NUMERIC(18,4),
    unit_of_measure VARCHAR(30),
    unit_rate NUMERIC(18,4),
    is_billable BOOLEAN NOT NULL DEFAULT false,
    is_capitalizable BOOLEAN NOT NULL DEFAULT false,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    distribution_id UUID,
    original_transaction_id UUID,
    adjustment_type VARCHAR(30),
    adjustment_reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    approved_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, transaction_number)
);

CREATE INDEX IF NOT EXISTS idx_pjct_org ON _atlas.project_cost_transactions(organization_id);
CREATE INDEX IF NOT EXISTS idx_pjct_project ON _atlas.project_cost_transactions(project_id);
CREATE INDEX IF NOT EXISTS idx_pjct_status ON _atlas.project_cost_transactions(status);
CREATE INDEX IF NOT EXISTS idx_pjct_type ON _atlas.project_cost_transactions(cost_type);
CREATE INDEX IF NOT EXISTS idx_pjct_date ON _atlas.project_cost_transactions(transaction_date);

-- Burden schedules
CREATE TABLE IF NOT EXISTS _atlas.burden_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    effective_from DATE NOT NULL,
    effective_to DATE,
    is_default BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_bs_org ON _atlas.burden_schedules(organization_id);
CREATE INDEX IF NOT EXISTS idx_bs_status ON _atlas.burden_schedules(status);

-- Burden schedule lines
CREATE TABLE IF NOT EXISTS _atlas.burden_schedule_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_id UUID NOT NULL REFERENCES _atlas.burden_schedules(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    cost_type VARCHAR(30) NOT NULL,
    expenditure_category VARCHAR(100),
    burden_rate_percent NUMERIC(8,4) NOT NULL DEFAULT 0,
    burden_account_code VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_bsl_schedule ON _atlas.burden_schedule_lines(schedule_id);
CREATE INDEX IF NOT EXISTS idx_bsl_type ON _atlas.burden_schedule_lines(cost_type);

-- Cost adjustments
CREATE TABLE IF NOT EXISTS _atlas.project_cost_adjustments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    adjustment_number VARCHAR(50) NOT NULL,
    original_transaction_id UUID NOT NULL,
    adjustment_type VARCHAR(30) NOT NULL,       -- increase, decrease, transfer, reversal
    adjustment_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    new_raw_cost NUMERIC(18,2) NOT NULL DEFAULT 0,
    new_burdened_cost NUMERIC(18,2) NOT NULL DEFAULT 0,
    reason TEXT NOT NULL,
    description TEXT,
    effective_date DATE NOT NULL,
    transfer_to_project_id UUID,
    transfer_to_task_id UUID,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    created_transaction_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, adjustment_number)
);

CREATE INDEX IF NOT EXISTS idx_pjca_org ON _atlas.project_cost_adjustments(organization_id);
CREATE INDEX IF NOT EXISTS idx_pjca_status ON _atlas.project_cost_adjustments(status);
CREATE INDEX IF NOT EXISTS idx_pjca_original ON _atlas.project_cost_adjustments(original_transaction_id);

-- Cost distributions (GL postings)
CREATE TABLE IF NOT EXISTS _atlas.project_cost_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    transaction_id UUID NOT NULL,
    line_number INT NOT NULL,
    debit_account_code VARCHAR(50) NOT NULL,
    credit_account_code VARCHAR(50) NOT NULL,
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    distribution_type VARCHAR(30) NOT NULL DEFAULT 'raw_cost',
    gl_date DATE NOT NULL,
    is_posted BOOLEAN NOT NULL DEFAULT false,
    gl_batch_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pjcd_org ON _atlas.project_cost_distributions(organization_id);
CREATE INDEX IF NOT EXISTS idx_pjcd_txn ON _atlas.project_cost_distributions(transaction_id);
CREATE INDEX IF NOT EXISTS idx_pjcd_posted ON _atlas.project_cost_distributions(is_posted);
