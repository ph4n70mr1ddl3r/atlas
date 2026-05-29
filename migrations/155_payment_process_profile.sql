-- 155_payment_process_profile.sql
-- Oracle Fusion Financial Feature: Payment Process Profile (PPP)
-- Defines how payments are grouped, formatted, and transmitted.

CREATE TABLE IF NOT EXISTS financials.payment_process_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    profile_code VARCHAR(100) NOT NULL,
    profile_name VARCHAR(200) NOT NULL,
    
    -- Processing Rules
    payment_format VARCHAR(100) NOT NULL, -- e.g., 'NACHA', 'SEPA', 'SWIFT_MT103'
    processing_type VARCHAR(50) NOT NULL DEFAULT 'ELECTRONIC', -- 'ELECTRONIC', 'PRINTED'
    
    -- Grouping Rules (boolean flags)
    group_by_due_date BOOLEAN NOT NULL DEFAULT false,
    group_by_payment_currency BOOLEAN NOT NULL DEFAULT true,
    group_by_payee BOOLEAN NOT NULL DEFAULT true,
    
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, profile_code)
);

CREATE TABLE IF NOT EXISTS financials.payment_profile_usage_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES financials.payment_process_profiles(id) ON DELETE CASCADE,
    -- Rules restricting where this profile can be used
    business_unit_id UUID,
    internal_bank_account_id UUID,
    currency_code VARCHAR(3),
    payment_method VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT now()
);

COMMENT ON TABLE financials.payment_process_profiles IS 'Payment Process Profiles - configures formatting, grouping, and transmission for AP payments';
COMMENT ON TABLE financials.payment_profile_usage_rules IS 'Usage Rules - limits the applicability of a payment process profile based on context';
