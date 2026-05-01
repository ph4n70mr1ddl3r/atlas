-- 107_financial_controls.sql
-- Advanced Financial Controls tables
-- Oracle Fusion equivalent: Financials > Advanced Controls

CREATE TABLE IF NOT EXISTS _atlas.control_monitor_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(30) NOT NULL, -- transaction, access, master_data, period_close, master_record
    risk_level VARCHAR(20) NOT NULL, -- critical, high, medium, low
    control_type VARCHAR(30) NOT NULL, -- threshold, pattern, frequency, segregation, approval, custom
    conditions JSONB DEFAULT '{}',
    threshold_value TEXT,
    target_entity VARCHAR(100) NOT NULL,
    target_fields JSONB DEFAULT '[]',
    actions JSONB DEFAULT '[]', -- alert, block, escalate, review
    auto_resolve BOOLEAN NOT NULL DEFAULT false,
    check_schedule VARCHAR(20) NOT NULL DEFAULT 'daily', -- realtime, daily, weekly, monthly
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    last_check_at TIMESTAMPTZ,
    last_violation_at TIMESTAMPTZ,
    total_violations INT NOT NULL DEFAULT 0,
    total_resolved INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.control_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_id UUID NOT NULL,
    rule_code VARCHAR(100),
    rule_name VARCHAR(200),
    violation_number VARCHAR(50) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID,
    description TEXT NOT NULL,
    findings JSONB DEFAULT '{}',
    risk_level VARCHAR(20) NOT NULL, -- critical, high, medium, low
    status VARCHAR(20) NOT NULL DEFAULT 'open', -- open, under_review, resolved, false_positive, escalated, waived
    assigned_to UUID,
    assigned_to_name VARCHAR(200),
    resolution_notes TEXT,
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    escalated_to UUID,
    escalated_at TIMESTAMPTZ,
    related_entities JSONB DEFAULT '[]',
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, violation_number)
);

CREATE INDEX IF NOT EXISTS idx_control_rules_org ON _atlas.control_monitor_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_control_rules_risk ON _atlas.control_monitor_rules(organization_id, risk_level);
CREATE INDEX IF NOT EXISTS idx_control_violations_org ON _atlas.control_violations(organization_id);
CREATE INDEX IF NOT EXISTS idx_control_violations_status ON _atlas.control_violations(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_control_violations_rule ON _atlas.control_violations(rule_id);
CREATE INDEX IF NOT EXISTS idx_control_violations_risk ON _atlas.control_violations(risk_level);
