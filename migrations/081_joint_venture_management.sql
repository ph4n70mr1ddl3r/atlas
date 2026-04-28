-- Joint Venture Management (Oracle Fusion Cloud Financials > Joint Venture Management)
-- Provides: Joint ventures, venture partners with ownership stakes,
--   cost distributions, revenue distributions, joint interest billing (JIB),
--   AFEs (Authorizations for Expenditure), and partner billing.

-- ============================================================================
-- Joint Ventures: top-level venture agreements
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_ventures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    venture_number VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',            -- draft, active, on_hold, closed
    operator_id UUID,                                         -- the managing partner
    operator_name VARCHAR(300),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    start_date DATE,
    end_date DATE,
    accounting_method VARCHAR(50) DEFAULT 'proportional',    -- proportional, equity, cost_method
    billing_cycle VARCHAR(50) DEFAULT 'monthly',             -- monthly, quarterly, semi_annual, annual
    cost_cap_amount NUMERIC(18,4),
    cost_cap_currency VARCHAR(3),
    gl_revenue_account VARCHAR(50),
    gl_cost_account VARCHAR(50),
    gl_billing_account VARCHAR(50),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, venture_number)
);

CREATE INDEX IF NOT EXISTS idx_joint_ventures_org ON _atlas.joint_ventures(organization_id);
CREATE INDEX IF NOT EXISTS idx_joint_ventures_status ON _atlas.joint_ventures(status);
CREATE INDEX IF NOT EXISTS idx_joint_ventures_operator ON _atlas.joint_ventures(operator_id);

-- ============================================================================
-- Venture Partners: ownership interests per joint venture
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_partners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    venture_id UUID NOT NULL REFERENCES _atlas.joint_ventures(id),
    partner_id UUID NOT NULL,                                -- external partner entity
    partner_name VARCHAR(300) NOT NULL,
    partner_type VARCHAR(50) NOT NULL DEFAULT 'owner',       -- operator, non_operator, carried_interest
    ownership_percentage NUMERIC(8,4) NOT NULL DEFAULT 0,    -- 0.0000 - 100.0000
    revenue_interest_pct NUMERIC(8,4),                       -- may differ from ownership
    cost_bearing_pct NUMERIC(8,4),                           -- may differ from ownership
    role VARCHAR(50) DEFAULT 'partner',                      -- operator, partner, carried
    billing_contact VARCHAR(300),
    billing_email VARCHAR(300),
    billing_address TEXT,
    effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_to DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'active',            -- active, withdrawn, suspended
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_jv_partners_venture ON _atlas.joint_venture_partners(venture_id);
CREATE INDEX IF NOT EXISTS idx_jv_partners_partner ON _atlas.joint_venture_partners(partner_id);
CREATE INDEX IF NOT EXISTS idx_jv_partners_org ON _atlas.joint_venture_partners(organization_id);

-- ============================================================================
-- AFEs (Authorizations for Expenditure): spending authorizations
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_afes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    venture_id UUID NOT NULL REFERENCES _atlas.joint_ventures(id),
    afe_number VARCHAR(100) NOT NULL,
    title VARCHAR(300) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',             -- draft, submitted, approved, rejected, closed
    estimated_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    actual_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    committed_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    remaining_budget NUMERIC(18,4) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    cost_center VARCHAR(100),
    work_area VARCHAR(200),
    well_name VARCHAR(200),
    requested_by UUID,
    requested_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, afe_number)
);

CREATE INDEX IF NOT EXISTS idx_jv_afes_venture ON _atlas.joint_venture_afes(venture_id);
CREATE INDEX IF NOT EXISTS idx_jv_afes_status ON _atlas.joint_venture_afes(status);
CREATE INDEX IF NOT EXISTS idx_jv_afes_org ON _atlas.joint_venture_afes(organization_id);

-- ============================================================================
-- Cost Distributions: allocate costs across partners
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_cost_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    venture_id UUID NOT NULL REFERENCES _atlas.joint_ventures(id),
    distribution_number VARCHAR(100) NOT NULL,
    afe_id UUID REFERENCES _atlas.joint_venture_afes(id),
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',             -- draft, posted, reversed
    total_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    cost_type VARCHAR(50) NOT NULL DEFAULT 'operating',      -- operating, capital, aba, overhead
    distribution_date DATE NOT NULL DEFAULT CURRENT_DATE,
    gl_posting_date DATE,
    gl_posted_at TIMESTAMPTZ,
    source_type VARCHAR(100),                                 -- ap_invoice, gl_journal, project_cost
    source_id UUID,
    source_number VARCHAR(100),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, distribution_number)
);

CREATE INDEX IF NOT EXISTS idx_jv_cost_dist_venture ON _atlas.joint_venture_cost_distributions(venture_id);
CREATE INDEX IF NOT EXISTS idx_jv_cost_dist_status ON _atlas.joint_venture_cost_distributions(status);
CREATE INDEX IF NOT EXISTS idx_jv_cost_dist_org ON _atlas.joint_venture_cost_distributions(organization_id);

-- ============================================================================
-- Cost Distribution Lines: per-partner split
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_cost_distribution_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    distribution_id UUID NOT NULL REFERENCES _atlas.joint_venture_cost_distributions(id),
    partner_id UUID NOT NULL,
    partner_name VARCHAR(300),
    ownership_pct NUMERIC(8,4) NOT NULL DEFAULT 0,
    cost_bearing_pct NUMERIC(8,4) NOT NULL DEFAULT 0,
    distributed_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    gl_account_code VARCHAR(50),
    line_description TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_jv_cost_dist_lines_dist ON _atlas.joint_venture_cost_distribution_lines(distribution_id);
CREATE INDEX IF NOT EXISTS idx_jv_cost_dist_lines_partner ON _atlas.joint_venture_cost_distribution_lines(partner_id);

-- ============================================================================
-- Revenue Distributions: allocate revenue across partners
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_revenue_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    venture_id UUID NOT NULL REFERENCES _atlas.joint_ventures(id),
    distribution_number VARCHAR(100) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',             -- draft, posted, reversed
    total_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    revenue_type VARCHAR(50) NOT NULL DEFAULT 'sales',       -- sales, royalty, bonus, other
    distribution_date DATE NOT NULL DEFAULT CURRENT_DATE,
    gl_posting_date DATE,
    gl_posted_at TIMESTAMPTZ,
    source_type VARCHAR(100),
    source_id UUID,
    source_number VARCHAR(100),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, distribution_number)
);

CREATE INDEX IF NOT EXISTS idx_jv_rev_dist_venture ON _atlas.joint_venture_revenue_distributions(venture_id);
CREATE INDEX IF NOT EXISTS idx_jv_rev_dist_status ON _atlas.joint_venture_revenue_distributions(status);
CREATE INDEX IF NOT EXISTS idx_jv_rev_dist_org ON _atlas.joint_venture_revenue_distributions(organization_id);

-- ============================================================================
-- Revenue Distribution Lines: per-partner split
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_revenue_distribution_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    distribution_id UUID NOT NULL REFERENCES _atlas.joint_venture_revenue_distributions(id),
    partner_id UUID NOT NULL,
    partner_name VARCHAR(300),
    revenue_interest_pct NUMERIC(8,4) NOT NULL DEFAULT 0,
    distributed_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    gl_account_code VARCHAR(50),
    line_description TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_jv_rev_dist_lines_dist ON _atlas.joint_venture_revenue_distribution_lines(distribution_id);
CREATE INDEX IF NOT EXISTS idx_jv_rev_dist_lines_partner ON _atlas.joint_venture_revenue_distribution_lines(partner_id);

-- ============================================================================
-- Joint Interest Billings (JIB): partner invoices for cost shares
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_billings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    venture_id UUID NOT NULL REFERENCES _atlas.joint_ventures(id),
    billing_number VARCHAR(100) NOT NULL,
    partner_id UUID NOT NULL,
    partner_name VARCHAR(300),
    billing_type VARCHAR(50) NOT NULL DEFAULT 'jib',         -- jib (cost), revenue, adjustment
    status VARCHAR(50) NOT NULL DEFAULT 'draft',             -- draft, submitted, approved, paid, disputed, cancelled
    total_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    total_with_tax NUMERIC(18,4) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    billing_period_start DATE NOT NULL,
    billing_period_end DATE NOT NULL,
    due_date DATE,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    payment_reference VARCHAR(200),
    dispute_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, billing_number)
);

CREATE INDEX IF NOT EXISTS idx_jv_billings_venture ON _atlas.joint_venture_billings(venture_id);
CREATE INDEX IF NOT EXISTS idx_jv_billings_partner ON _atlas.joint_venture_billings(partner_id);
CREATE INDEX IF NOT EXISTS idx_jv_billings_status ON _atlas.joint_venture_billings(status);
CREATE INDEX IF NOT EXISTS idx_jv_billings_org ON _atlas.joint_venture_billings(organization_id);

-- ============================================================================
-- Billing Lines: detail lines on a JIB invoice
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_billing_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    billing_id UUID NOT NULL REFERENCES _atlas.joint_venture_billings(id),
    line_number INT NOT NULL DEFAULT 1,
    cost_distribution_id UUID REFERENCES _atlas.joint_venture_cost_distributions(id),
    revenue_distribution_id UUID REFERENCES _atlas.joint_venture_revenue_distributions(id),
    description TEXT,
    cost_type VARCHAR(50),
    amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    ownership_pct NUMERIC(8,4),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_jv_billing_lines_billing ON _atlas.joint_venture_billing_lines(billing_id);

-- ============================================================================
-- Joint Venture Dashboard Summary View (convenience)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.joint_venture_dashboard (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    total_ventures INT DEFAULT 0,
    active_ventures INT DEFAULT 0,
    total_partners INT DEFAULT 0,
    total_cost_distributed NUMERIC(18,4) DEFAULT 0,
    total_revenue_distributed NUMERIC(18,4) DEFAULT 0,
    total_billed NUMERIC(18,4) DEFAULT 0,
    total_collected NUMERIC(18,4) DEFAULT 0,
    outstanding_balance NUMERIC(18,4) DEFAULT 0,
    pending_afes INT DEFAULT 0,
    ventures_by_status JSONB DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id)
);
