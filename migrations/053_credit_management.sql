-- 053: Credit Management (Oracle Fusion Cloud Credit Management)
-- Implements: Credit Profiles, Credit Scoring Models, Credit Limits,
--   Credit Exposure Tracking, Credit Check Rules, Credit Holds, Credit Reviews
-- Oracle Fusion equivalent: Receivables > Credit Management

-- Credit scoring models define how customer creditworthiness is assessed
CREATE TABLE IF NOT EXISTS _atlas.credit_scoring_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    model_type VARCHAR(50) NOT NULL DEFAULT 'manual',
    -- manual, scorecard, risk_category, external
    scoring_criteria JSONB DEFAULT '[]',
    -- e.g. [{"factor":"payment_history","weight":0.3,"max_score":100}]
    score_ranges JSONB DEFAULT '[]',
    -- e.g. [{"min":0,"max":40,"label":"High Risk","rating":"D"},
    --       {"min":41,"max":70,"label":"Medium Risk","rating":"C"},
    --       {"min":71,"max":100,"label":"Low Risk","rating":"A"}]
    is_active BOOLEAN DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Credit profiles per customer or customer group
CREATE TABLE IF NOT EXISTS _atlas.credit_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    profile_number VARCHAR(100) NOT NULL,
    profile_name VARCHAR(200) NOT NULL,
    description TEXT,
    profile_type VARCHAR(50) NOT NULL DEFAULT 'customer',
    -- customer, customer_group, global
    customer_id UUID,
    customer_name VARCHAR(300),
    customer_group_id UUID,
    customer_group_name VARCHAR(300),
    scoring_model_id UUID REFERENCES _atlas.credit_scoring_models(id),
    credit_score DOUBLE PRECISION,
    credit_rating VARCHAR(10),
    -- e.g. A, B, C, D
    risk_level VARCHAR(50) DEFAULT 'medium',
    -- low, medium, high, very_high, blocked
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- active, inactive, suspended, blocked
    review_frequency_days INT DEFAULT 90,
    last_review_date DATE,
    next_review_date DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, profile_number)
);

-- Credit limits (supports multi-currency and global limits)
CREATE TABLE IF NOT EXISTS _atlas.credit_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    profile_id UUID NOT NULL REFERENCES _atlas.credit_profiles(id) ON DELETE CASCADE,
    limit_type VARCHAR(50) NOT NULL DEFAULT 'overall',
    -- overall, order, delivery, currency
    currency_code VARCHAR(10),
    credit_limit DOUBLE PRECISION NOT NULL DEFAULT 0,
    temp_limit_increase DOUBLE PRECISION DEFAULT 0,
    temp_limit_expiry DATE,
    used_amount DOUBLE PRECISION DEFAULT 0,
    available_amount DOUBLE PRECISION DEFAULT 0,
    hold_amount DOUBLE PRECISION DEFAULT 0,
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN DEFAULT true,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Credit check rules define when credit checks are triggered
CREATE TABLE IF NOT EXISTS _atlas.credit_check_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    check_point VARCHAR(50) NOT NULL DEFAULT 'order_entry',
    -- order_entry, shipment, invoice, delivery, payment
    check_type VARCHAR(50) NOT NULL DEFAULT 'automatic',
    -- automatic, manual
    condition JSONB DEFAULT '{}',
    -- e.g. {"min_order_amount": 1000, "customer_type": "external"}
    action_on_failure VARCHAR(50) NOT NULL DEFAULT 'hold',
    -- hold, warn, reject, notify
    priority INT DEFAULT 10,
    is_active BOOLEAN DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, name)
);

-- Credit exposure tracks the total exposure per profile
CREATE TABLE IF NOT EXISTS _atlas.credit_exposure (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    profile_id UUID NOT NULL REFERENCES _atlas.credit_profiles(id) ON DELETE CASCADE,
    exposure_date DATE NOT NULL,
    open_receivables DOUBLE PRECISION DEFAULT 0,
    open_orders DOUBLE PRECISION DEFAULT 0,
    open_shipments DOUBLE PRECISION DEFAULT 0,
    open_invoices DOUBLE PRECISION DEFAULT 0,
    unapplied_cash DOUBLE PRECISION DEFAULT 0,
    on_hold_amount DOUBLE PRECISION DEFAULT 0,
    total_exposure DOUBLE PRECISION DEFAULT 0,
    credit_limit DOUBLE PRECISION DEFAULT 0,
    available_credit DOUBLE PRECISION DEFAULT 0,
    utilization_percent DOUBLE PRECISION DEFAULT 0,
    currency_code VARCHAR(10) DEFAULT 'USD',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(profile_id, exposure_date, currency_code)
);

-- Credit holds placed on transactions when credit limits exceeded
CREATE TABLE IF NOT EXISTS _atlas.credit_holds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    profile_id UUID NOT NULL REFERENCES _atlas.credit_profiles(id),
    hold_number VARCHAR(100) NOT NULL,
    hold_type VARCHAR(50) NOT NULL DEFAULT 'credit_limit',
    -- credit_limit, overdue, review, manual, scoring
    entity_type VARCHAR(100) NOT NULL,
    -- sales_order, invoice, shipment, delivery
    entity_id UUID NOT NULL,
    entity_number VARCHAR(200),
    hold_amount DOUBLE PRECISION,
    reason TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- active, released, overridden, cancelled
    released_by UUID,
    released_at TIMESTAMPTZ,
    release_reason TEXT,
    overridden_by UUID,
    overridden_at TIMESTAMPTZ,
    override_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, hold_number)
);

-- Credit reviews (periodic or triggered reviews of credit profiles)
CREATE TABLE IF NOT EXISTS _atlas.credit_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    profile_id UUID NOT NULL REFERENCES _atlas.credit_profiles(id),
    review_number VARCHAR(100) NOT NULL,
    review_type VARCHAR(50) NOT NULL DEFAULT 'periodic',
    -- periodic, triggered, ad_hoc, escalation
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- pending, in_review, completed, cancelled
    previous_credit_limit DOUBLE PRECISION,
    recommended_credit_limit DOUBLE PRECISION,
    approved_credit_limit DOUBLE PRECISION,
    previous_score DOUBLE PRECISION,
    new_score DOUBLE PRECISION,
    previous_rating VARCHAR(10),
    new_rating VARCHAR(10),
    findings TEXT,
    recommendations TEXT,
    reviewer_id UUID,
    reviewer_name VARCHAR(200),
    reviewed_at TIMESTAMPTZ,
    approver_id UUID,
    approver_name VARCHAR(200),
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    due_date DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, review_number)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_cm_profiles_org ON _atlas.credit_profiles(organization_id);
CREATE INDEX IF NOT EXISTS idx_cm_profiles_customer ON _atlas.credit_profiles(customer_id);
CREATE INDEX IF NOT EXISTS idx_cm_profiles_status ON _atlas.credit_profiles(status);
CREATE INDEX IF NOT EXISTS idx_cm_limits_profile ON _atlas.credit_limits(profile_id);
CREATE INDEX IF NOT EXISTS idx_cm_limits_active ON _atlas.credit_limits(is_active);
CREATE INDEX IF NOT EXISTS idx_cm_exposure_profile ON _atlas.credit_exposure(profile_id);
CREATE INDEX IF NOT EXISTS idx_cm_exposure_date ON _atlas.credit_exposure(exposure_date);
CREATE INDEX IF NOT EXISTS idx_cm_holds_profile ON _atlas.credit_holds(profile_id);
CREATE INDEX IF NOT EXISTS idx_cm_holds_entity ON _atlas.credit_holds(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_cm_holds_status ON _atlas.credit_holds(status);
CREATE INDEX IF NOT EXISTS idx_cm_reviews_profile ON _atlas.credit_reviews(profile_id);
CREATE INDEX IF NOT EXISTS idx_cm_reviews_status ON _atlas.credit_reviews(status);
CREATE INDEX IF NOT EXISTS idx_cm_rules_org ON _atlas.credit_check_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_cm_scoring_org ON _atlas.credit_scoring_models(organization_id);
