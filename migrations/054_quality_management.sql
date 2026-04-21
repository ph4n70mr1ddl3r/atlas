-- Quality Management Module
-- Oracle Fusion Cloud: Quality Management > Inspections > Non-Conformance > CAPA
-- Provides inspection plans, quality inspections, non-conformance reports,
-- corrective & preventive actions, and quality holds.

-- ========================================================================
-- Inspection Plans
-- ========================================================================

CREATE TABLE IF NOT EXISTS _atlas.quality_inspection_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    plan_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    plan_type VARCHAR(30) NOT NULL DEFAULT 'receiving',
    item_id UUID,
    item_code VARCHAR(100),
    supplier_id UUID,
    supplier_name VARCHAR(200),
    inspection_trigger VARCHAR(50) NOT NULL DEFAULT 'every_receipt',
    sampling_method VARCHAR(30) NOT NULL DEFAULT 'full',
    sample_size_percent NUMERIC(5,2),
    accept_number INT,
    reject_number INT,
    frequency VARCHAR(30) NOT NULL DEFAULT 'per_lot',
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    total_criteria INT NOT NULL DEFAULT 0,
    total_inspections INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, plan_code)
);

CREATE INDEX IF NOT EXISTS idx_qip_org ON _atlas.quality_inspection_plans(organization_id);
CREATE INDEX IF NOT EXISTS idx_qip_item ON _atlas.quality_inspection_plans(item_id) WHERE item_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_qip_supplier ON _atlas.quality_inspection_plans(supplier_id) WHERE supplier_id IS NOT NULL;

-- ========================================================================
-- Plan Criteria
-- ========================================================================

CREATE TABLE IF NOT EXISTS _atlas.quality_plan_criteria (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    plan_id UUID NOT NULL REFERENCES _atlas.quality_inspection_plans(id) ON DELETE CASCADE,
    criterion_number INT NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    characteristic VARCHAR(200) NOT NULL,
    measurement_type VARCHAR(30) NOT NULL DEFAULT 'pass_fail',
    target_value NUMERIC(18,6),
    lower_spec_limit NUMERIC(18,6),
    upper_spec_limit NUMERIC(18,6),
    unit_of_measure VARCHAR(30),
    is_mandatory BOOLEAN NOT NULL DEFAULT true,
    weight NUMERIC(5,2) NOT NULL DEFAULT 1,
    criticality VARCHAR(20) NOT NULL DEFAULT 'major',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(plan_id, criterion_number)
);

CREATE INDEX IF NOT EXISTS idx_qpc_plan ON _atlas.quality_plan_criteria(plan_id);

-- ========================================================================
-- Inspections
-- ========================================================================

CREATE TABLE IF NOT EXISTS _atlas.quality_inspections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    inspection_number VARCHAR(50) NOT NULL,
    plan_id UUID NOT NULL REFERENCES _atlas.quality_inspection_plans(id),
    source_type VARCHAR(50) NOT NULL DEFAULT 'receiving',
    source_id UUID,
    source_number VARCHAR(100),
    item_id UUID,
    item_code VARCHAR(100),
    item_description TEXT,
    lot_number VARCHAR(100),
    quantity_inspected NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_accepted NUMERIC(18,4) NOT NULL DEFAULT 0,
    quantity_rejected NUMERIC(18,4) NOT NULL DEFAULT 0,
    unit_of_measure VARCHAR(30),
    status VARCHAR(20) NOT NULL DEFAULT 'planned',
    verdict VARCHAR(20) NOT NULL DEFAULT 'pending',
    overall_score NUMERIC(5,2) NOT NULL DEFAULT 0,
    notes TEXT,
    inspector_id UUID,
    inspector_name VARCHAR(200),
    inspection_date DATE NOT NULL,
    completed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, inspection_number)
);

CREATE INDEX IF NOT EXISTS idx_qi_org ON _atlas.quality_inspections(organization_id);
CREATE INDEX IF NOT EXISTS idx_qi_plan ON _atlas.quality_inspections(plan_id);
CREATE INDEX IF NOT EXISTS idx_qi_status ON _atlas.quality_inspections(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_qi_item ON _atlas.quality_inspections(item_id) WHERE item_id IS NOT NULL;

-- ========================================================================
-- Inspection Results
-- ========================================================================

CREATE TABLE IF NOT EXISTS _atlas.quality_inspection_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    inspection_id UUID NOT NULL REFERENCES _atlas.quality_inspections(id) ON DELETE CASCADE,
    criterion_id UUID REFERENCES _atlas.quality_plan_criteria(id),
    criterion_name VARCHAR(200) NOT NULL,
    characteristic VARCHAR(200) NOT NULL,
    measurement_type VARCHAR(30) NOT NULL DEFAULT 'pass_fail',
    observed_value NUMERIC(18,6),
    target_value NUMERIC(18,6),
    lower_spec_limit NUMERIC(18,6),
    upper_spec_limit NUMERIC(18,6),
    unit_of_measure VARCHAR(30),
    result_status VARCHAR(20) NOT NULL DEFAULT 'not_evaluated',
    deviation NUMERIC(18,6),
    notes TEXT,
    evaluated_by UUID,
    evaluated_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_qir_inspection ON _atlas.quality_inspection_results(inspection_id);

-- ========================================================================
-- Non-Conformance Reports
-- ========================================================================

CREATE TABLE IF NOT EXISTS _atlas.quality_non_conformance_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ncr_number VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    ncr_type VARCHAR(50) NOT NULL DEFAULT 'defect',
    severity VARCHAR(20) NOT NULL DEFAULT 'major',
    origin VARCHAR(50) NOT NULL DEFAULT 'inspection',
    source_type VARCHAR(50),
    source_id UUID,
    source_number VARCHAR(100),
    item_id UUID,
    item_code VARCHAR(100),
    supplier_id UUID,
    supplier_name VARCHAR(200),
    detected_date DATE NOT NULL,
    detected_by VARCHAR(200),
    responsible_party VARCHAR(200),
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    resolution_description TEXT,
    resolution_type VARCHAR(50),
    resolved_by VARCHAR(200),
    resolved_at TIMESTAMPTZ,
    total_corrective_actions INT NOT NULL DEFAULT 0,
    open_corrective_actions INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, ncr_number)
);

CREATE INDEX IF NOT EXISTS idx_ncr_org ON _atlas.quality_non_conformance_reports(organization_id);
CREATE INDEX IF NOT EXISTS idx_ncr_status ON _atlas.quality_non_conformance_reports(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_ncr_severity ON _atlas.quality_non_conformance_reports(severity);
CREATE INDEX IF NOT EXISTS idx_ncr_item ON _atlas.quality_non_conformance_reports(item_id) WHERE item_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ncr_supplier ON _atlas.quality_non_conformance_reports(supplier_id) WHERE supplier_id IS NOT NULL;

-- ========================================================================
-- Corrective & Preventive Actions
-- ========================================================================

CREATE TABLE IF NOT EXISTS _atlas.quality_corrective_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    ncr_id UUID NOT NULL REFERENCES _atlas.quality_non_conformance_reports(id) ON DELETE CASCADE,
    action_number VARCHAR(50) NOT NULL,
    action_type VARCHAR(20) NOT NULL DEFAULT 'corrective',
    title VARCHAR(500) NOT NULL,
    description TEXT,
    root_cause TEXT,
    corrective_action_desc TEXT,
    preventive_action_desc TEXT,
    assigned_to VARCHAR(200),
    due_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    completed_at TIMESTAMPTZ,
    effectiveness_rating INT CHECK (effectiveness_rating IS NULL OR (effectiveness_rating >= 1 AND effectiveness_rating <= 5)),
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(ncr_id, action_number)
);

CREATE INDEX IF NOT EXISTS idx_capa_ncr ON _atlas.quality_corrective_actions(ncr_id);
CREATE INDEX IF NOT EXISTS idx_capa_status ON _atlas.quality_corrective_actions(organization_id, status);

-- ========================================================================
-- Quality Holds
-- ========================================================================

CREATE TABLE IF NOT EXISTS _atlas.quality_holds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    hold_number VARCHAR(50) NOT NULL,
    reason TEXT NOT NULL,
    description TEXT,
    item_id UUID,
    item_code VARCHAR(100),
    lot_number VARCHAR(100),
    supplier_id UUID,
    supplier_name VARCHAR(200),
    source_type VARCHAR(50),
    source_id UUID,
    source_number VARCHAR(100),
    hold_type VARCHAR(30) NOT NULL DEFAULT 'item',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    released_by UUID,
    released_at TIMESTAMPTZ,
    release_notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, hold_number)
);

CREATE INDEX IF NOT EXISTS idx_qh_org ON _atlas.quality_holds(organization_id);
CREATE INDEX IF NOT EXISTS idx_qh_status ON _atlas.quality_holds(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_qh_item ON _atlas.quality_holds(item_id) WHERE item_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_qh_supplier ON _atlas.quality_holds(supplier_id) WHERE supplier_id IS NOT NULL;
