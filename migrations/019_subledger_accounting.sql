-- Atlas ERP - Subledger Accounting
-- Oracle Fusion Cloud ERP: Financials > General Ledger > Subledger Accounting
--
-- Bridges subledger transactions (AP, AR, Expenses, etc.) to the General Ledger.
-- Provides accounting methods, derivation rules, journal entry generation,
-- posting, and transfer to GL.
--
-- This is a core Oracle Fusion feature: every subledger transaction must
-- flow through Subledger Accounting before hitting the GL.

-- ============================================================================
-- Accounting Methods
-- Oracle Fusion: Subledger Accounting > Accounting Methods
-- Defines how a given subledger transaction type is accounted for.
-- ============================================================================
CREATE TABLE _atlas.accounting_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Which subledger application this method applies to
    -- e.g., 'payables', 'receivables', 'expenses', 'assets', 'projects'
    application VARCHAR(50) NOT NULL,

    -- Transaction type within the application
    -- e.g., 'invoice', 'payment', 'credit_memo', 'expense_report'
    transaction_type VARCHAR(50) NOT NULL,

    -- Event class: 'create', 'update', 'cancel', 'reverse'
    event_class VARCHAR(50) NOT NULL DEFAULT 'create',

    -- Whether this method automatically creates journal entries
    auto_accounting BOOLEAN NOT NULL DEFAULT true,

    -- Whether manual journal entry creation is allowed
    allow_manual_entries BOOLEAN NOT NULL DEFAULT false,

    -- Whether rounding adjustments are applied
    apply_rounding BOOLEAN NOT NULL DEFAULT true,

    -- Rounding account code for rounding differences
    rounding_account_code VARCHAR(50),

    -- Threshold for rounding (amounts below this are rounded)
    rounding_threshold NUMERIC(16, 4) DEFAULT 0.01,

    -- Whether balancing is required (debits = credits)
    require_balancing BOOLEAN NOT NULL DEFAULT true,

    -- Intercompany balancing account code
    intercompany_balancing_account VARCHAR(50),

    -- Effective dating
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, application, transaction_type, event_class)
);

CREATE INDEX idx_accounting_methods_org ON _atlas.accounting_methods(organization_id);
CREATE INDEX idx_accounting_methods_app ON _atlas.accounting_methods(application, transaction_type);

-- ============================================================================
-- Accounting Derivation Rules
-- Oracle Fusion: Subledger Accounting > Derivation Rules
-- Rules for deriving account codes from transaction attributes.
-- e.g., "If expense category is 'Travel', debit account = '6100.Travel'"
-- ============================================================================
CREATE TABLE _atlas.accounting_derivation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    accounting_method_id UUID NOT NULL REFERENCES _atlas.accounting_methods(id),

    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Line type this rule applies to: 'debit', 'credit', 'tax', 'discount'
    line_type VARCHAR(20) NOT NULL DEFAULT 'debit',

    -- Priority order (lower = higher priority; first match wins)
    priority INT NOT NULL DEFAULT 10,

    -- Condition JSON: rule is applied only when conditions match
    -- e.g., {"expense_category": "Travel", "amount_gt": 5000}
    conditions JSONB DEFAULT '{}',

    -- Source field from the transaction to derive from
    -- e.g., 'expense_category', 'cost_center', 'department'
    source_field VARCHAR(100),

    -- Derivation type:
    -- 'constant' - always use fixed_account_code
    -- 'lookup' - look up from account_derivation_lookup using source_field value
    -- 'formula' - evaluate a formula
    derivation_type VARCHAR(20) NOT NULL DEFAULT 'constant',

    -- Fixed account code for 'constant' derivation type
    fixed_account_code VARCHAR(50),

    -- Lookup table JSON for 'lookup' derivation type
    -- e.g., {"Travel": "6100", "Meals": "6200", "Office": "6300"}
    account_derivation_lookup JSONB DEFAULT '{}',

    -- Formula expression for 'formula' derivation type
    formula_expression TEXT,

    -- The resulting account code (populated after derivation)
    -- This is set during journal entry generation

    sequence INT NOT NULL DEFAULT 10,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, accounting_method_id, code)
);

CREATE INDEX idx_derivation_rules_method ON _atlas.accounting_derivation_rules(accounting_method_id);
CREATE INDEX idx_derivation_rules_org ON _atlas.accounting_derivation_rules(organization_id);

-- ============================================================================
-- Subledger Journal Entries
-- Oracle Fusion: Subledger Accounting > Journal Entries
-- The accounting representation of a subledger transaction.
-- ============================================================================
CREATE TABLE _atlas.subledger_journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,

    -- Reference to the subledger transaction
    source_application VARCHAR(50) NOT NULL,  -- 'payables', 'receivables', etc.
    source_transaction_type VARCHAR(50) NOT NULL,  -- 'invoice', 'payment', etc.
    source_transaction_id UUID NOT NULL,
    source_transaction_number VARCHAR(100),

    -- Accounting method applied
    accounting_method_id UUID REFERENCES _atlas.accounting_methods(id),

    -- Journal entry header
    entry_number VARCHAR(100) NOT NULL,
    description TEXT,
    reference_number VARCHAR(100),

    -- Accounting date (GL date)
    accounting_date DATE NOT NULL,

    -- Period name (e.g., 'Jan-2024')
    period_name VARCHAR(50),

    -- Currency
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    entered_currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    currency_conversion_date DATE,
    currency_conversion_type VARCHAR(20),  -- 'spot', 'average', 'corporate'
    currency_conversion_rate NUMERIC(20, 10),

    -- Totals
    total_debit NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_credit NUMERIC(18, 2) NOT NULL DEFAULT 0,
    entered_debit NUMERIC(18, 2) DEFAULT 0,
    entered_credit NUMERIC(18, 2) DEFAULT 0,

    -- Status: 'draft', 'accounted', 'posted', 'transferred', 'reversed', 'error'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',

    -- Error details (if status = 'error')
    error_message TEXT,

    -- Balancing
    balancing_segment VARCHAR(50),
    is_balanced BOOLEAN NOT NULL DEFAULT false,

    -- GL transfer tracking
    gl_transfer_status VARCHAR(20) DEFAULT 'pending',  -- 'pending', 'transferred', 'failed'
    gl_transfer_date TIMESTAMPTZ,
    gl_journal_entry_id UUID,  -- Reference to GL journal entry after transfer

    -- Reversal tracking
    is_reversal BOOLEAN NOT NULL DEFAULT false,
    reversal_of_id UUID,  -- Reference to original entry if this is a reversal
    reversal_reason TEXT,

    -- Original and reversal entries link to this entry
    -- (Multiple reversals could be linked to one original)

    -- Audit
    created_by UUID,
    posted_by UUID,
    accounted_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, entry_number)
);

CREATE INDEX idx_sla_entries_org ON _atlas.subledger_journal_entries(organization_id);
CREATE INDEX idx_sla_entries_source ON _atlas.subledger_journal_entries(source_application, source_transaction_type, source_transaction_id);
CREATE INDEX idx_sla_entries_status ON _atlas.subledger_journal_entries(status);
CREATE INDEX idx_sla_entries_date ON _atlas.subledger_journal_entries(accounting_date);
CREATE INDEX idx_sla_entries_period ON _atlas.subledger_journal_entries(period_name);
CREATE INDEX idx_sla_entries_gl_transfer ON _atlas.subledger_journal_entries(gl_transfer_status);

-- ============================================================================
-- Subledger Journal Entry Lines
-- Oracle Fusion: Subledger Accounting > Journal Entry Lines
-- Individual debit/credit lines within a journal entry.
-- ============================================================================
CREATE TABLE _atlas.subledger_journal_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    journal_entry_id UUID NOT NULL REFERENCES _atlas.subledger_journal_entries(id) ON DELETE CASCADE,

    line_number INT NOT NULL,

    -- Line type: 'debit', 'credit', 'tax', 'discount', 'rounding'
    line_type VARCHAR(20) NOT NULL,

    -- Account derivation
    account_code VARCHAR(50) NOT NULL,
    account_description VARCHAR(200),
    derivation_rule_id UUID REFERENCES _atlas.accounting_derivation_rules(id),

    -- Amounts (accounted currency)
    entered_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    accounted_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,

    -- Currency
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    conversion_date DATE,
    conversion_rate NUMERIC(20, 10),

    -- Descriptive flexfield attributes (extensible)
    -- These allow subledger-specific attributes to flow through
    attribute_category VARCHAR(50),  -- e.g., 'payables', 'receivables'
    attribute1 VARCHAR(150),  -- e.g., supplier number
    attribute2 VARCHAR(150),  -- e.g., invoice number
    attribute3 VARCHAR(150),  -- e.g., expense category
    attribute4 VARCHAR(150),  -- e.g., cost center
    attribute5 VARCHAR(150),  -- e.g., project number
    attribute6 VARCHAR(150),
    attribute7 VARCHAR(150),
    attribute8 VARCHAR(150),
    attribute9 VARCHAR(150),
    attribute10 VARCHAR(150),

    -- Tax reference
    tax_code VARCHAR(50),
    tax_rate NUMERIC(10, 4) DEFAULT 0,
    tax_amount NUMERIC(18, 2) DEFAULT 0,

    -- Reference to source line (in subledger transaction)
    source_line_id UUID,
    source_line_type VARCHAR(50),

    -- Reversal
    is_reversal_line BOOLEAN NOT NULL DEFAULT false,
    reversal_of_line_id UUID,

    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_sla_lines_entry ON _atlas.subledger_journal_lines(journal_entry_id);
CREATE INDEX idx_sla_lines_account ON _atlas.subledger_journal_lines(account_code);
CREATE INDEX idx_sla_lines_type ON _atlas.subledger_journal_lines(line_type);

-- ============================================================================
-- Subledger Journal Entry Distributions
-- Oracle Fusion tracks distribution lines on SLA entries
-- ============================================================================
CREATE TABLE _atlas.subledger_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    journal_entry_id UUID NOT NULL REFERENCES _atlas.subledger_journal_entries(id) ON DELETE CASCADE,
    journal_line_id UUID NOT NULL REFERENCES _atlas.subledger_journal_lines(id) ON DELETE CASCADE,

    distribution_number INT NOT NULL,

    -- Reference to original subledger distribution
    source_distribution_id UUID,
    source_distribution_type VARCHAR(50),

    -- Account override (if distribution maps to a different account)
    account_code VARCHAR(50),
    account_description VARCHAR(200),

    amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    percentage NUMERIC(10, 4) DEFAULT 100,

    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_sla_dist_entry ON _atlas.subledger_distributions(journal_entry_id);

-- ============================================================================
-- Subledger Accounting Events
-- Oracle Fusion: Subledger Accounting > Events
-- Audit trail of accounting events and processing history.
-- ============================================================================
CREATE TABLE _atlas.sla_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,

    event_number VARCHAR(100) NOT NULL,
    event_type VARCHAR(50) NOT NULL,  -- 'creation', 'modification', 'cancellation', 'reversal', 'posting', 'transfer'

    -- Reference to the subledger transaction
    source_application VARCHAR(50) NOT NULL,
    source_transaction_type VARCHAR(50) NOT NULL,
    source_transaction_id UUID NOT NULL,

    -- Reference to the journal entry (if generated)
    journal_entry_id UUID REFERENCES _atlas.subledger_journal_entries(id),

    event_date DATE NOT NULL,
    event_status VARCHAR(20) NOT NULL DEFAULT 'processed',  -- 'processed', 'error', 'skipped'

    description TEXT,
    error_message TEXT,

    processed_by UUID,
    processed_at TIMESTAMPTZ DEFAULT now(),

    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_sla_events_org ON _atlas.sla_events(organization_id);
CREATE INDEX idx_sla_events_source ON _atlas.sla_events(source_application, source_transaction_id);
CREATE INDEX idx_sla_events_type ON _atlas.sla_events(event_type);
CREATE INDEX idx_sla_events_date ON _atlas.sla_events(event_date);

-- ============================================================================
-- GL Transfer Log
-- Tracks transfers of subledger entries to the General Ledger
-- ============================================================================
CREATE TABLE _atlas.gl_transfer_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,

    transfer_number VARCHAR(100) NOT NULL,
    transfer_date TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Source period
    from_period VARCHAR(50),

    -- Transfer status: 'pending', 'in_progress', 'completed', 'failed', 'reversed'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,

    -- Summary
    total_entries INT NOT NULL DEFAULT 0,
    total_debit NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_credit NUMERIC(18, 2) NOT NULL DEFAULT 0,

    -- Which applications were included
    included_applications JSONB DEFAULT '[]',

    transferred_by UUID,
    completed_at TIMESTAMPTZ,

    entries JSONB DEFAULT '[]',  -- Array of {entry_id, entry_number, gl_journal_entry_id}

    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, transfer_number)
);

CREATE INDEX idx_gl_transfer_log_org ON _atlas.gl_transfer_log(organization_id);
CREATE INDEX idx_gl_transfer_log_status ON _atlas.gl_transfer_log(status);
CREATE INDEX idx_gl_transfer_log_date ON _atlas.gl_transfer_log(transfer_date);