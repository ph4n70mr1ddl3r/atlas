-- ============================================================================
-- Tax Registration Management
-- Oracle Fusion Cloud ERP: Financials > Tax > Tax Registrations
--
-- Manages taxpayer identification numbers (TIN, VAT, GST, EIN, etc.)
-- for first-party legal entities and third-party organizations across
-- tax jurisdictions, with validation, status tracking, and compliance.
--
-- Lifecycle: pending → active → suspended → deregistered / expired
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Tax Registrations
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.tax_registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,

    -- Registration details
    registration_number VARCHAR(100) NOT NULL,
    registration_type VARCHAR(30) NOT NULL
        CHECK (registration_type IN ('tin', 'vat', 'gst', 'ein', 'sst', 'pan', 'cst',
                                      'sales_tax', 'withholding_tax', 'excise', 'customs', 'other')),
    tax_purpose VARCHAR(30) NOT NULL DEFAULT 'both'
        CHECK (tax_purpose IN ('input_tax', 'output_tax', 'both', 'reporting_only',
                               'withholding', 'reverse_charge', 'intracommunity')),

    -- Party information
    party_type VARCHAR(20) NOT NULL
        CHECK (party_type IN ('first_party', 'third_party')),
    party_id UUID,
    party_name VARCHAR(300),

    -- Jurisdiction
    jurisdiction_code VARCHAR(50) NOT NULL,
    country_code VARCHAR(2) NOT NULL,  -- ISO 3166-1 alpha-2
    state_code VARCHAR(10),

    -- Effectiveness
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('active', 'suspended', 'deregistered', 'expired', 'pending')),
    effective_from DATE NOT NULL,
    effective_to DATE,
    is_default BOOLEAN NOT NULL DEFAULT false,

    -- Reporting
    reporting_name VARCHAR(300),
    legal_entity_id UUID,

    -- Validation
    validation_status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (validation_status IN ('pending', 'validated', 'failed', 'not_applicable')),
    last_validated_at TIMESTAMPTZ,
    validation_errors JSONB DEFAULT '[]',

    -- Source
    source VARCHAR(20) NOT NULL DEFAULT 'manual'
        CHECK (source IN ('manual', 'import', 'integration', 'migration')),

    -- Deregistration
    deregistration_date DATE,
    deregistered_by UUID,
    deregistration_reason TEXT,

    -- Audit
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(organization_id, registration_number)
);

CREATE INDEX IF NOT EXISTS idx_tax_reg_org
    ON _atlas.tax_registrations(organization_id);
CREATE INDEX IF NOT EXISTS idx_tax_reg_status
    ON _atlas.tax_registrations(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_tax_reg_party_type
    ON _atlas.tax_registrations(organization_id, party_type);
CREATE INDEX IF NOT EXISTS idx_tax_reg_country
    ON _atlas.tax_registrations(organization_id, country_code);
CREATE INDEX IF NOT EXISTS idx_tax_reg_jurisdiction
    ON _atlas.tax_registrations(organization_id, jurisdiction_code);
CREATE INDEX IF NOT EXISTS idx_tax_reg_type
    ON _atlas.tax_registrations(organization_id, registration_type);

-- ============================================================================
-- Tax Registration Activities (Audit Trail)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.tax_registration_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registration_id UUID NOT NULL REFERENCES _atlas.tax_registrations(id) ON DELETE CASCADE,
    activity_type VARCHAR(30) NOT NULL,
    description TEXT,
    old_status VARCHAR(20),
    new_status VARCHAR(20),
    performed_by UUID,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_tax_reg_act_reg
    ON _atlas.tax_registration_activities(registration_id);
CREATE INDEX IF NOT EXISTS idx_tax_reg_act_type
    ON _atlas.tax_registration_activities(registration_id, activity_type);
