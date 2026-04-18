-- 026_financial_reporting.sql
-- Financial Reporting tables (Oracle Fusion GL > Financial Reporting Center)
-- Provides report templates, report columns, report rows, report execution,
-- trial balance, income statement, balance sheet, and custom financial reports.

-- Report Templates (defines the structure of a financial report)
CREATE TABLE IF NOT EXISTS _atlas.financial_report_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    report_type VARCHAR(50) NOT NULL DEFAULT 'custom',
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    row_display_order VARCHAR(20) NOT NULL DEFAULT 'sequential',
    column_display_order VARCHAR(20) NOT NULL DEFAULT 'sequential',
    rounding_option VARCHAR(20) NOT NULL DEFAULT 'none',
    show_zero_amounts BOOLEAN NOT NULL DEFAULT false,
    segment_filter JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Report Rows (line items on a financial report)
CREATE TABLE IF NOT EXISTS _atlas.financial_report_rows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.financial_report_templates(id) ON DELETE CASCADE,
    row_number INT NOT NULL,
    line_type VARCHAR(30) NOT NULL DEFAULT 'data',
    label VARCHAR(500) NOT NULL,
    indent_level INT NOT NULL DEFAULT 0,
    -- Account filter for data rows
    account_range_from VARCHAR(100),
    account_range_to VARCHAR(100),
    account_filter JSONB DEFAULT '{}',
    -- Computation for calculated rows
    compute_action VARCHAR(30),
    compute_source_rows JSONB DEFAULT '[]',
    -- Display options
    show_line BOOLEAN NOT NULL DEFAULT true,
    bold BOOLEAN NOT NULL DEFAULT false,
    underline BOOLEAN NOT NULL DEFAULT false,
    double_underline BOOLEAN NOT NULL DEFAULT false,
    page_break_before BOOLEAN NOT NULL DEFAULT false,
    -- Scaling
    scaling_factor VARCHAR(20),
    -- Children for hierarchical reports
    parent_row_id UUID REFERENCES _atlas.financial_report_rows(id),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Report Columns (periods or scenarios displayed across the top)
CREATE TABLE IF NOT EXISTS _atlas.financial_report_columns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.financial_report_templates(id) ON DELETE CASCADE,
    column_number INT NOT NULL,
    column_type VARCHAR(30) NOT NULL DEFAULT 'actuals',
    header_label VARCHAR(200) NOT NULL,
    sub_header_label VARCHAR(200),
    -- Period specification
    period_offset INT NOT NULL DEFAULT 0,
    period_type VARCHAR(20) NOT NULL DEFAULT 'period',
    -- Computation
    compute_action VARCHAR(30),
    compute_source_columns JSONB DEFAULT '[]',
    -- Display
    show_column BOOLEAN NOT NULL DEFAULT true,
    column_width INT,
    format_override VARCHAR(20),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Report Executions (instances of generated reports)
CREATE TABLE IF NOT EXISTS _atlas.financial_report_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.financial_report_templates(id) ON DELETE CASCADE,
    run_number VARCHAR(100) NOT NULL,
    name VARCHAR(200),
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    -- Parameters used for this run
    as_of_date DATE,
    period_from DATE,
    period_to DATE,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    segment_filter JSONB DEFAULT '{}',
    include_unposted BOOLEAN NOT NULL DEFAULT false,
    -- Summary totals
    total_debit NUMERIC(20,2) DEFAULT 0,
    total_credit NUMERIC(20,2) DEFAULT 0,
    net_change NUMERIC(20,2) DEFAULT 0,
    beginning_balance NUMERIC(20,2) DEFAULT 0,
    ending_balance NUMERIC(20,2) DEFAULT 0,
    row_count INT DEFAULT 0,
    -- Lifecycle
    generated_by UUID,
    generated_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    published_by UUID,
    published_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Report Run Results (the actual data cells of a generated report)
CREATE TABLE IF NOT EXISTS _atlas.financial_report_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.financial_report_runs(id) ON DELETE CASCADE,
    row_id UUID NOT NULL REFERENCES _atlas.financial_report_rows(id),
    column_id UUID NOT NULL REFERENCES _atlas.financial_report_columns(id),
    row_number INT NOT NULL,
    column_number INT NOT NULL,
    -- Cell data
    amount NUMERIC(20,2) DEFAULT 0,
    debit_amount NUMERIC(20,2) DEFAULT 0,
    credit_amount NUMERIC(20,2) DEFAULT 0,
    beginning_balance NUMERIC(20,2) DEFAULT 0,
    ending_balance NUMERIC(20,2) DEFAULT 0,
    -- Computed vs data
    is_computed BOOLEAN NOT NULL DEFAULT false,
    compute_note TEXT,
    -- Display metadata
    display_amount VARCHAR(100),
    display_format VARCHAR(20),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Favourite Reports (user bookmarks for quick access)
CREATE TABLE IF NOT EXISTS _atlas.financial_report_favourites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.financial_report_templates(id) ON DELETE CASCADE,
    display_name VARCHAR(200),
    position INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, user_id, template_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_frt_org_active ON _atlas.financial_report_templates(organization_id, is_active);
CREATE INDEX IF NOT EXISTS idx_frr_template ON _atlas.financial_report_rows(template_id);
CREATE INDEX IF NOT EXISTS idx_frc_template ON _atlas.financial_report_columns(template_id);
CREATE INDEX IF NOT EXISTS idx_frerun_template ON _atlas.financial_report_runs(template_id);
CREATE INDEX IF NOT EXISTS idx_frerun_status ON _atlas.financial_report_runs(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_freresult_run ON _atlas.financial_report_results(run_id);
CREATE INDEX IF NOT EXISTS idx_frfav_user ON _atlas.financial_report_favourites(organization_id, user_id);
