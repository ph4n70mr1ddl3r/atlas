-- Revenue Management (ASC 606 / IFRS 15) tables
CREATE TABLE IF NOT EXISTS _atlas.revenue_mgmt_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_number VARCHAR(100) NOT NULL,
    customer_id UUID NOT NULL,
    customer_name TEXT NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    transaction_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_allocated NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_recognized NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_unrecognized NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    contract_start_date DATE NOT NULL,
    contract_end_date DATE,
    performance_obligation_count INT NOT NULL DEFAULT 0,
    satisfied_obligation_count INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, contract_number)
);

CREATE TABLE IF NOT EXISTS _atlas.revenue_mgmt_obligations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.revenue_mgmt_contracts(id),
    contract_number VARCHAR(100),
    obligation_number VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    obligation_type VARCHAR(20) NOT NULL,
    satisfaction_status VARCHAR(30) NOT NULL DEFAULT 'not_started',
    satisfaction_method VARCHAR(20) NOT NULL,
    recognition_pattern VARCHAR(30) NOT NULL,
    standalone_selling_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    allocated_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    recognized_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    unrecognized_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    recognition_start_date DATE,
    recognition_end_date DATE,
    percent_complete NUMERIC(5,2) NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.revenue_mgmt_ssp (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    item_code VARCHAR(100) NOT NULL,
    item_name TEXT NOT NULL,
    estimation_method VARCHAR(30) NOT NULL,
    price NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    effective_from DATE NOT NULL,
    effective_to DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.revenue_mgmt_recognition_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL,
    obligation_id UUID NOT NULL,
    event_number VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    event_type VARCHAR(30) NOT NULL,
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    recognition_date DATE NOT NULL,
    gl_account_code VARCHAR(50),
    is_posted BOOLEAN NOT NULL DEFAULT false,
    posted_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Cash Flow Forecasting tables
CREATE TABLE IF NOT EXISTS _atlas.cash_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    forecast_number VARCHAR(100) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    forecast_horizon VARCHAR(20) NOT NULL,
    periods_out INT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    base_currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    total_inflows NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_outflows NUMERIC(18,2) NOT NULL DEFAULT 0,
    net_cash_flow NUMERIC(18,2) NOT NULL DEFAULT 0,
    opening_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    closing_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    approved_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, forecast_number)
);

CREATE TABLE IF NOT EXISTS _atlas.cash_forecast_scenarios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    forecast_id UUID NOT NULL REFERENCES _atlas.cash_forecasts(id),
    scenario_number VARCHAR(100) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    scenario_type VARCHAR(20) NOT NULL,
    adjustment_factor NUMERIC(5,2) NOT NULL DEFAULT 1.00,
    total_inflows NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_outflows NUMERIC(18,2) NOT NULL DEFAULT 0,
    net_cash_flow NUMERIC(18,2) NOT NULL DEFAULT 0,
    opening_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    closing_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.cash_forecast_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    forecast_id UUID NOT NULL REFERENCES _atlas.cash_forecasts(id),
    scenario_id UUID REFERENCES _atlas.cash_forecast_scenarios(id),
    period_name VARCHAR(50) NOT NULL,
    period_start_date DATE NOT NULL,
    period_end_date DATE NOT NULL,
    source_category VARCHAR(30) NOT NULL,
    flow_direction VARCHAR(10) NOT NULL,
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    probability NUMERIC(5,2) NOT NULL DEFAULT 1.00,
    weighted_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    is_manual BOOLEAN NOT NULL DEFAULT false,
    description TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Regulatory Reporting tables
CREATE TABLE IF NOT EXISTS _atlas.reg_report_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    authority VARCHAR(50) NOT NULL,
    report_category VARCHAR(20) NOT NULL,
    filing_frequency VARCHAR(20) NOT NULL,
    output_format VARCHAR(10) NOT NULL,
    row_definitions JSONB NOT NULL DEFAULT '[]',
    column_definitions JSONB NOT NULL DEFAULT '[]',
    validation_rules JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.reg_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.reg_report_templates(id),
    template_code VARCHAR(100),
    report_number VARCHAR(100) NOT NULL,
    name TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    authority VARCHAR(50) NOT NULL,
    output_format VARCHAR(10) NOT NULL,
    total_debits NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credits NUMERIC(18,2) NOT NULL DEFAULT 0,
    line_count INT NOT NULL DEFAULT 0,
    generated_at TIMESTAMPTZ,
    reviewed_by UUID,
    reviewed_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    filing_reference VARCHAR(200),
    rejection_reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, report_number)
);

CREATE TABLE IF NOT EXISTS _atlas.reg_report_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    report_id UUID NOT NULL REFERENCES _atlas.reg_reports(id),
    line_number INT NOT NULL,
    row_code VARCHAR(50) NOT NULL,
    row_label TEXT NOT NULL,
    column_code VARCHAR(50) NOT NULL,
    column_label TEXT NOT NULL,
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    description TEXT,
    account_range VARCHAR(100),
    is_subtotal BOOLEAN NOT NULL DEFAULT false,
    is_total BOOLEAN NOT NULL DEFAULT false,
    indent_level INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.reg_filing_calendar (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID,
    template_code VARCHAR(100),
    authority VARCHAR(50) NOT NULL,
    report_name TEXT NOT NULL,
    filing_frequency VARCHAR(20) NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    due_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'upcoming',
    assigned_to UUID,
    report_id UUID,
    filed_at TIMESTAMPTZ,
    filed_by UUID,
    filing_reference VARCHAR(200),
    notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_rm_contracts_org ON _atlas.revenue_mgmt_contracts(organization_id);
CREATE INDEX IF NOT EXISTS idx_rm_obligations_contract ON _atlas.revenue_mgmt_obligations(contract_id);
CREATE INDEX IF NOT EXISTS idx_rm_ssp_item ON _atlas.revenue_mgmt_ssp(organization_id, item_code);
CREATE INDEX IF NOT EXISTS idx_rm_events_contract ON _atlas.revenue_mgmt_recognition_events(contract_id);
CREATE INDEX IF NOT EXISTS idx_cf_org ON _atlas.cash_forecasts(organization_id);
CREATE INDEX IF NOT EXISTS idx_cf_entries_forecast ON _atlas.cash_forecast_entries(forecast_id);
CREATE INDEX IF NOT EXISTS idx_rr_templates_org ON _atlas.reg_report_templates(organization_id);
CREATE INDEX IF NOT EXISTS idx_rr_reports_org ON _atlas.reg_reports(organization_id);
CREATE INDEX IF NOT EXISTS idx_rr_lines_report ON _atlas.reg_report_lines(report_id);
CREATE INDEX IF NOT EXISTS idx_rr_filings_org ON _atlas.reg_filing_calendar(organization_id);
