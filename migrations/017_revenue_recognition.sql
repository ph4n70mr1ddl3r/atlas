-- Atlas ERP - Revenue Recognition (ASC 606 / IFRS 15)
-- Oracle Fusion Cloud ERP: Financials > Revenue Management
--
-- Implements the five-step revenue recognition model:
-- 1. Identify the contract with a customer
-- 2. Identify the performance obligations in the contract
-- 3. Determine the transaction price
-- 4. Allocate the transaction price to performance obligations
-- 5. Recognize revenue when (or as) performance obligations are satisfied
--
-- Supports:
-- - Revenue policies (recognition methods, allocation bases)
-- - Revenue contracts (customer agreements)
-- - Performance obligations (distinct goods/services)
-- - Revenue recognition schedules (planned recognition events)
-- - Contract modifications (amendments)
--
-- This is a standard Oracle Fusion Cloud ERP feature for ASC 606 / IFRS 15 compliance.

-- ============================================================================
-- Revenue Policies
-- Oracle Fusion: Revenue Management > Policies
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.revenue_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Unique policy code (e.g., "STD_SaaS", "STD_CONSULTING")
    code VARCHAR(50) NOT NULL,
    -- Human-readable name
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Recognition method: 'over_time', 'point_in_time'
    recognition_method VARCHAR(20) NOT NULL,
    -- Over-time method (when recognition_method = over_time):
    -- 'output', 'input', 'straight_line'
    over_time_method VARCHAR(20),
    -- Allocation basis: 'standalone_selling_price', 'residual', 'equal'
    allocation_basis VARCHAR(30) NOT NULL DEFAULT 'standalone_selling_price',
    -- Default standalone selling price
    default_selling_price NUMERIC(18,2),
    -- Variable consideration constraint
    constrain_variable_consideration BOOLEAN NOT NULL DEFAULT false,
    constraint_threshold_percent NUMERIC(5,2),
    -- Default GL account codes
    revenue_account_code VARCHAR(50),
    deferred_revenue_account_code VARCHAR(50),
    contra_revenue_account_code VARCHAR(50),
    -- Active flag
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

COMMENT ON TABLE _atlas.revenue_policies IS 'Revenue recognition policies defining how revenue is recognized for different product/service types';
COMMENT ON COLUMN _atlas.revenue_policies.recognition_method IS 'ASC 606 recognition method: over_time or point_in_time';
COMMENT ON COLUMN _atlas.revenue_policies.over_time_method IS 'Method for over-time recognition: output, input, or straight_line';
COMMENT ON COLUMN _atlas.revenue_policies.allocation_basis IS 'How transaction price is allocated to obligations: standalone_selling_price, residual, or equal';

-- ============================================================================
-- Revenue Contracts (Revenue Arrangements)
-- Oracle Fusion: Revenue Management > Revenue Contracts
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.revenue_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Auto-generated contract number (e.g., "RC-A1B2C3D4")
    contract_number VARCHAR(50) NOT NULL,
    -- Source document reference (sales order, agreement, etc.)
    source_type VARCHAR(50),
    source_id UUID,
    source_number VARCHAR(50),
    -- Customer information
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    -- Contract dates
    contract_date DATE,
    start_date DATE,
    end_date DATE,
    -- Financial amounts
    total_transaction_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_allocated_revenue NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_recognized_revenue NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_deferred_revenue NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Contract status: 'draft', 'active', 'completed', 'cancelled', 'modified'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- ASC 606 five-step tracking
    step1_contract_identified BOOLEAN NOT NULL DEFAULT false,
    step2_obligations_identified BOOLEAN NOT NULL DEFAULT false,
    step3_price_determined BOOLEAN NOT NULL DEFAULT false,
    step4_price_allocated BOOLEAN NOT NULL DEFAULT false,
    step5_recognition_scheduled BOOLEAN NOT NULL DEFAULT false,
    -- Currency
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    -- Notes
    notes TEXT,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, contract_number)
);

COMMENT ON TABLE _atlas.revenue_contracts IS 'Revenue contracts implementing ASC 606 Step 1: Identify the contract with a customer';
COMMENT ON COLUMN _atlas.revenue_contracts.step1_contract_identified IS 'ASC 606 Step 1: Contract has been identified';
COMMENT ON COLUMN _atlas.revenue_contracts.step2_obligations_identified IS 'ASC 606 Step 2: Performance obligations have been identified';
COMMENT ON COLUMN _atlas.revenue_contracts.step3_price_determined IS 'ASC 606 Step 3: Transaction price has been determined';
COMMENT ON COLUMN _atlas.revenue_contracts.step4_price_allocated IS 'ASC 606 Step 4: Transaction price has been allocated to obligations';
COMMENT ON COLUMN _atlas.revenue_contracts.step5_recognition_scheduled IS 'ASC 606 Step 5: Revenue recognition has been scheduled';

CREATE INDEX IF NOT EXISTS idx_revenue_contracts_org ON _atlas.revenue_contracts(organization_id);
CREATE INDEX IF NOT EXISTS idx_revenue_contracts_customer ON _atlas.revenue_contracts(customer_id);
CREATE INDEX IF NOT EXISTS idx_revenue_contracts_status ON _atlas.revenue_contracts(status);

-- ============================================================================
-- Performance Obligations
-- Oracle Fusion: Revenue Management > Performance Obligations
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.performance_obligations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Parent revenue contract
    contract_id UUID NOT NULL REFERENCES _atlas.revenue_contracts(id),
    -- Line number within the contract
    line_number INT NOT NULL,
    -- Description of the distinct good or service
    description TEXT,
    -- Product/service reference
    product_id UUID,
    product_name VARCHAR(200),
    -- Source line reference
    source_line_id UUID,
    -- Revenue policy applied
    revenue_policy_id UUID REFERENCES _atlas.revenue_policies(id),
    -- Recognition method (can override policy default)
    recognition_method VARCHAR(20),
    over_time_method VARCHAR(20),
    -- Standalone selling price (SSP)
    standalone_selling_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Allocated transaction price (after SSP allocation - ASC 606 Step 4)
    allocated_transaction_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Revenue tracking
    total_recognized_revenue NUMERIC(18,2) NOT NULL DEFAULT 0,
    deferred_revenue NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Recognition period
    recognition_start_date DATE,
    recognition_end_date DATE,
    -- Percent complete (for over-time recognition)
    percent_complete NUMERIC(5,2),
    -- Satisfaction method: 'over_time', 'point_in_time'
    satisfaction_method VARCHAR(20) NOT NULL DEFAULT 'point_in_time',
    -- Status: 'pending', 'in_progress', 'satisfied', 'partially_satisfied', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- GL account overrides
    revenue_account_code VARCHAR(50),
    deferred_revenue_account_code VARCHAR(50),
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(contract_id, line_number)
);

COMMENT ON TABLE _atlas.performance_obligations IS 'Performance obligations implementing ASC 606 Step 2: Identify distinct goods/services promised in a contract';
COMMENT ON COLUMN _atlas.performance_obligations.standalone_selling_price IS 'The price at which the entity would sell the good/service separately (SSP)';
COMMENT ON COLUMN _atlas.performance_obligations.allocated_transaction_price IS 'Transaction price allocated to this obligation based on SSP (ASC 606 Step 4)';

CREATE INDEX IF NOT EXISTS idx_perf_obligations_contract ON _atlas.performance_obligations(contract_id);
CREATE INDEX IF NOT EXISTS idx_perf_obligations_status ON _atlas.performance_obligations(status);

-- ============================================================================
-- Revenue Schedule Lines
-- Oracle Fusion: Revenue Management > Revenue Schedules
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.revenue_schedule_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Parent performance obligation
    obligation_id UUID NOT NULL REFERENCES _atlas.performance_obligations(id),
    -- Parent contract (denormalized for efficient querying)
    contract_id UUID NOT NULL REFERENCES _atlas.revenue_contracts(id),
    -- Line number within the obligation's schedule
    line_number INT NOT NULL,
    -- Planned recognition date
    recognition_date DATE NOT NULL,
    -- Amount to recognize in this period
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Amount actually recognized
    recognized_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Status: 'planned', 'recognized', 'reversed', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'planned',
    -- Recognition method used for this line
    recognition_method VARCHAR(20),
    -- Percentage of total obligation revenue
    percent_of_total NUMERIC(8,4),
    -- Journal entry reference (posted to GL)
    journal_entry_id UUID,
    -- When recognition was actually posted
    recognized_at TIMESTAMPTZ,
    -- Reversal tracking
    reversed_by_id UUID,
    reversal_reason TEXT,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(obligation_id, line_number)
);

COMMENT ON TABLE _atlas.revenue_schedule_lines IS 'Revenue recognition schedule implementing ASC 606 Step 5: Recognize revenue when/as obligations are satisfied';
COMMENT ON COLUMN _atlas.revenue_schedule_lines.status IS 'planned=scheduled for future recognition, recognized=revenue posted to GL, reversed=reversed due to correction, cancelled=cancelled';

CREATE INDEX IF NOT EXISTS idx_revenue_schedule_obligation ON _atlas.revenue_schedule_lines(obligation_id);
CREATE INDEX IF NOT EXISTS idx_revenue_schedule_contract ON _atlas.revenue_schedule_lines(contract_id);
CREATE INDEX IF NOT EXISTS idx_revenue_schedule_date ON _atlas.revenue_schedule_lines(recognition_date);
CREATE INDEX IF NOT EXISTS idx_revenue_schedule_status ON _atlas.revenue_schedule_lines(status);

-- ============================================================================
-- Revenue Contract Modifications
-- Oracle Fusion: Revenue Management > Contract Modifications
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.revenue_modifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Contract being modified
    contract_id UUID NOT NULL REFERENCES _atlas.revenue_contracts(id),
    -- Sequential modification number
    modification_number INT NOT NULL,
    -- Type: 'price_change', 'scope_change', 'term_extension',
    --       'termination', 'add_obligation', 'remove_obligation'
    modification_type VARCHAR(30) NOT NULL,
    description TEXT,
    -- Price changes
    previous_transaction_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    new_transaction_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Date changes
    previous_end_date DATE,
    new_end_date DATE,
    effective_date DATE NOT NULL,
    -- Status: 'draft', 'active', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(contract_id, modification_number)
);

COMMENT ON TABLE _atlas.revenue_modifications IS 'Tracks amendments to revenue contracts (price changes, scope changes, term extensions, etc.)';

CREATE INDEX IF NOT EXISTS idx_revenue_mods_contract ON _atlas.revenue_modifications(contract_id);
