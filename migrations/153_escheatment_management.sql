-- 153_escheatment_management.sql
-- Oracle Fusion Financial Feature: Escheatment (Unclaimed Property)
-- Manages the identification and transfer of uncashed payments to legal authorities.

CREATE TABLE IF NOT EXISTS financials.escheatment_authorities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    authority_name VARCHAR(200) NOT NULL,
    jurisdiction VARCHAR(100) NOT NULL,
    remittance_supplier_id UUID, -- Supplier representing the authority for remittance
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, authority_name)
);

CREATE TABLE IF NOT EXISTS financials.escheated_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_id UUID NOT NULL, -- Reference to the original AP Payment
    authority_id UUID NOT NULL REFERENCES financials.escheatment_authorities(id),
    escheated_amount DECIMAL(15,2) NOT NULL,
    escheatment_date DATE NOT NULL DEFAULT CURRENT_DATE,
    remittance_invoice_id UUID, -- Reference to the new AP invoice created to pay the authority
    status VARCHAR(20) NOT NULL DEFAULT 'INITIATED', -- 'INITIATED', 'REMITTED'
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

COMMENT ON TABLE financials.escheatment_authorities IS 'Escheatment Authorities - Legal bodies receiving unclaimed properties';
COMMENT ON TABLE financials.escheated_payments IS 'Escheated Payments - Audit trail of uncashed payments transferred to an authority';
