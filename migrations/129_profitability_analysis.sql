-- Profitability Analysis
-- Oracle Fusion: Financials > Profitability Analysis
-- Manages profitability segment definitions, revenue/cost assignments,
-- margin calculations (gross, operating, net), and period-over-period
-- comparisons by product, customer, channel, and geography.

-- Profitability segment definitions (dimensions to analyze)
CREATE TABLE IF NOT EXISTS _atlas.profitability_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    segment_code VARCHAR(50) NOT NULL,
    segment_name VARCHAR(200) NOT NULL,
    segment_type VARCHAR(30) NOT NULL,
    description TEXT,
    parent_segment_id UUID,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, segment_code)
);

-- Profitability analysis runs (periodic snapshots)
CREATE TABLE IF NOT EXISTS _atlas.profitability_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_number VARCHAR(50) NOT NULL,
    run_name VARCHAR(200) NOT NULL,
    analysis_type VARCHAR(30) NOT NULL DEFAULT 'standard',
    period_from DATE NOT NULL,
    period_to DATE NOT NULL,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    total_revenue DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_cogs DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_gross_margin DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_operating_expenses DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_operating_margin DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_net_margin DOUBLE PRECISION NOT NULL DEFAULT 0,
    gross_margin_pct DOUBLE PRECISION DEFAULT 0,
    operating_margin_pct DOUBLE PRECISION DEFAULT 0,
    net_margin_pct DOUBLE PRECISION DEFAULT 0,
    segment_count INT DEFAULT 0,
    comparison_run_id UUID,
    notes TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, run_number)
);

-- Profitability analysis lines (per-segment detail)
CREATE TABLE IF NOT EXISTS _atlas.profitability_run_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    run_id UUID NOT NULL REFERENCES _atlas.profitability_runs(id) ON DELETE CASCADE,
    segment_id UUID,
    segment_code VARCHAR(50),
    segment_name VARCHAR(200),
    segment_type VARCHAR(30),
    line_number INT NOT NULL,
    revenue DOUBLE PRECISION NOT NULL DEFAULT 0,
    cost_of_goods_sold DOUBLE PRECISION NOT NULL DEFAULT 0,
    gross_margin DOUBLE PRECISION NOT NULL DEFAULT 0,
    gross_margin_pct DOUBLE PRECISION DEFAULT 0,
    operating_expenses DOUBLE PRECISION NOT NULL DEFAULT 0,
    operating_margin DOUBLE PRECISION NOT NULL DEFAULT 0,
    operating_margin_pct DOUBLE PRECISION DEFAULT 0,
    other_income DOUBLE PRECISION DEFAULT 0,
    other_expense DOUBLE PRECISION DEFAULT 0,
    net_margin DOUBLE PRECISION NOT NULL DEFAULT 0,
    net_margin_pct DOUBLE PRECISION DEFAULT 0,
    revenue_contribution_pct DOUBLE PRECISION DEFAULT 0,
    margin_contribution_pct DOUBLE PRECISION DEFAULT 0,
    prior_period_revenue DOUBLE PRECISION DEFAULT 0,
    prior_period_cogs DOUBLE PRECISION DEFAULT 0,
    prior_period_net_margin DOUBLE PRECISION DEFAULT 0,
    revenue_change_pct DOUBLE PRECISION DEFAULT 0,
    margin_change_pct DOUBLE PRECISION DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Profitability templates (reusable analysis definitions)
CREATE TABLE IF NOT EXISTS _atlas.profitability_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_code VARCHAR(50) NOT NULL,
    template_name VARCHAR(200) NOT NULL,
    description TEXT,
    segment_type VARCHAR(30) NOT NULL,
    includes_cogs BOOLEAN DEFAULT true,
    includes_operating BOOLEAN DEFAULT true,
    includes_other BOOLEAN DEFAULT true,
    auto_calculate BOOLEAN DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, template_code)
);

CREATE INDEX IF NOT EXISTS idx_profit_segments_org ON _atlas.profitability_segments(organization_id);
CREATE INDEX IF NOT EXISTS idx_profit_segments_type ON _atlas.profitability_segments(organization_id, segment_type);
CREATE INDEX IF NOT EXISTS idx_profit_runs_org ON _atlas.profitability_runs(organization_id);
CREATE INDEX IF NOT EXISTS idx_profit_runs_status ON _atlas.profitability_runs(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_profit_run_lines_run ON _atlas.profitability_run_lines(run_id);
CREATE INDEX IF NOT EXISTS idx_profit_run_lines_segment ON _atlas.profitability_run_lines(run_id, segment_id);
CREATE INDEX IF NOT EXISTS idx_profit_templates_org ON _atlas.profitability_templates(organization_id);
