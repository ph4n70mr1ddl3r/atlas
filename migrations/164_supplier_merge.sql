-- 164_supplier_merge.sql
-- Oracle Fusion Financial Feature: Supplier Merge (Payables)
-- Manages the process of merging a duplicate supplier into a primary supplier.

CREATE TABLE IF NOT EXISTS financials.ap_supplier_merges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    
    -- The duplicate supplier to be merged and inactivated
    from_supplier_id UUID NOT NULL,
    -- The primary supplier that will inherit the transactions
    to_supplier_id UUID NOT NULL,
    
    merge_date DATE NOT NULL DEFAULT CURRENT_DATE,
    
    -- Options
    transfer_invoices BOOLEAN NOT NULL DEFAULT true,
    transfer_purchase_orders BOOLEAN NOT NULL DEFAULT true,
    inactivate_from_supplier BOOLEAN NOT NULL DEFAULT true,
    
    status VARCHAR(20) NOT NULL DEFAULT 'COMPLETED', -- 'DRAFT', 'PROCESSING', 'COMPLETED', 'FAILED'
    merge_reason VARCHAR(200),
    
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS financials.ap_supplier_merge_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merge_id UUID NOT NULL REFERENCES financials.ap_supplier_merges(id) ON DELETE CASCADE,
    
    entity_type VARCHAR(50) NOT NULL, -- e.g., 'INVOICE', 'PURCHASE_ORDER'
    entity_id UUID NOT NULL,
    
    created_at TIMESTAMPTZ DEFAULT now()
);

COMMENT ON TABLE financials.ap_supplier_merges IS 'Supplier Merges - tracks the merging of duplicate suppliers in Payables';
COMMENT ON TABLE financials.ap_supplier_merge_history IS 'Audit trail of specific transactions (invoices, POs) transferred during a merge';
