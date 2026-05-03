-- Cash Flow Statements, Receivable Application Rules,
-- Accounting Event Processor, Tax Jurisdiction Rules,
-- and Asset Depreciation Schedules
-- Oracle Fusion Financial features implementation

-- ============================================================================
-- Cash Flow Statements (Oracle Fusion: GL > Financial Reports > Cash Flow)
-- ============================================================================
CREATE TABLE IF NOT EXISTS cash_flow_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_number VARCHAR(50) NOT NULL,
    method VARCHAR(20) NOT NULL CHECK (method IN ('direct', 'indirect')),
    period_type VARCHAR(20) NOT NULL CHECK (period_type IN ('monthly', 'quarterly', 'yearly')),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    opening_cash_balance DECIMAL(18,2) NOT NULL DEFAULT 0,
    operating_cash_flow DECIMAL(18,2) NOT NULL DEFAULT 0,
    investing_cash_flow DECIMAL(18,2) NOT NULL DEFAULT 0,
    financing_cash_flow DECIMAL(18,2) NOT NULL DEFAULT 0,
    net_change_in_cash DECIMAL(18,2) NOT NULL DEFAULT 0,
    closing_cash_balance DECIMAL(18,2) NOT NULL DEFAULT 0,
    exchange_rate_effect DECIMAL(18,2) NOT NULL DEFAULT 0,
    prepared_by UUID,
    reviewed_by UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'calculated', 'reviewed', 'published', 'archived')),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, statement_number)
);

CREATE INDEX idx_cash_flow_stmts_org ON cash_flow_statements(organization_id);
CREATE INDEX idx_cash_flow_stmts_status ON cash_flow_statements(status);
CREATE INDEX idx_cash_flow_stmts_period ON cash_flow_statements(period_start, period_end);

-- ============================================================================
-- Cash Flow Statement Lines
-- ============================================================================
CREATE TABLE IF NOT EXISTS cash_flow_statement_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    statement_id UUID NOT NULL REFERENCES cash_flow_statements(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    category VARCHAR(20) NOT NULL CHECK (category IN ('operating', 'investing', 'financing')),
    description TEXT,
    line_type VARCHAR(20) NOT NULL CHECK (line_type IN ('header', 'detail', 'subtotal', 'total')),
    amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    account_range_from VARCHAR(50),
    account_range_to VARCHAR(50),
    is_non_cash BOOLEAN NOT NULL DEFAULT FALSE,
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cash_flow_lines_stmt ON cash_flow_statement_lines(statement_id);

-- ============================================================================
-- Receivable Application Rules (Oracle Fusion: AR > Receipts > Application Rules)
-- ============================================================================
CREATE TABLE IF NOT EXISTS receivable_application_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    application_method VARCHAR(30) NOT NULL CHECK (application_method IN ('transaction_number', 'invoice_date', 'due_date', 'amount_match', 'custom')),
    matching_priority VARCHAR(30) NOT NULL CHECK (matching_priority IN ('transaction_number_first', 'oldest_first', 'largest_first', 'custom_order')),
    allow_over_application BOOLEAN NOT NULL DEFAULT FALSE,
    allow_under_application BOOLEAN NOT NULL DEFAULT TRUE,
    auto_apply_unapplied BOOLEAN NOT NULL DEFAULT FALSE,
    over_application_tolerance DECIMAL(18,2) NOT NULL DEFAULT 0,
    under_application_tolerance DECIMAL(18,2) NOT NULL DEFAULT 0,
    priority INT NOT NULL DEFAULT 1,
    effective_from DATE,
    effective_to DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'inactive')),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, rule_code)
);

CREATE INDEX idx_recv_app_rules_org ON receivable_application_rules(organization_id);

-- ============================================================================
-- Accounting Event Definitions (Oracle Fusion: SLA > Event Definitions)
-- ============================================================================
CREATE TABLE IF NOT EXISTS accounting_event_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    event_code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    event_class VARCHAR(30) NOT NULL CHECK (event_class IN ('create', 'update', 'cancel', 'reverse', 'complete')),
    entity_type VARCHAR(30) NOT NULL CHECK (entity_type IN ('invoice', 'payment', 'receipt', 'journal', 'asset', 'order')),
    journal_entry_template TEXT,
    requires_manual_review BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    processing_order INT NOT NULL DEFAULT 100,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, event_code)
);

CREATE INDEX idx_acct_event_defs_org ON accounting_event_definitions(organization_id);
CREATE INDEX idx_acct_event_defs_entity ON accounting_event_definitions(entity_type);

-- ============================================================================
-- Accounting Event Line Templates
-- ============================================================================
CREATE TABLE IF NOT EXISTS accounting_event_line_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_definition_id UUID NOT NULL REFERENCES accounting_event_definitions(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    description TEXT,
    entry_side VARCHAR(10) NOT NULL CHECK (entry_side IN ('debit', 'credit')),
    account_source VARCHAR(200),
    amount_source VARCHAR(200),
    segment_rules TEXT,
    is_tax_line BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_acct_event_line_tmpls_evt ON accounting_event_line_templates(event_definition_id);

-- ============================================================================
-- Tax Jurisdiction Rules (Oracle Fusion: Tax > Jurisdiction Rules)
-- ============================================================================
CREATE TABLE IF NOT EXISTS tax_jurisdiction_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    ship_from_country VARCHAR(3),
    ship_to_country VARCHAR(3),
    ship_from_region VARCHAR(100),
    ship_to_region VARCHAR(100),
    tax_regime_code VARCHAR(50) NOT NULL,
    tax_jurisdiction_code VARCHAR(50) NOT NULL,
    place_of_supply VARCHAR(20) NOT NULL CHECK (place_of_supply IN ('ship_from', 'ship_to', 'origin', 'destination')),
    priority INT NOT NULL DEFAULT 1,
    effective_from DATE,
    effective_to DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'inactive')),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, rule_code)
);

CREATE INDEX idx_tax_jurisdiction_rules_org ON tax_jurisdiction_rules(organization_id);
CREATE INDEX idx_tax_jurisdiction_rules_regime ON tax_jurisdiction_rules(tax_regime_code);

-- ============================================================================
-- Asset Depreciation Schedules (Oracle Fusion: FA > Depreciation Schedules)
-- ============================================================================
CREATE TABLE IF NOT EXISTS asset_depreciation_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    schedule_number VARCHAR(50) NOT NULL,
    asset_id UUID NOT NULL,
    book_id UUID,
    depreciation_method VARCHAR(30) NOT NULL CHECK (depreciation_method IN ('straight_line', 'declining_balance', 'sum_of_years_digits', 'units_of_production')),
    original_cost DECIMAL(18,2) NOT NULL DEFAULT 0,
    salvage_value DECIMAL(18,2) NOT NULL DEFAULT 0,
    depreciable_basis DECIMAL(18,2) NOT NULL DEFAULT 0,
    useful_life_months INT NOT NULL,
    in_service_date DATE,
    total_periods INT NOT NULL,
    total_depreciation DECIMAL(18,2) NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'generated', 'reviewed', 'posted', 'reversed')),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, schedule_number)
);

CREATE INDEX idx_asset_dep_schedules_org ON asset_depreciation_schedules(organization_id);
CREATE INDEX idx_asset_dep_schedules_asset ON asset_depreciation_schedules(asset_id);

-- ============================================================================
-- Depreciation Schedule Lines
-- ============================================================================
CREATE TABLE IF NOT EXISTS depreciation_schedule_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID NOT NULL REFERENCES asset_depreciation_schedules(id) ON DELETE CASCADE,
    fiscal_year INT NOT NULL,
    period_number INT NOT NULL,
    period_start_date DATE,
    period_end_date DATE,
    beginning_net_book_value DECIMAL(18,2) NOT NULL DEFAULT 0,
    depreciation_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    accumulated_depreciation DECIMAL(18,2) NOT NULL DEFAULT 0,
    ending_net_book_value DECIMAL(18,2) NOT NULL DEFAULT 0,
    depreciation_rate DECIMAL(8,6) NOT NULL DEFAULT 0,
    gl_account VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'planned' CHECK (status IN ('planned', 'posted', 'reversed')),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_dep_schedule_lines_schedule ON depreciation_schedule_lines(schedule_id);
CREATE INDEX idx_dep_schedule_lines_period ON depreciation_schedule_lines(fiscal_year, period_number);
