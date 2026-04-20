-- Performance Management (Oracle Fusion HCM Performance Review)
-- Implements: Review Cycles, Performance Documents, Goals, Competency Assessments, Feedback
-- Oracle Fusion equivalent: My Client Groups > Performance > Performance Documents

-- Rating models define the rating scale (e.g., 1-5, Meets/Exceeds/Below)
CREATE TABLE IF NOT EXISTS _atlas.performance_rating_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    rating_scale JSONB NOT NULL DEFAULT '[]',
    -- e.g. [{"value":1,"label":"Below Expectations"},{"value":2,"label":"Needs Improvement"},{"value":3,"label":"Meets Expectations"},{"value":4,"label":"Exceeds Expectations"},{"value":5,"label":"Outstanding"}]
    is_active BOOLEAN DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Review cycles define review periods (e.g., "2025 Annual Review", "Q2 2025 Mid-Year")
CREATE TABLE IF NOT EXISTS _atlas.performance_review_cycles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    cycle_type VARCHAR(50) NOT NULL DEFAULT 'annual',
    -- annual, mid_year, quarterly, project_end, probation
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- draft, planning, goal_setting, self_evaluation, manager_evaluation, calibration, completed, cancelled
    rating_model_id UUID REFERENCES _atlas.performance_rating_models(id),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    goal_setting_start DATE,
    goal_setting_end DATE,
    self_evaluation_start DATE,
    self_evaluation_end DATE,
    manager_evaluation_start DATE,
    manager_evaluation_end DATE,
    calibration_date DATE,
    require_goals BOOLEAN DEFAULT true,
    require_competencies BOOLEAN DEFAULT true,
    min_goals INT DEFAULT 3,
    max_goals INT DEFAULT 10,
    goal_weight_total DECIMAL(5,2) DEFAULT 100.00,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Competency definitions (e.g., "Communication", "Leadership", "Technical Skills")
CREATE TABLE IF NOT EXISTS _atlas.performance_competencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    -- e.g., "core", "leadership", "technical", "functional"
    rating_model_id UUID REFERENCES _atlas.performance_rating_models(id),
    behavioral_indicators JSONB DEFAULT '[]',
    -- e.g. [{"level":1,"description":"Demonstrates basic..."}]
    is_active BOOLEAN DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Performance documents (one per employee per cycle)
CREATE TABLE IF NOT EXISTS _atlas.performance_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    review_cycle_id UUID NOT NULL REFERENCES _atlas.performance_review_cycles(id),
    employee_id UUID NOT NULL,
    employee_name VARCHAR(200),
    manager_id UUID,
    manager_name VARCHAR(200),
    document_number VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'not_started',
    -- not_started, goal_setting, self_evaluation, manager_evaluation, calibration, completed, cancelled
    overall_rating DECIMAL(5,2),
    overall_rating_label VARCHAR(100),
    self_overall_rating DECIMAL(5,2),
    self_comments TEXT,
    manager_overall_rating DECIMAL(5,2),
    manager_comments TEXT,
    calibration_rating DECIMAL(5,2),
    calibration_comments TEXT,
    final_rating DECIMAL(5,2),
    final_comments TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(review_cycle_id, employee_id)
);

-- Performance goals (linked to a performance document)
CREATE TABLE IF NOT EXISTS _atlas.performance_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES _atlas.performance_documents(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL,
    goal_name VARCHAR(300) NOT NULL,
    description TEXT,
    goal_category VARCHAR(100),
    -- performance, development, project, behavioral
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- draft, active, completed, cancelled
    weight DECIMAL(5,2) NOT NULL DEFAULT 0.00,
    target_metric TEXT,
    actual_result TEXT,
    self_rating DECIMAL(5,2),
    self_comments TEXT,
    manager_rating DECIMAL(5,2),
    manager_comments TEXT,
    start_date DATE,
    due_date DATE,
    completed_date DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Competency assessments (linked to a performance document)
CREATE TABLE IF NOT EXISTS _atlas.performance_competency_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES _atlas.performance_documents(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL,
    competency_id UUID NOT NULL REFERENCES _atlas.performance_competencies(id),
    self_rating DECIMAL(5,2),
    self_comments TEXT,
    manager_rating DECIMAL(5,2),
    manager_comments TEXT,
    calibration_rating DECIMAL(5,2),
    calibration_comments TEXT,
    final_rating DECIMAL(5,2),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(document_id, competency_id)
);

-- Feedback entries (360-degree feedback, ad-hoc feedback)
CREATE TABLE IF NOT EXISTS _atlas.performance_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    document_id UUID REFERENCES _atlas.performance_documents(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL,
    from_user_id UUID NOT NULL,
    from_user_name VARCHAR(200),
    feedback_type VARCHAR(50) NOT NULL DEFAULT 'peer',
    -- peer, manager, direct_report, external, self
    subject VARCHAR(300),
    content TEXT NOT NULL,
    visibility VARCHAR(50) NOT NULL DEFAULT 'manager_only',
    -- private, manager_only, manager_and_employee, everyone
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- draft, submitted, acknowledged, withdrawn
    acknowledged_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Performance dashboard summary (materialized view-like)
CREATE TABLE IF NOT EXISTS _atlas.performance_dashboard (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    review_cycle_id UUID NOT NULL REFERENCES _atlas.performance_review_cycles(id),
    total_documents INT DEFAULT 0,
    not_started_count INT DEFAULT 0,
    goal_setting_count INT DEFAULT 0,
    self_evaluation_count INT DEFAULT 0,
    manager_evaluation_count INT DEFAULT 0,
    calibration_count INT DEFAULT 0,
    completed_count INT DEFAULT 0,
    cancelled_count INT DEFAULT 0,
    average_rating DECIMAL(5,2),
    goals_total INT DEFAULT 0,
    goals_completed INT DEFAULT 0,
    feedback_count INT DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, review_cycle_id)
);
