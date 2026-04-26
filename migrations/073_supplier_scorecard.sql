-- Supplier Scorecard Management (Oracle Fusion Supplier Portal > Supplier Performance)
-- Tracks supplier KPIs, performance metrics, and periodic reviews

-- Scorecard Templates: define KPI categories and their weights
CREATE TABLE IF NOT EXISTS _atlas.scorecard_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    evaluation_period VARCHAR(50) NOT NULL DEFAULT 'quarterly',  -- quarterly, monthly, annual
    is_active BOOLEAN NOT NULL DEFAULT true,
    total_weight NUMERIC(5,2) NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Scorecard Categories: KPI categories within a template (e.g., Quality, Delivery, Cost, Responsiveness)
CREATE TABLE IF NOT EXISTS _atlas.scorecard_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.scorecard_templates(id) ON DELETE CASCADE,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    weight DOUBLE PRECISION NOT NULL DEFAULT 0,  -- percentage weight (0-100)
    sort_order INT NOT NULL DEFAULT 0,
    scoring_model VARCHAR(50) NOT NULL DEFAULT 'manual',  -- manual, auto, formula
    target_score DOUBLE PRECISION,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(template_id, code)
);

-- Supplier Scorecards: individual scorecard instances for suppliers
CREATE TABLE IF NOT EXISTS _atlas.supplier_scorecards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.scorecard_templates(id),
    scorecard_number VARCHAR(100) NOT NULL,
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(200),
    supplier_number VARCHAR(100),
    evaluation_period_start DATE NOT NULL,
    evaluation_period_end DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',  -- draft, in_review, submitted, approved, rejected
    overall_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    overall_grade VARCHAR(10),
    reviewer_id UUID,
    reviewer_name VARCHAR(200),
    review_date TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, scorecard_number)
);

-- Scorecard Lines: individual KPI scores within a scorecard
CREATE TABLE IF NOT EXISTS _atlas.scorecard_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    scorecard_id UUID NOT NULL REFERENCES _atlas.supplier_scorecards(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES _atlas.scorecard_categories(id),
    line_number INT NOT NULL DEFAULT 1,
    kpi_name VARCHAR(200) NOT NULL,
    kpi_description TEXT,
    weight DOUBLE PRECISION NOT NULL DEFAULT 0,
    target_value DOUBLE PRECISION,
    actual_value DOUBLE PRECISION,
    score DOUBLE PRECISION NOT NULL DEFAULT 0,  -- 0-100
    weighted_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    evidence TEXT,
    notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Performance Reviews: periodic supplier performance reviews
CREATE TABLE IF NOT EXISTS _atlas.supplier_performance_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    review_number VARCHAR(100) NOT NULL,
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(200),
    scorecard_id UUID REFERENCES _atlas.supplier_scorecards(id),
    review_type VARCHAR(50) NOT NULL DEFAULT 'periodic',  -- periodic, ad_hoc, annual
    review_period VARCHAR(100),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    previous_score DOUBLE PRECISION,
    current_score DOUBLE PRECISION,
    score_change DOUBLE PRECISION,
    rating VARCHAR(20),  -- excellent, good, acceptable, poor, critical
    strengths TEXT,
    improvement_areas TEXT,
    action_items TEXT,
    follow_up_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',  -- draft, scheduled, in_progress, completed, cancelled
    reviewer_id UUID,
    reviewer_name VARCHAR(200),
    completed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, review_number)
);

-- Review Action Items: track follow-up actions from reviews
CREATE TABLE IF NOT EXISTS _atlas.review_action_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    review_id UUID NOT NULL REFERENCES _atlas.supplier_performance_reviews(id) ON DELETE CASCADE,
    action_number INT NOT NULL DEFAULT 1,
    description TEXT NOT NULL,
    assignee_id UUID,
    assignee_name VARCHAR(200),
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',  -- low, medium, high, critical
    due_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'open',  -- open, in_progress, completed, cancelled
    completed_at TIMESTAMPTZ,
    notes TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
