-- 106_accounting_hub.sql
-- Accounting Hub tables for external system integration
-- Oracle Fusion equivalent: Financials > Accounting Hub

CREATE TABLE IF NOT EXISTS _atlas.external_systems (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    system_type VARCHAR(30) NOT NULL DEFAULT 'custom', -- erp, billing, pos, banking, insurance, custom
    connection_config JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_event_received TIMESTAMPTZ,
    total_events_received INT NOT NULL DEFAULT 0,
    total_events_processed INT NOT NULL DEFAULT 0,
    total_events_failed INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.accounting_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    event_number VARCHAR(50) NOT NULL,
    external_system_id UUID NOT NULL,
    external_system_code VARCHAR(100),
    event_type VARCHAR(100) NOT NULL,
    event_class VARCHAR(50) NOT NULL, -- invoice, payment, adjustment, transfer, custom
    source_event_id VARCHAR(200) NOT NULL,
    payload JSONB DEFAULT '{}',
    transaction_attributes JSONB DEFAULT '{}',
    accounting_method_id UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'received', -- received, validated, accounted, posted, transferred, error
    error_message TEXT,
    journal_entry_id UUID,
    event_date DATE NOT NULL,
    accounting_date DATE,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    total_amount DOUBLE PRECISION,
    description TEXT,
    processed_by UUID,
    processed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, event_number)
);

CREATE TABLE IF NOT EXISTS _atlas.transaction_mapping_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    external_system_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    event_type VARCHAR(100) NOT NULL,
    event_class VARCHAR(50) NOT NULL,
    priority INT NOT NULL DEFAULT 10,
    conditions JSONB DEFAULT '{}',
    field_mappings JSONB DEFAULT '{}',
    accounting_method_id UUID,
    stop_on_match BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_external_systems_org ON _atlas.external_systems(organization_id);
CREATE INDEX IF NOT EXISTS idx_accounting_events_org ON _atlas.accounting_events(organization_id);
CREATE INDEX IF NOT EXISTS idx_accounting_events_status ON _atlas.accounting_events(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_accounting_events_system ON _atlas.accounting_events(external_system_id);
CREATE INDEX IF NOT EXISTS idx_mapping_rules_system ON _atlas.transaction_mapping_rules(external_system_id);
