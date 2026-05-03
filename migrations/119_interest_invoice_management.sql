-- Interest Invoice Management
-- Oracle Fusion: Receivables > Late Charges
-- Enables automatic calculation of interest/late charges on overdue invoices,
-- interest rate schedule management, and interest invoice generation.
-- Uses VARCHAR for monetary amounts to avoid SQLx NUMERIC decoding issues.

-- Interest rate schedules define how interest is calculated
CREATE TABLE IF NOT EXISTS _atlas.interest_rate_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    annual_rate VARCHAR(50) NOT NULL,
    compounding_frequency VARCHAR(20) NOT NULL DEFAULT 'daily',
    charge_type VARCHAR(20) NOT NULL DEFAULT 'interest',
    grace_period_days INT NOT NULL DEFAULT 0,
    minimum_charge VARCHAR(50) DEFAULT '0',
    maximum_charge VARCHAR(50),
    currency_code VARCHAR(3) DEFAULT 'USD',
    effective_from DATE NOT NULL,
    effective_to DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, schedule_code)
);

-- Overdue invoice tracking
CREATE TABLE IF NOT EXISTS _atlas.overdue_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    invoice_number VARCHAR(100) NOT NULL,
    customer_id UUID NOT NULL,
    customer_name VARCHAR(200),
    original_amount VARCHAR(50) NOT NULL,
    outstanding_amount VARCHAR(50) NOT NULL,
    due_date DATE NOT NULL,
    overdue_days INT NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    last_interest_date DATE,
    total_interest_charged VARCHAR(50) DEFAULT '0',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Interest calculation runs
CREATE TABLE IF NOT EXISTS _atlas.interest_calculation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(50) NOT NULL,
    description TEXT,
    calculation_date DATE NOT NULL,
    schedule_id UUID REFERENCES _atlas.interest_rate_schedules(id),
    total_invoices_processed INT DEFAULT 0,
    total_interest_calculated VARCHAR(50) DEFAULT '0',
    currency_code VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    generated_by UUID,
    posted_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, run_number)
);

-- Individual interest calculation lines
CREATE TABLE IF NOT EXISTS _atlas.interest_calculation_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.interest_calculation_runs(id) ON DELETE CASCADE,
    overdue_invoice_id UUID REFERENCES _atlas.overdue_invoices(id),
    invoice_number VARCHAR(100) NOT NULL,
    customer_id UUID NOT NULL,
    customer_name VARCHAR(200),
    outstanding_amount VARCHAR(50) NOT NULL,
    overdue_days INT NOT NULL,
    annual_rate_used VARCHAR(50) NOT NULL,
    interest_amount VARCHAR(50) NOT NULL,
    currency_code VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'calculated',
    interest_invoice_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Interest invoices
CREATE TABLE IF NOT EXISTS _atlas.interest_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    invoice_number VARCHAR(50) NOT NULL,
    customer_id UUID NOT NULL,
    customer_name VARCHAR(200),
    calculation_run_id UUID REFERENCES _atlas.interest_calculation_runs(id),
    invoice_date DATE NOT NULL,
    due_date DATE,
    total_interest_amount VARCHAR(50) NOT NULL,
    currency_code VARCHAR(3) DEFAULT 'USD',
    line_count INT DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    gl_account_code VARCHAR(500),
    posted_at TIMESTAMPTZ,
    reversed_at TIMESTAMPTZ,
    reversal_invoice_id UUID,
    reference_invoice_number VARCHAR(100),
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, invoice_number)
);

-- Interest invoice lines
CREATE TABLE IF NOT EXISTS _atlas.interest_invoice_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    interest_invoice_id UUID NOT NULL REFERENCES _atlas.interest_invoices(id) ON DELETE CASCADE,
    calculation_line_id UUID REFERENCES _atlas.interest_calculation_lines(id),
    line_number INT NOT NULL,
    line_type VARCHAR(20) NOT NULL DEFAULT 'interest',
    description TEXT,
    reference_invoice_number VARCHAR(100),
    overdue_days INT,
    outstanding_amount VARCHAR(50),
    annual_rate_used VARCHAR(50),
    interest_amount VARCHAR(50) NOT NULL,
    currency_code VARCHAR(3) DEFAULT 'USD',
    gl_account_code VARCHAR(500),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Dashboard cache
CREATE TABLE IF NOT EXISTS _atlas.interest_dashboard_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
    total_active_schedules INT DEFAULT 0,
    total_overdue_invoices INT DEFAULT 0,
    total_overdue_amount VARCHAR(50) DEFAULT '0',
    total_interest_ytd VARCHAR(50) DEFAULT '0',
    total_pending_invoices INT DEFAULT 0,
    total_pending_amount VARCHAR(50) DEFAULT '0',
    avg_overdue_days VARCHAR(50) DEFAULT '0',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, snapshot_date)
);

CREATE INDEX IF NOT EXISTS idx_overdue_invoices_org_status ON _atlas.overdue_invoices(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_interest_invoices_org_status ON _atlas.interest_invoices(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_interest_calc_runs_org ON _atlas.interest_calculation_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_interest_calc_lines_run ON _atlas.interest_calculation_lines(run_id);
CREATE INDEX IF NOT EXISTS idx_interest_invoice_lines_invoice ON _atlas.interest_invoice_lines(interest_invoice_id);
