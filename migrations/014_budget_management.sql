-- Atlas ERP - Budget Management
-- Oracle Fusion Cloud ERP: General Ledger > Budgets
--
-- Manages budget definitions, budget versions with approval workflow,
-- budget lines (by account, period, department), budget vs. actuals
-- variance reporting, budget transfers, and budget controls.
--
-- This is a standard Oracle Fusion Cloud ERP feature for planning
-- and controlling expenditures across fiscal periods.

-- ============================================================================
-- Budget Definitions (Templates)
-- Oracle Fusion: Define budget structures with control settings
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.budget_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Budget code / identifier (e.g., 'FY2024_OPEx', 'FY2024_CAPEx')
    code VARCHAR(50) NOT NULL,
    -- Display name
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Which fiscal calendar this budget is for
    calendar_id UUID,
    fiscal_year INT,
    -- Budget type: 'operating', 'capital', 'project', 'cash_flow'
    budget_type VARCHAR(30) NOT NULL DEFAULT 'operating',
    -- Budget control level
    -- 'none' = no control, 'advisory' = warn on over-budget, 'absolute' = block over-budget
    control_level VARCHAR(20) NOT NULL DEFAULT 'none',
    -- Whether budget allows carry-forward of unspent amounts
    allow_carry_forward BOOLEAN DEFAULT false,
    -- Whether budget transfers between accounts are allowed
    allow_transfers BOOLEAN DEFAULT true,
    -- Default currency for this budget
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Status: 'active', 'inactive'
    is_active BOOLEAN DEFAULT true,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_budget_definitions_org ON _atlas.budget_definitions(organization_id);
CREATE INDEX idx_budget_definitions_active ON _atlas.budget_definitions(organization_id, is_active) WHERE is_active = true;
CREATE INDEX idx_budget_definitions_fiscal_year ON _atlas.budget_definitions(organization_id, fiscal_year);

-- ============================================================================
-- Budget Versions (Snapshots with workflow)
-- Oracle Fusion: Create and submit budget versions for approval
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.budget_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- The budget definition this version belongs to
    definition_id UUID NOT NULL REFERENCES _atlas.budget_definitions(id),
    -- Version number (auto-incremented per definition)
    version_number INT NOT NULL DEFAULT 1,
    -- Version label (e.g., 'Original', 'Revised Q2', 'Final')
    label VARCHAR(100),
    -- Status: 'draft', 'submitted', 'approved', 'active', 'closed', 'rejected'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Totals (calculated from budget lines)
    total_budget_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_committed_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_actual_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_variance_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Approval workflow
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    -- Effective dates
    effective_from DATE,
    effective_to DATE,
    -- Notes
    notes TEXT,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(definition_id, version_number)
);

CREATE INDEX idx_budget_versions_org ON _atlas.budget_versions(organization_id);
CREATE INDEX idx_budget_versions_definition ON _atlas.budget_versions(definition_id);
CREATE INDEX idx_budget_versions_status ON _atlas.budget_versions(organization_id, status);

-- ============================================================================
-- Budget Lines (Individual budget amounts)
-- Oracle Fusion: Enter budget amounts by account, period, and dimension
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.budget_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- The budget version this line belongs to
    version_id UUID NOT NULL REFERENCES _atlas.budget_versions(id) ON DELETE CASCADE,
    -- Line number within the version
    line_number INT NOT NULL,
    -- Account reference
    account_code VARCHAR(50) NOT NULL,
    account_name VARCHAR(200),
    -- Period reference
    period_name VARCHAR(50),
    period_start_date DATE,
    period_end_date DATE,
    fiscal_year INT,
    quarter INT,
    -- Dimension references (optional segmentation)
    department_id UUID,
    department_name VARCHAR(200),
    project_id UUID,
    project_name VARCHAR(200),
    cost_center VARCHAR(50),
    -- Budget amounts
    budget_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    committed_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    actual_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    variance_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    variance_percent NUMERIC(8,4) NOT NULL DEFAULT 0,
    -- Carry-forward
    carry_forward_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Transfer tracking
    transferred_in_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    transferred_out_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Description
    description TEXT,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_budget_lines_org ON _atlas.budget_lines(organization_id);
CREATE INDEX idx_budget_lines_version ON _atlas.budget_lines(version_id);
CREATE INDEX idx_budget_lines_account ON _atlas.budget_lines(account_code);
CREATE INDEX idx_budget_lines_period ON _atlas.budget_lines(period_name);
CREATE INDEX idx_budget_lines_dept ON _atlas.budget_lines(department_id);
CREATE INDEX idx_budget_lines_project ON _atlas.budget_lines(project_id);

-- ============================================================================
-- Budget Transfers
-- Oracle Fusion: Transfer budget amounts between accounts/periods
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.budget_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- The budget version
    version_id UUID NOT NULL REFERENCES _atlas.budget_versions(id),
    -- Transfer identification
    transfer_number VARCHAR(50) NOT NULL,
    description TEXT,
    -- Source
    from_account_code VARCHAR(50) NOT NULL,
    from_period_name VARCHAR(50),
    from_department_id UUID,
    from_cost_center VARCHAR(50),
    -- Destination
    to_account_code VARCHAR(50) NOT NULL,
    to_period_name VARCHAR(50),
    to_department_id UUID,
    to_cost_center VARCHAR(50),
    -- Amount
    amount NUMERIC(18,2) NOT NULL,
    -- Status: 'pending', 'approved', 'rejected', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Approval
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, transfer_number)
);

CREATE INDEX idx_budget_transfers_org ON _atlas.budget_transfers(organization_id);
CREATE INDEX idx_budget_transfers_version ON _atlas.budget_transfers(version_id);
CREATE INDEX idx_budget_transfers_status ON _atlas.budget_transfers(organization_id, status);
