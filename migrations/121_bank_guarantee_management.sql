-- ============================================================================
-- Migration 121: Bank Guarantee Management
-- Oracle Fusion: Treasury > Bank Guarantees
--
-- Manages bank guarantees (bid bonds, performance guarantees, advance payment
-- guarantees, etc.) with full lifecycle, amendment tracking, and expiry monitoring.
-- ============================================================================

-- ============================================================================
-- Bank Guarantees
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_bank_guarantees (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                  UUID NOT NULL,
    guarantee_number        VARCHAR(50) NOT NULL,
    guarantee_type          VARCHAR(30) NOT NULL,
    description             TEXT,

    -- Parties
    beneficiary_name        VARCHAR(200) NOT NULL,
    beneficiary_code        VARCHAR(50),
    applicant_name          VARCHAR(200) NOT NULL,
    applicant_code          VARCHAR(50),
    issuing_bank_name       VARCHAR(200) NOT NULL,
    issuing_bank_code       VARCHAR(50),
    bank_account_number     VARCHAR(50),

    -- Financial details
    guarantee_amount        DECIMAL(18, 2) NOT NULL,
    currency_code           VARCHAR(3) NOT NULL DEFAULT 'USD',
    margin_percentage       DECIMAL(8, 4) NOT NULL DEFAULT 0,
    margin_amount           DECIMAL(18, 2) NOT NULL DEFAULT 0,
    commission_rate         DECIMAL(8, 4) NOT NULL DEFAULT 0,
    commission_amount       DECIMAL(18, 2) NOT NULL DEFAULT 0,

    -- Dates
    issue_date              DATE,
    effective_date          DATE,
    expiry_date             DATE,
    claim_expiry_date       DATE,
    renewal_date            DATE,
    auto_renew              BOOLEAN NOT NULL DEFAULT false,

    -- References
    reference_contract_number   VARCHAR(50),
    reference_purchase_order    VARCHAR(50),
    purpose                     TEXT,

    -- Collateral
    collateral_type         VARCHAR(30),
    collateral_amount       DECIMAL(18, 2),

    -- Status & metadata
    status                  VARCHAR(20) NOT NULL DEFAULT 'draft',
    amendment_count         INTEGER NOT NULL DEFAULT 0,
    latest_amendment_number VARCHAR(60),
    created_by_id           UUID,
    approved_by_id          UUID,
    notes                   TEXT,

    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(org_id, guarantee_number)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_bg_org_status ON fin_bank_guarantees(org_id, status);
CREATE INDEX IF NOT EXISTS idx_bg_org_type ON fin_bank_guarantees(org_id, guarantee_type);
CREATE INDEX IF NOT EXISTS idx_bg_expiry ON fin_bank_guarantees(expiry_date) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_bg_beneficiary ON fin_bank_guarantees(beneficiary_name);

-- ============================================================================
-- Bank Guarantee Amendments
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_bank_guarantee_amendments (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                  UUID NOT NULL,
    guarantee_id            UUID NOT NULL REFERENCES fin_bank_guarantees(id),
    guarantee_number        VARCHAR(50) NOT NULL,
    amendment_number        VARCHAR(60) NOT NULL,
    amendment_type          VARCHAR(30) NOT NULL,

    -- Amendment details
    previous_amount         DECIMAL(18, 2),
    new_amount              DECIMAL(18, 2),
    previous_expiry_date    DATE,
    new_expiry_date         DATE,
    previous_terms          TEXT,
    new_terms               TEXT,
    reason                  TEXT,

    -- Status & metadata
    status                  VARCHAR(20) NOT NULL DEFAULT 'pending_approval',
    effective_date          DATE,
    approved_by_id          UUID,
    created_by_id           UUID,

    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(guarantee_id, amendment_number)
);

CREATE INDEX IF NOT EXISTS idx_bga_org ON fin_bank_guarantee_amendments(org_id);
CREATE INDEX IF NOT EXISTS idx_bga_guarantee ON fin_bank_guarantee_amendments(guarantee_id);
CREATE INDEX IF NOT EXISTS idx_bga_status ON fin_bank_guarantee_amendments(org_id, status);

-- ============================================================================
-- Comments for documentation
-- ============================================================================

COMMENT ON TABLE fin_bank_guarantees IS 'Bank guarantee management - Oracle Fusion Treasury > Bank Guarantees';
COMMENT ON TABLE fin_bank_guarantee_amendments IS 'Amendments to bank guarantees';
