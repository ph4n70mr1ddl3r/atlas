-- 022_procurement_sourcing.sql
-- Procurement Sourcing Management (Oracle Fusion SCM > Procurement > Sourcing)
--
-- Provides sourcing events (RFQ/RFP/RFI), supplier responses/bids,
-- scoring & evaluation, award management, and sourcing templates.

-- ========================================================================
-- Sourcing Templates
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.sourcing_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    default_event_type VARCHAR(20) NOT NULL DEFAULT 'rfq',
    default_style VARCHAR(20) NOT NULL DEFAULT 'sealed',
    default_scoring_method VARCHAR(20) NOT NULL DEFAULT 'weighted',
    default_response_deadline_days INT NOT NULL DEFAULT 14,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    default_bids_visible BOOLEAN NOT NULL DEFAULT false,
    default_terms TEXT,
    default_scoring_criteria JSONB NOT NULL DEFAULT '[]',
    default_lines JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ========================================================================
-- Sourcing Events
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.sourcing_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    event_number VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    event_type VARCHAR(20) NOT NULL DEFAULT 'rfq',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    style VARCHAR(20) NOT NULL DEFAULT 'sealed',
    response_deadline DATE NOT NULL,
    published_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    template_id UUID REFERENCES _atlas.sourcing_templates(id),
    template_name VARCHAR(200),
    scoring_method VARCHAR(20) NOT NULL DEFAULT 'weighted',
    evaluation_lead_id UUID,
    evaluation_lead_name VARCHAR(200),
    contact_person_id UUID,
    contact_person_name VARCHAR(200),
    are_bids_visible BOOLEAN NOT NULL DEFAULT false,
    allow_supplier_rank_visibility BOOLEAN NOT NULL DEFAULT false,
    terms_and_conditions TEXT,
    attachments JSONB NOT NULL DEFAULT '[]',
    invited_supplier_count INT NOT NULL DEFAULT 0,
    response_count INT NOT NULL DEFAULT 0,
    award_summary JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, event_number)
);

CREATE INDEX IF NOT EXISTS idx_sourcing_events_org ON _atlas.sourcing_events(organization_id);
CREATE INDEX IF NOT EXISTS idx_sourcing_events_status ON _atlas.sourcing_events(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_sourcing_events_type ON _atlas.sourcing_events(organization_id, event_type);

-- ========================================================================
-- Sourcing Event Lines
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.sourcing_event_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    event_id UUID NOT NULL REFERENCES _atlas.sourcing_events(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    description TEXT NOT NULL,
    item_number VARCHAR(100),
    category VARCHAR(200),
    quantity NUMERIC(18,4) NOT NULL,
    uom VARCHAR(20) NOT NULL DEFAULT 'EA',
    target_price NUMERIC(18,4),
    target_total NUMERIC(18,4),
    need_by_date DATE,
    ship_to VARCHAR(500),
    specifications JSONB,
    allow_partial_quantity BOOLEAN NOT NULL DEFAULT false,
    min_award_quantity NUMERIC(18,4),
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    awarded_supplier_id UUID,
    awarded_supplier_name VARCHAR(200),
    awarded_price NUMERIC(18,4),
    awarded_quantity NUMERIC(18,4),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(event_id, line_number)
);

CREATE INDEX IF NOT EXISTS idx_sourcing_event_lines_event ON _atlas.sourcing_event_lines(event_id);

-- ========================================================================
-- Sourcing Invites
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.sourcing_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    event_id UUID NOT NULL REFERENCES _atlas.sourcing_events(id) ON DELETE CASCADE,
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(200),
    supplier_email VARCHAR(500),
    is_viewed BOOLEAN NOT NULL DEFAULT false,
    viewed_at TIMESTAMPTZ,
    has_responded BOOLEAN NOT NULL DEFAULT false,
    responded_at TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'invited',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(event_id, supplier_id)
);

CREATE INDEX IF NOT EXISTS idx_sourcing_invites_event ON _atlas.sourcing_invites(event_id);

-- ========================================================================
-- Supplier Responses (Bids)
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.supplier_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    event_id UUID NOT NULL REFERENCES _atlas.sourcing_events(id) ON DELETE CASCADE,
    response_number VARCHAR(50) NOT NULL,
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(200),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    total_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    total_score NUMERIC(8,2),
    rank INT,
    is_compliant BOOLEAN,
    cover_letter TEXT,
    valid_until DATE,
    payment_terms VARCHAR(200),
    lead_time_days INT,
    warranty_months INT,
    attachments JSONB NOT NULL DEFAULT '[]',
    evaluation_notes TEXT,
    submitted_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    evaluated_by UUID,
    evaluated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(event_id, response_number)
);

CREATE INDEX IF NOT EXISTS idx_supplier_responses_event ON _atlas.supplier_responses(event_id);
CREATE INDEX IF NOT EXISTS idx_supplier_responses_supplier ON _atlas.supplier_responses(supplier_id);
CREATE INDEX IF NOT EXISTS idx_supplier_responses_status ON _atlas.supplier_responses(event_id, status);

-- ========================================================================
-- Supplier Response Lines
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.supplier_response_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    response_id UUID NOT NULL REFERENCES _atlas.supplier_responses(id) ON DELETE CASCADE,
    event_line_id UUID NOT NULL REFERENCES _atlas.sourcing_event_lines(id),
    line_number INT NOT NULL,
    unit_price NUMERIC(18,4) NOT NULL,
    quantity NUMERIC(18,4) NOT NULL,
    line_amount NUMERIC(18,4) NOT NULL,
    discount_percent NUMERIC(5,2),
    effective_price NUMERIC(18,4),
    promised_delivery_date DATE,
    lead_time_days INT,
    is_compliant BOOLEAN,
    score NUMERIC(8,2),
    supplier_notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(response_id, event_line_id)
);

CREATE INDEX IF NOT EXISTS idx_supplier_response_lines_response ON _atlas.supplier_response_lines(response_id);

-- ========================================================================
-- Scoring Criteria
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.scoring_criteria (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    event_id UUID NOT NULL REFERENCES _atlas.sourcing_events(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    weight NUMERIC(5,2) NOT NULL,
    max_score NUMERIC(8,2) NOT NULL,
    criterion_type VARCHAR(20) NOT NULL DEFAULT 'custom',
    display_order INT NOT NULL DEFAULT 10,
    is_mandatory BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_scoring_criteria_event ON _atlas.scoring_criteria(event_id);

-- ========================================================================
-- Response Scores
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.response_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    response_id UUID NOT NULL REFERENCES _atlas.supplier_responses(id) ON DELETE CASCADE,
    criterion_id UUID NOT NULL REFERENCES _atlas.scoring_criteria(id) ON DELETE CASCADE,
    score NUMERIC(8,2) NOT NULL,
    weighted_score NUMERIC(8,2) NOT NULL,
    notes TEXT,
    scored_by UUID,
    scored_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(response_id, criterion_id)
);

CREATE INDEX IF NOT EXISTS idx_response_scores_response ON _atlas.response_scores(response_id);

-- ========================================================================
-- Sourcing Awards
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.sourcing_awards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    event_id UUID NOT NULL REFERENCES _atlas.sourcing_events(id) ON DELETE CASCADE,
    award_number VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    award_method VARCHAR(20) NOT NULL DEFAULT 'single',
    total_awarded_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
    award_rationale TEXT,
    lines JSONB NOT NULL DEFAULT '[]',
    metadata JSONB NOT NULL DEFAULT '{}',
    awarded_by UUID,
    awarded_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(event_id, award_number)
);

CREATE INDEX IF NOT EXISTS idx_sourcing_awards_event ON _atlas.sourcing_awards(event_id);

-- ========================================================================
-- Sourcing Award Lines
-- ========================================================================
CREATE TABLE IF NOT EXISTS _atlas.sourcing_award_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    award_id UUID NOT NULL REFERENCES _atlas.sourcing_awards(id) ON DELETE CASCADE,
    event_line_id UUID NOT NULL REFERENCES _atlas.sourcing_event_lines(id),
    response_id UUID NOT NULL REFERENCES _atlas.supplier_responses(id),
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(200),
    awarded_quantity NUMERIC(18,4) NOT NULL,
    awarded_unit_price NUMERIC(18,4) NOT NULL,
    awarded_amount NUMERIC(18,4) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sourcing_award_lines_award ON _atlas.sourcing_award_lines(award_id);
