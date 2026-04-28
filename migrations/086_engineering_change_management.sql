-- Engineering Change Management (ECM)
-- Oracle Fusion Cloud: Product Development > Engineering Change Management
-- Provides: Engineering Change Requests (ECR), Engineering Change Orders (ECO),
--   Engineering Change Notices (ECN), change lines, affected items,
--   revision management, impact analysis, approval workflows.
--
-- Inspired by Oracle Fusion Cloud Product Development / PLM module.

-- ============================================================================
-- Change Types: ECR, ECO, ECN definitions
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.engineering_change_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    type_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Type category: ecr (request), eco (order), ecn (notice)
    category VARCHAR(30) NOT NULL DEFAULT 'eco',
    -- Approval required before implementation
    approval_required BOOLEAN DEFAULT true,
    -- Default priority for this type
    default_priority VARCHAR(20) DEFAULT 'medium',
    -- Auto-numbering prefix (e.g., ECR-, ECO-, ECN-)
    number_prefix VARCHAR(20) NOT NULL,
    -- Status list as JSON array
    statuses JSONB DEFAULT '["draft","submitted","approved","in_review","rejected","implemented","closed","cancelled"]'::jsonb,
    -- Template for change description
    description_template TEXT,
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, type_code)
);

CREATE INDEX IF NOT EXISTS idx_eng_change_types_org ON _atlas.engineering_change_types(organization_id);

-- ============================================================================
-- Engineering Change Orders / Requests / Notices
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.engineering_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    change_number VARCHAR(100) NOT NULL,
    -- Type reference
    change_type_id UUID REFERENCES _atlas.engineering_change_types(id),
    -- Classification
    category VARCHAR(30) NOT NULL DEFAULT 'eco',  -- ecr, eco, ecn
    title VARCHAR(500) NOT NULL,
    description TEXT,
    -- Reason / justification
    change_reason VARCHAR(200),
    change_reason_description TEXT,
    -- Priority: low, medium, high, critical
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    -- Status lifecycle: draft -> submitted -> in_review -> approved/rejected -> implemented -> closed
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- Revision tracking
    revision VARCHAR(20) DEFAULT 'A',
    -- Routing
    assigned_to UUID,
    assigned_to_name VARCHAR(200),
    -- Dates
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    implemented_at TIMESTAMPTZ,
    target_date DATE,
    effective_date DATE,
    -- Resolution
    resolution_code VARCHAR(50),  -- implemented, partially_implemented, withdrawn, superseded
    resolution_notes TEXT,
    -- Parent change (for linked/related changes)
    parent_change_id UUID REFERENCES _atlas.engineering_changes(id),
    -- Superseding
    superseded_by_id UUID REFERENCES _atlas.engineering_changes(id),
    -- Impact assessment
    impact_analysis JSONB DEFAULT '{}'::jsonb,
    -- Estimated vs actual
    estimated_cost NUMERIC(18, 2),
    actual_cost NUMERIC(18, 2),
    currency_code VARCHAR(10) DEFAULT 'USD',
    estimated_hours NUMERIC(10, 2),
    actual_hours NUMERIC(10, 2),
    -- Compliance
    regulatory_impact VARCHAR(200),
    safety_impact VARCHAR(200),
    validation_required BOOLEAN DEFAULT false,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, change_number)
);

CREATE INDEX IF NOT EXISTS idx_eng_changes_org ON _atlas.engineering_changes(organization_id);
CREATE INDEX IF NOT EXISTS idx_eng_changes_status ON _atlas.engineering_changes(status);
CREATE INDEX IF NOT EXISTS idx_eng_changes_category ON _atlas.engineering_changes(category);
CREATE INDEX IF NOT EXISTS idx_eng_changes_type ON _atlas.engineering_changes(change_type_id);
CREATE INDEX IF NOT EXISTS idx_eng_changes_assigned ON _atlas.engineering_changes(assigned_to);

-- ============================================================================
-- Change Lines: individual changes within an engineering change
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.engineering_change_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    change_id UUID NOT NULL REFERENCES _atlas.engineering_changes(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    -- Affected item
    item_id UUID,
    item_number VARCHAR(100),
    item_name VARCHAR(500),
    -- Change details
    change_category VARCHAR(50) NOT NULL,  -- item_update, bom_add, bom_remove, bom_change, revision_change, specification_change
    field_name VARCHAR(200),
    -- Old and new values
    old_value TEXT,
    new_value TEXT,
    old_revision VARCHAR(20),
    new_revision VARCHAR(20),
    -- BOM-specific
    component_item_id UUID,
    component_item_number VARCHAR(100),
    bom_quantity_old NUMERIC(18, 4),
    bom_quantity_new NUMERIC(18, 4),
    -- Effectivity
    effectivity_date DATE,
    effectivity_end_date DATE,
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- pending, in_progress, completed, failed, skipped
    completion_notes TEXT,
    -- Ordering
    sequence_number INT DEFAULT 0,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(change_id, line_number)
);

CREATE INDEX IF NOT EXISTS idx_eng_change_lines_change ON _atlas.engineering_change_lines(change_id);
CREATE INDEX IF NOT EXISTS idx_eng_change_lines_item ON _atlas.engineering_change_lines(item_id);

-- ============================================================================
-- Affected Items: items impacted by the change (may have no direct line edits)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.engineering_change_affected_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    change_id UUID NOT NULL REFERENCES _atlas.engineering_changes(id) ON DELETE CASCADE,
    item_id UUID NOT NULL,
    item_number VARCHAR(100) NOT NULL,
    item_name VARCHAR(500),
    -- Impact type
    impact_type VARCHAR(50) NOT NULL DEFAULT 'direct',  -- direct, indirect, dependent
    impact_description TEXT,
    -- Current revision before change
    current_revision VARCHAR(20),
    -- New revision after change
    new_revision VARCHAR(20),
    -- Disposition
    disposition VARCHAR(50),  -- use_existing, scrap, rework, return_to_supplier
    old_item_status VARCHAR(50),
    new_item_status VARCHAR(50),
    -- Phased rollout
    phase_in_date DATE,
    phase_out_date DATE,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(change_id, item_id)
);

CREATE INDEX IF NOT EXISTS idx_eng_affected_items_change ON _atlas.engineering_change_affected_items(change_id);
CREATE INDEX IF NOT EXISTS idx_eng_affected_items_item ON _atlas.engineering_change_affected_items(item_id);

-- ============================================================================
-- Change Approvals: approval workflow records
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.engineering_change_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    change_id UUID NOT NULL REFERENCES _atlas.engineering_changes(id) ON DELETE CASCADE,
    -- Approval level (multi-level approval chain)
    approval_level INT NOT NULL DEFAULT 1,
    -- Approver
    approver_id UUID,
    approver_name VARCHAR(200),
    approver_role VARCHAR(100),
    -- Status: pending, approved, rejected, returned, delegated
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- Action
    action_date TIMESTAMPTZ,
    comments TEXT,
    -- Delegation
    delegated_from_id UUID,
    -- Conditions / instructions
    approval_conditions TEXT,
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_eng_approvals_change ON _atlas.engineering_change_approvals(change_id);
CREATE INDEX IF NOT EXISTS idx_eng_approvals_approver ON _atlas.engineering_change_approvals(approver_id);
CREATE INDEX IF NOT EXISTS idx_eng_approvals_status ON _atlas.engineering_change_approvals(status);

-- ============================================================================
-- Change Attachments
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.engineering_change_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    change_id UUID NOT NULL REFERENCES _atlas.engineering_changes(id) ON DELETE CASCADE,
    file_name VARCHAR(500) NOT NULL,
    file_type VARCHAR(100),
    file_size BIGINT,
    storage_path TEXT,
    description TEXT,
    -- Category: supporting_doc, cad_drawing, test_result, specification, photo, other
    attachment_category VARCHAR(50) DEFAULT 'supporting_doc',
    version INT DEFAULT 1,
    uploaded_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_eng_attachments_change ON _atlas.engineering_change_attachments(change_id);

-- ============================================================================
-- ECM Dashboard View (materialized summary)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.ecm_dashboard (
    organization_id UUID PRIMARY KEY,
    total_changes INT DEFAULT 0,
    open_changes INT DEFAULT 0,
    pending_approval INT DEFAULT 0,
    approved_changes INT DEFAULT 0,
    implemented_changes INT DEFAULT 0,
    rejected_changes INT DEFAULT 0,
    -- By category
    ecr_count INT DEFAULT 0,
    eco_count INT DEFAULT 0,
    ecn_count INT DEFAULT 0,
    -- By priority
    critical_open INT DEFAULT 0,
    high_open INT DEFAULT 0,
    medium_open INT DEFAULT 0,
    low_open INT DEFAULT 0,
    -- Averages
    avg_days_to_implement NUMERIC(10, 2) DEFAULT 0,
    avg_days_to_approve NUMERIC(10, 2) DEFAULT 0,
    -- Items affected
    total_items_affected INT DEFAULT 0,
    -- Cost tracking
    total_estimated_cost NUMERIC(18, 2) DEFAULT 0,
    total_actual_cost NUMERIC(18, 2) DEFAULT 0,
    -- Breakdown by change reason
    changes_by_reason JSONB DEFAULT '{}'::jsonb,
    changes_by_status JSONB DEFAULT '{}'::jsonb,
    changes_trend JSONB DEFAULT '[]'::jsonb,
    updated_at TIMESTAMPTZ DEFAULT now()
);
