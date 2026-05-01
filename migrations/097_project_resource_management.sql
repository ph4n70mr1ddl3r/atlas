-- ============================================================================
-- Project Resource Management (Oracle Fusion Cloud: Project Management)
-- ============================================================================
-- Manages resource profiles, resource requests, assignments, utilization
-- tracking, and resource analytics for project staffing.
-- ============================================================================

-- Resource Profiles
CREATE TABLE IF NOT EXISTS _atlas.resource_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    resource_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    email VARCHAR(200) NOT NULL DEFAULT '',
    resource_type VARCHAR(50) NOT NULL DEFAULT 'employee',
    department VARCHAR(200) NOT NULL DEFAULT '',
    job_title VARCHAR(200) NOT NULL DEFAULT '',
    skills TEXT NOT NULL DEFAULT '',
    certifications TEXT NOT NULL DEFAULT '',
    availability_status VARCHAR(50) NOT NULL DEFAULT 'available',
    available_hours_per_week DOUBLE PRECISION NOT NULL DEFAULT 40,
    cost_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    cost_rate_currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    bill_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    bill_rate_currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    location VARCHAR(200) NOT NULL DEFAULT '',
    manager_id UUID,
    manager_name VARCHAR(200) NOT NULL DEFAULT '',
    hire_date DATE,
    notes TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, resource_number)
);

CREATE INDEX IF NOT EXISTS idx_resource_profiles_org ON _atlas.resource_profiles(organization_id);
CREATE INDEX IF NOT EXISTS idx_resource_profiles_status ON _atlas.resource_profiles(availability_status);
CREATE INDEX IF NOT EXISTS idx_resource_profiles_type ON _atlas.resource_profiles(resource_type);

-- Resource Requests
CREATE TABLE IF NOT EXISTS _atlas.resource_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    request_number VARCHAR(50) NOT NULL,
    project_id UUID,
    project_name VARCHAR(200) NOT NULL DEFAULT '',
    project_number VARCHAR(50) NOT NULL DEFAULT '',
    requested_role VARCHAR(200) NOT NULL DEFAULT '',
    required_skills TEXT NOT NULL DEFAULT '',
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    hours_per_week DOUBLE PRECISION NOT NULL DEFAULT 40,
    total_planned_hours DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_cost_rate DOUBLE PRECISION,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    resource_type_preference VARCHAR(30) NOT NULL DEFAULT 'any',
    location_requirement VARCHAR(200) NOT NULL DEFAULT '',
    fulfilled_by UUID,
    fulfilled_at TIMESTAMPTZ,
    notes TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, request_number)
);

CREATE INDEX IF NOT EXISTS idx_resource_requests_org ON _atlas.resource_requests(organization_id);
CREATE INDEX IF NOT EXISTS idx_resource_requests_status ON _atlas.resource_requests(status);
CREATE INDEX IF NOT EXISTS idx_resource_requests_project ON _atlas.resource_requests(project_id);

-- Resource Assignments
CREATE TABLE IF NOT EXISTS _atlas.resource_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    assignment_number VARCHAR(50) NOT NULL,
    resource_id UUID NOT NULL REFERENCES _atlas.resource_profiles(id),
    resource_name VARCHAR(200) NOT NULL DEFAULT '',
    resource_email VARCHAR(200) NOT NULL DEFAULT '',
    project_id UUID,
    project_name VARCHAR(200) NOT NULL DEFAULT '',
    project_number VARCHAR(50) NOT NULL DEFAULT '',
    request_id UUID REFERENCES _atlas.resource_requests(id),
    role VARCHAR(200) NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'planned',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    planned_hours DOUBLE PRECISION NOT NULL DEFAULT 0,
    actual_hours DOUBLE PRECISION NOT NULL DEFAULT 0,
    remaining_hours DOUBLE PRECISION NOT NULL DEFAULT 0,
    utilization_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
    cost_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    bill_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    notes TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, assignment_number)
);

CREATE INDEX IF NOT EXISTS idx_resource_assignments_org ON _atlas.resource_assignments(organization_id);
CREATE INDEX IF NOT EXISTS idx_resource_assignments_resource ON _atlas.resource_assignments(resource_id);
CREATE INDEX IF NOT EXISTS idx_resource_assignments_project ON _atlas.resource_assignments(project_id);
CREATE INDEX IF NOT EXISTS idx_resource_assignments_status ON _atlas.resource_assignments(status);

-- Utilization Entries
CREATE TABLE IF NOT EXISTS _atlas.utilization_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    assignment_id UUID NOT NULL REFERENCES _atlas.resource_assignments(id),
    resource_id UUID NOT NULL REFERENCES _atlas.resource_profiles(id),
    entry_date DATE NOT NULL,
    hours_worked DOUBLE PRECISION NOT NULL DEFAULT 0,
    description TEXT NOT NULL DEFAULT '',
    billable BOOLEAN NOT NULL DEFAULT true,
    status VARCHAR(20) NOT NULL DEFAULT 'submitted',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    notes TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_utilization_entries_org ON _atlas.utilization_entries(organization_id);
CREATE INDEX IF NOT EXISTS idx_utilization_entries_assignment ON _atlas.utilization_entries(assignment_id);
CREATE INDEX IF NOT EXISTS idx_utilization_entries_resource ON _atlas.utilization_entries(resource_id);
CREATE INDEX IF NOT EXISTS idx_utilization_entries_date ON _atlas.utilization_entries(entry_date);
