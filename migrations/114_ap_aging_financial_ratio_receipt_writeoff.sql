-- AP Aging Analysis
-- Oracle Fusion: Payables > Aging Reports

CREATE TABLE IF NOT EXISTS _atlas.ap_aging_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    aging_basis VARCHAR(30) NOT NULL DEFAULT 'due_date',
    num_buckets INT NOT NULL DEFAULT 5,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, definition_code)
);

CREATE TABLE IF NOT EXISTS _atlas.ap_aging_buckets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_id UUID NOT NULL REFERENCES _atlas.ap_aging_definitions(id) ON DELETE CASCADE,
    bucket_number INT NOT NULL,
    name VARCHAR(100) NOT NULL,
    from_days INT NOT NULL DEFAULT 0,
    to_days INT,
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.ap_aging_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    definition_id UUID NOT NULL REFERENCES _atlas.ap_aging_definitions(id),
    snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
    as_of_date DATE NOT NULL,
    total_open_amount NUMERIC(18,2) DEFAULT 0,
    total_overdue_amount NUMERIC(18,2) DEFAULT 0,
    total_past_due_count INT DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'completed',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.ap_aging_snapshot_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    snapshot_id UUID NOT NULL REFERENCES _atlas.ap_aging_snapshots(id) ON DELETE CASCADE,
    supplier_id UUID,
    supplier_number VARCHAR(50) NOT NULL,
    supplier_name VARCHAR(200),
    invoice_id UUID,
    invoice_number VARCHAR(50) NOT NULL,
    invoice_date DATE NOT NULL,
    due_date DATE NOT NULL,
    original_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    open_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    days_past_due INT NOT NULL DEFAULT 0,
    bucket_number INT NOT NULL DEFAULT 0,
    bucket_name VARCHAR(100) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Financial Ratio Analysis
-- Oracle Fusion: Financial Reporting Center > Ratio Analysis

CREATE TABLE IF NOT EXISTS _atlas.financial_ratio_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ratio_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(30) NOT NULL,
    formula TEXT NOT NULL,
    numerator_accounts JSONB DEFAULT '[]',
    denominator_accounts JSONB DEFAULT '[]',
    unit VARCHAR(20) NOT NULL DEFAULT 'ratio',
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, ratio_code)
);

CREATE TABLE IF NOT EXISTS _atlas.financial_ratio_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    total_ratios INT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.financial_ratio_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    snapshot_id UUID NOT NULL REFERENCES _atlas.financial_ratio_snapshots(id) ON DELETE CASCADE,
    ratio_id UUID NOT NULL REFERENCES _atlas.financial_ratio_definitions(id),
    ratio_code VARCHAR(50) NOT NULL,
    ratio_name VARCHAR(200) NOT NULL,
    category VARCHAR(30) NOT NULL,
    numerator_value NUMERIC(18,4) NOT NULL DEFAULT 0,
    denominator_value NUMERIC(18,4) NOT NULL DEFAULT 0,
    result_value VARCHAR(50) NOT NULL,
    unit VARCHAR(20) NOT NULL DEFAULT 'ratio',
    previous_value VARCHAR(50),
    change_amount VARCHAR(50),
    change_percent VARCHAR(50),
    trend_direction VARCHAR(20),
    benchmark_value VARCHAR(50),
    status_flag VARCHAR(30),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.financial_ratio_benchmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ratio_id UUID NOT NULL REFERENCES _atlas.financial_ratio_definitions(id),
    name VARCHAR(200) NOT NULL,
    benchmark_value NUMERIC(18,4) NOT NULL,
    min_acceptable NUMERIC(18,4),
    max_acceptable NUMERIC(18,4),
    industry VARCHAR(100),
    effective_from DATE NOT NULL,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Receipt Write-Off
-- Oracle Fusion: Receivables > Receipts > Write-Off

CREATE TABLE IF NOT EXISTS _atlas.receipt_write_off_reasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    reason_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    default_gl_account VARCHAR(50),
    requires_approval BOOLEAN DEFAULT true,
    max_auto_approve_amount NUMERIC(18,2),
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, reason_code)
);

CREATE TABLE IF NOT EXISTS _atlas.receipt_write_off_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    request_number VARCHAR(50) NOT NULL UNIQUE,
    receipt_id UUID NOT NULL,
    receipt_number VARCHAR(50) NOT NULL,
    customer_id UUID,
    customer_number VARCHAR(50),
    write_off_amount NUMERIC(18,2) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    reason_id UUID NOT NULL REFERENCES _atlas.receipt_write_off_reasons(id),
    reason_code VARCHAR(50) NOT NULL,
    comments TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    gl_account_code VARCHAR(50),
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    journal_entry_id UUID,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.receipt_write_off_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    total_amount NUMERIC(18,2) DEFAULT 0,
    total_count INT DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.receipt_write_off_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    min_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    max_amount NUMERIC(18,2) NOT NULL,
    requires_approval BOOLEAN DEFAULT true,
    auto_approve_below NUMERIC(18,2),
    default_gl_account VARCHAR(50),
    aging_threshold_days INT,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes for performance
CREATE INDEX idx_ap_aging_defs_org ON _atlas.ap_aging_definitions(organization_id);
CREATE INDEX idx_ap_aging_snapshots_org ON _atlas.ap_aging_snapshots(organization_id);
CREATE INDEX idx_ap_aging_lines_snapshot ON _atlas.ap_aging_snapshot_lines(snapshot_id);
CREATE INDEX idx_fin_ratio_defs_org ON _atlas.financial_ratio_definitions(organization_id);
CREATE INDEX idx_fin_ratio_snapshots_org ON _atlas.financial_ratio_snapshots(organization_id);
CREATE INDEX idx_fin_ratio_results_snapshot ON _atlas.financial_ratio_results(snapshot_id);
CREATE INDEX idx_wo_reasons_org ON _atlas.receipt_write_off_reasons(organization_id);
CREATE INDEX idx_wo_requests_org ON _atlas.receipt_write_off_requests(organization_id);
CREATE INDEX idx_wo_batches_org ON _atlas.receipt_write_off_batches(organization_id);
CREATE INDEX idx_wo_policies_org ON _atlas.receipt_write_off_policies(organization_id);
