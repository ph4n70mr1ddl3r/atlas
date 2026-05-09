-- ============================================================================
-- Doubtful Account Allowance / Bad Debt Provision
-- Oracle Fusion: Receivables > Collections > Allowance for Doubtful Accounts
--
-- Manages provision policies, aging bucket definitions, provision runs,
-- and provision history for AR bad debt allowance calculation.
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Provision Policies
-- Defines how the allowance for doubtful accounts is calculated.
-- Methods: aging_based (uses aging bucket percentages),
--          percentage_based (flat % of total AR balance),
--          specific_identification (manual per-customer amounts)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.doubtful_account_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    policy_code VARCHAR(50) NOT NULL,
    policy_name VARCHAR(200) NOT NULL,
    description TEXT,
    calculation_method VARCHAR(30) NOT NULL DEFAULT 'aging_based',
    -- aging_based, percentage_based, specific_identification
    flat_percentage DOUBLE PRECISION DEFAULT 0,
    -- For percentage_based method: flat % applied to total outstanding AR
    default_provision_account VARCHAR(100),
    -- GL account for the provision credit (Allowance for Doubtful Accounts)
    default_expense_account VARCHAR(100),
    -- GL account for the provision debit (Bad Debt Expense)
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    effective_from DATE NOT NULL,
    effective_to DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    -- active, inactive, archived
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, policy_code)
);

-- ============================================================================
-- Aging Bucket Definitions
-- Defines aging bucket ranges and their provision percentages.
-- Each policy can have multiple buckets (e.g., 0-30, 31-60, 61-90, 91+).
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.doubtful_account_aging_buckets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    policy_id UUID NOT NULL REFERENCES _atlas.doubtful_account_policies(id) ON DELETE CASCADE,
    bucket_name VARCHAR(100) NOT NULL,
    -- e.g., "Current", "1-30 Days", "31-60 Days", "61-90 Days", "91+ Days"
    from_days INTEGER NOT NULL DEFAULT 0,
    -- Start of aging range in days (inclusive)
    to_days INTEGER,
    -- End of aging range in days (inclusive), NULL = unlimited
    provision_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Percentage of outstanding balance to provision
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================================
-- Provision Runs
-- Records each execution of the provision calculation.
-- Captures the policy used, total provision amount, and run status.
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.doubtful_account_provision_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(50) NOT NULL,
    policy_id UUID NOT NULL REFERENCES _atlas.doubtful_account_policies(id),
    policy_code VARCHAR(50) NOT NULL,
    run_date DATE NOT NULL,
    as_of_date DATE NOT NULL,
    -- The date for which the provision is calculated
    calculation_method VARCHAR(30) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- draft, calculated, posted, reversed, cancelled
    total_outstanding_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Total AR outstanding balance as of as_of_date
    total_provision_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Total calculated provision
    total_prior_provision DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Prior provision amount (for incremental calculation)
    incremental_provision DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Change from prior provision
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    journal_batch_id UUID,
    -- Reference to the posted journal batch
    journal_entry_number VARCHAR(50),
    description TEXT,
    customer_count INTEGER NOT NULL DEFAULT 0,
    -- Number of customers included in the run
    transaction_count INTEGER NOT NULL DEFAULT 0,
    -- Number of open AR transactions
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, run_number)
);

-- ============================================================================
-- Provision Run Details
-- Line-level detail of provision calculation per aging bucket.
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.doubtful_account_provision_details (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.doubtful_account_provision_runs(id) ON DELETE CASCADE,
    bucket_id UUID REFERENCES _atlas.doubtful_account_aging_buckets(id),
    bucket_name VARCHAR(100) NOT NULL,
    from_days INTEGER NOT NULL DEFAULT 0,
    to_days INTEGER,
    provision_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
    outstanding_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Total outstanding balance in this bucket
    transaction_count INTEGER NOT NULL DEFAULT 0,
    -- Number of transactions in this bucket
    customer_count INTEGER NOT NULL DEFAULT 0,
    -- Number of distinct customers
    provision_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Calculated provision = outstanding_amount * provision_percentage / 100
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================================
-- Provision Run Activities / Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.doubtful_account_provision_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID REFERENCES _atlas.doubtful_account_provision_runs(id) ON DELETE CASCADE,
    policy_id UUID,
    action VARCHAR(50) NOT NULL,
    -- created, calculated, posted, reversed, cancelled
    description TEXT,
    performed_by UUID,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    old_status VARCHAR(20),
    new_status VARCHAR(20),
    metadata JSONB
);

-- ============================================================================
-- ============================================================================
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_doubtful_policies_org ON _atlas.doubtful_account_policies(organization_id);
CREATE INDEX IF NOT EXISTS idx_doubtful_policies_status ON _atlas.doubtful_account_policies(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_doubtful_buckets_policy ON _atlas.doubtful_account_aging_buckets(policy_id);
CREATE INDEX IF NOT EXISTS idx_doubtful_runs_org ON _atlas.doubtful_account_provision_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_doubtful_runs_policy ON _atlas.doubtful_account_provision_runs(policy_id);
CREATE INDEX IF NOT EXISTS idx_doubtful_runs_status ON _atlas.doubtful_account_provision_runs(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_doubtful_runs_date ON _atlas.doubtful_account_provision_runs(run_date);
CREATE INDEX IF NOT EXISTS idx_doubtful_details_run ON _atlas.doubtful_account_provision_details(run_id);
CREATE INDEX IF NOT EXISTS idx_doubtful_activities_run ON _atlas.doubtful_account_provision_activities(run_id);
