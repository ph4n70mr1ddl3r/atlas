-- Recruiting Management (Oracle Fusion HCM > Recruiting)
-- Job Requisitions, Candidates, Applications, Interviews, Offers

-- Job Requisitions
CREATE TABLE IF NOT EXISTS _atlas.job_requisitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    requisition_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    department VARCHAR(200),
    location VARCHAR(200),
    employment_type VARCHAR(50) DEFAULT 'full_time',  -- full_time, part_time, contract, internship
    position_type VARCHAR(50) DEFAULT 'new',           -- new, replacement, additional
    vacancies INT DEFAULT 1,
    priority VARCHAR(50) DEFAULT 'medium',             -- low, medium, high, critical
    salary_min NUMERIC(12, 2),
    salary_max NUMERIC(12, 2),
    currency VARCHAR(3) DEFAULT 'USD',
    required_skills JSONB DEFAULT '[]',
    qualifications TEXT,
    experience_years_min INT,
    experience_years_max INT,
    education_level VARCHAR(100),
    hiring_manager_id UUID,
    recruiter_id UUID,
    target_start_date DATE,
    status VARCHAR(50) DEFAULT 'draft',               -- draft, open, on_hold, filled, cancelled, closed
    posted_date TIMESTAMPTZ,
    closed_date TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, requisition_number)
);

-- Candidates
CREATE TABLE IF NOT EXISTS _atlas.candidates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    candidate_number VARCHAR(100),
    first_name VARCHAR(200) NOT NULL,
    last_name VARCHAR(200) NOT NULL,
    email VARCHAR(500),
    phone VARCHAR(100),
    address TEXT,
    city VARCHAR(200),
    state VARCHAR(200),
    country VARCHAR(200),
    postal_code VARCHAR(50),
    linkedin_url VARCHAR(1000),
    source VARCHAR(100),                              -- referral, job_board, agency, website, internal, other
    source_detail VARCHAR(500),
    resume_url VARCHAR(1000),
    cover_letter_url VARCHAR(1000),
    current_employer VARCHAR(500),
    current_title VARCHAR(500),
    years_of_experience INT,
    education_level VARCHAR(100),
    skills JSONB DEFAULT '[]',
    notes TEXT,
    status VARCHAR(50) DEFAULT 'active',              -- active, inactive, hired, rejected, blacklisted
    tags JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Job Applications
CREATE TABLE IF NOT EXISTS _atlas.job_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    application_number VARCHAR(100),
    requisition_id UUID NOT NULL REFERENCES _atlas.job_requisitions(id),
    candidate_id UUID NOT NULL REFERENCES _atlas.candidates(id),
    status VARCHAR(50) DEFAULT 'applied',             -- applied, screening, interview, assessment, offer, hired, rejected, withdrawn
    match_score NUMERIC(5, 2) DEFAULT 0,
    screening_notes TEXT,
    rejection_reason TEXT,
    applied_at TIMESTAMPTZ DEFAULT now(),
    last_status_change TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, application_number)
);

-- Interviews
CREATE TABLE IF NOT EXISTS _atlas.interviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    application_id UUID NOT NULL REFERENCES _atlas.job_applications(id),
    interview_type VARCHAR(50) DEFAULT 'phone',       -- phone, video, on_site, panel, technical, group
    round INT DEFAULT 1,
    scheduled_at TIMESTAMPTZ,
    duration_minutes INT DEFAULT 60,
    location VARCHAR(500),
    meeting_link VARCHAR(1000),
    interviewer_ids JSONB DEFAULT '[]',
    interviewer_names JSONB DEFAULT '[]',
    status VARCHAR(50) DEFAULT 'scheduled',           -- scheduled, in_progress, completed, cancelled, no_show
    feedback TEXT,
    rating INT CHECK (rating >= 1 AND rating <= 5),
    recommendation VARCHAR(50),                       -- strong_hire, hire, lean_hire, no_hire, strong_no_hire
    completed_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Job Offers
CREATE TABLE IF NOT EXISTS _atlas.job_offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    application_id UUID NOT NULL REFERENCES _atlas.job_applications(id),
    offer_number VARCHAR(100),
    job_title VARCHAR(500) NOT NULL,
    department VARCHAR(200),
    location VARCHAR(200),
    employment_type VARCHAR(50) DEFAULT 'full_time',
    start_date DATE,
    salary_offered NUMERIC(12, 2),
    salary_currency VARCHAR(3) DEFAULT 'USD',
    salary_frequency VARCHAR(50) DEFAULT 'annual',    -- annual, monthly, hourly
    signing_bonus NUMERIC(12, 2) DEFAULT 0,
    benefits_summary TEXT,
    terms_and_conditions TEXT,
    status VARCHAR(50) DEFAULT 'draft',               -- draft, pending_approval, approved, extended, accepted, declined, withdrawn, expired
    offer_date TIMESTAMPTZ,
    response_deadline TIMESTAMPTZ,
    responded_at TIMESTAMPTZ,
    response_notes TEXT,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, offer_number)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_requisitions_org ON _atlas.job_requisitions(organization_id);
CREATE INDEX IF NOT EXISTS idx_requisitions_status ON _atlas.job_requisitions(status);
CREATE INDEX IF NOT EXISTS idx_candidates_org ON _atlas.candidates(organization_id);
CREATE INDEX IF NOT EXISTS idx_candidates_email ON _atlas.candidates(email);
CREATE INDEX IF NOT EXISTS idx_applications_req ON _atlas.job_applications(requisition_id);
CREATE INDEX IF NOT EXISTS idx_applications_candidate ON _atlas.job_applications(candidate_id);
CREATE INDEX IF NOT EXISTS idx_applications_status ON _atlas.job_applications(status);
CREATE INDEX IF NOT EXISTS idx_interviews_app ON _atlas.interviews(application_id);
CREATE INDEX IF NOT EXISTS idx_offers_app ON _atlas.job_offers(application_id);
CREATE INDEX IF NOT EXISTS idx_offers_status ON _atlas.job_offers(status);
