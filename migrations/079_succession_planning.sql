-- Succession Planning (Oracle Fusion HCM > Succession Management)
-- Provides: Succession plans, talent pools, talent reviews, career paths,
--   candidate assessments, and nine-box grid analytics.

-- ============================================================================
-- Succession Plans: plans for key positions with backup candidates
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.succession_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    plan_type VARCHAR(50) NOT NULL DEFAULT 'position',       -- position, role, key_person
    position_id UUID,
    position_title VARCHAR(300),
    job_id UUID,
    department_id UUID,
    current_incumbent_id UUID,
    current_incumbent_name VARCHAR(300),
    risk_level VARCHAR(50) NOT NULL DEFAULT 'medium',        -- low, medium, high, critical
    urgency VARCHAR(50) NOT NULL DEFAULT 'medium_term',      -- immediate, short_term, medium_term, long_term
    status VARCHAR(50) NOT NULL DEFAULT 'draft',             -- draft, active, completed, cancelled
    effective_date DATE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_succession_plans_org ON _atlas.succession_plans(organization_id);
CREATE INDEX IF NOT EXISTS idx_succession_plans_status ON _atlas.succession_plans(status);
CREATE INDEX IF NOT EXISTS idx_succession_plans_position ON _atlas.succession_plans(position_id);
CREATE INDEX IF NOT EXISTS idx_succession_plans_incumbent ON _atlas.succession_plans(current_incumbent_id);

-- ============================================================================
-- Succession Candidates: people nominated as successors for a plan
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.succession_candidates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    plan_id UUID NOT NULL REFERENCES _atlas.succession_plans(id) ON DELETE CASCADE,
    person_id UUID NOT NULL,
    person_name VARCHAR(300),
    employee_number VARCHAR(50),
    readiness VARCHAR(50) NOT NULL DEFAULT 'not_ready',      -- ready_now, ready_1_2_years, ready_3_5_years, not_ready
    ranking INT,
    performance_rating VARCHAR(50),
    potential_rating VARCHAR(50),
    flight_risk VARCHAR(50),                                  -- low, medium, high
    development_notes TEXT,
    recommended_actions TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'proposed',           -- proposed, approved, rejected, development
    metadata JSONB DEFAULT '{}'::jsonb,
    added_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(plan_id, person_id)
);

CREATE INDEX IF NOT EXISTS idx_succession_candidates_plan ON _atlas.succession_candidates(plan_id);
CREATE INDEX IF NOT EXISTS idx_succession_candidates_person ON _atlas.succession_candidates(person_id);

-- ============================================================================
-- Talent Pools: named groups of high-potential employees
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.talent_pools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    pool_type VARCHAR(50) NOT NULL DEFAULT 'high_potential',  -- leadership, technical, high_potential, diversity, custom
    owner_id UUID,
    max_members INT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',              -- draft, active, archived
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_talent_pools_org ON _atlas.talent_pools(organization_id);

-- ============================================================================
-- Talent Pool Members: individuals in a talent pool
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.talent_pool_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    pool_id UUID NOT NULL REFERENCES _atlas.talent_pools(id) ON DELETE CASCADE,
    person_id UUID NOT NULL,
    person_name VARCHAR(300),
    performance_rating VARCHAR(50),
    potential_rating VARCHAR(50),
    readiness VARCHAR(50) NOT NULL DEFAULT 'not_ready',       -- ready_now, ready_1_2_years, ready_3_5_years, not_ready
    development_plan TEXT,
    notes TEXT,
    added_date DATE,
    review_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'active',             -- active, on_hold, removed, graduated
    metadata JSONB DEFAULT '{}'::jsonb,
    added_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(pool_id, person_id)
);

CREATE INDEX IF NOT EXISTS idx_talent_pool_members_pool ON _atlas.talent_pool_members(pool_id);
CREATE INDEX IF NOT EXISTS idx_talent_pool_members_person ON _atlas.talent_pool_members(person_id);

-- ============================================================================
-- Talent Reviews: formal assessment meetings
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.talent_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    review_type VARCHAR(50) NOT NULL DEFAULT 'nine_box',      -- calibration, performance_potential, nine_box, leadership
    facilitator_id UUID,
    department_id UUID,
    review_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'scheduled',          -- scheduled, in_progress, completed, cancelled
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_talent_reviews_org ON _atlas.talent_reviews(organization_id);

-- ============================================================================
-- Talent Review Assessments: individual assessments within a review
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.talent_review_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    review_id UUID NOT NULL REFERENCES _atlas.talent_reviews(id) ON DELETE CASCADE,
    person_id UUID NOT NULL,
    person_name VARCHAR(300),
    performance_rating VARCHAR(50),
    potential_rating VARCHAR(50),
    nine_box_position VARCHAR(100),                            -- star, workhorse, puzzle, solid_citizen, etc.
    strengths TEXT,
    weaknesses TEXT,
    career_aspiration TEXT,
    development_needs TEXT,
    succession_readiness VARCHAR(50),
    assessor_id UUID,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(review_id, person_id)
);

CREATE INDEX IF NOT EXISTS idx_talent_review_assessments_review ON _atlas.talent_review_assessments(review_id);
CREATE INDEX IF NOT EXISTS idx_talent_review_assessments_person ON _atlas.talent_review_assessments(person_id);

-- ============================================================================
-- Career Paths: defined progression paths between jobs/roles
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.career_paths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    path_type VARCHAR(50) NOT NULL DEFAULT 'linear',          -- linear, branching, lattice, dual_track
    from_job_id UUID,
    from_job_title VARCHAR(300),
    to_job_id UUID,
    to_job_title VARCHAR(300),
    typical_duration_months INT,
    required_competencies TEXT,
    required_certifications TEXT,
    development_activities TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',              -- draft, active, archived
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_career_paths_org ON _atlas.career_paths(organization_id);
CREATE INDEX IF NOT EXISTS idx_career_paths_from_job ON _atlas.career_paths(from_job_id);
CREATE INDEX IF NOT EXISTS idx_career_paths_to_job ON _atlas.career_paths(to_job_id);
