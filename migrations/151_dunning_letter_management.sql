-- 151_dunning_letter_management.sql
-- Oracle Fusion Financial Feature: Dunning Letter Management
-- Manages the creation, configuration, and generation of dunning letters
-- (escalating payment reminders) for overdue customer invoices.
-- Oracle Fusion: Financials > Receivables > Dunning Letters

-- ============================================================================
-- Dunning Letter Sets (collection of letters at escalating severity levels)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.dunning_letter_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    set_name VARCHAR(100) NOT NULL,
    description TEXT,
    -- 'draft', 'active', 'inactive'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Number of dunning levels in this set
    number_of_levels INT NOT NULL DEFAULT 3,
    -- Minimum days overdue before any dunning starts
    minimum_overdue_days INT NOT NULL DEFAULT 1,
    -- Currency for minimum amounts
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Whether to include finance charges in dunning letters
    include_finance_charges BOOLEAN NOT NULL DEFAULT false,
    -- Whether to include unapplied receipts
    include_unapplied_receipts BOOLEAN NOT NULL DEFAULT false,
    -- Default aging bucket: 'days_overdue', 'invoice_date', 'due_date'
    aging_basis VARCHAR(20) NOT NULL DEFAULT 'days_overdue',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, set_name)
);

CREATE INDEX IF NOT EXISTS idx_dls_org ON financials.dunning_letter_sets(organization_id);
CREATE INDEX IF NOT EXISTS idx_dls_status ON financials.dunning_letter_sets(status);

-- ============================================================================
-- Dunning Letter Set Lines (individual letter level definitions)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.dunning_letter_set_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    set_id UUID NOT NULL REFERENCES financials.dunning_letter_sets(id) ON DELETE CASCADE,
    -- Level number (1 = reminder, 2 = first notice, 3 = final demand, etc.)
    level_number INT NOT NULL,
    -- Display name for this level
    level_name VARCHAR(100) NOT NULL,
    -- Minimum days overdue to trigger this level
    min_days_overdue INT NOT NULL DEFAULT 0,
    -- Maximum days overdue for this level (null = no upper limit)
    max_days_overdue INT,
    -- Minimum overdue amount to trigger this level
    minimum_amount VARCHAR(50) NOT NULL DEFAULT '0',
    -- Letter template name/reference
    letter_template VARCHAR(200),
    -- Whether to include a copy to a designated recipient
    cc_to VARCHAR(200),
    -- 'print', 'email', 'both'
    delivery_method VARCHAR(20) NOT NULL DEFAULT 'print',
    -- Whether to place the customer on credit hold at this level
    apply_credit_hold BOOLEAN NOT NULL DEFAULT false,
    -- Whether to assess finance charges at this level
    assess_finance_charges BOOLEAN NOT NULL DEFAULT false,
    -- Additional instructions/notes printed on the letter
    letter_text TEXT,
    -- Days before escalating to the next level
    escalation_days INT,
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(set_id, level_number)
);

CREATE INDEX IF NOT EXISTS idx_dlsl_set ON financials.dunning_letter_set_lines(set_id);

-- ============================================================================
-- Dunning Profiles (customer-level dunning configuration)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.dunning_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    customer_id UUID NOT NULL,
    customer_name VARCHAR(200),
    -- Reference to the dunning letter set to use
    letter_set_id UUID REFERENCES financials.dunning_letter_sets(id),
    -- 'enabled', 'disabled', 'hold'
    dunning_status VARCHAR(20) NOT NULL DEFAULT 'enabled',
    -- Minimum overdue amount before dunning this customer
    minimum_overdue_amount VARCHAR(50) NOT NULL DEFAULT '0',
    -- Contact name for dunning letters
    contact_name VARCHAR(200),
    -- Contact email
    contact_email VARCHAR(200),
    -- Contact address
    contact_address TEXT,
    -- Last dunning level sent to this customer
    last_dunning_level INT DEFAULT 0,
    -- Date of last dunning letter sent
    last_dunning_date DATE,
    -- Number of dunning letters sent
    dunning_letter_count INT NOT NULL DEFAULT 0,
    -- 'print', 'email', 'both'
    preferred_delivery_method VARCHAR(20) DEFAULT 'print',
    -- Reason if dunning is on hold
    hold_reason TEXT,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, customer_id)
);

CREATE INDEX IF NOT EXISTS idx_dp_org ON financials.dunning_profiles(organization_id);
CREATE INDEX IF NOT EXISTS idx_dp_customer ON financials.dunning_profiles(customer_id);
CREATE INDEX IF NOT EXISTS idx_dp_status ON financials.dunning_profiles(dunning_status);

-- ============================================================================
-- Dunning Letter Runs (batch dunning letter generation)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.dunning_letter_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(50) NOT NULL,
    description TEXT,
    -- The letter set used for this run
    letter_set_id UUID REFERENCES financials.dunning_letter_sets(id),
    -- Run date
    run_date DATE NOT NULL DEFAULT CURRENT_DATE,
    -- Aging as-of date for calculating overdue days
    aging_as_of_date DATE NOT NULL DEFAULT CURRENT_DATE,
    -- 'draft', 'submitted', 'completed', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Filters applied to the run
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Minimum overdue amount filter
    minimum_amount_filter VARCHAR(50) DEFAULT '0',
    -- Specific letter level to generate (null = all eligible levels)
    specific_level INT,
    -- Customer ID filter (null = all customers with profiles)
    customer_id_filter UUID,
    -- Statistics
    total_customers INT NOT NULL DEFAULT 0,
    total_letters_generated INT NOT NULL DEFAULT 0,
    total_overdue_amount VARCHAR(50) NOT NULL DEFAULT '0',
    -- Error/warning count
    error_count INT NOT NULL DEFAULT 0,
    warning_count INT NOT NULL DEFAULT 0,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, run_number)
);

CREATE INDEX IF NOT EXISTS idx_dlr_org ON financials.dunning_letter_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_dlr_status ON financials.dunning_letter_runs(status);
CREATE INDEX IF NOT EXISTS idx_dlr_date ON financials.dunning_letter_runs(run_date);

-- ============================================================================
-- Dunning Letter Run Results (individual customer results per run)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.dunning_letter_run_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES financials.dunning_letter_runs(id) ON DELETE CASCADE,
    -- Customer info
    customer_id UUID NOT NULL,
    customer_name VARCHAR(200),
    customer_number VARCHAR(50),
    -- Dunning profile
    profile_id UUID REFERENCES financials.dunning_profiles(id),
    -- Dunning level assigned
    dunning_level INT NOT NULL,
    -- Level name from the set line
    level_name VARCHAR(100),
    -- Overdue information
    number_of_overdue_items INT NOT NULL DEFAULT 0,
    total_overdue_amount VARCHAR(50) NOT NULL DEFAULT '0',
    oldest_overdue_date DATE,
    days_overdue INT NOT NULL DEFAULT 0,
    -- Finance charges included
    finance_charge_amount VARCHAR(50) NOT NULL DEFAULT '0',
    -- Letter details
    letter_template VARCHAR(200),
    -- 'pending', 'generated', 'sent', 'failed', 'skipped'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Delivery tracking
    delivery_method VARCHAR(20) DEFAULT 'print',
    sent_date DATE,
    delivery_confirmation VARCHAR(200),
    -- Error or skip reason
    reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_dlrr_run ON financials.dunning_letter_run_results(run_id);
CREATE INDEX IF NOT EXISTS idx_dlrr_customer ON financials.dunning_letter_run_results(customer_id);
CREATE INDEX IF NOT EXISTS idx_dlrr_status ON financials.dunning_letter_run_results(status);

-- ============================================================================
-- Dunning Letter Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.dunning_letter_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID,
    result_id UUID,
    set_id UUID,
    profile_id UUID,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_dla_run ON financials.dunning_letter_audit(run_id);
CREATE INDEX IF NOT EXISTS idx_dla_action ON financials.dunning_letter_audit(action);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.dunning_dashboard AS
SELECT
    runs.organization_id,
    COUNT(DISTINCT sets.id) AS total_letter_sets,
    COUNT(DISTINCT sets.id) FILTER (WHERE sets.status = 'active') AS active_letter_sets,
    COUNT(DISTINCT profiles.id) AS total_profiles,
    COUNT(DISTINCT profiles.id) FILTER (WHERE profiles.dunning_status = 'enabled') AS enabled_profiles,
    COUNT(DISTINCT profiles.id) FILTER (WHERE profiles.dunning_status = 'disabled') AS disabled_profiles,
    COUNT(DISTINCT profiles.id) FILTER (WHERE profiles.dunning_status = 'hold') AS hold_profiles,
    COUNT(DISTINCT runs.id) AS total_runs,
    COUNT(DISTINCT runs.id) FILTER (WHERE runs.status = 'draft') AS draft_runs,
    COUNT(DISTINCT runs.id) FILTER (WHERE runs.status = 'submitted') AS submitted_runs,
    COUNT(DISTINCT runs.id) FILTER (WHERE runs.status = 'completed') AS completed_runs,
    COALESCE(SUM(runs.total_letters_generated), 0) AS total_letters_generated,
    COALESCE(SUM(runs.error_count), 0) AS total_errors,
    COUNT(DISTINCT results.id) FILTER (WHERE results.status = 'sent') AS letters_sent,
    COUNT(DISTINCT results.id) FILTER (WHERE results.status = 'failed') AS letters_failed,
    COUNT(DISTINCT results.id) FILTER (WHERE results.status = 'pending') AS letters_pending
FROM financials.dunning_letter_runs runs
LEFT JOIN financials.dunning_letter_run_results results ON results.run_id = runs.id
LEFT JOIN financials.dunning_letter_sets sets ON sets.organization_id = runs.organization_id
LEFT JOIN financials.dunning_profiles profiles ON profiles.organization_id = runs.organization_id
GROUP BY runs.organization_id;

COMMENT ON TABLE financials.dunning_letter_sets IS 'Dunning Letter Sets - collections of escalating severity letters for overdue invoices';
COMMENT ON TABLE financials.dunning_letter_set_lines IS 'Dunning Letter Set Lines - individual letter level definitions within a set';
COMMENT ON TABLE financials.dunning_profiles IS 'Dunning Profiles - customer-level configuration for dunning letter management';
COMMENT ON TABLE financials.dunning_letter_runs IS 'Dunning Letter Runs - batch generation runs of dunning letters';
COMMENT ON TABLE financials.dunning_letter_run_results IS 'Dunning Letter Run Results - individual customer results per dunning run';
COMMENT ON TABLE financials.dunning_letter_audit IS 'Audit trail for dunning letter management actions';
COMMENT ON VIEW financials.dunning_dashboard IS 'Dashboard aggregation for dunning letter management statistics';
