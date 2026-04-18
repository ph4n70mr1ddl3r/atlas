-- Grant Management (Oracle Fusion Grants Management)
-- Manages grant sponsors, grant awards, budgets, expenditures, billing, and compliance.
-- Oracle Fusion equivalent: Financials > Grants Management > Awards

-- Grant Sponsors (funding organizations)
CREATE TABLE IF NOT EXISTS _atlas.grant_sponsors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    sponsor_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    sponsor_type VARCHAR(30) NOT NULL DEFAULT 'government', -- government, foundation, corporate, internal, university
    country_code VARCHAR(3),
    taxpayer_id VARCHAR(50),
    contact_name VARCHAR(200),
    contact_email VARCHAR(200),
    contact_phone VARCHAR(50),
    address_line1 VARCHAR(200),
    address_line2 VARCHAR(200),
    city VARCHAR(100),
    state_province VARCHAR(100),
    postal_code VARCHAR(20),
    payment_terms VARCHAR(50),
    billing_frequency VARCHAR(30) NOT NULL DEFAULT 'monthly', -- monthly, quarterly, annual, on_demand, milestone
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    credit_limit NUMERIC(18, 2),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, sponsor_code)
);

-- Indirect Cost Rate Agreements
CREATE TABLE IF NOT EXISTS _atlas.grant_indirect_cost_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rate_name VARCHAR(100) NOT NULL,
    rate_type VARCHAR(30) NOT NULL DEFAULT 'negotiated', -- negotiated, predetermined, fixed, provisional
    rate_percentage NUMERIC(8, 4) NOT NULL DEFAULT 0,
    base_type VARCHAR(30) NOT NULL DEFAULT 'modified_total_direct_costs', -- modified_total_direct_costs, total_direct_costs, salaries_and_wages
    effective_from DATE NOT NULL,
    effective_to DATE,
    negotiated_by VARCHAR(200),
    approved_by UUID,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Grant Awards
CREATE TABLE IF NOT EXISTS _atlas.grant_awards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    award_number VARCHAR(50) NOT NULL UNIQUE,
    award_title VARCHAR(500) NOT NULL,
    sponsor_id UUID NOT NULL REFERENCES _atlas.grant_sponsors(id),
    sponsor_name VARCHAR(200),
    sponsor_award_number VARCHAR(50),
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, active, suspended, completed, terminated, closed
    award_type VARCHAR(30) NOT NULL DEFAULT 'research', -- research, training, fellowship, contract, cooperative_agreement, other
    award_purpose TEXT,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    budget_start_date DATE,
    budget_end_date DATE,
    total_award_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    direct_costs_total NUMERIC(18, 2) NOT NULL DEFAULT 0,
    indirect_costs_total NUMERIC(18, 2) NOT NULL DEFAULT 0,
    cost_sharing_total NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_funded NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_billed NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_collected NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_expenditures NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_commitments NUMERIC(18, 2) NOT NULL DEFAULT 0,
    available_balance NUMERIC(18, 2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    indirect_cost_rate_id UUID REFERENCES _atlas.grant_indirect_cost_rates(id),
    indirect_cost_rate NUMERIC(8, 4) NOT NULL DEFAULT 0,
    cost_sharing_required BOOLEAN NOT NULL DEFAULT false,
    cost_sharing_percent NUMERIC(5, 2) NOT NULL DEFAULT 0,
    principal_investigator_id UUID,
    principal_investigator_name VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    project_id UUID,
    cost_center VARCHAR(50),
    gl_revenue_account VARCHAR(50),
    gl_receivable_account VARCHAR(50),
    gl_deferred_account VARCHAR(50),
    billing_frequency VARCHAR(30) NOT NULL DEFAULT 'monthly',
    billing_basis VARCHAR(30) NOT NULL DEFAULT 'cost', -- cost, milestone, fixed_price, deliverable
    reporting_requirements TEXT,
    compliance_notes TEXT,
    closeout_date DATE,
    closeout_notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_grant_awards_org ON _atlas.grant_awards(organization_id);
CREATE INDEX idx_grant_awards_sponsor ON _atlas.grant_awards(sponsor_id);
CREATE INDEX idx_grant_awards_status ON _atlas.grant_awards(organization_id, status);
CREATE INDEX idx_grant_awards_pi ON _atlas.grant_awards(principal_investigator_id);
CREATE INDEX idx_grant_awards_dates ON _atlas.grant_awards(start_date, end_date) WHERE status = 'active';

-- Grant Budget Lines
CREATE TABLE IF NOT EXISTS _atlas.grant_budget_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    award_id UUID NOT NULL REFERENCES _atlas.grant_awards(id),
    line_number INT NOT NULL,
    budget_category VARCHAR(50) NOT NULL, -- personnel, fringe, travel, equipment, supplies, contractual, other_direct, indirect, cost_sharing
    description TEXT,
    account_code VARCHAR(50),
    budget_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    committed_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    expended_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    billed_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    available_balance NUMERIC(18, 2) NOT NULL DEFAULT 0,
    period_start DATE,
    period_end DATE,
    fiscal_year INT,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(award_id, line_number)
);

CREATE INDEX idx_grant_budget_award ON _atlas.grant_budget_lines(award_id);
CREATE INDEX idx_grant_budget_category ON _atlas.grant_budget_lines(award_id, budget_category);

-- Grant Expenditures
CREATE TABLE IF NOT EXISTS _atlas.grant_expenditures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    award_id UUID NOT NULL REFERENCES _atlas.grant_awards(id),
    expenditure_number VARCHAR(50) NOT NULL,
    expenditure_type VARCHAR(30) NOT NULL DEFAULT 'actual', -- actual, commitment, encumbrance, adjustment
    expenditure_date DATE NOT NULL,
    description TEXT,
    budget_line_id UUID REFERENCES _atlas.grant_budget_lines(id),
    budget_category VARCHAR(50),
    amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    indirect_cost_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    cost_sharing_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    employee_id UUID,
    employee_name VARCHAR(200),
    vendor_id UUID,
    vendor_name VARCHAR(200),
    source_entity_type VARCHAR(50), -- ap_invoice, expense_report, journal_entry, purchase_order
    source_entity_id UUID,
    source_entity_number VARCHAR(50),
    gl_debit_account VARCHAR(50),
    gl_credit_account VARCHAR(50),
    status VARCHAR(30) NOT NULL DEFAULT 'pending', -- pending, approved, billed, reversed, hold
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    billed_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(award_id, expenditure_number)
);

CREATE INDEX idx_grant_expenditures_award ON _atlas.grant_expenditures(award_id);
CREATE INDEX idx_grant_expenditures_status ON _atlas.grant_expenditures(award_id, status);
CREATE INDEX idx_grant_expenditures_type ON _atlas.grant_expenditures(organization_id, expenditure_type);
CREATE INDEX idx_grant_expenditures_date ON _atlas.grant_expenditures(expenditure_date);

-- Grant Billing (invoices to sponsor)
CREATE TABLE IF NOT EXISTS _atlas.grant_billings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    award_id UUID NOT NULL REFERENCES _atlas.grant_awards(id),
    invoice_number VARCHAR(50) NOT NULL,
    invoice_date DATE NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    due_date DATE,
    direct_costs_billed NUMERIC(18, 2) NOT NULL DEFAULT 0,
    indirect_costs_billed NUMERIC(18, 2) NOT NULL DEFAULT 0,
    cost_sharing_billed NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    amount_received NUMERIC(18, 2) NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, submitted, approved, paid, partial, disputed, cancelled
    expenditure_ids JSONB DEFAULT '[]',
    notes TEXT,
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    payment_reference VARCHAR(100),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(award_id, invoice_number)
);

CREATE INDEX idx_grant_billings_award ON _atlas.grant_billings(award_id);
CREATE INDEX idx_grant_billings_status ON _atlas.grant_billings(organization_id, status);
CREATE INDEX idx_grant_billings_date ON _atlas.grant_billings(invoice_date);

-- Grant Compliance Reports
CREATE TABLE IF NOT EXISTS _atlas.grant_compliance_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    award_id UUID NOT NULL REFERENCES _atlas.grant_awards(id),
    report_type VARCHAR(50) NOT NULL, -- federal_financial_report_sf425, progress_report, invention_report, property_report, closeout_report, audit_report, budget_modification
    report_title VARCHAR(300),
    reporting_period_start DATE NOT NULL,
    reporting_period_end DATE NOT NULL,
    due_date DATE,
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, in_review, approved, submitted, rejected
    total_expenditures NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_billed NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_received NUMERIC(18, 2) NOT NULL DEFAULT 0,
    cash_draws NUMERIC(18, 2) NOT NULL DEFAULT 0,
    obligations NUMERIC(18, 2) NOT NULL DEFAULT 0,
    content JSONB DEFAULT '{}',
    notes TEXT,
    prepared_by UUID,
    reviewed_by UUID,
    approved_by UUID,
    submitted_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_grant_compliance_award ON _atlas.grant_compliance_reports(award_id);
CREATE INDEX idx_grant_compliance_status ON _atlas.grant_compliance_reports(organization_id, status);
CREATE INDEX idx_grant_compliance_due ON _atlas.grant_compliance_reports(due_date) WHERE status IN ('draft', 'in_review');
