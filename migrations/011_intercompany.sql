-- Atlas ERP - Intercompany Transactions
-- Oracle Fusion Cloud ERP: Intercompany > Intercompany Transactions
--
-- Manages financial transactions between legal entities / business units
-- within the same enterprise. Supports:
-- - Intercompany invoices (AP/AR between entities)
-- - Automatic due-to/due-from balancing entries
-- - Intercompany transaction batches
-- - Settlement tracking and netting
--
-- This is a standard Oracle Fusion Cloud ERP feature required by every
-- multi-entity organization. Oracle Fusion's Intercompany module handles
-- the creation, approval, posting, and settlement of intercompany
-- transactions with automatic elimination during consolidation.

-- ============================================================================
-- Intercompany Transaction Batches
-- Oracle Fusion: Group intercompany transactions for processing
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.intercompany_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Batch number (auto-generated or manual)
    batch_number VARCHAR(50) NOT NULL,
    -- Batch description
    description TEXT,
    -- Status: 'draft', 'submitted', 'approved', 'posted', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- From entity (initiator)
    from_entity_id UUID NOT NULL,
    from_entity_name VARCHAR(200) NOT NULL,
    -- To entity (receiver)
    to_entity_id UUID NOT NULL,
    to_entity_name VARCHAR(200) NOT NULL,
    -- Currency
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Totals
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    transaction_count INT NOT NULL DEFAULT 0,
    -- Posting references
    from_journal_id UUID,
    to_journal_id UUID,
    -- Accounting date
    accounting_date DATE,
    posted_at TIMESTAMPTZ,
    -- Rejection
    rejected_reason TEXT,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    approved_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, batch_number)
);

CREATE INDEX idx_ic_batches_org ON _atlas.intercompany_batches(organization_id);
CREATE INDEX idx_ic_batches_status ON _atlas.intercompany_batches(organization_id, status);
CREATE INDEX idx_ic_batches_from_entity ON _atlas.intercompany_batches(from_entity_id);
CREATE INDEX idx_ic_batches_to_entity ON _atlas.intercompany_batches(to_entity_id);

-- ============================================================================
-- Intercompany Transactions
-- Oracle Fusion: Individual intercompany transaction lines
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.intercompany_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Batch reference
    batch_id UUID NOT NULL REFERENCES _atlas.intercompany_batches(id),
    -- Transaction number (auto-generated)
    transaction_number VARCHAR(50) NOT NULL,
    -- Transaction type: 'invoice', 'journal_entry', 'payment', 'charge', 'allocation'
    transaction_type VARCHAR(30) NOT NULL DEFAULT 'invoice',
    -- Description
    description TEXT,
    -- From entity (initiator / debtor)
    from_entity_id UUID NOT NULL,
    from_entity_name VARCHAR(200) NOT NULL,
    -- To entity (receiver / creditor)
    to_entity_id UUID NOT NULL,
    to_entity_name VARCHAR(200) NOT NULL,
    -- Amount and currency
    amount NUMERIC(18,2) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    exchange_rate NUMERIC(18,6),
    -- Account references (from initiator's perspective)
    from_debit_account VARCHAR(50),
    from_credit_account VARCHAR(50),
    -- Account references (from receiver's perspective)
    to_debit_account VARCHAR(50),
    to_credit_account VARCHAR(50),
    -- Intercompany due-to/due-from accounts
    from_ic_account VARCHAR(50) NOT NULL,  -- Due-To (liability on from side)
    to_ic_account VARCHAR(50) NOT NULL,    -- Due-From (asset on to side)
    -- Status: 'draft', 'approved', 'posted', 'settled', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Dates
    transaction_date DATE NOT NULL DEFAULT CURRENT_DATE,
    due_date DATE,
    settlement_date DATE,
    -- Reference to source document
    source_entity_type VARCHAR(100),
    source_entity_id UUID,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, transaction_number)
);

CREATE INDEX idx_ic_transactions_org ON _atlas.intercompany_transactions(organization_id);
CREATE INDEX idx_ic_transactions_batch ON _atlas.intercompany_transactions(batch_id);
CREATE INDEX idx_ic_transactions_status ON _atlas.intercompany_transactions(organization_id, status);
CREATE INDEX idx_ic_transactions_from ON _atlas.intercompany_transactions(from_entity_id);
CREATE INDEX idx_ic_transactions_to ON _atlas.intercompany_transactions(to_entity_id);
CREATE INDEX idx_ic_transactions_date ON _atlas.intercompany_transactions(transaction_date);
CREATE INDEX idx_ic_transactions_type ON _atlas.intercompany_transactions(transaction_type);

-- ============================================================================
-- Intercompany Settlements
-- Oracle Fusion: Track settlement / netting of intercompany balances
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.intercompany_settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Settlement number
    settlement_number VARCHAR(50) NOT NULL,
    -- Settlement method: 'cash', 'netting', 'offset'
    settlement_method VARCHAR(20) NOT NULL DEFAULT 'cash',
    -- From and to entities
    from_entity_id UUID NOT NULL,
    to_entity_id UUID NOT NULL,
    -- Amount settled
    settled_amount NUMERIC(18,2) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Payment reference
    payment_reference VARCHAR(100),
    -- Status: 'pending', 'completed', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Settlement date
    settlement_date DATE NOT NULL DEFAULT CURRENT_DATE,
    -- Linked transactions
    transaction_ids JSONB DEFAULT '[]',
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, settlement_number)
);

CREATE INDEX idx_ic_settlements_org ON _atlas.intercompany_settlements(organization_id);
CREATE INDEX idx_ic_settlements_from ON _atlas.intercompany_settlements(from_entity_id);
CREATE INDEX idx_ic_settlements_to ON _atlas.intercompany_settlements(to_entity_id);
CREATE INDEX idx_ic_settlements_status ON _atlas.intercompany_settlements(organization_id, status);

-- ============================================================================
-- Intercompany Balances (materialized view of outstanding balances)
-- Oracle Fusion: Intercompany balances dashboard
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.intercompany_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    from_entity_id UUID NOT NULL,
    to_entity_id UUID NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Running balance
    total_outstanding NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_posted NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_settled NUMERIC(18,2) NOT NULL DEFAULT 0,
    open_transaction_count INT NOT NULL DEFAULT 0,
    -- Period snapshot
    as_of_date DATE NOT NULL DEFAULT CURRENT_DATE,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, from_entity_id, to_entity_id, currency_code, as_of_date)
);

CREATE INDEX idx_ic_balances_org ON _atlas.intercompany_balances(organization_id);
CREATE INDEX idx_ic_balances_entities ON _atlas.intercompany_balances(from_entity_id, to_entity_id);
CREATE INDEX idx_ic_balances_date ON _atlas.intercompany_balances(as_of_date);
