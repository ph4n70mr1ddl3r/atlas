-- Account Monitor & Balance Inquiry
-- Oracle Fusion General Ledger > Account Monitor and Balance Inquiry
--
-- Provides real-time GL account balance monitoring, account groups,
-- threshold alerts, and period-over-period comparisons.
--
-- Oracle Fusion equivalent: General Ledger > Journals > Account Monitor,
--                           General Ledger > Journals > Smart Account Inquiry

-- Account Groups: user-defined collections of accounts to monitor together
CREATE TABLE IF NOT EXISTS _atlas.account_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    owner_id UUID,
    is_shared BOOLEAN NOT NULL DEFAULT false,
    threshold_warning_pct NUMERIC(8,4),
    threshold_critical_pct NUMERIC(8,4),
    comparison_type VARCHAR(30) NOT NULL DEFAULT 'prior_period',  -- prior_period, prior_year, budget
    status VARCHAR(30) NOT NULL DEFAULT 'active',  -- active, inactive
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Account Group Members: individual accounts within a group
CREATE TABLE IF NOT EXISTS _atlas.account_group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES _atlas.account_groups(id) ON DELETE CASCADE,
    account_segment VARCHAR(200) NOT NULL,
    account_label VARCHAR(200),
    display_order INT NOT NULL DEFAULT 0,
    include_children BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_agm_group ON _atlas.account_group_members(group_id);

-- Balance Snapshots: point-in-time GL balance data for monitored accounts
CREATE TABLE IF NOT EXISTS _atlas.balance_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    account_group_id UUID NOT NULL REFERENCES _atlas.account_groups(id) ON DELETE CASCADE,
    member_id UUID REFERENCES _atlas.account_group_members(id) ON DELETE CASCADE,
    account_segment VARCHAR(200) NOT NULL,
    period_name VARCHAR(50) NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    fiscal_year INT NOT NULL,
    period_number INT NOT NULL,
    beginning_balance NUMERIC(20,4) NOT NULL DEFAULT 0,
    total_debits NUMERIC(20,4) NOT NULL DEFAULT 0,
    total_credits NUMERIC(20,4) NOT NULL DEFAULT 0,
    net_activity NUMERIC(20,4) NOT NULL DEFAULT 0,
    ending_balance NUMERIC(20,4) NOT NULL DEFAULT 0,
    journal_entry_count INT NOT NULL DEFAULT 0,
    comparison_balance NUMERIC(20,4),
    comparison_period_name VARCHAR(50),
    variance_amount NUMERIC(20,4),
    variance_pct NUMERIC(8,4),
    alert_status VARCHAR(30) NOT NULL DEFAULT 'none',  -- none, info, warning, critical
    snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_bs_group ON _atlas.balance_snapshots(account_group_id);
CREATE INDEX IF NOT EXISTS idx_bs_member ON _atlas.balance_snapshots(member_id);
CREATE INDEX IF NOT EXISTS idx_bs_period ON _atlas.balance_snapshots(organization_id, period_name);
CREATE INDEX IF NOT EXISTS idx_bs_date ON _atlas.balance_snapshots(snapshot_date);
CREATE INDEX IF NOT EXISTS idx_bs_alert ON _atlas.balance_snapshots(alert_status) WHERE alert_status != 'none';

-- Saved Balance Inquiries: personalized balance inquiry configurations
-- Oracle Fusion: General Ledger > Journals > Save Balance Inquiry
CREATE TABLE IF NOT EXISTS _atlas.saved_balance_inquiries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    account_segments JSONB NOT NULL DEFAULT '[]'::jsonb,
    period_from VARCHAR(50) NOT NULL,
    period_to VARCHAR(50) NOT NULL,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    amount_type VARCHAR(30) NOT NULL DEFAULT 'ending_balance',  -- beginning_balance, ending_balance, net_activity, debits, credits
    include_zero_balances BOOLEAN NOT NULL DEFAULT false,
    comparison_enabled BOOLEAN NOT NULL DEFAULT false,
    comparison_type VARCHAR(30) DEFAULT 'prior_period',
    sort_by VARCHAR(50) NOT NULL DEFAULT 'account_segment',
    sort_direction VARCHAR(10) NOT NULL DEFAULT 'asc',
    is_shared BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sbi_user ON _atlas.saved_balance_inquiries(organization_id, user_id);
