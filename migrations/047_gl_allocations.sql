-- ==============================================================================
-- General Ledger Allocations
-- Oracle Fusion Cloud ERP: General Ledger > Allocations
--
-- Supports allocation pools, allocation bases, basis details,
-- allocation rules with target lines, and allocation runs with run lines.
-- ==============================================================================

-- GL Allocation pools (source of costs to distribute)
CREATE TABLE IF NOT EXISTS _atlas.gl_allocation_pools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    pool_type VARCHAR(50) NOT NULL DEFAULT 'cost_center',
    source_account_code VARCHAR(100),
    source_account_range_from VARCHAR(100),
    source_account_range_to VARCHAR(100),
    source_department_id UUID,
    source_project_id UUID,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Allocation bases (method of distribution: headcount, revenue, sqft, etc.)
CREATE TABLE IF NOT EXISTS _atlas.gl_allocation_bases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    basis_type VARCHAR(50) NOT NULL DEFAULT 'statistical',
    unit_of_measure VARCHAR(50),
    is_manual BOOLEAN NOT NULL DEFAULT true,
    source_account_code VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Allocation basis details (individual target entries in a basis)
CREATE TABLE IF NOT EXISTS _atlas.gl_allocation_basis_details (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    basis_id UUID NOT NULL REFERENCES _atlas.gl_allocation_bases(id) ON DELETE CASCADE,
    target_department_id UUID,
    target_department_name VARCHAR(200),
    target_cost_center VARCHAR(100),
    target_project_id UUID,
    target_project_name VARCHAR(200),
    target_account_code VARCHAR(100),
    basis_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    percentage NUMERIC(12, 6) NOT NULL DEFAULT 0,
    period_name VARCHAR(50),
    period_start_date DATE,
    period_end_date DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Allocation rules (mapping from pool to targets using a basis)
CREATE TABLE IF NOT EXISTS _atlas.gl_allocation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    pool_id UUID NOT NULL REFERENCES _atlas.gl_allocation_pools(id) ON DELETE RESTRICT,
    pool_code VARCHAR(100) NOT NULL,
    basis_id UUID NOT NULL REFERENCES _atlas.gl_allocation_bases(id) ON DELETE RESTRICT,
    basis_code VARCHAR(100) NOT NULL,
    allocation_method VARCHAR(50) NOT NULL DEFAULT 'proportional',
    offset_method VARCHAR(50) NOT NULL DEFAULT 'same_account',
    offset_account_code VARCHAR(100),
    journal_batch_prefix VARCHAR(50),
    round_to_largest BOOLEAN NOT NULL DEFAULT false,
    minimum_threshold NUMERIC(18, 2),
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Allocation target lines (for fixed_percentage method)
CREATE TABLE IF NOT EXISTS _atlas.gl_allocation_target_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_id UUID NOT NULL REFERENCES _atlas.gl_allocation_rules(id) ON DELETE CASCADE,
    line_number INT NOT NULL DEFAULT 1,
    target_department_id UUID,
    target_department_name VARCHAR(200),
    target_cost_center VARCHAR(100),
    target_project_id UUID,
    target_project_name VARCHAR(200),
    target_account_code VARCHAR(100) NOT NULL,
    target_account_name VARCHAR(200),
    fixed_percentage NUMERIC(12, 6),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Allocation runs (execution of an allocation rule)
CREATE TABLE IF NOT EXISTS _atlas.gl_allocation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(100) NOT NULL,
    rule_id UUID NOT NULL REFERENCES _atlas.gl_allocation_rules(id) ON DELETE RESTRICT,
    rule_code VARCHAR(100) NOT NULL,
    rule_name VARCHAR(200) NOT NULL,
    period_name VARCHAR(50) NOT NULL,
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    pool_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    allocation_method VARCHAR(50) NOT NULL DEFAULT 'proportional',
    total_allocated NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_offset NUMERIC(18, 2) NOT NULL DEFAULT 0,
    rounding_difference NUMERIC(18, 2) NOT NULL DEFAULT 0,
    target_count INT NOT NULL DEFAULT 0,
    journal_batch_id UUID,
    journal_batch_name VARCHAR(200),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    run_date DATE NOT NULL DEFAULT CURRENT_DATE,
    posted_at TIMESTAMPTZ,
    reversed_at TIMESTAMPTZ,
    posted_by UUID,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Allocation run lines (individual debit/credit entries)
CREATE TABLE IF NOT EXISTS _atlas.gl_allocation_run_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.gl_allocation_runs(id) ON DELETE CASCADE,
    line_number INT NOT NULL DEFAULT 1,
    target_department_id UUID,
    target_department_name VARCHAR(200),
    target_cost_center VARCHAR(100),
    target_project_id UUID,
    target_project_name VARCHAR(200),
    target_account_code VARCHAR(100) NOT NULL,
    target_account_name VARCHAR(200),
    source_account_code VARCHAR(100),
    basis_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    basis_percentage NUMERIC(12, 6) NOT NULL DEFAULT 0,
    allocated_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    offset_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    line_type VARCHAR(20) NOT NULL DEFAULT 'allocation',
    journal_line_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_gl_alloc_pools_org ON _atlas.gl_allocation_pools(organization_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_pools_code ON _atlas.gl_allocation_pools(organization_id, code);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_bases_org ON _atlas.gl_allocation_bases(organization_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_basis_details_basis ON _atlas.gl_allocation_basis_details(basis_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_rules_org ON _atlas.gl_allocation_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_rules_pool ON _atlas.gl_allocation_rules(pool_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_rules_basis ON _atlas.gl_allocation_rules(basis_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_target_lines_rule ON _atlas.gl_allocation_target_lines(rule_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_runs_org ON _atlas.gl_allocation_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_runs_rule ON _atlas.gl_allocation_runs(rule_id);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_runs_status ON _atlas.gl_allocation_runs(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_gl_alloc_run_lines_run ON _atlas.gl_allocation_run_lines(run_id);