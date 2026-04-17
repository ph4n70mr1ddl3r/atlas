-- Atlas ERP - Fixed Assets Management
-- Oracle Fusion Cloud ERP: Fixed Assets
--
-- Manages asset categories, asset books (corporate/tax), asset registration
-- with depreciation parameters, depreciation calculation methods
-- (straight-line, declining balance, sum-of-years-digits), asset lifecycle
-- (acquisition → in-service → depreciation → retirement), asset transfers,
-- and asset retirement with gain/loss.
--
-- This is a standard Oracle Fusion Cloud ERP feature for tracking and
-- depreciating capital assets over their useful life.

-- ============================================================================
-- Asset Categories
-- Oracle Fusion: Fixed Assets > Asset Categories
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.asset_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Category code (e.g., 'IT_EQUIP', 'VEHICLES', 'BUILDINGS')
    code VARCHAR(50) NOT NULL,
    -- Display name
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Default depreciation method for assets in this category
    -- 'straight_line', 'declining_balance', 'sum_of_years_digits'
    default_depreciation_method VARCHAR(30) NOT NULL DEFAULT 'straight_line',
    -- Default useful life in months
    default_useful_life_months INT NOT NULL DEFAULT 60,
    -- Default salvage value percentage (0-100)
    default_salvage_value_percent NUMERIC(5,2) NOT NULL DEFAULT 0,
    -- Default asset account code (debit account for the asset)
    default_asset_account_code VARCHAR(50),
    -- Default accumulated depreciation account code (contra-asset)
    default_accum_depr_account_code VARCHAR(50),
    -- Default depreciation expense account code
    default_depr_expense_account_code VARCHAR(50),
    -- Default gain/loss account for asset retirement
    default_gain_loss_account_code VARCHAR(50),
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_asset_categories_org ON _atlas.asset_categories(organization_id);
CREATE INDEX idx_asset_categories_active ON _atlas.asset_categories(organization_id, is_active) WHERE is_active = true;

-- ============================================================================
-- Asset Books
-- Oracle Fusion: Fixed Assets > Books (corporate, tax)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.asset_books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Book code (e.g., 'CORPORATE', 'TAX_US', 'TAX_LOCAL')
    code VARCHAR(50) NOT NULL,
    -- Display name
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Book type: 'corporate', 'tax'
    book_type VARCHAR(20) NOT NULL DEFAULT 'corporate',
    -- Whether depreciation is automatically calculated
    auto_depreciation BOOLEAN DEFAULT true,
    -- Depreciation calendar: 'monthly', 'quarterly', 'yearly'
    depreciation_calendar VARCHAR(20) NOT NULL DEFAULT 'monthly',
    -- Current fiscal year
    current_fiscal_year INT,
    -- Last depreciation run date
    last_depreciation_date DATE,
    -- Status
    is_active BOOLEAN DEFAULT true,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_asset_books_org ON _atlas.asset_books(organization_id);

-- ============================================================================
-- Fixed Assets
-- Oracle Fusion: Fixed Assets > Assets
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.fixed_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Asset identification
    asset_number VARCHAR(50) NOT NULL,
    asset_name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Category reference
    category_id UUID REFERENCES _atlas.asset_categories(id),
    category_code VARCHAR(50),
    -- Book reference (which book this asset belongs to)
    book_id UUID REFERENCES _atlas.asset_books(id),
    book_code VARCHAR(50),
    -- Asset type: 'tangible', 'intangible', 'leased', 'cipc'
    asset_type VARCHAR(20) NOT NULL DEFAULT 'tangible',
    -- Asset lifecycle status:
    -- 'draft', 'acquired', 'in_service', 'under_construction',
    -- 'disposed', 'retired', 'transferred'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Financial details
    original_cost NUMERIC(18,2) NOT NULL DEFAULT 0,
    current_cost NUMERIC(18,2) NOT NULL DEFAULT 0,
    salvage_value NUMERIC(18,2) NOT NULL DEFAULT 0,
    salvage_value_percent NUMERIC(5,2) NOT NULL DEFAULT 0,
    -- Depreciation parameters
    depreciation_method VARCHAR(30) NOT NULL DEFAULT 'straight_line',
    useful_life_months INT NOT NULL DEFAULT 60,
    -- Declining balance rate (for declining balance method)
    declining_balance_rate NUMERIC(5,2),
    -- Depreciation calculations
    depreciable_basis NUMERIC(18,2) NOT NULL DEFAULT 0,
    accumulated_depreciation NUMERIC(18,2) NOT NULL DEFAULT 0,
    net_book_value NUMERIC(18,2) NOT NULL DEFAULT 0,
    depreciation_per_period NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Depreciation tracking
    periods_depreciated INT NOT NULL DEFAULT 0,
    last_depreciation_date DATE,
    last_depreciation_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Date tracking
    acquisition_date DATE,
    in_service_date DATE,
    disposal_date DATE,
    retirement_date DATE,
    -- Location and assignment
    location VARCHAR(200),
    department_id UUID,
    department_name VARCHAR(200),
    custodian_id UUID,
    custodian_name VARCHAR(200),
    -- Serial / tag tracking
    serial_number VARCHAR(100),
    tag_number VARCHAR(100),
    manufacturer VARCHAR(200),
    model VARCHAR(200),
    -- Warranty
    warranty_expiry DATE,
    -- Insurance
    insurance_policy_number VARCHAR(100),
    insurance_expiry DATE,
    -- Lease info
    lease_number VARCHAR(50),
    lease_expiry DATE,
    -- Asset account codes (overriding category defaults)
    asset_account_code VARCHAR(50),
    accum_depr_account_code VARCHAR(50),
    depr_expense_account_code VARCHAR(50),
    gain_loss_account_code VARCHAR(50),
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, asset_number)
);

CREATE INDEX idx_fixed_assets_org ON _atlas.fixed_assets(organization_id);
CREATE INDEX idx_fixed_assets_status ON _atlas.fixed_assets(organization_id, status);
CREATE INDEX idx_fixed_assets_category ON _atlas.fixed_assets(category_id);
CREATE INDEX idx_fixed_assets_book ON _atlas.fixed_assets(book_id);
CREATE INDEX idx_fixed_assets_dept ON _atlas.fixed_assets(department_id);
CREATE INDEX idx_fixed_assets_number ON _atlas.fixed_assets(asset_number);

-- ============================================================================
-- Asset Depreciation History
-- Oracle Fusion: Fixed Assets > Depreciation History
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.asset_depreciation_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    asset_id UUID NOT NULL REFERENCES _atlas.fixed_assets(id) ON DELETE CASCADE,
    -- Period information
    fiscal_year INT NOT NULL,
    period_number INT NOT NULL,
    period_name VARCHAR(50),
    depreciation_date DATE NOT NULL,
    -- Depreciation amounts
    depreciation_amount NUMERIC(18,2) NOT NULL,
    accumulated_depreciation NUMERIC(18,2) NOT NULL,
    net_book_value NUMERIC(18,2) NOT NULL,
    -- Method used for this period
    depreciation_method VARCHAR(30) NOT NULL,
    -- Journal entry reference (posting to GL)
    journal_entry_id UUID,
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_asset_depr_hist_org ON _atlas.asset_depreciation_history(organization_id);
CREATE INDEX idx_asset_depr_hist_asset ON _atlas.asset_depreciation_history(asset_id);
CREATE INDEX idx_asset_depr_hist_period ON _atlas.asset_depreciation_history(fiscal_year, period_number);

-- ============================================================================
-- Asset Transfers
-- Oracle Fusion: Fixed Assets > Asset Transfers
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.asset_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    transfer_number VARCHAR(50) NOT NULL,
    asset_id UUID NOT NULL REFERENCES _atlas.fixed_assets(id),
    -- From
    from_department_id UUID,
    from_department_name VARCHAR(200),
    from_location VARCHAR(200),
    from_custodian_id UUID,
    from_custodian_name VARCHAR(200),
    -- To
    to_department_id UUID,
    to_department_name VARCHAR(200),
    to_location VARCHAR(200),
    to_custodian_id UUID,
    to_custodian_name VARCHAR(200),
    -- Transfer details
    transfer_date DATE NOT NULL,
    reason TEXT,
    -- Status: 'pending', 'approved', 'rejected', 'completed'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, transfer_number)
);

CREATE INDEX idx_asset_transfers_org ON _atlas.asset_transfers(organization_id);
CREATE INDEX idx_asset_transfers_asset ON _atlas.asset_transfers(asset_id);
CREATE INDEX idx_asset_transfers_status ON _atlas.asset_transfers(organization_id, status);

-- ============================================================================
-- Asset Retirements
-- Oracle Fusion: Fixed Assets > Asset Retirements
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.asset_retirements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    retirement_number VARCHAR(50) NOT NULL,
    asset_id UUID NOT NULL REFERENCES _atlas.fixed_assets(id),
    -- Retirement type: 'sale', 'scrap', 'donation', 'write_off', 'casualty'
    retirement_type VARCHAR(20) NOT NULL,
    -- Retirement details
    retirement_date DATE NOT NULL,
    -- Proceeds from sale (if any)
    proceeds NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Removal cost
    removal_cost NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Calculated amounts
    -- Gain/Loss = Proceeds - Net Book Value - Removal Cost
    net_book_value_at_retirement NUMERIC(18,2) NOT NULL DEFAULT 0,
    accumulated_depreciation_at_retirement NUMERIC(18,2) NOT NULL DEFAULT 0,
    gain_loss_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- 'gain' or 'loss'
    gain_loss_type VARCHAR(10),
    -- Account references
    gain_account_code VARCHAR(50),
    loss_account_code VARCHAR(50),
    cash_account_code VARCHAR(50),
    asset_account_code VARCHAR(50),
    accum_depr_account_code VARCHAR(50),
    -- Reference / buyer information
    reference_number VARCHAR(100),
    buyer_name VARCHAR(200),
    notes TEXT,
    -- Status: 'pending', 'approved', 'completed', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    -- Journal entry reference
    journal_entry_id UUID,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, retirement_number)
);

CREATE INDEX idx_asset_retirements_org ON _atlas.asset_retirements(organization_id);
CREATE INDEX idx_asset_retirements_asset ON _atlas.asset_retirements(asset_id);
CREATE INDEX idx_asset_retirements_status ON _atlas.asset_retirements(organization_id, status);
