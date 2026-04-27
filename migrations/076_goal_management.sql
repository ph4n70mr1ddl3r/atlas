-- Goal Management (Oracle Fusion HCM > Goal Management)
-- Provides: Goal library templates, goal plans, goals with cascading hierarchy,
-- progress tracking, goal alignment, and goal notes.

-- ============================================================================
-- Goal Library: predefined goal templates organized by category
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.goal_library_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    display_order INT DEFAULT 0,
    status VARCHAR(50) DEFAULT 'active',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.goal_library_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    category_id UUID REFERENCES _atlas.goal_library_categories(id),
    code VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    goal_type VARCHAR(50) NOT NULL DEFAULT 'individual',  -- individual, team, organization
    success_criteria TEXT,
    target_metric VARCHAR(200),
    target_value NUMERIC(18,4),
    uom VARCHAR(50),                                        -- unit of measure
    suggested_weight NUMERIC(5,2),
    estimated_duration_days INT,
    status VARCHAR(50) DEFAULT 'active',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Goal Plans: performance/review periods that contain goals
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.goal_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    plan_type VARCHAR(50) NOT NULL DEFAULT 'performance',   -- performance, development, stretch
    review_period_start DATE NOT NULL,
    review_period_end DATE NOT NULL,
    goal_creation_deadline DATE,
    status VARCHAR(50) DEFAULT 'draft',                     -- draft, active, closed
    allow_self_goals BOOLEAN DEFAULT true,
    allow_team_goals BOOLEAN DEFAULT true,
    max_weight_sum NUMERIC(5,2) DEFAULT 100.00,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Goals: individual, team, and organizational goals
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    plan_id UUID REFERENCES _atlas.goal_plans(id),
    parent_goal_id UUID REFERENCES _atlas.goals(id),        -- cascading hierarchy
    library_template_id UUID REFERENCES _atlas.goal_library_templates(id),

    code VARCHAR(100),
    name VARCHAR(500) NOT NULL,
    description TEXT,
    goal_type VARCHAR(50) NOT NULL DEFAULT 'individual',     -- individual, team, organization
    category VARCHAR(100),

    -- Owner
    owner_id UUID NOT NULL,                                  -- person responsible
    owner_type VARCHAR(50) DEFAULT 'employee',               -- employee, team, department, organization
    assigned_by UUID,

    -- Metrics
    success_criteria TEXT,
    target_metric VARCHAR(200),
    target_value NUMERIC(18,4),
    actual_value NUMERIC(18,4),
    uom VARCHAR(50),
    progress_pct NUMERIC(5,2) DEFAULT 0.00,                  -- 0-100

    -- Weighting & status
    weight NUMERIC(5,2),                                     -- relative weight in plan
    status VARCHAR(50) NOT NULL DEFAULT 'not_started',       -- not_started, in_progress, on_track, at_risk, completed, cancelled
    priority VARCHAR(50) DEFAULT 'medium',                   -- low, medium, high, critical

    -- Dates
    start_date DATE,
    target_date DATE,
    completed_date DATE,

    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_goals_org ON _atlas.goals(organization_id);
CREATE INDEX IF NOT EXISTS idx_goals_plan ON _atlas.goals(plan_id);
CREATE INDEX IF NOT EXISTS idx_goals_parent ON _atlas.goals(parent_goal_id);
CREATE INDEX IF NOT EXISTS idx_goals_owner ON _atlas.goals(owner_id);
CREATE INDEX IF NOT EXISTS idx_goals_status ON _atlas.goals(status);

-- ============================================================================
-- Goal Alignments: explicit alignment links between goals
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.goal_alignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    source_goal_id UUID NOT NULL REFERENCES _atlas.goals(id) ON DELETE CASCADE,
    aligned_to_goal_id UUID NOT NULL REFERENCES _atlas.goals(id) ON DELETE CASCADE,
    alignment_type VARCHAR(50) NOT NULL DEFAULT 'supports',  -- supports, depends_on, cascaded_from
    description TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(source_goal_id, aligned_to_goal_id)
);

-- ============================================================================
-- Goal Notes: comments and feedback on goals
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.goal_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    goal_id UUID NOT NULL REFERENCES _atlas.goals(id) ON DELETE CASCADE,
    author_id UUID NOT NULL,
    note_type VARCHAR(50) DEFAULT 'comment',                 -- comment, feedback, status_change, check_in
    content TEXT NOT NULL,
    visibility VARCHAR(50) DEFAULT 'private',                -- private, manager, public
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_goal_notes_goal ON _atlas.goal_notes(goal_id);
