-- Atlas ERP - Expense Management
-- Oracle Fusion Cloud ERP: Expenses > Expense Reports, Categories, Policies
--
-- Manages expense categories, expense policies (limits, per-diem, mileage),
-- expense reports with line items, and reimbursement workflow.
--
-- This is a standard Oracle Fusion Cloud ERP feature that every
-- implementation requires for employee expense tracking and reimbursement.

-- ============================================================================
-- Expense Categories
-- Oracle Fusion: Define expense categories (e.g., Travel, Meals, Lodging)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.expense_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Category code (e.g., 'TRAVEL', 'MEALS', 'LODGING', 'MISC')
    code VARCHAR(50) NOT NULL,
    -- Display name
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Receipt requirements
    receipt_required BOOLEAN DEFAULT false,
    receipt_threshold NUMERIC(18,2),
    -- Per-diem support
    is_per_diem BOOLEAN DEFAULT false,
    default_per_diem_rate NUMERIC(18,2),
    -- Mileage support
    is_mileage BOOLEAN DEFAULT false,
    default_mileage_rate NUMERIC(18,4),
    -- GL posting
    expense_account_code VARCHAR(50),
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_expense_categories_org ON _atlas.expense_categories(organization_id);
CREATE INDEX idx_expense_categories_active ON _atlas.expense_categories(organization_id, is_active) WHERE is_active = true;

-- ============================================================================
-- Expense Policies
-- Oracle Fusion: Define expense policies with limits and violation actions
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.expense_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- The category this policy applies to (NULL = applies to all categories)
    category_id UUID REFERENCES _atlas.expense_categories(id),
    -- Amount limits
    min_amount NUMERIC(18,2),
    max_amount NUMERIC(18,2),
    -- Daily and report-level limits
    daily_limit NUMERIC(18,2),
    report_limit NUMERIC(18,2),
    -- Violation handling
    requires_approval_on_violation BOOLEAN DEFAULT false,
    violation_action VARCHAR(30) NOT NULL DEFAULT 'warn', -- 'warn', 'block', 'require_justification'
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Effective dates
    effective_from DATE,
    effective_to DATE,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_expense_policies_org ON _atlas.expense_policies(organization_id);
CREATE INDEX idx_expense_policies_category ON _atlas.expense_policies(category_id);
CREATE INDEX idx_expense_policies_active ON _atlas.expense_policies(organization_id, is_active) WHERE is_active = true;

-- ============================================================================
-- Expense Reports (Header)
-- Oracle Fusion: Employee expense reports with workflow
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.expense_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Report identification
    report_number VARCHAR(50) NOT NULL,
    title VARCHAR(200) NOT NULL,
    description TEXT,
    -- Status: 'draft', 'submitted', 'approved', 'rejected', 'reimbursed', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Employee who created the report
    employee_id UUID NOT NULL,
    employee_name VARCHAR(200),
    -- Cost center / department
    department_id UUID,
    -- Purpose of the expense
    purpose TEXT,
    -- Project reference (for project billing)
    project_id UUID,
    -- Currency
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Totals (calculated from expense lines)
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    reimbursable_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    receipt_required_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    receipt_count INT NOT NULL DEFAULT 0,
    -- Business trip dates
    trip_start_date DATE,
    trip_end_date DATE,
    -- Cost center override
    cost_center VARCHAR(50),
    -- Approval information
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,
    -- Payment information
    payment_method VARCHAR(30),
    payment_reference VARCHAR(100),
    reimbursed_at TIMESTAMPTZ,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, report_number)
);

CREATE INDEX idx_expense_reports_org ON _atlas.expense_reports(organization_id);
CREATE INDEX idx_expense_reports_employee ON _atlas.expense_reports(employee_id);
CREATE INDEX idx_expense_reports_status ON _atlas.expense_reports(organization_id, status);
CREATE INDEX idx_expense_reports_dates ON _atlas.expense_reports(created_at);

-- ============================================================================
-- Expense Lines (Individual expenses within a report)
-- Oracle Fusion: Itemized expense lines with per-diem, mileage support
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.expense_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    report_id UUID NOT NULL REFERENCES _atlas.expense_reports(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    -- Category reference
    expense_category_id UUID REFERENCES _atlas.expense_categories(id),
    expense_category_name VARCHAR(200),
    -- Expense type: 'expense', 'per_diem', 'mileage', 'credit_card'
    expense_type VARCHAR(20) NOT NULL DEFAULT 'expense',
    -- Description
    description TEXT,
    -- Date expense was incurred
    expense_date DATE NOT NULL,
    -- Amount (in report currency)
    amount NUMERIC(18,2) NOT NULL,
    -- Foreign currency support
    original_currency VARCHAR(3),
    original_amount NUMERIC(18,2),
    exchange_rate NUMERIC(18,6),
    -- Reimbursability
    is_reimbursable BOOLEAN DEFAULT true,
    -- Receipt tracking
    has_receipt BOOLEAN DEFAULT false,
    receipt_reference VARCHAR(100),
    -- Merchant / vendor
    merchant_name VARCHAR(200),
    -- Location
    location VARCHAR(200),
    -- Attendees (for entertainment / meals)
    attendees JSONB,
    -- Per-diem fields
    per_diem_days NUMERIC(8,2),
    per_diem_rate NUMERIC(18,2),
    -- Mileage fields
    mileage_distance NUMERIC(12,2),
    mileage_rate NUMERIC(18,4),
    mileage_unit VARCHAR(10), -- 'miles' or 'km'
    mileage_from VARCHAR(200),
    mileage_to VARCHAR(200),
    -- Policy validation
    policy_violation BOOLEAN DEFAULT false,
    policy_violation_message TEXT,
    -- GL account override
    expense_account_code VARCHAR(50),
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_expense_lines_report ON _atlas.expense_lines(report_id);
CREATE INDEX idx_expense_lines_org ON _atlas.expense_lines(organization_id);
CREATE INDEX idx_expense_lines_category ON _atlas.expense_lines(expense_category_id);
CREATE INDEX idx_expense_lines_date ON _atlas.expense_lines(expense_date);
