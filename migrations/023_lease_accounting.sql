-- Lease Accounting Tables (ASC 842 / IFRS 16)
-- Oracle Fusion Cloud ERP equivalent: Financials > Lease Management
--
-- Provides lease contract management, right-of-use (ROU) asset tracking,
-- lease liability amortization, payment scheduling, modifications,
-- impairment, and termination accounting.

-- Lease Contracts
CREATE TABLE IF NOT EXISTS _atlas.lease_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lease_number VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    classification VARCHAR(20) NOT NULL DEFAULT 'operating', -- 'operating' or 'finance'
    
    -- Lessor / Supplier
    lessor_id UUID,
    lessor_name VARCHAR(200),
    
    -- Asset information
    asset_description TEXT,
    location VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    
    -- Lease dates and term
    commencement_date DATE NOT NULL,
    end_date DATE NOT NULL,
    lease_term_months INT NOT NULL,
    
    -- Options
    purchase_option_exists BOOLEAN DEFAULT false,
    purchase_option_likely BOOLEAN DEFAULT false,
    renewal_option_exists BOOLEAN DEFAULT false,
    renewal_option_months INT,
    renewal_option_likely BOOLEAN DEFAULT false,
    
    -- Financial parameters
    discount_rate NUMERIC(10, 6) NOT NULL, -- Incremental borrowing rate
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    payment_frequency VARCHAR(20) NOT NULL DEFAULT 'monthly', -- 'monthly', 'quarterly', 'annually'
    annual_payment_amount NUMERIC(18, 2) NOT NULL,
    escalation_rate NUMERIC(8, 6),
    escalation_frequency_months INT,
    
    -- Financial summary (initial values)
    total_lease_payments NUMERIC(18, 2) NOT NULL DEFAULT 0,
    initial_lease_liability NUMERIC(18, 2) NOT NULL DEFAULT 0,
    initial_rou_asset_value NUMERIC(18, 2) NOT NULL DEFAULT 0,
    residual_guarantee_amount NUMERIC(18, 2),
    
    -- Current balances
    current_lease_liability NUMERIC(18, 2) NOT NULL DEFAULT 0,
    current_rou_asset_value NUMERIC(18, 2) NOT NULL DEFAULT 0,
    accumulated_rou_depreciation NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_payments_made NUMERIC(18, 2) NOT NULL DEFAULT 0,
    periods_elapsed INT NOT NULL DEFAULT 0,
    
    -- GL account codes
    rou_asset_account_code VARCHAR(50),
    rou_depreciation_account_code VARCHAR(50),
    lease_liability_account_code VARCHAR(50),
    lease_expense_account_code VARCHAR(50),
    interest_expense_account_code VARCHAR(50),
    
    -- Status and impairment
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    impairment_amount NUMERIC(18, 2),
    impairment_date DATE,
    
    -- Metadata and audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    
    UNIQUE(organization_id, lease_number)
);

CREATE INDEX IF NOT EXISTS idx_lease_contracts_org ON _atlas.lease_contracts(organization_id);
CREATE INDEX IF NOT EXISTS idx_lease_contracts_status ON _atlas.lease_contracts(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_lease_contracts_classification ON _atlas.lease_contracts(organization_id, classification);

-- Lease Payment Schedule
CREATE TABLE IF NOT EXISTS _atlas.lease_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lease_id UUID NOT NULL REFERENCES _atlas.lease_contracts(id) ON DELETE CASCADE,
    period_number INT NOT NULL,
    payment_date DATE NOT NULL,
    
    -- Payment breakdown
    payment_amount NUMERIC(18, 2) NOT NULL,
    interest_amount NUMERIC(18, 2) NOT NULL,
    principal_amount NUMERIC(18, 2) NOT NULL,
    remaining_liability NUMERIC(18, 2) NOT NULL,
    
    -- ROU asset tracking
    rou_asset_value NUMERIC(18, 2) NOT NULL,
    rou_depreciation NUMERIC(18, 2) NOT NULL,
    accumulated_depreciation NUMERIC(18, 2) NOT NULL,
    
    -- Straight-line expense (for operating leases)
    lease_expense NUMERIC(18, 2) NOT NULL DEFAULT 0,
    
    -- Payment tracking
    is_paid BOOLEAN DEFAULT false,
    payment_reference VARCHAR(100),
    journal_entry_id UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'scheduled',
    
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    
    UNIQUE(lease_id, period_number)
);

CREATE INDEX IF NOT EXISTS idx_lease_payments_lease ON _atlas.lease_payments(lease_id);
CREATE INDEX IF NOT EXISTS idx_lease_payments_status ON _atlas.lease_payments(lease_id, status);

-- Lease Modifications
CREATE TABLE IF NOT EXISTS _atlas.lease_modifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lease_id UUID NOT NULL REFERENCES _atlas.lease_contracts(id) ON DELETE CASCADE,
    modification_number INT NOT NULL,
    modification_type VARCHAR(30) NOT NULL, -- 'term_extension', 'scope_change', 'payment_change', 'rate_change', 'reclassification'
    description TEXT,
    effective_date DATE NOT NULL,
    
    -- Before/after values
    previous_term_months INT,
    new_term_months INT,
    previous_end_date DATE,
    new_end_date DATE,
    previous_discount_rate NUMERIC(10, 6),
    new_discount_rate NUMERIC(10, 6),
    
    -- Financial impact
    liability_adjustment NUMERIC(18, 2) NOT NULL DEFAULT 0,
    rou_asset_adjustment NUMERIC(18, 2) NOT NULL DEFAULT 0,
    
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    
    UNIQUE(lease_id, modification_number)
);

CREATE INDEX IF NOT EXISTS idx_lease_modifications_lease ON _atlas.lease_modifications(lease_id);

-- Lease Terminations
CREATE TABLE IF NOT EXISTS _atlas.lease_terminations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lease_id UUID NOT NULL REFERENCES _atlas.lease_contracts(id) ON DELETE CASCADE,
    termination_type VARCHAR(30) NOT NULL, -- 'early', 'end_of_term', 'mutual_agreement', 'default'
    termination_date DATE NOT NULL,
    reason TEXT,
    
    -- Financial details
    remaining_liability NUMERIC(18, 2) NOT NULL,
    remaining_rou_asset NUMERIC(18, 2) NOT NULL,
    termination_penalty NUMERIC(18, 2) NOT NULL DEFAULT 0,
    gain_loss_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    gain_loss_type VARCHAR(10), -- 'gain' or 'loss'
    
    journal_entry_id UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_lease_terminations_lease ON _atlas.lease_terminations(lease_id);
