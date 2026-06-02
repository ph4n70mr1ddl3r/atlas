-- 138_cash_flow_statement.sql
-- Oracle Fusion Financial Feature: Cash Flow Statements
-- Generates cash flow statements using direct or indirect methods
-- with line-level detail for operating, investing, and financing activities.
-- Oracle Fusion: Financials > General Ledger > Financial Reports > Cash Flow Statements

-- ============================================================================
-- Cash Flow Statement Header
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.cash_flow_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    statement_number VARCHAR(50) NOT NULL,
    method VARCHAR(20) NOT NULL DEFAULT 'indirect',
    period_type VARCHAR(20) NOT NULL DEFAULT 'monthly',
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    opening_cash_balance NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    operating_cash_flow NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    investing_cash_flow NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    financing_cash_flow NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    net_change_in_cash NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    closing_cash_balance NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    exchange_rate_effect NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    prepared_by UUID,
    reviewed_by UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, statement_number)
);

CREATE INDEX IF NOT EXISTS idx_cfs_org ON financials.cash_flow_statements(organization_id);
CREATE INDEX IF NOT EXISTS idx_cfs_status ON financials.cash_flow_statements(status);
CREATE INDEX IF NOT EXISTS idx_cfs_method ON financials.cash_flow_statements(method);
CREATE INDEX IF NOT EXISTS idx_cfs_period ON financials.cash_flow_statements(period_start, period_end);

-- ============================================================================
-- Cash Flow Statement Lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.cash_flow_statement_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    statement_id UUID NOT NULL REFERENCES financials.cash_flow_statements(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    category VARCHAR(20) NOT NULL,
    description TEXT,
    line_type VARCHAR(20) NOT NULL DEFAULT 'detail',
    amount NUMERIC(18,2) NOT NULL DEFAULT 0.00,
    account_range_from VARCHAR(50),
    account_range_to VARCHAR(50),
    is_non_cash BOOLEAN NOT NULL DEFAULT false,
    display_order INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cfsl_statement ON financials.cash_flow_statement_lines(statement_id);
CREATE INDEX IF NOT EXISTS idx_cfsl_category ON financials.cash_flow_statement_lines(category);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.cash_flow_statement_dashboard AS
SELECT
    cfs.organization_id,
    COUNT(DISTINCT cfs.id) AS total_statements,
    COUNT(DISTINCT cfs.id) FILTER (WHERE cfs.status = 'draft') AS draft_statements,
    COUNT(DISTINCT cfs.id) FILTER (WHERE cfs.status = 'published') AS published_statements,
    COALESCE(SUM(cfs.operating_cash_flow), 0) AS total_operating,
    COALESCE(SUM(cfs.investing_cash_flow), 0) AS total_investing,
    COALESCE(SUM(cfs.financing_cash_flow), 0) AS total_financing
FROM financials.cash_flow_statements cfs
GROUP BY cfs.organization_id;

COMMENT ON TABLE financials.cash_flow_statements IS 'Cash flow statement headers - direct/indirect method cash flow reports';
COMMENT ON TABLE financials.cash_flow_statement_lines IS 'Cash flow statement lines - individual line items categorized as operating/investing/financing';
COMMENT ON VIEW financials.cash_flow_statement_dashboard IS 'Dashboard aggregation for cash flow statement statistics';
