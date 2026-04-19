-- Cross-Validation Rules (Oracle Fusion GL > Chart of Accounts > Cross-Validation)
-- Prevents invalid combinations of account segment values.

-- Cross-validation rule definitions
CREATE TABLE IF NOT EXISTS _atlas.cross_validation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    rule_type VARCHAR(20) NOT NULL DEFAULT 'deny',  -- deny, allow
    error_message TEXT NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    priority INT NOT NULL DEFAULT 10,
    segment_names JSONB NOT NULL DEFAULT '[]',  -- ["company", "department", "account"]
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Rule lines (from/to patterns)
CREATE TABLE IF NOT EXISTS _atlas.cross_validation_rule_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    rule_id UUID NOT NULL REFERENCES _atlas.cross_validation_rules(id) ON DELETE CASCADE,
    line_type VARCHAR(20) NOT NULL,  -- from, to
    patterns JSONB NOT NULL DEFAULT '[]',  -- ["1000", "%", "5000"]  (% = any)
    display_order INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_cvr_rules_org ON _atlas.cross_validation_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_cvr_rules_org_enabled ON _atlas.cross_validation_rules(organization_id, is_enabled);
CREATE INDEX IF NOT EXISTS idx_cvr_rule_lines_rule ON _atlas.cross_validation_rule_lines(rule_id);
