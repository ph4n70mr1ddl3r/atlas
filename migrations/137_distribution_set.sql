-- 137_distribution_set.sql
-- Oracle Fusion Financial Feature: Distribution Sets
-- Reusable GL account distribution templates for AP invoices.
-- Oracle Fusion: Financials > Payables > Distribution Sets

-- ============================================================================
-- Distribution Set Header
-- Oracle Fusion: Payables > Setup > Distribution Sets
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.distribution_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    set_code VARCHAR(50) NOT NULL,
    set_name VARCHAR(200) NOT NULL,
    description TEXT,
    distribution_type VARCHAR(20) NOT NULL DEFAULT 'percentage',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    total_percentage NUMERIC(8,4) NOT NULL DEFAULT 0.0000,
    total_amount NUMERIC(18,2),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    is_default BOOLEAN NOT NULL DEFAULT false,
    effective_from DATE,
    effective_to DATE,
    usage_count INT NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, set_code)
);

CREATE INDEX idx_distribution_sets_org ON financials.distribution_sets(organization_id);
CREATE INDEX idx_distribution_sets_status ON financials.distribution_sets(status);
CREATE INDEX idx_distribution_sets_type ON financials.distribution_sets(distribution_type);
CREATE INDEX idx_distribution_sets_default ON financials.distribution_sets(organization_id) WHERE is_default = true;

-- ============================================================================
-- Distribution Set Lines (individual account distributions)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.distribution_set_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    distribution_set_id UUID NOT NULL REFERENCES financials.distribution_sets(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    account_combination VARCHAR(500) NOT NULL,
    account_description TEXT,
    segment1 VARCHAR(50),
    segment2 VARCHAR(50),
    segment3 VARCHAR(50),
    segment4 VARCHAR(50),
    segment5 VARCHAR(50),
    percentage NUMERIC(8,4) NOT NULL DEFAULT 0.0000,
    amount NUMERIC(18,2),
    description TEXT,
    cost_center VARCHAR(50),
    department VARCHAR(100),
    project_code VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_distribution_set_lines_set ON financials.distribution_set_lines(distribution_set_id);
CREATE INDEX idx_distribution_set_lines_active ON financials.distribution_set_lines(is_active);

-- ============================================================================
-- Distribution Set Usage Log (audit trail of when sets are applied)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.distribution_set_usage_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    distribution_set_id UUID NOT NULL REFERENCES financials.distribution_sets(id),
    set_code VARCHAR(50) NOT NULL,
    target_entity_type VARCHAR(50) NOT NULL,
    target_entity_id UUID NOT NULL,
    target_entity_number VARCHAR(50),
    applied_by UUID,
    applied_at TIMESTAMPTZ DEFAULT now(),
    line_count INT NOT NULL DEFAULT 0,
    total_amount NUMERIC(18,2),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_distribution_set_usage_set ON financials.distribution_set_usage_log(distribution_set_id);
CREATE INDEX idx_distribution_set_usage_target ON financials.distribution_set_usage_log(target_entity_type, target_entity_id);
CREATE INDEX idx_distribution_set_usage_applied ON financials.distribution_set_usage_log(applied_at);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.distribution_set_dashboard AS
SELECT
    ds.organization_id,
    COUNT(DISTINCT ds.id) AS total_sets,
    COUNT(DISTINCT ds.id) FILTER (WHERE ds.status = 'active') AS active_sets,
    COUNT(DISTINCT ds.id) FILTER (WHERE ds.status = 'inactive') AS inactive_sets,
    COUNT(DISTINCT ds.id) FILTER (WHERE ds.is_default = true) AS default_sets,
    COALESCE(SUM(ds.usage_count), 0) AS total_usages,
    COUNT(DISTINCT ds.id) FILTER (WHERE ds.distribution_type = 'percentage') AS percentage_sets,
    COUNT(DISTINCT ds.id) FILTER (WHERE ds.distribution_type = 'amount') AS amount_sets,
    AVG(ds.usage_count) FILTER (WHERE ds.status = 'active') AS avg_usage_per_set
FROM financials.distribution_sets ds
GROUP BY ds.organization_id;

COMMENT ON TABLE financials.distribution_sets IS 'Distribution set headers - reusable GL account distribution templates for AP invoices';
COMMENT ON TABLE financials.distribution_set_lines IS 'Distribution set line items - individual GL account distributions within a set';
COMMENT ON TABLE financials.distribution_set_usage_log IS 'Audit trail of distribution set applications to invoices and other entities';
COMMENT ON VIEW financials.distribution_set_dashboard IS 'Dashboard aggregation for distribution set statistics';
