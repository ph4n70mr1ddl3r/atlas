-- ============================================================================
-- Migration 120: Expense Policy Compliance Engine
-- Oracle Fusion: Expenses > Policies > Expense Policy Compliance
--
-- Implements configurable expense policy rules, compliance evaluation,
-- violation tracking, and audit scoring for corporate expense management.
-- ============================================================================

-- ============================================================================
-- Expense Policy Rules
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_expense_policy_rules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL,
    rule_code       VARCHAR(50) NOT NULL,
    name            VARCHAR(200) NOT NULL,
    description     TEXT,

    -- Rule definition
    rule_type       VARCHAR(30) NOT NULL,  -- amount_limit, daily_limit, category_limit, receipt_required, time_restriction, duplicate_check, approval_required, per_diem_override
    expense_category VARCHAR(30) NOT NULL, -- airfare, hotel, meals, etc., or 'all'
    severity        VARCHAR(20) NOT NULL DEFAULT 'warning', -- warning, violation, block
    evaluation_scope VARCHAR(20) NOT NULL DEFAULT 'per_line', -- per_line, per_day, per_report, per_trip

    -- Thresholds and limits
    threshold_amount    DECIMAL(18, 2) DEFAULT 0,
    maximum_amount      DECIMAL(18, 2) DEFAULT 0,
    threshold_days      INTEGER DEFAULT 0,

    -- Requirements
    requires_receipt      BOOLEAN DEFAULT FALSE,
    requires_justification BOOLEAN DEFAULT FALSE,

    -- Status and lifecycle
    is_active             BOOLEAN DEFAULT TRUE,
    effective_from        DATE,
    effective_to          DATE,

    -- Applicability
    applies_to_department VARCHAR(200),
    applies_to_cost_center VARCHAR(100),

    -- Audit
    created_by_id  UUID,
    status         VARCHAR(20) NOT NULL DEFAULT 'draft',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(org_id, rule_code)
);

CREATE INDEX idx_expense_policy_rules_org ON fin_expense_policy_rules(org_id);
CREATE INDEX idx_expense_policy_rules_type ON fin_expense_policy_rules(org_id, rule_type);
CREATE INDEX idx_expense_policy_rules_category ON fin_expense_policy_rules(org_id, expense_category);
CREATE INDEX idx_expense_policy_rules_active ON fin_expense_policy_rules(org_id, is_active) WHERE is_active = TRUE;

-- ============================================================================
-- Expense Compliance Audits
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_expense_compliance_audits (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL,
    audit_number    VARCHAR(50) NOT NULL,

    -- References
    report_id       UUID NOT NULL,
    report_number   VARCHAR(50),
    employee_id     UUID,
    employee_name   VARCHAR(200),
    department_id   UUID,

    -- Audit details
    audit_date      DATE NOT NULL DEFAULT CURRENT_DATE,
    audit_trigger   VARCHAR(30) NOT NULL DEFAULT 'automatic', -- automatic, random_sample, high_amount, policy_violation, manual

    -- Statistics
    total_lines     INTEGER DEFAULT 0,
    violations_count INTEGER DEFAULT 0,
    warnings_count  INTEGER DEFAULT 0,
    blocks_count    INTEGER DEFAULT 0,
    compliance_score DECIMAL(5, 2) DEFAULT 100.00,
    risk_level      VARCHAR(20) NOT NULL DEFAULT 'low', -- low, medium, high, critical

    -- Amounts
    total_flagged_amount  DECIMAL(18, 2) DEFAULT 0,
    total_approved_amount DECIMAL(18, 2) DEFAULT 0,

    -- Review flags
    requires_manager_review  BOOLEAN DEFAULT FALSE,
    requires_finance_review  BOOLEAN DEFAULT FALSE,

    -- Workflow status
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',
    reviewed_by_id  UUID,
    review_notes    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(org_id, audit_number)
);

CREATE INDEX idx_expense_compliance_audits_org ON fin_expense_compliance_audits(org_id);
CREATE INDEX idx_expense_compliance_audits_report ON fin_expense_compliance_audits(report_id);
CREATE INDEX idx_expense_compliance_audits_employee ON fin_expense_compliance_audits(employee_id);
CREATE INDEX idx_expense_compliance_audits_status ON fin_expense_compliance_audits(org_id, status);
CREATE INDEX idx_expense_compliance_audits_risk ON fin_expense_compliance_audits(org_id, risk_level) WHERE risk_level IN ('high', 'critical');

-- ============================================================================
-- Expense Compliance Violations
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_expense_compliance_violations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL,

    -- References
    audit_id        UUID NOT NULL REFERENCES fin_expense_compliance_audits(id) ON DELETE CASCADE,
    report_id       UUID NOT NULL,
    report_line_id  UUID,
    policy_rule_id  UUID REFERENCES fin_expense_policy_rules(id),

    -- Rule details (denormalized for audit trail)
    rule_code       VARCHAR(50) NOT NULL,
    rule_name       VARCHAR(200),
    rule_type       VARCHAR(30) NOT NULL,
    severity        VARCHAR(20) NOT NULL,

    -- Violation details
    violation_description TEXT,
    expense_amount  DECIMAL(18, 2) NOT NULL DEFAULT 0,
    threshold_amount DECIMAL(18, 2) DEFAULT 0,
    excess_amount   DECIMAL(18, 2) DEFAULT 0,

    -- Resolution
    resolution_status VARCHAR(20) NOT NULL DEFAULT 'open', -- open, justified, adjusted, upheld, escalated
    justification    TEXT,
    resolved_by_id   UUID,
    resolution_date  DATE,

    -- Audit
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_expense_compliance_violations_audit ON fin_expense_compliance_violations(audit_id);
CREATE INDEX idx_expense_compliance_violations_report ON fin_expense_compliance_violations(report_id);
CREATE INDEX idx_expense_compliance_violations_rule ON fin_expense_compliance_violations(policy_rule_id);
CREATE INDEX idx_expense_compliance_violations_severity ON fin_expense_compliance_violations(org_id, severity);
CREATE INDEX idx_expense_compliance_violations_resolution ON fin_expense_compliance_violations(resolution_status) WHERE resolution_status = 'open';

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE fin_expense_policy_rules IS 'Oracle Fusion: Expenses > Policies > Expense Policy Rules - Configurable rules for expense compliance enforcement';
COMMENT ON TABLE fin_expense_compliance_audits IS 'Oracle Fusion: Expenses > Audit > Compliance Audits - Audit results for expense report compliance evaluation';
COMMENT ON TABLE fin_expense_compliance_violations IS 'Oracle Fusion: Expenses > Audit > Violation Details - Individual compliance violations from policy evaluation';

COMMENT ON COLUMN fin_expense_policy_rules.rule_type IS 'Type of policy rule: amount_limit, daily_limit, category_limit, receipt_required, time_restriction, duplicate_check, approval_required, per_diem_override';
COMMENT ON COLUMN fin_expense_policy_rules.severity IS 'Severity when violated: warning (informational), violation (requires resolution), block (prevents submission)';
COMMENT ON COLUMN fin_expense_policy_rules.evaluation_scope IS 'How the rule is evaluated: per_line (single line), per_day (daily aggregate), per_report (report total), per_trip (trip total)';
COMMENT ON COLUMN fin_expense_compliance_audits.compliance_score IS 'Compliance score from 0-100, based on passed evaluations minus penalties for blocks and violations';
COMMENT ON COLUMN fin_expense_compliance_audits.risk_level IS 'Risk classification: low (>=80), medium (>=60), high (>=40), critical (<40)';
