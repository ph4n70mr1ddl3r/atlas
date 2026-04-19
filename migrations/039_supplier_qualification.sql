-- ============================================================================
-- Supplier Qualification Management
-- Oracle Fusion Cloud ERP: Procurement > Supplier Qualification
-- Migration 039
--
-- Manages supplier qualification lifecycle including:
--   - Qualification areas (criteria categories)
--   - Qualification questions (questionnaires for suppliers)
--   - Supplier qualification initiatives (qualification runs)
--   - Supplier responses & evaluations
--   - Supplier certifications with expiry tracking
--   - Scoring models for objective evaluation
--   - Qualification status tracking (initiated → pending → under_evaluation → qualified/disqualified/expired)
--
-- Oracle Fusion equivalent: Procurement > Supplier Qualification > Initiatives
-- ============================================================================

-- ============================================================================
-- Qualification Areas
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.qualification_areas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    area_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    area_type VARCHAR(30) NOT NULL DEFAULT 'questionnaire',  -- questionnaire, certificate, financial, site_visit, reference, other
    scoring_model VARCHAR(30) NOT NULL DEFAULT 'manual',     -- manual, weighted, pass_fail
    passing_score NUMERIC(5,2) DEFAULT 70.00,
    is_mandatory BOOLEAN NOT NULL DEFAULT false,
    renewal_period_days INT DEFAULT 365,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, area_code)
);

CREATE INDEX idx_qual_areas_org ON _atlas.qualification_areas(organization_id);

-- ============================================================================
-- Qualification Questions
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.qualification_questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    area_id UUID NOT NULL REFERENCES _atlas.qualification_areas(id),
    question_number INT NOT NULL,
    question_text TEXT NOT NULL,
    description TEXT,
    response_type VARCHAR(30) NOT NULL DEFAULT 'text',  -- text, yes_no, numeric, date, multi_choice, file_upload
    choices JSONB,                                        -- for multi_choice: ["Option A", "Option B", ...]
    is_required BOOLEAN NOT NULL DEFAULT true,
    weight NUMERIC(5,2) NOT NULL DEFAULT 1.00,           -- weight for weighted scoring
    max_score NUMERIC(5,2) NOT NULL DEFAULT 10.00,       -- maximum possible score for this question
    help_text TEXT,
    display_order INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_qual_questions_area ON _atlas.qualification_questions(area_id);

-- ============================================================================
-- Supplier Qualification Initiatives (qualification runs/campaigns)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.supplier_qualification_initiatives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    initiative_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    area_id UUID NOT NULL REFERENCES _atlas.qualification_areas(id),
    qualification_purpose VARCHAR(50) NOT NULL DEFAULT 'new_supplier',  -- new_supplier, requalification, compliance, ad_hoc
    status VARCHAR(30) NOT NULL DEFAULT 'draft',  -- draft, active, pending_evaluations, completed, cancelled
    deadline DATE,
    total_invited INT NOT NULL DEFAULT 0,
    total_responded INT NOT NULL DEFAULT 0,
    total_qualified INT NOT NULL DEFAULT 0,
    total_disqualified INT NOT NULL DEFAULT 0,
    total_pending INT NOT NULL DEFAULT 0,
    completed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, initiative_number)
);

CREATE INDEX idx_qual_initiatives_org ON _atlas.supplier_qualification_initiatives(organization_id);
CREATE INDEX idx_qual_initiatives_status ON _atlas.supplier_qualification_initiatives(organization_id, status);

-- ============================================================================
-- Supplier Qualification Invitations (per supplier per initiative)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.supplier_qualification_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    initiative_id UUID NOT NULL REFERENCES _atlas.supplier_qualification_initiatives(id),
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(200) NOT NULL,
    supplier_contact_name VARCHAR(200),
    supplier_contact_email VARCHAR(300),
    status VARCHAR(30) NOT NULL DEFAULT 'initiated',  -- initiated, pending_response, under_evaluation, qualified, disqualified, expired, withdrawn
    invitation_date TIMESTAMPTZ,
    response_date TIMESTAMPTZ,
    evaluation_date TIMESTAMPTZ,
    expiry_date DATE,
    overall_score NUMERIC(5,2) DEFAULT 0.00,
    max_possible_score NUMERIC(5,2) DEFAULT 0.00,
    score_percentage NUMERIC(5,2) DEFAULT 0.00,
    qualified_by UUID,
    disqualified_reason TEXT,
    evaluation_notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(initiative_id, supplier_id)
);

CREATE INDEX idx_qual_invitations_initiative ON _atlas.supplier_qualification_invitations(initiative_id);
CREATE INDEX idx_qual_invitations_supplier ON _atlas.supplier_qualification_invitations(supplier_id);
CREATE INDEX idx_qual_invitations_status ON _atlas.supplier_qualification_invitations(organization_id, status);

-- ============================================================================
-- Supplier Qualification Responses (answers to individual questions)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.supplier_qualification_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    invitation_id UUID NOT NULL REFERENCES _atlas.supplier_qualification_invitations(id),
    question_id UUID NOT NULL REFERENCES _atlas.qualification_questions(id),
    response_text TEXT,
    response_value JSONB,                          -- for structured responses (yes/no, numeric, multi_choice, date)
    file_reference VARCHAR(500),                    -- for file_upload type
    score NUMERIC(5,2) DEFAULT 0.00,
    max_score NUMERIC(5,2) DEFAULT 0.00,
    evaluator_notes TEXT,
    evaluated_by UUID,
    evaluated_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(invitation_id, question_id)
);

CREATE INDEX idx_qual_responses_invitation ON _atlas.supplier_qualification_responses(invitation_id);

-- ============================================================================
-- Supplier Certifications (track ongoing certifications / qualifications)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.supplier_certifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(200) NOT NULL,
    certification_type VARCHAR(100) NOT NULL,       -- e.g. "ISO 9001", "ISO 14001", " diversity_certified", etc.
    certification_name VARCHAR(300) NOT NULL,
    certifying_body VARCHAR(200),
    certificate_number VARCHAR(100),
    status VARCHAR(30) NOT NULL DEFAULT 'active',  -- active, expired, revoked, pending_renewal
    issued_date DATE,
    expiry_date DATE,
    renewal_date DATE,
    qualification_invitation_id UUID,               -- link to the qualification that granted this certification
    document_reference VARCHAR(500),
    notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_supplier_certs_supplier ON _atlas.supplier_certifications(supplier_id);
CREATE INDEX idx_supplier_certs_status ON _atlas.supplier_certifications(organization_id, status);
CREATE INDEX idx_supplier_certs_expiry ON _atlas.supplier_certifications(expiry_date);
