-- Transfer Pricing Management (Oracle Fusion Financials > Transfer Pricing)
-- Manages intercompany transfer price policies, transactions, benchmarking,
-- and BEPS/OECD documentation for tax compliance.

-- ═══════════════════════════════════════════════════════════════════
-- Transfer Pricing Policies
-- ═══════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS _atlas.transfer_pricing_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    policy_code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    pricing_method VARCHAR(50) NOT NULL, -- CUP, resale_price, cost_plus, profit_split, tnmm, other
    from_entity_id UUID,
    from_entity_name VARCHAR(300),
    to_entity_id UUID,
    to_entity_name VARCHAR(300),
    product_category VARCHAR(200),
    item_id UUID,
    item_code VARCHAR(100),
    geography VARCHAR(100),
    tax_jurisdiction VARCHAR(100),
    effective_from DATE,
    effective_to DATE,
    arm_length_range_low NUMERIC(18,4),
    arm_length_range_mid NUMERIC(18,4),
    arm_length_range_high NUMERIC(18,4),
    margin_pct NUMERIC(8,4),
    cost_base VARCHAR(50), -- full_cost, variable_cost, total_cost, custom
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, active, inactive, expired
    version INT NOT NULL DEFAULT 1,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_by UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, policy_code)
);

-- ═══════════════════════════════════════════════════════════════════
-- Transfer Price Transactions
-- ═══════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS _atlas.transfer_pricing_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    transaction_number VARCHAR(100) NOT NULL,
    policy_id UUID REFERENCES _atlas.transfer_pricing_policies(id),
    from_entity_id UUID,
    from_entity_name VARCHAR(300),
    to_entity_id UUID,
    to_entity_name VARCHAR(300),
    item_id UUID,
    item_code VARCHAR(100),
    item_description TEXT,
    quantity NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    transfer_price NUMERIC(18,4) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    transaction_date DATE NOT NULL,
    gl_date DATE,
    source_type VARCHAR(50), -- intercompany, sales_order, purchase_order, manual
    source_id UUID,
    source_number VARCHAR(100),
    margin_applied NUMERIC(8,4),
    margin_amount NUMERIC(18,4),
    is_arm_length_compliant BOOLEAN,
    compliance_notes TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, submitted, approved, rejected, completed
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_by UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- ═══════════════════════════════════════════════════════════════════
-- Benchmark Studies (Arm's-Length Analyses)
-- ═══════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS _atlas.transfer_pricing_benchmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    study_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    policy_id UUID REFERENCES _atlas.transfer_pricing_policies(id),
    analysis_method VARCHAR(50) NOT NULL, -- cup, resale_price, cost_plus, profit_split, tnmm, berry_ratio
    fiscal_year INT,
    from_entity_id UUID,
    from_entity_name VARCHAR(300),
    to_entity_id UUID,
    to_entity_name VARCHAR(300),
    product_category VARCHAR(200),
    tested_party VARCHAR(300),
    interquartile_range_low NUMERIC(18,4),
    interquartile_range_mid NUMERIC(18,4),
    interquartile_range_high NUMERIC(18,4),
    tested_result NUMERIC(18,4),
    is_within_range BOOLEAN,
    conclusion TEXT,
    prepared_by UUID,
    prepared_by_name VARCHAR(300),
    reviewed_by UUID,
    reviewed_by_name VARCHAR(300),
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, in_review, approved, rejected, superseded
    approved_at TIMESTAMPTZ,
    created_by UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- ═══════════════════════════════════════════════════════════════════
-- Benchmark Comparables
-- ═══════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS _atlas.transfer_pricing_comparables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    benchmark_id UUID NOT NULL REFERENCES _atlas.transfer_pricing_benchmarks(id) ON DELETE CASCADE,
    comparable_number INT NOT NULL,
    company_name VARCHAR(300) NOT NULL,
    country VARCHAR(100),
    industry_code VARCHAR(50),
    industry_description VARCHAR(300),
    fiscal_year INT,
    revenue NUMERIC(18,4),
    operating_income NUMERIC(18,4),
    operating_margin_pct NUMERIC(8,4),
    net_income NUMERIC(18,4),
    total_assets NUMERIC(18,4),
    employees INT,
    data_source VARCHAR(100),
    is_included BOOLEAN NOT NULL DEFAULT true,
    exclusion_reason TEXT,
    relevance_score NUMERIC(5,2),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- ═══════════════════════════════════════════════════════════════════
-- Documentation Packages (BEPS / Local File / Master File / CbCR)
-- ═══════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS _atlas.transfer_pricing_documentation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    doc_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    doc_type VARCHAR(50) NOT NULL, -- master_file, local_file, cbcr, country_by_country, other
    fiscal_year INT NOT NULL,
    country VARCHAR(100),
    reporting_entity_id UUID,
    reporting_entity_name VARCHAR(300),
    description TEXT,
    content_summary TEXT,
    policy_ids UUID[] DEFAULT '{}',
    benchmark_ids UUID[] DEFAULT '{}',
    filing_date DATE,
    filing_deadline DATE,
    responsible_party VARCHAR(300),
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, in_review, approved, filed, superseded
    reviewed_by UUID,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    filed_at TIMESTAMPTZ,
    created_by UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tp_policies_org ON _atlas.transfer_pricing_policies(organization_id);
CREATE INDEX IF NOT EXISTS idx_tp_policies_status ON _atlas.transfer_pricing_policies(status);
CREATE INDEX IF NOT EXISTS idx_tp_transactions_org ON _atlas.transfer_pricing_transactions(organization_id);
CREATE INDEX IF NOT EXISTS idx_tp_transactions_policy ON _atlas.transfer_pricing_transactions(policy_id);
CREATE INDEX IF NOT EXISTS idx_tp_transactions_date ON _atlas.transfer_pricing_transactions(transaction_date);
CREATE INDEX IF NOT EXISTS idx_tp_benchmarks_org ON _atlas.transfer_pricing_benchmarks(organization_id);
CREATE INDEX IF NOT EXISTS idx_tp_comparables_bench ON _atlas.transfer_pricing_comparables(benchmark_id);
CREATE INDEX IF NOT EXISTS idx_tp_docs_org ON _atlas.transfer_pricing_documentation(organization_id);
CREATE INDEX IF NOT EXISTS idx_tp_docs_type ON _atlas.transfer_pricing_documentation(doc_type);
