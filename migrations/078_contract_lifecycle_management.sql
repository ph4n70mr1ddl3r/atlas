-- Contract Lifecycle Management (Oracle Fusion Enterprise Contracts)
-- Provides: Contract templates, clause library, contract management with parties,
-- milestones, deliverables, amendments, risk assessment, and renewal tracking.
--
-- Oracle Fusion equivalent: Enterprise Contracts > Contract Management

-- ============================================================================
-- Contract Types: categorize contracts (sales, procurement, service, nda, etc.)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_contract_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    contract_category VARCHAR(50) NOT NULL DEFAULT 'general', -- sales, procurement, service, nda, partnership, employment, general
    default_duration_days INT,
    requires_approval BOOLEAN DEFAULT true,
    is_auto_renew BOOLEAN DEFAULT false,
    risk_scoring_enabled BOOLEAN DEFAULT false,
    status VARCHAR(50) DEFAULT 'active',                      -- active, inactive
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Clause Library: reusable contract clauses and terms
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_clauses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    body TEXT NOT NULL,
    clause_type VARCHAR(50) NOT NULL DEFAULT 'standard',      -- standard, optional, mandatory, boilerplate
    clause_category VARCHAR(50) DEFAULT 'general',            -- indemnification, confidentiality, termination, payment, liability, general
    applicability VARCHAR(50) DEFAULT 'all',                  -- all, sales, procurement, service
    is_locked BOOLEAN DEFAULT false,                          -- locked clauses cannot be modified in contracts
    version INT DEFAULT 1,
    status VARCHAR(50) DEFAULT 'active',                      -- active, deprecated, archived
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- ============================================================================
-- Contract Templates: pre-defined contract structures
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    contract_type_id UUID REFERENCES _atlas.clm_contract_types(id),
    default_currency VARCHAR(10) DEFAULT 'USD',
    default_duration_days INT,
    terms_and_conditions TEXT,
    is_standard BOOLEAN DEFAULT false,
    status VARCHAR(50) DEFAULT 'active',                      -- active, inactive, archived
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Template Clauses: which clauses belong to which templates
CREATE TABLE IF NOT EXISTS _atlas.clm_template_clauses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES _atlas.clm_templates(id) ON DELETE CASCADE,
    clause_id UUID NOT NULL REFERENCES _atlas.clm_clauses(id),
    section VARCHAR(100),                                     -- e.g., "payment_terms", "liability"
    display_order INT DEFAULT 0,
    is_required BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(template_id, clause_id)
);

-- ============================================================================
-- Contracts: the main contract header
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    contract_type_id UUID REFERENCES _atlas.clm_contract_types(id),
    template_id UUID REFERENCES _atlas.clm_templates(id),
    contract_category VARCHAR(50) NOT NULL DEFAULT 'general', -- sales, procurement, service, nda, partnership, employment, general
    currency VARCHAR(10) DEFAULT 'USD',
    total_value NUMERIC(18,4) DEFAULT 0,
    start_date DATE,
    end_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',              -- draft, in_review, pending_approval, approved, active, suspended, expired, terminated, completed, cancelled
    priority VARCHAR(20) DEFAULT 'normal',                    -- low, normal, high, critical
    risk_score INT,                                            -- 0-100
    risk_level VARCHAR(20),                                    -- low, medium, high, critical
    parent_contract_id UUID REFERENCES _atlas.clm_contracts(id),
    renewal_type VARCHAR(30) DEFAULT 'none',                  -- none, automatic, manual, with_notice
    auto_renew_months INT,
    renewal_notice_days INT DEFAULT 30,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    approved_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, contract_number)
);

CREATE INDEX IF NOT EXISTS idx_clm_contracts_type ON _atlas.clm_contracts(contract_type_id);
CREATE INDEX IF NOT EXISTS idx_clm_contracts_status ON _atlas.clm_contracts(status);
CREATE INDEX IF NOT EXISTS idx_clm_contracts_category ON _atlas.clm_contracts(contract_category);
CREATE INDEX IF NOT EXISTS idx_clm_contracts_parent ON _atlas.clm_contracts(parent_contract_id);

-- ============================================================================
-- Contract Parties: internal/external parties to a contract
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_contract_parties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.clm_contracts(id) ON DELETE CASCADE,
    party_type VARCHAR(30) NOT NULL DEFAULT 'external',       -- internal, external
    party_role VARCHAR(50) NOT NULL DEFAULT 'counterparty',   -- initiator, counterparty, beneficiary, guarantor, approver, reviewer
    party_name VARCHAR(300) NOT NULL,
    contact_name VARCHAR(300),
    contact_email VARCHAR(500),
    contact_phone VARCHAR(100),
    entity_reference VARCHAR(100),                            -- reference to customer/supplier/employee in other modules
    is_primary BOOLEAN DEFAULT false,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_clm_parties_contract ON _atlas.clm_contract_parties(contract_id);

-- ============================================================================
-- Contract Clauses: actual clauses used in a specific contract
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_contract_clauses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.clm_contracts(id) ON DELETE CASCADE,
    clause_id UUID REFERENCES _atlas.clm_clauses(id),
    section VARCHAR(100),
    title VARCHAR(500) NOT NULL,
    body TEXT NOT NULL,
    clause_type VARCHAR(50) DEFAULT 'standard',
    display_order INT DEFAULT 0,
    is_modified BOOLEAN DEFAULT false,                         -- true if edited from template
    original_body TEXT,                                        -- original clause text before modification
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_clm_cc_contract ON _atlas.clm_contract_clauses(contract_id);

-- ============================================================================
-- Contract Milestones: key dates and events in the contract lifecycle
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_contract_milestones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.clm_contracts(id) ON DELETE CASCADE,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    milestone_type VARCHAR(50) NOT NULL DEFAULT 'event',      -- event, payment, delivery, review, approval, renewal, termination
    due_date DATE,
    completed_date DATE,
    amount NUMERIC(18,4),
    currency VARCHAR(10) DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',            -- pending, in_progress, completed, overdue, cancelled
    responsible_party_id UUID REFERENCES _atlas.clm_contract_parties(id),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_clm_milestones_contract ON _atlas.clm_contract_milestones(contract_id);
CREATE INDEX IF NOT EXISTS idx_clm_milestones_status ON _atlas.clm_contract_milestones(status);

-- ============================================================================
-- Contract Deliverables: tangible outputs tied to a contract
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_contract_deliverables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.clm_contracts(id) ON DELETE CASCADE,
    milestone_id UUID REFERENCES _atlas.clm_contract_milestones(id),
    name VARCHAR(300) NOT NULL,
    description TEXT,
    deliverable_type VARCHAR(50) NOT NULL DEFAULT 'document', -- document, product, service, report, payment
    quantity NUMERIC(18,6) DEFAULT 1,
    unit_of_measure VARCHAR(50) DEFAULT 'each',
    due_date DATE,
    completed_date DATE,
    acceptance_date DATE,
    amount NUMERIC(18,4),
    currency VARCHAR(10) DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',            -- pending, submitted, under_review, accepted, rejected, overdue, cancelled
    accepted_by UUID,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_clm_deliv_contract ON _atlas.clm_contract_deliverables(contract_id);
CREATE INDEX IF NOT EXISTS idx_clm_deliv_status ON _atlas.clm_contract_deliverables(status);

-- ============================================================================
-- Contract Amendments: changes to an active contract
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_contract_amendments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.clm_contracts(id) ON DELETE CASCADE,
    amendment_number VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    amendment_type VARCHAR(50) NOT NULL DEFAULT 'modification', -- modification, extension, renewal, termination, scope_change
    previous_value TEXT,
    new_value TEXT,
    effective_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',              -- draft, pending_approval, approved, rejected, cancelled
    approved_by UUID,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, contract_id, amendment_number)
);

CREATE INDEX IF NOT EXISTS idx_clm_amend_contract ON _atlas.clm_contract_amendments(contract_id);

-- ============================================================================
-- Contract Risk Assessments: risk scoring for contracts
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.clm_contract_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    contract_id UUID NOT NULL REFERENCES _atlas.clm_contracts(id) ON DELETE CASCADE,
    risk_category VARCHAR(50) NOT NULL,                       -- financial, legal, operational, compliance, reputational
    risk_description TEXT NOT NULL,
    probability VARCHAR(20) NOT NULL DEFAULT 'medium',        -- low, medium, high, very_high
    impact VARCHAR(20) NOT NULL DEFAULT 'medium',             -- low, medium, high, critical
    mitigation_strategy TEXT,
    residual_risk VARCHAR(20),
    owner_id UUID,
    status VARCHAR(50) NOT NULL DEFAULT 'identified',         -- identified, assessing, mitigated, accepted, transferred, closed
    metadata JSONB DEFAULT '{}'::jsonb,
    assessed_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_clm_risk_contract ON _atlas.clm_contract_risks(contract_id);
