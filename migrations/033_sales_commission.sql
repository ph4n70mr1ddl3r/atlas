-- 033_sales_commission.sql
-- Sales Commission Management (Oracle Fusion Cloud ERP: Incentive Compensation)
-- Tables for sales representatives, commission plans, rate tiers, plan assignments,
-- sales quotas, commission transactions, payouts, and payout lines.

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ─── Sales Representatives ────────────────────────────────────────

CREATE TABLE IF NOT EXISTS _atlas.sales_reps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rep_code VARCHAR(50) NOT NULL,
    employee_id UUID,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email VARCHAR(200),
    territory_code VARCHAR(50),
    territory_name VARCHAR(200),
    manager_id UUID REFERENCES _atlas.sales_reps(id),
    manager_name VARCHAR(200),
    hire_date DATE,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, rep_code)
);

CREATE INDEX IF NOT EXISTS idx_sales_reps_org ON _atlas.sales_reps(organization_id);
CREATE INDEX IF NOT EXISTS idx_sales_reps_active ON _atlas.sales_reps(organization_id, is_active);
CREATE INDEX IF NOT EXISTS idx_sales_reps_territory ON _atlas.sales_reps(territory_code) WHERE territory_code IS NOT NULL;

-- ─── Commission Plans ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS _atlas.commission_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    plan_type VARCHAR(30) NOT NULL DEFAULT 'revenue',  -- revenue, margin, quantity, flat_bonus
    basis VARCHAR(30) NOT NULL DEFAULT 'revenue',       -- revenue, gross_margin, net_margin, quantity
    calculation_method VARCHAR(30) NOT NULL DEFAULT 'percentage', -- percentage, tiered, flat_rate, graduated
    default_rate NUMERIC(18,4) DEFAULT 0,
    effective_from DATE,
    effective_to DATE,
    status VARCHAR(20) DEFAULT 'draft',  -- draft, active, inactive, expired
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_commission_plans_org ON _atlas.commission_plans(organization_id);
CREATE INDEX IF NOT EXISTS idx_commission_plans_status ON _atlas.commission_plans(organization_id, status);

-- ─── Commission Rate Tiers ────────────────────────────────────────

CREATE TABLE IF NOT EXISTS _atlas.commission_rate_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    plan_id UUID NOT NULL REFERENCES _atlas.commission_plans(id) ON DELETE CASCADE,
    tier_number INT NOT NULL,
    from_amount NUMERIC(18,4) DEFAULT 0,
    to_amount NUMERIC(18,4),
    rate_percent NUMERIC(18,4) DEFAULT 0,
    flat_amount NUMERIC(18,4),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_commission_rate_tiers_plan ON _atlas.commission_rate_tiers(plan_id);

-- ─── Plan Assignments ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS _atlas.plan_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rep_id UUID NOT NULL REFERENCES _atlas.sales_reps(id),
    plan_id UUID NOT NULL REFERENCES _atlas.commission_plans(id),
    effective_from DATE NOT NULL,
    effective_to DATE,
    status VARCHAR(20) DEFAULT 'active',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_plan_assignments_rep ON _atlas.plan_assignments(rep_id);
CREATE INDEX IF NOT EXISTS idx_plan_assignments_plan ON _atlas.plan_assignments(plan_id);

-- ─── Sales Quotas ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS _atlas.sales_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rep_id UUID NOT NULL REFERENCES _atlas.sales_reps(id),
    plan_id UUID REFERENCES _atlas.commission_plans(id),
    quota_number VARCHAR(50) NOT NULL,
    period_name VARCHAR(100) NOT NULL,
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    quota_type VARCHAR(30) NOT NULL DEFAULT 'revenue',  -- revenue, units, margin, activities
    target_amount NUMERIC(18,4) DEFAULT 0,
    achieved_amount NUMERIC(18,4) DEFAULT 0,
    achievement_percent NUMERIC(18,4) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'active',  -- draft, active, closed
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sales_quotas_rep ON _atlas.sales_quotas(rep_id);
CREATE INDEX IF NOT EXISTS idx_sales_quotas_period ON _atlas.sales_quotas(period_start_date, period_end_date);

-- ─── Commission Transactions ──────────────────────────────────────

CREATE TABLE IF NOT EXISTS _atlas.commission_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rep_id UUID NOT NULL REFERENCES _atlas.sales_reps(id),
    plan_id UUID REFERENCES _atlas.commission_plans(id),
    quota_id UUID REFERENCES _atlas.sales_quotas(id),
    transaction_number VARCHAR(50) NOT NULL,
    source_type VARCHAR(50),           -- sales_order, invoice, etc.
    source_id UUID,
    source_number VARCHAR(100),
    transaction_date DATE NOT NULL,
    sale_amount NUMERIC(18,4) DEFAULT 0,
    commission_basis_amount NUMERIC(18,4) DEFAULT 0,
    commission_rate NUMERIC(18,4) DEFAULT 0,
    commission_amount NUMERIC(18,4) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(20) DEFAULT 'credited',  -- credited, included, paid, cancelled
    payout_id UUID REFERENCES _atlas.commission_payouts(id),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_commission_txns_rep ON _atlas.commission_transactions(rep_id);
CREATE INDEX IF NOT EXISTS idx_commission_txns_status ON _atlas.commission_transactions(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_commission_txns_date ON _atlas.commission_transactions(transaction_date);
CREATE INDEX IF NOT EXISTS idx_commission_txns_source ON _atlas.commission_transactions(source_type, source_id) WHERE source_type IS NOT NULL;

-- ─── Commission Payouts ───────────────────────────────────────────

CREATE TABLE IF NOT EXISTS _atlas.commission_payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payout_number VARCHAR(50) NOT NULL,
    period_name VARCHAR(100) NOT NULL,
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    total_payout_amount NUMERIC(18,4) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    rep_count INT DEFAULT 0,
    transaction_count INT DEFAULT 0,
    status VARCHAR(20) DEFAULT 'draft',  -- draft, approved, paid, rejected
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_commission_payouts_org ON _atlas.commission_payouts(organization_id);
CREATE INDEX IF NOT EXISTS idx_commission_payouts_status ON _atlas.commission_payouts(organization_id, status);

-- ─── Commission Payout Lines ──────────────────────────────────────

CREATE TABLE IF NOT EXISTS _atlas.commission_payout_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payout_id UUID NOT NULL REFERENCES _atlas.commission_payouts(id) ON DELETE CASCADE,
    rep_id UUID NOT NULL REFERENCES _atlas.sales_reps(id),
    rep_name VARCHAR(200) NOT NULL,
    plan_id UUID REFERENCES _atlas.commission_plans(id),
    plan_code VARCHAR(50),
    gross_commission NUMERIC(18,4) DEFAULT 0,
    adjustment_amount NUMERIC(18,4) DEFAULT 0,
    net_commission NUMERIC(18,4) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    transaction_count INT DEFAULT 0,
    status VARCHAR(20) DEFAULT 'pending',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_commission_payout_lines_payout ON _atlas.commission_payout_lines(payout_id);
CREATE INDEX IF NOT EXISTS idx_commission_payout_lines_rep ON _atlas.commission_payout_lines(rep_id);
