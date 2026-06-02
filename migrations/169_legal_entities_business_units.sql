-- Migration 169: Legal Entities and Business Units
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Enterprise Structure
--
-- This migration adds core enterprise structure entities:
-- 1. Legal Entities: Legal existence for reporting and tax compliance.
-- 2. Business Units: Groups transactions and management reporting.

BEGIN;

-- ============================================================================
-- Legal Entities
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_legal_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    legal_entity_identifier VARCHAR(100),
    registration_number VARCHAR(100),
    inception_date DATE,
    registration_date DATE,
    place_of_registration VARCHAR(200),
    is_primary_legal_entity BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, name)
);

CREATE INDEX IF NOT EXISTS idx_legal_entities_org ON _atlas.fin_legal_entities(organization_id);

-- ============================================================================
-- Business Units
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.fin_business_units (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    code VARCHAR(50) NOT NULL,
    manager_id UUID, -- References employees(id)
    default_legal_entity_id UUID REFERENCES _atlas.fin_legal_entities(id),
    default_ledger_id UUID, -- References accounting_books(id)
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_business_units_org ON _atlas.fin_business_units(organization_id);
CREATE INDEX IF NOT EXISTS idx_business_units_legal_entity ON _atlas.fin_business_units(default_legal_entity_id);

COMMENT ON TABLE _atlas.fin_legal_entities IS 'Legal entities representing legal existence for reporting and tax';
COMMENT ON TABLE _atlas.fin_business_units IS 'Business units for grouping transactions and management reporting';

COMMIT;
