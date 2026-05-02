-- Migration 109: Create missing core financial / PIM tables
--
-- The Rust repository layer references table names that were not created by
-- earlier migrations.  This migration adds every missing table so that the
-- code and schema stay in sync.

BEGIN;

-- ═══════════════════════════════════════════════════════════════════════════════
-- General Ledger
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.gl_accounts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID        NOT NULL,
    account_code        VARCHAR(100) NOT NULL,
    account_name        VARCHAR(200) NOT NULL,
    description         TEXT,
    account_type        VARCHAR(30)  NOT NULL,  -- asset, liability, equity, revenue, expense
    subtype             VARCHAR(50),
    parent_account_id   UUID REFERENCES _atlas.gl_accounts(id),
    is_active           BOOLEAN      NOT NULL DEFAULT true,
    natural_balance     VARCHAR(10)  NOT NULL,  -- Debit / Credit
    third_party_control BOOLEAN      NOT NULL DEFAULT false,
    reconciliation_enabled BOOLEAN   NOT NULL DEFAULT false,
    metadata            JSONB        NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (organization_id, account_code)
);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_org   ON _atlas.gl_accounts (organization_id);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_type  ON _atlas.gl_accounts (organization_id, account_type);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_parent ON _atlas.gl_accounts (parent_account_id);

CREATE TABLE IF NOT EXISTS _atlas.gl_journal_entries (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID        NOT NULL,
    entry_number        VARCHAR(50)  NOT NULL,
    ledger_id           UUID,
    entry_date          DATE         NOT NULL,
    gl_date             DATE         NOT NULL,
    entry_type          VARCHAR(30)  NOT NULL DEFAULT 'manual',
    description         TEXT,
    currency_code       VARCHAR(10)  NOT NULL DEFAULT 'USD',
    total_debit         NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credit        NUMERIC(18,2) NOT NULL DEFAULT 0,
    is_balanced         BOOLEAN      NOT NULL DEFAULT false,
    status              VARCHAR(30)  NOT NULL DEFAULT 'draft',
    posted_by           UUID,
    posted_at           TIMESTAMPTZ,
    reversal_entry_id   UUID REFERENCES _atlas.gl_journal_entries(id),
    source_type         VARCHAR(50),
    source_id           UUID,
    metadata            JSONB        NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (organization_id, entry_number)
);
CREATE INDEX IF NOT EXISTS idx_gl_je_org     ON _atlas.gl_journal_entries (organization_id);
CREATE INDEX IF NOT EXISTS idx_gl_je_status  ON _atlas.gl_journal_entries (organization_id, status);
CREATE INDEX IF NOT EXISTS idx_gl_je_date    ON _atlas.gl_journal_entries (gl_date);
CREATE INDEX IF NOT EXISTS idx_gl_je_source  ON _atlas.gl_journal_entries (source_type, source_id);

CREATE TABLE IF NOT EXISTS _atlas.gl_journal_lines (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID        NOT NULL,
    journal_entry_id    UUID        NOT NULL REFERENCES _atlas.gl_journal_entries(id) ON DELETE CASCADE,
    line_number         INT         NOT NULL,
    line_type           VARCHAR(20)  NOT NULL DEFAULT 'debit',
    account_code        VARCHAR(100) NOT NULL,
    account_name        VARCHAR(200),
    description         TEXT,
    entered_dr          NUMERIC(18,2) NOT NULL DEFAULT 0,
    entered_cr          NUMERIC(18,2) NOT NULL DEFAULT 0,
    accounted_dr        NUMERIC(18,2) NOT NULL DEFAULT 0,
    accounted_cr        NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code       VARCHAR(10)  NOT NULL DEFAULT 'USD',
    exchange_rate       NUMERIC(18,6),
    reference           VARCHAR(200),
    tax_code            VARCHAR(50),
    metadata            JSONB        NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_gl_jl_entry  ON _atlas.gl_journal_lines (journal_entry_id);
CREATE INDEX IF NOT EXISTS idx_gl_jl_acct   ON _atlas.gl_journal_lines (account_code);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Accounts Payable
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.ap_invoices (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id         UUID         NOT NULL,
    invoice_number          VARCHAR(50)   NOT NULL,
    invoice_date            DATE          NOT NULL,
    invoice_type            VARCHAR(30)   NOT NULL DEFAULT 'standard',
    description             TEXT,
    supplier_id             UUID,
    supplier_number         VARCHAR(50),
    supplier_name           VARCHAR(200),
    supplier_site           VARCHAR(100),
    invoice_currency_code   VARCHAR(10)   NOT NULL DEFAULT 'USD',
    payment_currency_code   VARCHAR(10)   NOT NULL DEFAULT 'USD',
    exchange_rate           NUMERIC(18,6) DEFAULT 1,
    exchange_rate_type      VARCHAR(30),
    exchange_date           DATE,
    invoice_amount          NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_amount              NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_amount            NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_paid             NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_remaining        NUMERIC(18,2) NOT NULL DEFAULT 0,
    discount_available      NUMERIC(18,2) NOT NULL DEFAULT 0,
    discount_taken          NUMERIC(18,2) NOT NULL DEFAULT 0,
    payment_terms           VARCHAR(50),
    payment_method          VARCHAR(50),
    payment_due_date        DATE,
    discount_date           DATE,
    gl_date                 DATE,
    gl_posted_date          TIMESTAMPTZ,
    status                  VARCHAR(30)   NOT NULL DEFAULT 'draft',
    approval_status         VARCHAR(30),
    approved_by             UUID,
    approved_at             TIMESTAMPTZ,
    cancelled_reason        TEXT,
    cancelled_by            UUID,
    cancelled_at            TIMESTAMPTZ,
    po_number               VARCHAR(50),
    receipt_number          VARCHAR(50),
    source                  VARCHAR(50),
    batch_id                UUID,
    metadata                JSONB         NOT NULL DEFAULT '{}',
    created_by              UUID,
    created_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, invoice_number)
);
CREATE INDEX IF NOT EXISTS idx_ap_inv_org     ON _atlas.ap_invoices (organization_id);
CREATE INDEX IF NOT EXISTS idx_ap_inv_supplier ON _atlas.ap_invoices (supplier_id);
CREATE INDEX IF NOT EXISTS idx_ap_inv_status   ON _atlas.ap_invoices (organization_id, status);

CREATE TABLE IF NOT EXISTS _atlas.ap_invoice_lines (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID         NOT NULL,
    invoice_id          UUID         NOT NULL REFERENCES _atlas.ap_invoices(id) ON DELETE CASCADE,
    line_number         INT          NOT NULL,
    line_type           VARCHAR(30)  NOT NULL DEFAULT 'item',
    description         TEXT,
    amount              NUMERIC(18,2) NOT NULL DEFAULT 0,
    unit_price          NUMERIC(18,2),
    quantity_invoiced   NUMERIC(18,2),
    unit_of_measure     VARCHAR(30),
    po_line_id          UUID,
    po_line_number      VARCHAR(50),
    product_code        VARCHAR(100),
    tax_code            VARCHAR(50),
    tax_amount          NUMERIC(18,2),
    asset_category_code VARCHAR(50),
    project_id          UUID,
    task_id             UUID,
    expenditure_type    VARCHAR(50),
    metadata            JSONB        NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_ap_inv_line_invoice ON _atlas.ap_invoice_lines (invoice_id);

CREATE TABLE IF NOT EXISTS _atlas.ap_invoice_distributions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id             UUID         NOT NULL,
    invoice_id                  UUID         NOT NULL REFERENCES _atlas.ap_invoices(id) ON DELETE CASCADE,
    invoice_line_id             UUID         REFERENCES _atlas.ap_invoice_lines(id) ON DELETE CASCADE,
    distribution_line_number    INT          NOT NULL,
    distribution_type           VARCHAR(30),
    account_combination         VARCHAR(500),
    description                 TEXT,
    amount                      NUMERIC(18,2) NOT NULL DEFAULT 0,
    base_amount                 NUMERIC(18,2),
    currency_code               VARCHAR(10)  DEFAULT 'USD',
    exchange_rate               NUMERIC(18,6),
    gl_account                  VARCHAR(100),
    cost_center                 VARCHAR(100),
    department                  VARCHAR(100),
    project_id                  UUID,
    task_id                     UUID,
    expenditure_type            VARCHAR(50),
    tax_code                    VARCHAR(50),
    tax_recoverable             BOOLEAN      DEFAULT false,
    tax_recoverable_amount      NUMERIC(18,2) DEFAULT 0,
    accounting_date             DATE,
    posted_status               VARCHAR(30)  DEFAULT 'unposted',
    posted_at                   TIMESTAMPTZ,
    metadata                    JSONB        NOT NULL DEFAULT '{}',
    created_by                  UUID,
    created_at                  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_ap_dist_invoice ON _atlas.ap_invoice_distributions (invoice_id);

CREATE TABLE IF NOT EXISTS _atlas.ap_invoice_holds (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID         NOT NULL,
    invoice_id          UUID         NOT NULL REFERENCES _atlas.ap_invoices(id) ON DELETE CASCADE,
    hold_type           VARCHAR(50)  NOT NULL,
    hold_reason         TEXT,
    hold_status         VARCHAR(30)  NOT NULL DEFAULT 'active',
    released_by         UUID,
    released_at         TIMESTAMPTZ,
    release_reason      TEXT,
    metadata            JSONB        NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_ap_hold_invoice ON _atlas.ap_invoice_holds (invoice_id);

CREATE TABLE IF NOT EXISTS _atlas.ap_payments (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id         UUID         NOT NULL,
    payment_number          VARCHAR(50)   NOT NULL,
    payment_date            DATE          NOT NULL,
    payment_method          VARCHAR(50),
    payment_currency_code   VARCHAR(10)   NOT NULL DEFAULT 'USD',
    payment_amount          NUMERIC(18,2) NOT NULL DEFAULT 0,
    bank_account_id         UUID,
    bank_account_name       VARCHAR(200),
    payment_document        VARCHAR(200),
    status                  VARCHAR(30)   NOT NULL DEFAULT 'draft',
    supplier_id             UUID,
    supplier_number         VARCHAR(50),
    supplier_name           VARCHAR(200),
    invoice_ids             JSONB         NOT NULL DEFAULT '[]',
    confirmed_by            UUID,
    confirmed_at            TIMESTAMPTZ,
    cancelled_reason        TEXT,
    metadata                JSONB         NOT NULL DEFAULT '{}',
    created_by              UUID,
    created_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, payment_number)
);
CREATE INDEX IF NOT EXISTS idx_ap_pay_org     ON _atlas.ap_payments (organization_id);
CREATE INDEX IF NOT EXISTS idx_ap_pay_supplier ON _atlas.ap_payments (supplier_id);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Accounts Receivable
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.ar_transactions (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id         UUID         NOT NULL,
    transaction_number      VARCHAR(50)   NOT NULL,
    transaction_type        VARCHAR(30)   NOT NULL DEFAULT 'invoice',
    transaction_date        DATE          NOT NULL,
    gl_date                 DATE,
    customer_id             UUID,
    customer_number         VARCHAR(50),
    customer_name           VARCHAR(200),
    bill_to_site            VARCHAR(200),
    currency_code           VARCHAR(10)   NOT NULL DEFAULT 'USD',
    exchange_rate           NUMERIC(18,6) DEFAULT 1,
    exchange_rate_type      VARCHAR(30),
    entered_amount          NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_amount              NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_amount            NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_due_original     NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_due_remaining    NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_applied          NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_adjusted         NUMERIC(18,2) NOT NULL DEFAULT 0,
    payment_terms           VARCHAR(50),
    due_date                DATE,
    discount_due_date       DATE,
    reference_number        VARCHAR(100),
    purchase_order          VARCHAR(50),
    sales_rep               VARCHAR(200),
    status                  VARCHAR(30)   NOT NULL DEFAULT 'draft',
    receipt_method          VARCHAR(50),
    notes                   TEXT,
    metadata                JSONB         NOT NULL DEFAULT '{}',
    created_by              UUID,
    created_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, transaction_number)
);
CREATE INDEX IF NOT EXISTS idx_ar_txn_org      ON _atlas.ar_transactions (organization_id);
CREATE INDEX IF NOT EXISTS idx_ar_txn_customer ON _atlas.ar_transactions (customer_id);
CREATE INDEX IF NOT EXISTS idx_ar_txn_status   ON _atlas.ar_transactions (organization_id, status);

CREATE TABLE IF NOT EXISTS _atlas.ar_transaction_lines (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID         NOT NULL,
    transaction_id      UUID         NOT NULL REFERENCES _atlas.ar_transactions(id) ON DELETE CASCADE,
    line_number         INT          NOT NULL,
    description         TEXT,
    line_type           VARCHAR(30)  NOT NULL DEFAULT 'line',
    item_code           VARCHAR(100),
    item_description    VARCHAR(200),
    unit_of_measure     VARCHAR(30),
    quantity            NUMERIC(18,2),
    unit_price          NUMERIC(18,2),
    line_amount         NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_amount          NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_code            VARCHAR(50),
    revenue_account     VARCHAR(100),
    tax_account         VARCHAR(100),
    metadata            JSONB        NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_ar_tl_txn ON _atlas.ar_transaction_lines (transaction_id);

CREATE TABLE IF NOT EXISTS _atlas.ar_adjustments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID         NOT NULL,
    adjustment_number   VARCHAR(50)   NOT NULL,
    transaction_id      UUID,
    transaction_number  VARCHAR(50),
    customer_id         UUID,
    customer_number     VARCHAR(50),
    adjustment_date     DATE          NOT NULL,
    gl_date             DATE,
    adjustment_type     VARCHAR(30)   NOT NULL,
    amount              NUMERIC(18,2) NOT NULL DEFAULT 0,
    receivable_account  VARCHAR(100),
    adjustment_account  VARCHAR(100),
    reason_code         VARCHAR(50),
    reason_description  TEXT,
    status              VARCHAR(30)   NOT NULL DEFAULT 'draft',
    approved_by         UUID,
    notes               TEXT,
    metadata            JSONB         NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, adjustment_number)
);
CREATE INDEX IF NOT EXISTS idx_ar_adj_org ON _atlas.ar_adjustments (organization_id);

CREATE TABLE IF NOT EXISTS _atlas.ar_receipts (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id             UUID         NOT NULL,
    receipt_number              VARCHAR(50)   NOT NULL,
    receipt_date                DATE          NOT NULL,
    receipt_type                VARCHAR(30)   NOT NULL DEFAULT 'cash',
    receipt_method              VARCHAR(50),
    amount                      NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code               VARCHAR(10)   NOT NULL DEFAULT 'USD',
    exchange_rate               NUMERIC(18,6) DEFAULT 1,
    customer_id                 UUID,
    customer_number             VARCHAR(50),
    customer_name               VARCHAR(200),
    reference_number            VARCHAR(100),
    bank_account_name           VARCHAR(200),
    check_number                VARCHAR(50),
    maturity_date               DATE,
    status                      VARCHAR(30)   NOT NULL DEFAULT 'draft',
    applied_transaction_number  VARCHAR(50),
    notes                       TEXT,
    metadata                    JSONB         NOT NULL DEFAULT '{}',
    created_by                  UUID,
    created_at                  TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, receipt_number)
);
CREATE INDEX IF NOT EXISTS idx_ar_rcpt_org      ON _atlas.ar_receipts (organization_id);
CREATE INDEX IF NOT EXISTS idx_ar_rcpt_customer ON _atlas.ar_receipts (customer_id);

CREATE TABLE IF NOT EXISTS _atlas.ar_credit_memos (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID         NOT NULL,
    credit_memo_number  VARCHAR(50)   NOT NULL,
    customer_id         UUID,
    customer_number     VARCHAR(50),
    customer_name       VARCHAR(200),
    transaction_id      UUID,
    transaction_number  VARCHAR(50),
    credit_memo_date    DATE          NOT NULL,
    gl_date             DATE,
    reason_code         VARCHAR(50),
    reason_description  TEXT,
    amount              NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_amount          NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_amount        NUMERIC(18,2) NOT NULL DEFAULT 0,
    status              VARCHAR(30)   NOT NULL DEFAULT 'draft',
    notes               TEXT,
    metadata            JSONB         NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, credit_memo_number)
);
CREATE INDEX IF NOT EXISTS idx_ar_cm_org ON _atlas.ar_credit_memos (organization_id);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Product Information Management (PIM)
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.pim_items (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id         UUID         NOT NULL,
    item_number             VARCHAR(100)  NOT NULL,
    item_name               VARCHAR(200)  NOT NULL,
    description             TEXT,
    long_description        TEXT,
    item_type               VARCHAR(50)   NOT NULL DEFAULT 'finished_good',
    status                  VARCHAR(30)   NOT NULL DEFAULT 'active',
    lifecycle_phase         VARCHAR(30),
    primary_uom_code        VARCHAR(10),
    secondary_uom_code      VARCHAR(10),
    weight                  NUMERIC(18,4),
    weight_uom              VARCHAR(10),
    volume                  NUMERIC(18,4),
    volume_uom              VARCHAR(10),
    hazmat_flag             BOOLEAN       NOT NULL DEFAULT false,
    lot_control_flag        BOOLEAN       NOT NULL DEFAULT false,
    serial_control_flag     BOOLEAN       NOT NULL DEFAULT false,
    shelf_life_days         INT,
    min_order_quantity      NUMERIC(18,2),
    max_order_quantity      NUMERIC(18,2),
    lead_time_days          INT,
    list_price              NUMERIC(18,2),
    cost_price              NUMERIC(18,2),
    currency_code           VARCHAR(10)   DEFAULT 'USD',
    inventory_item_flag     BOOLEAN       NOT NULL DEFAULT true,
    purchasable_flag        BOOLEAN       NOT NULL DEFAULT true,
    sellable_flag           BOOLEAN       NOT NULL DEFAULT true,
    stock_enabled_flag      BOOLEAN       NOT NULL DEFAULT true,
    invoice_enabled_flag    BOOLEAN       NOT NULL DEFAULT true,
    default_buyer_id        UUID,
    default_supplier_id     UUID,
    template_id             UUID,
    thumbnail_url           VARCHAR(500),
    image_url               VARCHAR(500),
    metadata                JSONB         NOT NULL DEFAULT '{}',
    created_by              UUID,
    created_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, item_number)
);
CREATE INDEX IF NOT EXISTS idx_pim_items_org   ON _atlas.pim_items (organization_id);
CREATE INDEX IF NOT EXISTS idx_pim_items_type  ON _atlas.pim_items (organization_id, item_type);
CREATE INDEX IF NOT EXISTS idx_pim_items_status ON _atlas.pim_items (status);

CREATE TABLE IF NOT EXISTS _atlas.pim_categories (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID         NOT NULL,
    code                VARCHAR(100)  NOT NULL,
    name                VARCHAR(200)  NOT NULL,
    description         TEXT,
    parent_category_id  UUID REFERENCES _atlas.pim_categories(id),
    level_number        INT          NOT NULL DEFAULT 1,
    item_count          INT          NOT NULL DEFAULT 0,
    is_active           BOOLEAN      NOT NULL DEFAULT true,
    metadata            JSONB        NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code)
);
CREATE INDEX IF NOT EXISTS idx_pim_cat_org ON _atlas.pim_categories (organization_id);

CREATE TABLE IF NOT EXISTS _atlas.pim_item_category_assignments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID         NOT NULL,
    item_id             UUID         NOT NULL REFERENCES _atlas.pim_items(id) ON DELETE CASCADE,
    category_id         UUID         NOT NULL REFERENCES _atlas.pim_categories(id) ON DELETE CASCADE,
    is_primary          BOOLEAN      NOT NULL DEFAULT false,
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_pim_ica_item ON _atlas.pim_item_category_assignments (item_id);
CREATE INDEX IF NOT EXISTS idx_pim_ica_cat  ON _atlas.pim_item_category_assignments (category_id);

CREATE TABLE IF NOT EXISTS _atlas.pim_item_cross_references (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id         UUID         NOT NULL,
    item_id                 UUID         NOT NULL REFERENCES _atlas.pim_items(id) ON DELETE CASCADE,
    cross_reference_type    VARCHAR(50)   NOT NULL,
    cross_reference_value   VARCHAR(200)  NOT NULL,
    description             TEXT,
    source_system           VARCHAR(100),
    effective_from          DATE,
    effective_to            DATE,
    is_active               BOOLEAN      NOT NULL DEFAULT true,
    metadata                JSONB        NOT NULL DEFAULT '{}',
    created_by              UUID,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_pim_xref_item ON _atlas.pim_item_cross_references (item_id);

CREATE TABLE IF NOT EXISTS _atlas.pim_item_templates (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id             UUID         NOT NULL,
    code                        VARCHAR(100)  NOT NULL,
    name                        VARCHAR(200)  NOT NULL,
    description                 TEXT,
    item_type                   VARCHAR(50),
    default_uom_code            VARCHAR(10),
    default_category_id         UUID,
    default_inventory_flag      BOOLEAN      NOT NULL DEFAULT true,
    default_purchasable_flag    BOOLEAN      NOT NULL DEFAULT true,
    default_sellable_flag       BOOLEAN      NOT NULL DEFAULT true,
    default_stock_enabled_flag  BOOLEAN      NOT NULL DEFAULT true,
    attribute_defaults          JSONB        NOT NULL DEFAULT '{}',
    is_active                   BOOLEAN      NOT NULL DEFAULT true,
    metadata                    JSONB        NOT NULL DEFAULT '{}',
    created_by                  UUID,
    created_at                  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.pim_new_item_requests (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id         UUID         NOT NULL,
    request_number          VARCHAR(50)   NOT NULL,
    title                   VARCHAR(200)  NOT NULL,
    description             TEXT,
    item_type               VARCHAR(50),
    priority                VARCHAR(20)   NOT NULL DEFAULT 'medium',
    status                  VARCHAR(30)   NOT NULL DEFAULT 'submitted',
    requested_item_number   VARCHAR(100),
    requested_item_name     VARCHAR(200),
    requested_category_id   UUID,
    justification           TEXT,
    target_launch_date      DATE,
    estimated_cost          NUMERIC(18,2),
    currency_code           VARCHAR(10)   DEFAULT 'USD',
    requested_by            UUID,
    approved_by             UUID,
    approved_at             TIMESTAMPTZ,
    rejection_reason        TEXT,
    implemented_item_id     UUID,
    implemented_at          TIMESTAMPTZ,
    metadata                JSONB         NOT NULL DEFAULT '{}',
    created_by              UUID,
    created_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, request_number)
);
CREATE INDEX IF NOT EXISTS idx_pim_nir_org ON _atlas.pim_new_item_requests (organization_id);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Cost Allocation (missing base_values)
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.allocation_base_values (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID          NOT NULL,
    base_id             UUID          NOT NULL,
    base_code           VARCHAR(50)   NOT NULL,
    department_id       UUID,
    department_name     VARCHAR(200),
    cost_center         VARCHAR(100),
    project_id          UUID,
    value               NUMERIC(18,2) NOT NULL DEFAULT 0,
    effective_date      DATE          NOT NULL,
    source              VARCHAR(30)   NOT NULL DEFAULT 'manual',
    metadata            JSONB         NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ   NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_alloc_bv_unique
    ON _atlas.allocation_base_values (base_id, COALESCE(department_id, '00000000-0000-0000-0000-000000000000'),
                                       COALESCE(cost_center, ''), effective_date);
CREATE INDEX IF NOT EXISTS idx_alloc_bv_base ON _atlas.allocation_base_values (base_id);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Sales Commission (missing transactions)
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.commission_transactions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id             UUID         NOT NULL,
    rep_id                      UUID         NOT NULL,
    plan_id                     UUID,
    quota_id                    UUID,
    transaction_number          VARCHAR(50)   NOT NULL,
    source_type                 VARCHAR(50),
    source_id                   UUID,
    source_number               VARCHAR(50),
    transaction_date            DATE          NOT NULL,
    sale_amount                 NUMERIC(18,2) NOT NULL DEFAULT 0,
    commission_basis_amount     NUMERIC(18,2) NOT NULL DEFAULT 0,
    commission_rate             NUMERIC(10,4) NOT NULL DEFAULT 0,
    commission_amount           NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code               VARCHAR(10)   DEFAULT 'USD',
    status                      VARCHAR(30)   NOT NULL DEFAULT 'pending',
    payout_id                   UUID,
    metadata                    JSONB         NOT NULL DEFAULT '{}',
    created_by                  UUID,
    created_at                  TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (organization_id, transaction_number)
);
CREATE INDEX IF NOT EXISTS idx_comm_txn_rep ON _atlas.commission_transactions (rep_id);

-- ═══════════════════════════════════════════════════════════════════════════════
-- Pricing (missing discount_rules)
-- ═══════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _atlas.discount_rules (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID         NOT NULL,
    code                VARCHAR(100)  NOT NULL,
    name                VARCHAR(200)  NOT NULL,
    description         TEXT,
    discount_type       VARCHAR(30)   NOT NULL,  -- percentage, fixed_amount
    discount_value      NUMERIC(18,2) NOT NULL DEFAULT 0,
    application_method  VARCHAR(30),             -- line, order
    stacking_rule       VARCHAR(30),             -- exclusive, stackable
    priority            INT          DEFAULT 0,
    condition           JSONB,                   -- eligibility condition
    effective_from      DATE,
    effective_to        DATE,
    status              VARCHAR(30)   NOT NULL DEFAULT 'active',
    is_active           BOOLEAN      NOT NULL DEFAULT true,
    usage_count         INT          NOT NULL DEFAULT 0,
    max_usage           INT,
    metadata            JSONB        NOT NULL DEFAULT '{}',
    created_by          UUID,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code)
);
CREATE INDEX IF NOT EXISTS idx_disc_rules_org ON _atlas.discount_rules (organization_id);

COMMIT;
