-- Learning Management (Oracle Fusion HCM > Learning)
-- Provides: Learning items, categories, enrollments, learning paths,
--   path items, mandatory assignments, and completion analytics.

-- ============================================================================
-- Learning Categories: hierarchical catalog organization
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.learning_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    parent_category_id UUID REFERENCES _atlas.learning_categories(id),
    display_order INT DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'active',           -- active, inactive
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_learning_categories_org ON _atlas.learning_categories(organization_id);
CREATE INDEX IF NOT EXISTS idx_learning_categories_parent ON _atlas.learning_categories(parent_category_id);

-- ============================================================================
-- Learning Items: courses, certifications, specializations, etc.
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.learning_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    item_type VARCHAR(50) NOT NULL DEFAULT 'course',        -- course, certification, specialization, video, assessment, blended
    format VARCHAR(50) NOT NULL DEFAULT 'self_paced',       -- online, classroom, virtual_classroom, self_paced, blended
    category VARCHAR(100),
    provider VARCHAR(300),
    duration_hours NUMERIC(8,2),
    currency_code VARCHAR(3),
    cost NUMERIC(18,4),
    credits NUMERIC(8,2),
    credit_type VARCHAR(50),                                 -- ceu, cpe, pdu, college_credit, custom
    validity_months INT,
    recertification_required BOOLEAN DEFAULT false,
    max_enrollments INT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',            -- draft, active, inactive, archived
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_learning_items_org ON _atlas.learning_items(organization_id);
CREATE INDEX IF NOT EXISTS idx_learning_items_type ON _atlas.learning_items(item_type);
CREATE INDEX IF NOT EXISTS idx_learning_items_status ON _atlas.learning_items(status);
CREATE INDEX IF NOT EXISTS idx_learning_items_category ON _atlas.learning_items(category);

-- ============================================================================
-- Learning Enrollments: person enrollment and progress tracking
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.learning_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    learning_item_id UUID NOT NULL REFERENCES _atlas.learning_items(id),
    person_id UUID NOT NULL,
    person_name VARCHAR(300),
    enrollment_type VARCHAR(50) NOT NULL DEFAULT 'self',    -- self, manager, mandatory, auto_assigned
    enrolled_by UUID,
    status VARCHAR(50) NOT NULL DEFAULT 'enrolled',         -- enrolled, in_progress, completed, failed, withdrawn, expired
    progress_pct NUMERIC(5,2) DEFAULT 0.00,                 -- 0-100
    score NUMERIC(5,2),
    enrollment_date DATE,
    completion_date DATE,
    due_date DATE,
    certification_expiry DATE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(learning_item_id, person_id)
);

CREATE INDEX IF NOT EXISTS idx_learning_enrollments_org ON _atlas.learning_enrollments(organization_id);
CREATE INDEX IF NOT EXISTS idx_learning_enrollments_item ON _atlas.learning_enrollments(learning_item_id);
CREATE INDEX IF NOT EXISTS idx_learning_enrollments_person ON _atlas.learning_enrollments(person_id);
CREATE INDEX IF NOT EXISTS idx_learning_enrollments_status ON _atlas.learning_enrollments(status);

-- ============================================================================
-- Learning Paths: curricula / sequences of learning items
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.learning_paths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    path_type VARCHAR(50) NOT NULL DEFAULT 'sequential',    -- sequential, elective, milestone, tiered
    target_role VARCHAR(200),
    target_job_id UUID,
    estimated_duration_hours NUMERIC(8,2),
    total_items INT DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',            -- draft, active, inactive, archived
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_learning_paths_org ON _atlas.learning_paths(organization_id);

-- ============================================================================
-- Learning Path Items: individual steps within a learning path
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.learning_path_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    learning_path_id UUID NOT NULL REFERENCES _atlas.learning_paths(id) ON DELETE CASCADE,
    learning_item_id UUID NOT NULL REFERENCES _atlas.learning_items(id),
    sequence_number INT NOT NULL DEFAULT 1,
    is_required BOOLEAN DEFAULT true,
    milestone_name VARCHAR(200),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(learning_path_id, learning_item_id)
);

CREATE INDEX IF NOT EXISTS idx_learning_path_items_path ON _atlas.learning_path_items(learning_path_id);
CREATE INDEX IF NOT EXISTS idx_learning_path_items_item ON _atlas.learning_path_items(learning_item_id);

-- ============================================================================
-- Learning Assignments: mandatory training requirements
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.learning_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    learning_item_id UUID REFERENCES _atlas.learning_items(id),
    learning_path_id UUID REFERENCES _atlas.learning_paths(id),
    title VARCHAR(500) NOT NULL,
    description TEXT,
    assignment_type VARCHAR(50) NOT NULL DEFAULT 'individual', -- individual, organization, department, job, position
    target_id UUID,
    assigned_by UUID,
    priority VARCHAR(50) NOT NULL DEFAULT 'medium',         -- low, medium, high, critical
    due_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'active',           -- active, completed, cancelled
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_learning_assignments_org ON _atlas.learning_assignments(organization_id);
CREATE INDEX IF NOT EXISTS idx_learning_assignments_status ON _atlas.learning_assignments(status);
CREATE INDEX IF NOT EXISTS idx_learning_assignments_item ON _atlas.learning_assignments(learning_item_id);
CREATE INDEX IF NOT EXISTS idx_learning_assignments_path ON _atlas.learning_assignments(learning_path_id);
