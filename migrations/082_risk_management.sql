-- Risk Management & Internal Controls
-- Oracle Fusion Cloud: Governance, Risk, and Compliance (GRC) / Advanced Controls
-- Provides: Risk register, control registry, risk-control mappings,
--   control testing & certification, issue/remediation tracking,
--   risk scoring & heat map data.

-- ============================================================================
-- Risk Categories: group risks by business area
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.risk_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    parent_category_id UUID REFERENCES _atlas.risk_categories(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INT DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_risk_categories_org ON _atlas.risk_categories(organization_id);

-- ============================================================================
-- Risk Register: enterprise risk identification and assessment
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.risk_register (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    risk_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    category_id UUID REFERENCES _atlas.risk_categories(id),
    -- Risk source: operational, financial, compliance, strategic, reputational, technology
    risk_source VARCHAR(50) NOT NULL DEFAULT 'operational',
    -- Likelihood: 1 (rare) to 5 (almost certain)
    likelihood INT NOT NULL DEFAULT 3 CHECK (likelihood BETWEEN 1 AND 5),
    -- Impact: 1 (negligible) to 5 (catastrophic)
    impact INT NOT NULL DEFAULT 3 CHECK (impact BETWEEN 1 AND 5),
    -- Computed risk score: likelihood * impact
    risk_score INT GENERATED ALWAYS AS (likelihood * impact) STORED,
    -- Risk level derived from score
    risk_level VARCHAR(20) NOT NULL DEFAULT 'medium',
    -- Status: identified, assessed, mitigated, accepted, closed
    status VARCHAR(50) NOT NULL DEFAULT 'identified',
    -- Owner
    owner_id UUID,
    owner_name VARCHAR(300),
    -- Applicable business units / processes (JSON array)
    business_units JSONB DEFAULT '[]'::jsonb,
    -- Risk response strategy: avoid, mitigate, transfer, accept
    response_strategy VARCHAR(50),
    -- Residual risk after controls
    residual_likelihood INT CHECK (residual_likelihood BETWEEN 1 AND 5),
    residual_impact INT CHECK (residual_impact BETWEEN 1 AND 5),
    -- Dates
    identified_date DATE NOT NULL DEFAULT CURRENT_DATE,
    last_assessed_date DATE,
    next_review_date DATE,
    closed_date DATE,
    -- References
    related_entity_type VARCHAR(100),
    related_entity_id UUID,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, risk_number)
);

CREATE INDEX IF NOT EXISTS idx_risk_register_org ON _atlas.risk_register(organization_id);
CREATE INDEX IF NOT EXISTS idx_risk_register_status ON _atlas.risk_register(status);
CREATE INDEX IF NOT EXISTS idx_risk_register_category ON _atlas.risk_register(category_id);
CREATE INDEX IF NOT EXISTS idx_risk_register_owner ON _atlas.risk_register(owner_id);
CREATE INDEX IF NOT EXISTS idx_risk_register_risk_level ON _atlas.risk_register(risk_level);
CREATE INDEX IF NOT EXISTS idx_risk_register_score ON _atlas.risk_register(risk_score);

-- ============================================================================
-- Control Registry: internal controls linked to risks
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.control_registry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    control_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    -- Control type: preventive, detective, corrective
    control_type VARCHAR(50) NOT NULL DEFAULT 'preventive',
    -- Control nature: automated, manual, it_dependent
    control_nature VARCHAR(50) NOT NULL DEFAULT 'manual',
    -- Control frequency: daily, weekly, monthly, quarterly, annually, per_transaction
    frequency VARCHAR(50) NOT NULL DEFAULT 'monthly',
    -- Control objective
    objective TEXT,
    -- Test procedures description
    test_procedures TEXT,
    -- Control owner
    owner_id UUID,
    owner_name VARCHAR(300),
    -- Key control flag
    is_key_control BOOLEAN NOT NULL DEFAULT false,
    -- Effectiveness: effective, ineffective, not_tested
    effectiveness VARCHAR(50) NOT NULL DEFAULT 'not_tested',
    -- Status: draft, active, inactive, deprecated
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- Applicable business processes
    business_processes JSONB DEFAULT '[]'::jsonb,
    -- Related regulatory framework (SOX, GDPR, etc.)
    regulatory_frameworks JSONB DEFAULT '[]'::jsonb,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, control_number)
);

CREATE INDEX IF NOT EXISTS idx_control_registry_org ON _atlas.control_registry(organization_id);
CREATE INDEX IF NOT EXISTS idx_control_registry_status ON _atlas.control_registry(status);
CREATE INDEX IF NOT EXISTS idx_control_registry_type ON _atlas.control_registry(control_type);
CREATE INDEX IF NOT EXISTS idx_control_registry_owner ON _atlas.control_registry(owner_id);

-- ============================================================================
-- Risk-Control Mappings: which controls mitigate which risks
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.risk_control_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    risk_id UUID NOT NULL REFERENCES _atlas.risk_register(id),
    control_id UUID NOT NULL REFERENCES _atlas.control_registry(id),
    -- How much this control reduces the risk
    mitigation_effectiveness VARCHAR(20) NOT NULL DEFAULT 'partial',
    -- Mapping status: active, ineffective, superseded
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    description TEXT,
    mapped_by UUID,
    mapped_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(risk_id, control_id)
);

CREATE INDEX IF NOT EXISTS idx_rcm_risk ON _atlas.risk_control_mappings(risk_id);
CREATE INDEX IF NOT EXISTS idx_rcm_control ON _atlas.risk_control_mappings(control_id);
CREATE INDEX IF NOT EXISTS idx_rcm_org ON _atlas.risk_control_mappings(organization_id);

-- ============================================================================
-- Control Tests: periodic testing and certification of controls
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.control_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    control_id UUID NOT NULL REFERENCES _atlas.control_registry(id),
    test_number VARCHAR(100) NOT NULL,
    -- Test plan
    test_plan TEXT NOT NULL,
    test_period_start DATE NOT NULL,
    test_period_end DATE NOT NULL,
    -- Tester
    tester_id UUID,
    tester_name VARCHAR(300),
    -- Test result: pass, fail, not_tested, in_progress
    result VARCHAR(50) NOT NULL DEFAULT 'not_tested',
    -- Findings
    findings TEXT,
    -- Deficiency severity if failed: minor, significant, material
    deficiency_severity VARCHAR(50),
    -- Evidence
    evidence_document_ids JSONB DEFAULT '[]'::jsonb,
    -- Sample info
    sample_size INT,
    sample_exceptions INT,
    -- Dates
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    -- Reviewer (who reviews the test)
    reviewer_id UUID,
    reviewer_name VARCHAR(300),
    reviewed_at TIMESTAMPTZ,
    review_status VARCHAR(50) DEFAULT 'pending',
    -- Status: planned, in_progress, completed, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'planned',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, test_number)
);

CREATE INDEX IF NOT EXISTS idx_control_tests_control ON _atlas.control_tests(control_id);
CREATE INDEX IF NOT EXISTS idx_control_tests_status ON _atlas.control_tests(status);
CREATE INDEX IF NOT EXISTS idx_control_tests_org ON _atlas.control_tests(organization_id);
CREATE INDEX IF NOT EXISTS idx_control_tests_result ON _atlas.control_tests(result);

-- ============================================================================
-- Issues & Remediations: findings from control tests and risk events
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.risk_issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    issue_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    -- Source: control_test, risk_event, audit_finding, regulatory, self_identified
    source VARCHAR(50) NOT NULL DEFAULT 'self_identified',
    -- Related entities
    risk_id UUID REFERENCES _atlas.risk_register(id),
    control_id UUID REFERENCES _atlas.control_registry(id),
    control_test_id UUID REFERENCES _atlas.control_tests(id),
    -- Severity: low, medium, high, critical
    severity VARCHAR(50) NOT NULL DEFAULT 'medium',
    -- Priority: low, normal, high, urgent
    priority VARCHAR(50) NOT NULL DEFAULT 'normal',
    -- Status: open, investigating, remediation_in_progress, resolved, closed, accepted
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    -- Owner
    owner_id UUID,
    owner_name VARCHAR(300),
    -- Remediation plan
    remediation_plan TEXT,
    remediation_due_date DATE,
    remediation_completed_date DATE,
    -- Root cause
    root_cause TEXT,
    -- Corrective actions taken
    corrective_actions TEXT,
    -- Dates
    identified_date DATE NOT NULL DEFAULT CURRENT_DATE,
    resolved_date DATE,
    closed_date DATE,
    -- Related regulatory reference
    regulatory_reference VARCHAR(300),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, issue_number)
);

CREATE INDEX IF NOT EXISTS idx_risk_issues_org ON _atlas.risk_issues(organization_id);
CREATE INDEX IF NOT EXISTS idx_risk_issues_status ON _atlas.risk_issues(status);
CREATE INDEX IF NOT EXISTS idx_risk_issues_severity ON _atlas.risk_issues(severity);
CREATE INDEX IF NOT EXISTS idx_risk_issues_risk ON _atlas.risk_issues(risk_id);
CREATE INDEX IF NOT EXISTS idx_risk_issues_control ON _atlas.risk_issues(control_id);
CREATE INDEX IF NOT EXISTS idx_risk_issues_owner ON _atlas.risk_issues(owner_id);

-- ============================================================================
-- Risk Dashboard Summary (materialized view convenience table)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.risk_dashboard (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    total_risks INT DEFAULT 0,
    open_risks INT DEFAULT 0,
    mitigated_risks INT DEFAULT 0,
    accepted_risks INT DEFAULT 0,
    critical_risks INT DEFAULT 0,
    high_risks INT DEFAULT 0,
    medium_risks INT DEFAULT 0,
    low_risks INT DEFAULT 0,
    total_controls INT DEFAULT 0,
    active_controls INT DEFAULT 0,
    effective_controls INT DEFAULT 0,
    ineffective_controls INT DEFAULT 0,
    not_tested_controls INT DEFAULT 0,
    total_tests INT DEFAULT 0,
    passed_tests INT DEFAULT 0,
    failed_tests INT DEFAULT 0,
    open_issues INT DEFAULT 0,
    critical_issues INT DEFAULT 0,
    overdue_remediations INT DEFAULT 0,
    risks_by_source JSONB DEFAULT '{}'::jsonb,
    risks_by_level JSONB DEFAULT '{}'::jsonb,
    control_effectiveness_summary JSONB DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id)
);
