-- 160_customer_tax_exemptions.sql
-- Oracle Fusion Financial Feature: Customer Tax Exemptions
-- Manages tax exemption certificates and logic for exempting specific customers from taxes.

CREATE TABLE IF NOT EXISTS financials.customer_tax_exemptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    customer_id UUID NOT NULL,
    
    -- Specific tax regime and tax that this exemption applies to
    tax_regime_code VARCHAR(100) NOT NULL,
    tax_code VARCHAR(100), -- If null, applies to all taxes in the regime
    
    -- Exemption details
    exemption_certificate_number VARCHAR(100) NOT NULL,
    exemption_reason_code VARCHAR(100) NOT NULL, -- E.g., 'GOVERNMENT', 'RESELLER', 'EDUCATIONAL'
    exemption_percentage DECIMAL(5,2) NOT NULL DEFAULT 100.00, -- Usually 100%, but could be partial
    
    start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    end_date DATE,
    
    status VARCHAR(20) NOT NULL DEFAULT 'PRIMARY', -- 'PRIMARY', 'MANUAL', 'UNAPPROVED', 'EXPIRED'
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, customer_id, tax_regime_code, tax_code)
);

COMMENT ON TABLE financials.customer_tax_exemptions IS 'Customer Tax Exemptions - tracks tax exemption certificates to prevent tax calculation for specific customers';
