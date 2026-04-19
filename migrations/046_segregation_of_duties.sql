-- Segregation of Duties (SoD) Tables
-- Oracle Fusion Cloud ERP: Advanced Access Control > Segregation of Duties

-- SoD Rules: Define pairs of incompatible duties
CREATE TABLE IF NOT EXISTS _atlas.sod_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    first_duties JSONB NOT NULL DEFAULT '[]',
    second_duties JSONB NOT NULL DEFAULT '[]',
    enforcement_mode VARCHAR(20) NOT NULL DEFAULT 'detective',  -- 'preventive' or 'detective'
    risk_level VARCHAR(10) NOT NULL DEFAULT 'medium',  -- 'high', 'medium', 'low'
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Role Assignments: Track which duties each user has
CREATE TABLE IF NOT EXISTS _atlas.sod_role_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role_name VARCHAR(200) NOT NULL,
    duty_code VARCHAR(200) NOT NULL,
    assigned_by UUID,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- SoD Violations: Detected conflicts between duties for a user
CREATE TABLE IF NOT EXISTS _atlas.sod_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_id UUID NOT NULL REFERENCES _atlas.sod_rules(id),
    rule_code VARCHAR(100) NOT NULL,
    user_id UUID NOT NULL,
    first_matched_duties JSONB NOT NULL DEFAULT '[]',
    second_matched_duties JSONB NOT NULL DEFAULT '[]',
    violation_status VARCHAR(20) NOT NULL DEFAULT 'open',  -- 'open', 'mitigated', 'exception', 'resolved'
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolved_by UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Mitigating Controls: Compensating controls for accepted violations
CREATE TABLE IF NOT EXISTS _atlas.sod_mitigating_controls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    violation_id UUID NOT NULL REFERENCES _atlas.sod_violations(id),
    control_name VARCHAR(200) NOT NULL,
    control_description TEXT NOT NULL,
    control_owner_id UUID,
    review_frequency VARCHAR(20) NOT NULL DEFAULT 'monthly',  -- 'daily', 'weekly', 'monthly', 'quarterly'
    effective_from DATE,
    effective_to DATE,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'pending_approval',  -- 'pending_approval', 'active', 'expired', 'revoked'
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_sod_rules_org ON _atlas.sod_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_sod_rules_active ON _atlas.sod_rules(organization_id, is_active);
CREATE INDEX IF NOT EXISTS idx_sod_assignments_user ON _atlas.sod_role_assignments(organization_id, user_id, is_active);
CREATE INDEX IF NOT EXISTS idx_sod_violations_org ON _atlas.sod_violations(organization_id);
CREATE INDEX IF NOT EXISTS idx_sod_violations_user ON _atlas.sod_violations(user_id);
CREATE INDEX IF NOT EXISTS idx_sod_violations_status ON _atlas.sod_violations(violation_status);
CREATE INDEX IF NOT EXISTS idx_sod_violations_rule_user ON _atlas.sod_violations(rule_id, user_id, violation_status);
CREATE INDEX IF NOT EXISTS idx_sod_mitigations_violation ON _atlas.sod_mitigating_controls(violation_id);
CREATE INDEX IF NOT EXISTS idx_sod_mitigations_org ON _atlas.sod_mitigating_controls(organization_id);
