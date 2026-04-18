-- 025_cost_allocation.sql
-- Cost Allocation tables (Oracle Fusion GL > Allocations / Mass Allocations)
-- Provides cost pools, allocation bases, allocation rules, and execution runs.

-- Allocation Pools
CREATE TABLE IF NOT EXISTS _atlas.allocation_pools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    pool_type VARCHAR(50) NOT NULL DEFAULT 'cost_center',
    source_account_codes JSONB NOT NULL DEFAULT '[]',
    source_department_id UUID,
    source_cost_center VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Allocation Bases
CREATE TABLE IF NOT EXISTS _atlas.allocation_bases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    base_type VARCHAR(50) NOT NULL DEFAULT 'statistical',
    financial_account_code VARCHAR(100),
    unit_of_measure VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Statistical Base Values
CREATE TABLE IF NOT EXISTS _atlas.allocation_base_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    base_id UUID NOT NULL REFERENCES _atlas.allocation_bases(id) ON DELETE CASCADE,
    base_code VARCHAR(100) NOT NULL,
    department_id UUID,
    department_name VARCHAR(200),
    cost_center VARCHAR(100),
    project_id UUID,
    value NUMERIC(18,4) NOT NULL DEFAULT 0,
    effective_date DATE NOT NULL,
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(base_id, COALESCE(department_id, '00000000-0000-0000-0000-000000000000'),
           COALESCE(cost_center, ''), effective_date)
);

-- Allocation Rules
CREATE TABLE IF NOT EXISTS _atlas.allocation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    pool_id UUID NOT NULL REFERENCES _atlas.allocation_pools(id),
    pool_code VARCHAR(100) NOT NULL,
    base_id UUID NOT NULL REFERENCES _atlas.allocation_bases(id),
    base_code VARCHAR(100) NOT NULL,
    allocation_method VARCHAR(50) NOT NULL DEFAULT 'proportional',
    journal_description TEXT,
    offset_account_code VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    current_version INT NOT NULL DEFAULT 1,
    effective_from DATE,
    effective_to DATE,
    is_reversing BOOLEAN NOT NULL DEFAULT false,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, rule_number)
);

-- Allocation Rule Targets (lines)
CREATE TABLE IF NOT EXISTS _atlas.allocation_rule_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_id UUID NOT NULL REFERENCES _atlas.allocation_rules(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    department_id UUID,
    department_name VARCHAR(200),
    cost_center VARCHAR(100),
    project_id UUID,
    project_name VARCHAR(200),
    target_account_code VARCHAR(100) NOT NULL,
    fixed_percent NUMERIC(10,4),
    fixed_amount NUMERIC(18,4),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Allocation Runs (executions)
CREATE TABLE IF NOT EXISTS _atlas.allocation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(50) NOT NULL,
    rule_id UUID NOT NULL REFERENCES _atlas.allocation_rules(id),
    rule_name VARCHAR(200) NOT NULL,
    rule_number VARCHAR(50) NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    total_source_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    total_allocated_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    line_count INT NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    journal_entry_id UUID,
    run_date DATE NOT NULL,
    reversed_by_id UUID,
    reversal_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Allocation Run Lines (debit/credit entries)
CREATE TABLE IF NOT EXISTS _atlas.allocation_run_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.allocation_runs(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    line_type VARCHAR(20) NOT NULL,
    account_code VARCHAR(100) NOT NULL,
    department_id UUID,
    department_name VARCHAR(200),
    cost_center VARCHAR(100),
    project_id UUID,
    amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    base_value_used NUMERIC(18,4),
    percent_of_total NUMERIC(10,4),
    description TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_alloc_pools_org ON _atlas.allocation_pools(organization_id);
CREATE INDEX IF NOT EXISTS idx_alloc_bases_org ON _atlas.allocation_bases(organization_id);
CREATE INDEX IF NOT EXISTS idx_alloc_base_values_base_date ON _atlas.allocation_base_values(base_id, effective_date);
CREATE INDEX IF NOT EXISTS idx_alloc_rules_org ON _atlas.allocation_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_alloc_rules_status ON _atlas.allocation_rules(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_alloc_rule_targets_rule ON _atlas.allocation_rule_targets(rule_id);
CREATE INDEX IF NOT EXISTS idx_alloc_runs_org ON _atlas.allocation_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_alloc_runs_rule ON _atlas.allocation_runs(rule_id);
CREATE INDEX IF NOT EXISTS idx_alloc_run_lines_run ON _atlas.allocation_run_lines(run_id);
