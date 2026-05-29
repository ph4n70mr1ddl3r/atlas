-- 161_sla_mapping_sets.sql
-- Oracle Fusion Financial Feature: Subledger Accounting (SLA) Mapping Sets
-- Derives GL segment values from specific transaction input attributes.

CREATE TABLE IF NOT EXISTS financials.sla_mapping_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    mapping_set_code VARCHAR(100) NOT NULL,
    mapping_set_name VARCHAR(200) NOT NULL,
    
    -- The output type, e.g., 'SEGMENT', 'VALUE_SET'
    output_type VARCHAR(50) NOT NULL,
    
    -- The specific segment this mapping set outputs, e.g., 'Cost Center', 'Natural Account'
    output_segment VARCHAR(50) NOT NULL,
    
    -- If true, returns a default value when no specific input rule matches
    use_default_value BOOLEAN NOT NULL DEFAULT false,
    default_output_value VARCHAR(100),
    
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, mapping_set_code)
);

CREATE TABLE IF NOT EXISTS financials.sla_mapping_set_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mapping_set_id UUID NOT NULL REFERENCES financials.sla_mapping_sets(id) ON DELETE CASCADE,
    
    -- The value from the transaction source, e.g., an Item Category code 'IT-HDW'
    input_value VARCHAR(200) NOT NULL, 
    
    -- The derived segment value, e.g., '1200'
    output_value VARCHAR(100) NOT NULL, 
    
    effective_start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_end_date DATE,
    UNIQUE(mapping_set_id, input_value, effective_start_date)
);

COMMENT ON TABLE financials.sla_mapping_sets IS 'SLA Mapping Sets - defines sets of rules to map transaction data to GL segments';
COMMENT ON TABLE financials.sla_mapping_set_rules IS 'SLA Mapping Set Rules - individual input-to-output mappings for a mapping set';
