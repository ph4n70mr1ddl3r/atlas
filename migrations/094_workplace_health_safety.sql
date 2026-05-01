-- 094: Workplace Health & Safety (EHS)
-- Oracle Fusion Cloud: Environment, Health, and Safety
-- Safety incidents, hazards, inspections, corrective actions (CAPA)

-- ========================================================================
-- Safety Incidents
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.safety_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    incident_number VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    incident_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'reported',
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    incident_date DATE NOT NULL,
    incident_time VARCHAR(10),
    location TEXT,
    facility_id UUID,
    department_id UUID,
    reported_by_id UUID,
    reported_by_name VARCHAR(200),
    assigned_to_id UUID,
    assigned_to_name VARCHAR(200),
    root_cause TEXT,
    immediate_action TEXT,
    osha_recordable BOOLEAN NOT NULL DEFAULT false,
    osha_classification VARCHAR(50),
    days_away_from_work INT NOT NULL DEFAULT 0,
    days_restricted INT NOT NULL DEFAULT 0,
    body_part VARCHAR(100),
    injury_source VARCHAR(200),
    event_type VARCHAR(100),
    environment_factor VARCHAR(200),
    involved_parties JSONB NOT NULL DEFAULT '[]'::jsonb,
    witness_statements JSONB NOT NULL DEFAULT '[]'::jsonb,
    attachments JSONB NOT NULL DEFAULT '[]'::jsonb,
    resolution_date DATE,
    closed_date DATE,
    closed_by UUID,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, incident_number)
);

CREATE INDEX IF NOT EXISTS idx_safety_incidents_org ON _atlas.safety_incidents(organization_id);
CREATE INDEX IF NOT EXISTS idx_safety_incidents_status ON _atlas.safety_incidents(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_safety_incidents_severity ON _atlas.safety_incidents(organization_id, severity);
CREATE INDEX IF NOT EXISTS idx_safety_incidents_type ON _atlas.safety_incidents(organization_id, incident_type);
CREATE INDEX IF NOT EXISTS idx_safety_incidents_date ON _atlas.safety_incidents(organization_id, incident_date);
CREATE INDEX IF NOT EXISTS idx_safety_incidents_facility ON _atlas.safety_incidents(organization_id, facility_id) WHERE facility_id IS NOT NULL;

-- ========================================================================
-- Safety Hazards
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.safety_hazards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    hazard_code VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    hazard_category VARCHAR(50) NOT NULL,
    risk_level VARCHAR(20) NOT NULL,
    likelihood VARCHAR(20) NOT NULL,
    consequence VARCHAR(20) NOT NULL,
    risk_score INT NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'identified',
    location TEXT,
    facility_id UUID,
    department_id UUID,
    identified_by_id UUID,
    identified_by_name VARCHAR(200),
    identified_date DATE NOT NULL,
    mitigation_measures JSONB NOT NULL DEFAULT '[]'::jsonb,
    residual_risk_level VARCHAR(20),
    residual_risk_score INT,
    review_date DATE,
    owner_id UUID,
    owner_name VARCHAR(200),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, hazard_code)
);

CREATE INDEX IF NOT EXISTS idx_safety_hazards_org ON _atlas.safety_hazards(organization_id);
CREATE INDEX IF NOT EXISTS idx_safety_hazards_status ON _atlas.safety_hazards(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_safety_hazards_risk ON _atlas.safety_hazards(organization_id, risk_level);
CREATE INDEX IF NOT EXISTS idx_safety_hazards_category ON _atlas.safety_hazards(organization_id, hazard_category);
CREATE INDEX IF NOT EXISTS idx_safety_hazards_score ON _atlas.safety_hazards(organization_id, risk_score DESC);

-- ========================================================================
-- Safety Inspections
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.safety_inspections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    inspection_number VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    inspection_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'scheduled',
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    scheduled_date DATE NOT NULL,
    completed_date DATE,
    location TEXT,
    facility_id UUID,
    department_id UUID,
    inspector_id UUID,
    inspector_name VARCHAR(200),
    findings_summary TEXT,
    total_findings INT NOT NULL DEFAULT 0,
    critical_findings INT NOT NULL DEFAULT 0,
    non_conformities INT NOT NULL DEFAULT 0,
    observations INT NOT NULL DEFAULT 0,
    score DOUBLE PRECISION,
    max_score DOUBLE PRECISION,
    score_pct DOUBLE PRECISION,
    findings JSONB NOT NULL DEFAULT '[]'::jsonb,
    attachments JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, inspection_number)
);

CREATE INDEX IF NOT EXISTS idx_safety_inspections_org ON _atlas.safety_inspections(organization_id);
CREATE INDEX IF NOT EXISTS idx_safety_inspections_status ON _atlas.safety_inspections(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_safety_inspections_type ON _atlas.safety_inspections(organization_id, inspection_type);
CREATE INDEX IF NOT EXISTS idx_safety_inspections_date ON _atlas.safety_inspections(organization_id, scheduled_date);

-- ========================================================================
-- Corrective and Preventive Actions (CAPA)
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.corrective_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    action_number VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    action_type VARCHAR(30) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    source_type VARCHAR(50),
    source_id UUID,
    source_number VARCHAR(50),
    root_cause TEXT,
    corrective_action_plan TEXT,
    preventive_action_plan TEXT,
    assigned_to_id UUID,
    assigned_to_name VARCHAR(200),
    due_date DATE,
    completed_date DATE,
    verified_by UUID,
    verified_date DATE,
    effectiveness VARCHAR(30),
    facility_id UUID,
    department_id UUID,
    estimated_cost DOUBLE PRECISION,
    actual_cost DOUBLE PRECISION,
    currency_code VARCHAR(3),
    notes TEXT,
    attachments JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, action_number)
);

CREATE INDEX IF NOT EXISTS idx_corrective_actions_org ON _atlas.corrective_actions(organization_id);
CREATE INDEX IF NOT EXISTS idx_corrective_actions_status ON _atlas.corrective_actions(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_corrective_actions_type ON _atlas.corrective_actions(organization_id, action_type);
CREATE INDEX IF NOT EXISTS idx_corrective_actions_due ON _atlas.corrective_actions(organization_id, due_date) WHERE due_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_corrective_actions_source ON _atlas.corrective_actions(source_type, source_id) WHERE source_type IS NOT NULL;
