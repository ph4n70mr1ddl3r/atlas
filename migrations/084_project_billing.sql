-- Project Billing
-- Oracle Fusion Cloud: Project Management > Project Billing
-- Provides: Bill rate schedules, billing events, project invoices,
--   retention management, billing analytics.
--
-- Extends the existing project_costing module to enable client billing
-- for projects (Time & Materials, Fixed Price, Milestone-based).

-- ============================================================================
-- Bill Rate Schedules: define billable rates by role / labor category
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.bill_rate_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_number VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    -- Schedule type: standard, overtime, holiday, custom
    schedule_type VARCHAR(50) NOT NULL DEFAULT 'standard',
    -- Currency for rates in this schedule
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- Effectivity
    effective_start DATE NOT NULL,
    effective_end DATE,
    -- Status: draft, active, inactive
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- Markup percentage applied on top of raw cost (for cost-plus billing)
    default_markup_pct NUMERIC(8,4) DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, schedule_number)
);

CREATE INDEX IF NOT EXISTS idx_bill_rate_schedules_org ON _atlas.bill_rate_schedules(organization_id);
CREATE INDEX IF NOT EXISTS idx_bill_rate_schedules_status ON _atlas.bill_rate_schedules(status);

-- ============================================================================
-- Bill Rate Lines: individual rates within a schedule
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.bill_rate_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_id UUID NOT NULL REFERENCES _atlas.bill_rate_schedules(id) ON DELETE CASCADE,
    -- Role / labor category this rate applies to
    role_name VARCHAR(300) NOT NULL,
    -- Optional project-specific override
    project_id UUID,
    -- Rate
    bill_rate NUMERIC(18,4) NOT NULL,
    unit_of_measure VARCHAR(20) NOT NULL DEFAULT 'hours',
    -- Effectivity (may differ from parent schedule)
    effective_start DATE NOT NULL,
    effective_end DATE,
    -- Markup pct override for cost-plus
    markup_pct NUMERIC(8,4),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_bill_rate_lines_schedule ON _atlas.bill_rate_lines(schedule_id);
CREATE INDEX IF NOT EXISTS idx_bill_rate_lines_project ON _atlas.bill_rate_lines(project_id);

-- ============================================================================
-- Project Billing Config: billing arrangement per project
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.project_billing_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    project_id UUID NOT NULL,
    -- Billing method: time_and_materials, fixed_price, milestone, cost_plus, retention
    billing_method VARCHAR(50) NOT NULL DEFAULT 'time_and_materials',
    -- Link to bill rate schedule (for T&M / cost-plus)
    bill_rate_schedule_id UUID REFERENCES _atlas.bill_rate_schedules(id),
    -- Fixed price contract value (for fixed-price projects)
    contract_amount NUMERIC(18,4) DEFAULT 0,
    -- Billing currency
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- Invoice format: detailed, summary, consolidated
    invoice_format VARCHAR(50) NOT NULL DEFAULT 'detailed',
    -- Billing cycle: weekly, biweekly, monthly, milestone
    billing_cycle VARCHAR(50) NOT NULL DEFAULT 'monthly',
    -- Grace period (days after invoice before overdue)
    payment_terms_days INT NOT NULL DEFAULT 30,
    -- Retention
    retention_pct NUMERIC(8,4) DEFAULT 0,
    retention_amount_cap NUMERIC(18,4) DEFAULT 0,
    -- Customer reference
    customer_id UUID,
    customer_name VARCHAR(500),
    -- PO / contract reference
    customer_po_number VARCHAR(100),
    contract_number VARCHAR(100),
    -- Status: draft, active, completed, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, project_id)
);

CREATE INDEX IF NOT EXISTS idx_proj_billing_configs_org ON _atlas.project_billing_configs(organization_id);
CREATE INDEX IF NOT EXISTS idx_proj_billing_configs_project ON _atlas.project_billing_configs(project_id);
CREATE INDEX IF NOT EXISTS idx_proj_billing_configs_customer ON _atlas.project_billing_configs(customer_id);

-- ============================================================================
-- Billing Events: milestones, progress markers for billing
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.billing_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    project_id UUID NOT NULL,
    event_number VARCHAR(100) NOT NULL,
    event_name VARCHAR(500) NOT NULL,
    description TEXT,
    -- Event type: milestone, progress, completion, retention_release
    event_type VARCHAR(50) NOT NULL DEFAULT 'milestone',
    -- Billing amount for this event
    billing_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- Completion tracking
    completion_pct NUMERIC(8,4) DEFAULT 0,
    -- Status: planned, ready, invoiced, partially_invoiced, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'planned',
    -- Dates
    planned_date DATE,
    actual_date DATE,
    -- Task reference
    task_id UUID,
    task_name VARCHAR(300),
    -- Invoice reference (once invoiced)
    invoice_header_id UUID,
    -- Approvals
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, event_number)
);

CREATE INDEX IF NOT EXISTS idx_billing_events_org ON _atlas.billing_events(organization_id);
CREATE INDEX IF NOT EXISTS idx_billing_events_project ON _atlas.billing_events(project_id);
CREATE INDEX IF NOT EXISTS idx_billing_events_status ON _atlas.billing_events(status);
CREATE INDEX IF NOT EXISTS idx_billing_events_type ON _atlas.billing_events(event_type);

-- ============================================================================
-- Project Invoice Headers
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.project_invoice_headers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    invoice_number VARCHAR(100) NOT NULL,
    project_id UUID NOT NULL,
    project_number VARCHAR(100),
    project_name VARCHAR(500),
    -- Invoice type: progress, milestone, t_and_m, retention_release, debit_memo, credit_memo
    invoice_type VARCHAR(50) NOT NULL DEFAULT 't_and_m',
    -- Status: draft, submitted, approved, rejected, posted, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- Customer
    customer_id UUID,
    customer_name VARCHAR(500),
    -- Amounts
    invoice_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(18,4) DEFAULT 0,
    retention_held NUMERIC(18,4) DEFAULT 0,
    total_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    exchange_rate NUMERIC(18,8) DEFAULT 1,
    -- Billing period
    billing_period_start DATE,
    billing_period_end DATE,
    invoice_date DATE NOT NULL DEFAULT CURRENT_DATE,
    due_date DATE,
    -- References
    billing_event_id UUID,
    customer_po_number VARCHAR(100),
    contract_number VARCHAR(100),
    -- GL posting
    gl_posted_flag BOOLEAN DEFAULT false,
    gl_posted_date TIMESTAMPTZ,
    -- Approval
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    -- Payment
    payment_status VARCHAR(50) DEFAULT 'unpaid',
    payment_date TIMESTAMPTZ,
    -- Notes
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, invoice_number)
);

CREATE INDEX IF NOT EXISTS idx_proj_invoice_headers_org ON _atlas.project_invoice_headers(organization_id);
CREATE INDEX IF NOT EXISTS idx_proj_invoice_headers_project ON _atlas.project_invoice_headers(project_id);
CREATE INDEX IF NOT EXISTS idx_proj_invoice_headers_status ON _atlas.project_invoice_headers(status);
CREATE INDEX IF NOT EXISTS idx_proj_invoice_headers_customer ON _atlas.project_invoice_headers(customer_id);
CREATE INDEX IF NOT EXISTS idx_proj_invoice_headers_date ON _atlas.project_invoice_headers(invoice_date);

-- ============================================================================
-- Project Invoice Lines
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.project_invoice_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    invoice_header_id UUID NOT NULL REFERENCES _atlas.project_invoice_headers(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    -- Source: expenditure_item, billing_event, retention, manual
    line_source VARCHAR(50) NOT NULL DEFAULT 'expenditure_item',
    -- Reference back to source cost transaction
    expenditure_item_id UUID,
    -- Reference to billing event
    billing_event_id UUID,
    -- Task reference
    task_id UUID,
    task_number VARCHAR(100),
    task_name VARCHAR(300),
    -- Description
    description TEXT,
    -- Resource details
    employee_id UUID,
    employee_name VARCHAR(300),
    role_name VARCHAR(300),
    expenditure_type VARCHAR(100),
    -- Quantity & rate
    quantity NUMERIC(18,4) DEFAULT 0,
    unit_of_measure VARCHAR(20) DEFAULT 'hours',
    bill_rate NUMERIC(18,4) DEFAULT 0,
    -- Amounts
    raw_cost_amount NUMERIC(18,4) DEFAULT 0,
    bill_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    markup_amount NUMERIC(18,4) DEFAULT 0,
    retention_amount NUMERIC(18,4) DEFAULT 0,
    tax_amount NUMERIC(18,4) DEFAULT 0,
    -- Transaction date
    transaction_date DATE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_proj_invoice_lines_header ON _atlas.project_invoice_lines(invoice_header_id);
CREATE INDEX IF NOT EXISTS idx_proj_invoice_lines_task ON _atlas.project_invoice_lines(task_id);
CREATE INDEX IF NOT EXISTS idx_proj_invoice_lines_expenditure ON _atlas.project_invoice_lines(expenditure_item_id);

-- ============================================================================
-- Project Billing Dashboard Summary
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.project_billing_dashboard (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Totals
    total_projects_billable INT DEFAULT 0,
    total_contract_value NUMERIC(18,4) DEFAULT 0,
    total_billed NUMERIC(18,4) DEFAULT 0,
    total_unbilled NUMERIC(18,4) DEFAULT 0,
    total_retention_held NUMERIC(18,4) DEFAULT 0,
    total_retention_released NUMERIC(18,4) DEFAULT 0,
    -- Invoices
    total_invoices INT DEFAULT 0,
    draft_invoices INT DEFAULT 0,
    submitted_invoices INT DEFAULT 0,
    approved_invoices INT DEFAULT 0,
    posted_invoices INT DEFAULT 0,
    overdue_invoices INT DEFAULT 0,
    -- Revenue
    total_revenuerecognized NUMERIC(18,4) DEFAULT 0,
    -- By billing method
    by_billing_method JSONB DEFAULT '{}'::jsonb,
    -- By invoice status
    by_invoice_status JSONB DEFAULT '{}'::jsonb,
    -- Monthly billing trend
    billing_trend JSONB DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id)
);
