-- Bank Account Transfer (Internal Fund Transfers)
-- Oracle Fusion equivalent: Financials > Cash Management > Bank Account Transfers
-- Migration 103

-- Bank account transfer types
CREATE TABLE IF NOT EXISTS _atlas.bank_transfer_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    settlement_method VARCHAR(50) NOT NULL DEFAULT 'immediate', -- immediate, scheduled, batch
    requires_approval BOOLEAN DEFAULT true,
    approval_threshold DECIMAL(18,2),
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Bank account transfers
CREATE TABLE IF NOT EXISTS _atlas.bank_account_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    transfer_number VARCHAR(100) NOT NULL,
    transfer_type_id UUID REFERENCES _atlas.bank_transfer_types(id),
    from_bank_account_id UUID NOT NULL,
    from_bank_account_number VARCHAR(50),
    from_bank_name VARCHAR(200),
    to_bank_account_id UUID NOT NULL,
    to_bank_account_number VARCHAR(50),
    to_bank_name VARCHAR(200),
    amount DECIMAL(18,2) NOT NULL,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    exchange_rate DECIMAL(18,6),
    from_currency VARCHAR(10),
    to_currency VARCHAR(10),
    transferred_amount DECIMAL(18,2),
    transfer_date DATE NOT NULL,
    value_date DATE,
    settlement_date DATE,
    reference_number VARCHAR(100),
    description TEXT,
    purpose VARCHAR(200),
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, submitted, approved, in_transit, completed, cancelled, reversed, failed
    priority VARCHAR(20) DEFAULT 'normal', -- low, normal, high, urgent
    from_journal_id UUID,
    to_journal_id UUID,
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    completed_by UUID,
    completed_at TIMESTAMPTZ,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    failure_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, transfer_number)
);

-- Transfer audit trail
CREATE TABLE IF NOT EXISTS _atlas.bank_transfer_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    transfer_id UUID NOT NULL REFERENCES _atlas.bank_account_transfers(id),
    action VARCHAR(50) NOT NULL,
    from_status VARCHAR(50),
    to_status VARCHAR(50),
    performed_by UUID,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_bank_transfer_types_org ON _atlas.bank_transfer_types(organization_id);
CREATE INDEX IF NOT EXISTS idx_bank_transfers_org ON _atlas.bank_account_transfers(organization_id);
CREATE INDEX IF NOT EXISTS idx_bank_transfers_status ON _atlas.bank_account_transfers(status);
CREATE INDEX IF NOT EXISTS idx_bank_transfers_from_account ON _atlas.bank_account_transfers(from_bank_account_id);
CREATE INDEX IF NOT EXISTS idx_bank_transfers_to_account ON _atlas.bank_account_transfers(to_bank_account_id);
CREATE INDEX IF NOT EXISTS idx_bank_transfer_audit_transfer ON _atlas.bank_transfer_audit(transfer_id);
