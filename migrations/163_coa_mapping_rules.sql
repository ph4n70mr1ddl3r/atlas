-- 163_coa_mapping_rules.sql
-- Oracle Fusion Financial Feature: GL Chart of Accounts (COA) Mapping
-- Defines rules for translating account combinations from a Source COA to a Target COA.

CREATE TABLE IF NOT EXISTS financials.gl_coa_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    mapping_name VARCHAR(200) NOT NULL,
    
    source_coa_id UUID NOT NULL,
    target_coa_id UUID NOT NULL,
    
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, mapping_name)
);

CREATE TABLE IF NOT EXISTS financials.gl_coa_mapping_segment_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mapping_id UUID NOT NULL REFERENCES financials.gl_coa_mappings(id) ON DELETE CASCADE,
    
    -- The segment in the Target COA being populated (e.g., 'Target_Company', 'Target_Account')
    target_segment_name VARCHAR(50) NOT NULL,
    
    -- Action determines how the target segment is derived:
    -- 'COPY_FROM_SOURCE' (1:1 mapping from a source segment)
    -- 'ASSIGN_VALUE' (assigns a constant value)
    action_type VARCHAR(50) NOT NULL,
    
    -- Used if action is COPY_FROM_SOURCE (e.g., 'Source_Company')
    source_segment_name VARCHAR(50),
    
    -- Used if action is ASSIGN_VALUE (e.g., '000')
    constant_value VARCHAR(100),
    
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(mapping_id, target_segment_name)
);

COMMENT ON TABLE financials.gl_coa_mappings IS 'GL Chart of Accounts Mapping - links a Source COA to a Target COA for cross-ledger transfers';
COMMENT ON TABLE financials.gl_coa_mapping_segment_rules IS 'Segment-level rules dictating how each target segment is derived from the source';
