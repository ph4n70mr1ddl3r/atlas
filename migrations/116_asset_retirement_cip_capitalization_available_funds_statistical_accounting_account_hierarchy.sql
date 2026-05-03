-- Asset Retirement, CIP Capitalization, Available Funds Check,
-- Statistical Accounting, and Account Hierarchy tables
-- Oracle Fusion Financial features implementation

-- ============================================================================
-- Asset Retirement (Oracle Fusion: FA > Asset Retirements)
-- ============================================================================
CREATE TABLE IF NOT EXISTS asset_retirements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    retirement_number VARCHAR(50) NOT NULL,
    asset_id UUID NOT NULL,
    asset_number VARCHAR(100),
    asset_description TEXT,
    retirement_type VARCHAR(30) NOT NULL CHECK (retirement_type IN ('sale','scrap','donation','transfer','theft','destruction')),
    retirement_date DATE NOT NULL,
    cost DECIMAL(18,2) NOT NULL DEFAULT 0,
    accumulated_depreciation DECIMAL(18,2) NOT NULL DEFAULT 0,
    net_book_value DECIMAL(18,2) NOT NULL DEFAULT 0,
    proceeds DECIMAL(18,2) NOT NULL DEFAULT 0,
    removal_cost DECIMAL(18,2) NOT NULL DEFAULT 0,
    gain_loss_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    gain_loss_account VARCHAR(100),
    asset_account VARCHAR(100),
    depreciation_account VARCHAR(100),
    proceeds_account VARCHAR(100),
    removal_cost_account VARCHAR(100),
    buyer_name VARCHAR(200),
    reason TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','completed','reversed','cancelled')),
    approved_by UUID,
    posted_to_gl BOOLEAN NOT NULL DEFAULT FALSE,
    gl_batch_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, retirement_number)
);

CREATE INDEX idx_asset_retirements_org ON asset_retirements(organization_id);
CREATE INDEX idx_asset_retirements_asset ON asset_retirements(asset_id);
CREATE INDEX idx_asset_retirements_status ON asset_retirements(status);

-- ============================================================================
-- CIP Capitalization (Oracle Fusion: FA > CIP Capitalization)
-- ============================================================================
CREATE TABLE IF NOT EXISTS cip_capitalizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    capitalization_number VARCHAR(50) NOT NULL,
    cip_asset_id UUID NOT NULL,
    cip_asset_number VARCHAR(100),
    cip_asset_name VARCHAR(200),
    capitalized_asset_id UUID,
    capitalized_asset_number VARCHAR(100),
    asset_type VARCHAR(30) NOT NULL CHECK (asset_type IN ('tangible','intangible','leased','group')),
    category_id UUID,
    category_code VARCHAR(100),
    book_id UUID,
    book_code VARCHAR(100),
    depreciation_method VARCHAR(100),
    useful_life_months INTEGER,
    salvage_value DECIMAL(18,2) NOT NULL DEFAULT 0,
    total_cip_cost DECIMAL(18,2) NOT NULL DEFAULT 0,
    capitalized_cost DECIMAL(18,2) NOT NULL DEFAULT 0,
    capitalization_date DATE NOT NULL,
    date_placed_in_service DATE,
    location VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    asset_account VARCHAR(100),
    cip_account VARCHAR(100),
    depreciation_account VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','submitted','approved','capitalized','reversed','cancelled')),
    approved_by UUID,
    posted_to_gl BOOLEAN NOT NULL DEFAULT FALSE,
    gl_batch_id UUID,
    notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, capitalization_number)
);

CREATE TABLE IF NOT EXISTS cip_capitalization_cost_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    capitalization_id UUID NOT NULL REFERENCES cip_capitalizations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    source_type VARCHAR(50) NOT NULL,
    source_id UUID,
    source_number VARCHAR(100),
    description TEXT,
    cost_amount DECIMAL(18,2) NOT NULL,
    included BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cip_caps_org ON cip_capitalizations(organization_id);
CREATE INDEX idx_cip_caps_status ON cip_capitalizations(status);
CREATE INDEX idx_cip_cost_lines_cap ON cip_capitalization_cost_lines(capitalization_id);

-- ============================================================================
-- Available Funds Check (Oracle Fusion: GL > Budgetary Control)
-- ============================================================================
CREATE TABLE IF NOT EXISTS funds_check_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    check_number VARCHAR(50) NOT NULL,
    budget_code VARCHAR(100) NOT NULL,
    account_code VARCHAR(100) NOT NULL,
    fund_type VARCHAR(20) NOT NULL DEFAULT 'budget',
    fiscal_year INTEGER NOT NULL,
    period_number INTEGER NOT NULL,
    budget_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    encumbered_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    actual_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    available_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    requested_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    result_after DECIMAL(18,2) NOT NULL DEFAULT 0,
    check_result VARCHAR(20) NOT NULL CHECK (check_result IN ('pass','warning','fail','advisory')),
    tolerance_type VARCHAR(20),
    tolerance_value VARCHAR(50),
    reference_type VARCHAR(50),
    reference_id UUID,
    reference_number VARCHAR(100),
    metadata JSONB NOT NULL DEFAULT '{}',
    checked_by UUID,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, check_number)
);

CREATE TABLE IF NOT EXISTS funds_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    check_id UUID NOT NULL REFERENCES funds_check_results(id),
    override_number VARCHAR(50) NOT NULL,
    reason TEXT NOT NULL,
    approval_level VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected','expired')),
    approved_by UUID,
    approval_notes TEXT,
    expires_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    requested_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, override_number)
);

CREATE INDEX idx_funds_checks_org ON funds_check_results(organization_id);
CREATE INDEX idx_funds_checks_budget ON funds_check_results(budget_code);
CREATE INDEX idx_funds_checks_result ON funds_check_results(check_result);
CREATE INDEX idx_funds_overrides_org ON funds_overrides(organization_id);
CREATE INDEX idx_funds_overrides_status ON funds_overrides(status);

-- ============================================================================
-- Statistical Accounting (Oracle Fusion: GL > Statistical Accounting)
-- ============================================================================
CREATE TABLE IF NOT EXISTS statistical_units (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    stat_type VARCHAR(30) NOT NULL CHECK (stat_type IN ('headcount','square_footage','units_produced','machine_hours','labor_hours','transactions','vehicles','lines_of_code','custom')),
    unit_of_measure VARCHAR(30) NOT NULL CHECK (unit_of_measure IN ('people','sqft','sqm','units','hours','transactions','vehicles','kloc','each')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS statistical_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entry_number VARCHAR(50) NOT NULL,
    statistical_unit_id UUID NOT NULL REFERENCES statistical_units(id),
    statistical_unit_code VARCHAR(100),
    account_code VARCHAR(100),
    dimension1 VARCHAR(200),
    dimension2 VARCHAR(200),
    dimension3 VARCHAR(200),
    fiscal_year INTEGER NOT NULL,
    period_number INTEGER NOT NULL,
    quantity DECIMAL(18,4) NOT NULL,
    unit_cost DECIMAL(18,4),
    extended_amount DECIMAL(18,2),
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','posted','reversed')),
    source_type VARCHAR(50),
    source_id UUID,
    source_number VARCHAR(100),
    description TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, entry_number)
);

CREATE INDEX idx_stat_units_org ON statistical_units(organization_id);
CREATE INDEX idx_stat_entries_org ON statistical_entries(organization_id);
CREATE INDEX idx_stat_entries_unit ON statistical_entries(statistical_unit_id);
CREATE INDEX idx_stat_entries_period ON statistical_entries(fiscal_year, period_number);

-- ============================================================================
-- Account Hierarchy (Oracle Fusion: GL > Chart of Accounts > Hierarchies)
-- ============================================================================
CREATE TABLE IF NOT EXISTS account_hierarchies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    hierarchy_type VARCHAR(30) NOT NULL CHECK (hierarchy_type IN ('account','cost_center','entity','product','project','intercompany','custom')),
    version INTEGER NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','active','inactive')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS account_hierarchy_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hierarchy_id UUID NOT NULL REFERENCES account_hierarchies(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    parent_id UUID REFERENCES account_hierarchy_nodes(id) ON DELETE CASCADE,
    account_code VARCHAR(100) NOT NULL,
    account_name VARCHAR(200),
    node_type VARCHAR(20) NOT NULL CHECK (node_type IN ('root','summary','detail')),
    display_order INTEGER NOT NULL DEFAULT 0,
    level_depth INTEGER NOT NULL DEFAULT 0,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_acct_hierarchies_org ON account_hierarchies(organization_id);
CREATE INDEX idx_acct_hier_nodes_hierarchy ON account_hierarchy_nodes(hierarchy_id);
CREATE INDEX idx_acct_hier_nodes_parent ON account_hierarchy_nodes(parent_id);
CREATE INDEX idx_acct_hier_nodes_code ON account_hierarchy_nodes(account_code);
