//! Fixed Asset Engine Implementation
//!
//! Manages asset categories, asset books (corporate/tax), asset registration
//! with depreciation parameters, depreciation calculation methods
//! (straight-line, declining balance, sum-of-years-digits), asset lifecycle
//! management, asset transfers, and asset retirement with gain/loss.
//!
//! Oracle Fusion Cloud ERP equivalent: Fixed Assets

use atlas_shared::{
    AssetCategory, AssetBook, FixedAsset, AssetDepreciationHistory,
    AssetTransfer, AssetRetirement,
    AtlasError, AtlasResult,
};
use super::FixedAssetRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid depreciation methods
const VALID_DEPRECIATION_METHODS: &[&str] = &[
    "straight_line", "declining_balance", "sum_of_years_digits",
];

/// Valid asset types
const VALID_ASSET_TYPES: &[&str] = &[
    "tangible", "intangible", "leased", "cipc",
];

/// Valid asset statuses
const VALID_STATUSES: &[&str] = &[
    "draft", "acquired", "in_service", "under_construction",
    "disposed", "retired", "transferred",
];

/// Valid retirement types
const VALID_RETIREMENT_TYPES: &[&str] = &[
    "sale", "scrap", "donation", "write_off", "casualty",
];

/// Valid transfer statuses
const VALID_TRANSFER_STATUSES: &[&str] = &[
    "pending", "approved", "rejected", "completed",
];

/// Valid retirement statuses
const VALID_RETIREMENT_STATUSES: &[&str] = &[
    "pending", "approved", "completed", "cancelled",
];

/// Fixed Asset engine for managing assets, depreciation, transfers, and retirements
pub struct FixedAssetEngine {
    repository: Arc<dyn FixedAssetRepository>,
}

impl FixedAssetEngine {
    pub fn new(repository: Arc<dyn FixedAssetRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Asset Category Management
    // ========================================================================

    /// Create a new asset category
    pub async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        default_depreciation_method: &str,
        default_useful_life_months: i32,
        default_salvage_value_percent: &str,
        default_asset_account_code: Option<&str>,
        default_accum_depr_account_code: Option<&str>,
        default_depr_expense_account_code: Option<&str>,
        default_gain_loss_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetCategory> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Asset category code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Asset category name is required".to_string(),
            ));
        }
        if !VALID_DEPRECIATION_METHODS.contains(&default_depreciation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid depreciation method '{}'. Must be one of: {}",
                default_depreciation_method, VALID_DEPRECIATION_METHODS.join(", ")
            )));
        }
        if default_useful_life_months <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Useful life must be positive".to_string(),
            ));
        }

        info!("Creating asset category '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_category(
            org_id, &code_upper, name, description,
            default_depreciation_method, default_useful_life_months,
            default_salvage_value_percent,
            default_asset_account_code, default_accum_depr_account_code,
            default_depr_expense_account_code, default_gain_loss_account_code,
            created_by,
        ).await
    }

    /// Get an asset category by code
    pub async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AssetCategory>> {
        self.repository.get_category(org_id, &code.to_uppercase()).await
    }

    /// List all asset categories for an organization
    pub async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<AssetCategory>> {
        self.repository.list_categories(org_id).await
    }

    /// Deactivate an asset category
    pub async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating asset category '{}' for org {}", code, org_id);
        self.repository.delete_category(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Asset Book Management
    // ========================================================================

    /// Create a new asset book
    pub async fn create_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        book_type: &str,
        auto_depreciation: bool,
        depreciation_calendar: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetBook> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Asset book code is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Asset book name is required".to_string(),
            ));
        }
        if !["corporate", "tax"].contains(&book_type) {
            return Err(AtlasError::ValidationFailed(
                "Book type must be 'corporate' or 'tax'".to_string(),
            ));
        }
        if !["monthly", "quarterly", "yearly"].contains(&depreciation_calendar) {
            return Err(AtlasError::ValidationFailed(
                "Depreciation calendar must be 'monthly', 'quarterly', or 'yearly'".to_string(),
            ));
        }

        info!("Creating asset book '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_book(
            org_id, &code_upper, name, description,
            book_type, auto_depreciation, depreciation_calendar,
            created_by,
        ).await
    }

    /// Get an asset book by code
    pub async fn get_book(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AssetBook>> {
        self.repository.get_book(org_id, &code.to_uppercase()).await
    }

    /// List all asset books for an organization
    pub async fn list_books(&self, org_id: Uuid) -> AtlasResult<Vec<AssetBook>> {
        self.repository.list_books(org_id).await
    }

    /// Deactivate an asset book
    pub async fn delete_book(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating asset book '{}' for org {}", code, org_id);
        self.repository.delete_book(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Asset Management
    // ========================================================================

    /// Create a new fixed asset
    pub async fn create_asset(
        &self,
        org_id: Uuid,
        asset_number: &str,
        asset_name: &str,
        description: Option<&str>,
        category_code: Option<&str>,
        book_code: Option<&str>,
        asset_type: &str,
        original_cost: &str,
        salvage_value: &str,
        salvage_value_percent: &str,
        depreciation_method: Option<&str>,
        useful_life_months: Option<i32>,
        declining_balance_rate: Option<&str>,
        acquisition_date: Option<chrono::NaiveDate>,
        location: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        custodian_id: Option<Uuid>,
        custodian_name: Option<&str>,
        serial_number: Option<&str>,
        tag_number: Option<&str>,
        manufacturer: Option<&str>,
        model: Option<&str>,
        warranty_expiry: Option<chrono::NaiveDate>,
        insurance_policy_number: Option<&str>,
        insurance_expiry: Option<chrono::NaiveDate>,
        lease_number: Option<&str>,
        lease_expiry: Option<chrono::NaiveDate>,
        asset_account_code: Option<&str>,
        accum_depr_account_code: Option<&str>,
        depr_expense_account_code: Option<&str>,
        gain_loss_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FixedAsset> {
        if asset_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Asset number is required".to_string(),
            ));
        }
        if asset_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Asset name is required".to_string(),
            ));
        }
        if !VALID_ASSET_TYPES.contains(&asset_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid asset type '{}'. Must be one of: {}", asset_type, VALID_ASSET_TYPES.join(", ")
            )));
        }

        let cost: f64 = original_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Original cost must be a valid number".to_string(),
        ))?;
        if cost < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Original cost must be non-negative".to_string(),
            ));
        }

        let salvage: f64 = salvage_value.parse().map_err(|_| AtlasError::ValidationFailed(
            "Salvage value must be a valid number".to_string(),
        ))?;
        if salvage > cost {
            return Err(AtlasError::ValidationFailed(
                "Salvage value cannot exceed original cost".to_string(),
            ));
        }

        // Resolve category
        let mut category_id: Option<Uuid> = None;
        let mut resolved_dep_method = depreciation_method.unwrap_or("straight_line").to_string();
        let mut resolved_useful_life = useful_life_months.unwrap_or(60);
        let resolved_salvage_pct = salvage_value_percent.to_string();
        let mut resolved_asset_acct = asset_account_code.map(String::from);
        let mut resolved_accum_depr_acct = accum_depr_account_code.map(String::from);
        let mut resolved_depr_expense_acct = depr_expense_account_code.map(String::from);
        let mut resolved_gain_loss_acct = gain_loss_account_code.map(String::from);

        if let Some(cc) = category_code {
            let cat = self.get_category(org_id, cc).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Asset category '{}' not found", cc)
                ))?;
            category_id = Some(cat.id);
            if depreciation_method.is_none() {
                resolved_dep_method = cat.default_depreciation_method.clone();
            }
            if useful_life_months.is_none() {
                resolved_useful_life = cat.default_useful_life_months;
            }
            if asset_account_code.is_none() {
                resolved_asset_acct = cat.default_asset_account_code.clone();
            }
            if accum_depr_account_code.is_none() {
                resolved_accum_depr_acct = cat.default_accum_depr_account_code.clone();
            }
            if depr_expense_account_code.is_none() {
                resolved_depr_expense_acct = cat.default_depr_expense_account_code.clone();
            }
            if gain_loss_account_code.is_none() {
                resolved_gain_loss_acct = cat.default_gain_loss_account_code.clone();
            }
        }

        if !VALID_DEPRECIATION_METHODS.contains(&resolved_dep_method.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid depreciation method '{}'. Must be one of: {}",
                resolved_dep_method, VALID_DEPRECIATION_METHODS.join(", ")
            )));
        }

        if resolved_useful_life <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Useful life must be positive".to_string(),
            ));
        }

        // Resolve book
        let mut book_id: Option<Uuid> = None;
        if let Some(bc) = book_code {
            let book = self.get_book(org_id, bc).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Asset book '{}' not found", bc)
                ))?;
            book_id = Some(book.id);
        }

        info!("Creating fixed asset '{}' ({}) for org {}", asset_number, asset_name, org_id);

        self.repository.create_asset(
            org_id, asset_number, asset_name, description,
            category_id, category_code,
            book_id, book_code,
            asset_type,
            original_cost, salvage_value, &resolved_salvage_pct,
            &resolved_dep_method, resolved_useful_life,
            declining_balance_rate,
            acquisition_date,
            location, department_id, department_name,
            custodian_id, custodian_name,
            serial_number, tag_number, manufacturer, model,
            warranty_expiry, insurance_policy_number, insurance_expiry,
            lease_number, lease_expiry,
            resolved_asset_acct.as_deref(),
            resolved_accum_depr_acct.as_deref(),
            resolved_depr_expense_acct.as_deref(),
            resolved_gain_loss_acct.as_deref(),
            created_by,
        ).await
    }

    /// Get a fixed asset by ID
    pub async fn get_asset(&self, id: Uuid) -> AtlasResult<Option<FixedAsset>> {
        self.repository.get_asset(id).await
    }

    /// Get a fixed asset by number
    pub async fn get_asset_by_number(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<Option<FixedAsset>> {
        self.repository.get_asset_by_number(org_id, asset_number).await
    }

    /// List fixed assets with optional filters
    pub async fn list_assets(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        category_code: Option<&str>,
        book_code: Option<&str>,
    ) -> AtlasResult<Vec<FixedAsset>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_assets(org_id, status, category_code, book_code).await
    }

    // ========================================================================
    // Asset Lifecycle Management
    // ========================================================================

    /// Place an acquired asset in service
    pub async fn place_in_service(&self, asset_id: Uuid, in_service_date: Option<chrono::NaiveDate>) -> AtlasResult<FixedAsset> {
        let asset = self.repository.get_asset(asset_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Fixed asset {} not found", asset_id)
            ))?;

        if asset.status != "acquired" && asset.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot place asset in '{}' status in service. Must be 'acquired' or 'draft'.", asset.status)
            ));
        }

        let service_date = in_service_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        info!("Placing fixed asset {} in service on {}", asset.asset_number, service_date);

        self.repository.update_asset_status(asset_id, "in_service", Some(service_date), None, None).await
    }

    /// Mark an asset as acquired
    pub async fn acquire_asset(&self, asset_id: Uuid) -> AtlasResult<FixedAsset> {
        let asset = self.repository.get_asset(asset_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Fixed asset {} not found", asset_id)
            ))?;

        if asset.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot acquire asset in '{}' status. Must be 'draft'.", asset.status)
            ));
        }

        info!("Acquiring fixed asset {}", asset.asset_number);

        self.repository.update_asset_status(asset_id, "acquired", None, None, None).await
    }

    // ========================================================================
    // Depreciation Calculation
    // ========================================================================

    /// Calculate depreciation for a single asset for one period
    /// Returns the depreciation amount and updated asset
    pub async fn calculate_depreciation(
        &self,
        asset_id: Uuid,
        fiscal_year: i32,
        period_number: i32,
        period_name: Option<&str>,
        depreciation_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<(f64, FixedAsset)> {
        let asset = self.repository.get_asset(asset_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Fixed asset {} not found", asset_id)
            ))?;

        if asset.status != "in_service" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot depreciate asset in '{}' status. Must be 'in_service'.", asset.status)
            ));
        }

        // Check if already depreciated for this period
        if let Some(_existing) = self.repository.get_depreciation_for_period(
            asset_id, fiscal_year, period_number
        ).await? {
            return Err(AtlasError::ValidationFailed(
                format!("Asset already depreciated for FY{} period {}", fiscal_year, period_number)
            ));
        }

        let depreciable_basis: f64 = asset.depreciable_basis.parse().unwrap_or(0.0);
        let current_accum: f64 = asset.accumulated_depreciation.parse().unwrap_or(0.0);
        let useful_life = asset.useful_life_months;

        // Check if fully depreciated
        if asset.periods_depreciated >= useful_life {
            return Err(AtlasError::ValidationFailed(
                "Asset is fully depreciated".to_string(),
            ));
        }

        // Calculate depreciation amount based on method
        let dep_amount = match asset.depreciation_method.as_str() {
            "straight_line" => {
                self.calculate_straight_line(depreciable_basis, useful_life)
            }
            "declining_balance" => {
                let rate: f64 = asset.declining_balance_rate
                    .as_ref()
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(2.0); // Default: double declining
                let nbv: f64 = asset.net_book_value.parse().unwrap_or(0.0);
                self.calculate_declining_balance(nbv, rate, useful_life, asset.periods_depreciated)
            }
            "sum_of_years_digits" => {
                self.calculate_sum_of_years_digits(
                    depreciable_basis, useful_life, asset.periods_depreciated
                )
            }
            _ => {
                return Err(AtlasError::ValidationFailed(
                    format!("Unknown depreciation method: {}", asset.depreciation_method)
                ));
            }
        };

        // Don't depreciate past salvage value
        let new_accum = current_accum + dep_amount;
        let max_accum = depreciable_basis; // Can't exceed depreciable basis
        let actual_dep = if new_accum > max_accum {
            (max_accum - current_accum).max(0.0)
        } else {
            dep_amount
        };

        let final_accum = current_accum + actual_dep;
        let original_cost: f64 = asset.original_cost.parse().unwrap_or(0.0);
        let salvage: f64 = asset.salvage_value.parse().unwrap_or(0.0);
        let new_nbv = (original_cost - final_accum).max(salvage);

        info!("Depreciating asset {} by {:.2} (FY{} P{})", asset.asset_number, actual_dep, fiscal_year, period_number);

        // Create depreciation history entry
        self.repository.create_depreciation_entry(
            asset.organization_id, asset_id,
            fiscal_year, period_number, period_name,
            depreciation_date,
            &format!("{:.2}", actual_dep),
            &format!("{:.2}", final_accum),
            &format!("{:.2}", new_nbv),
            &asset.depreciation_method,
            created_by,
        ).await?;

        // Update asset depreciation
        let new_periods = asset.periods_depreciated + 1;
        let updated = self.repository.update_asset_depreciation(
            asset_id,
            &format!("{:.2}", final_accum),
            &format!("{:.2}", new_nbv),
            new_periods,
            Some(depreciation_date),
            &format!("{:.2}", actual_dep),
        ).await?;

        Ok((actual_dep, updated))
    }

    /// Straight-line depreciation: (Cost - Salvage) / Useful Life
    fn calculate_straight_line(&self, depreciable_basis: f64, useful_life_months: i32) -> f64 {
        if useful_life_months <= 0 {
            return 0.0;
        }
        depreciable_basis / useful_life_months as f64
    }

    /// Declining balance depreciation: NBV × (Rate / Useful Life)
    fn calculate_declining_balance(&self, net_book_value: f64, rate: f64, useful_life_months: i32, _periods_depreciated: i32) -> f64 {
        if useful_life_months <= 0 {
            return 0.0;
        }
        let straight_line_rate = 1.0 / useful_life_months as f64;
        let db_rate = straight_line_rate * rate;
        net_book_value * db_rate
    }

    /// Sum-of-years-digits depreciation
    fn calculate_sum_of_years_digits(&self, depreciable_basis: f64, useful_life_months: i32, periods_depreciated: i32) -> f64 {
        if useful_life_months <= 0 {
            return 0.0;
        }
        let n = useful_life_months as f64;
        let sum_of_years = n * (n + 1.0) / 2.0;
        let remaining_life = (useful_life_months - periods_depreciated) as f64;
        if remaining_life <= 0.0 || sum_of_years <= 0.0 {
            return 0.0;
        }
        depreciable_basis * (remaining_life / sum_of_years)
    }

    /// List depreciation history for an asset
    pub async fn list_depreciation_history(&self, asset_id: Uuid) -> AtlasResult<Vec<AssetDepreciationHistory>> {
        self.repository.list_depreciation_history(asset_id).await
    }

    // ========================================================================
    // Asset Transfers
    // ========================================================================

    /// Create an asset transfer request
    pub async fn create_transfer(
        &self,
        org_id: Uuid,
        asset_id: Uuid,
        to_department_id: Option<Uuid>,
        to_department_name: Option<&str>,
        to_location: Option<&str>,
        to_custodian_id: Option<Uuid>,
        to_custodian_name: Option<&str>,
        transfer_date: chrono::NaiveDate,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetTransfer> {
        let asset = self.repository.get_asset(asset_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Fixed asset {} not found", asset_id)
            ))?;

        if asset.status != "in_service" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot transfer asset in '{}' status. Must be 'in_service'.", asset.status)
            ));
        }

        let transfer_number = format!("ATR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating asset transfer {} for asset {}", transfer_number, asset.asset_number);

        self.repository.create_transfer(
            org_id, &transfer_number, asset_id,
            asset.department_id, asset.department_name.as_deref(),
            asset.location.as_deref(),
            asset.custodian_id, asset.custodian_name.as_deref(),
            to_department_id, to_department_name, to_location,
            to_custodian_id, to_custodian_name,
            transfer_date, reason, created_by,
        ).await
    }

    /// Approve an asset transfer
    pub async fn approve_transfer(&self, transfer_id: Uuid, approved_by: Uuid) -> AtlasResult<AssetTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Asset transfer {} not found", transfer_id)
            ))?;

        if transfer.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve transfer in '{}' status. Must be 'pending'.", transfer.status)
            ));
        }

        // Update the transfer
        let transfer = self.repository.update_transfer_status(transfer_id, "approved", Some(approved_by), None).await?;

        // Update the asset's assignment
        self.repository.update_asset_assignment(
            transfer.asset_id,
            transfer.to_department_id,
            transfer.to_department_name.as_deref(),
            transfer.to_location.as_deref(),
            transfer.to_custodian_id,
            transfer.to_custodian_name.as_deref(),
        ).await?;

        // Mark transfer as completed
        let transfer = self.repository.update_transfer_status(transfer_id, "completed", None, None).await?;

        info!("Approved and completed asset transfer {}", transfer_id);
        Ok(transfer)
    }

    /// Reject an asset transfer
    pub async fn reject_transfer(&self, transfer_id: Uuid, reason: Option<&str>) -> AtlasResult<AssetTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Asset transfer {} not found", transfer_id)
            ))?;

        if transfer.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject transfer in '{}' status. Must be 'pending'.", transfer.status)
            ));
        }

        info!("Rejecting asset transfer {}", transfer_id);
        self.repository.update_transfer_status(transfer_id, "rejected", None, reason).await
    }

    /// List asset transfers
    pub async fn list_transfers(&self, org_id: Uuid, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetTransfer>> {
        self.repository.list_transfers(org_id, asset_id).await
    }

    // ========================================================================
    // Asset Retirement
    // ========================================================================

    /// Create an asset retirement (disposal)
    pub async fn create_retirement(
        &self,
        org_id: Uuid,
        asset_id: Uuid,
        retirement_type: &str,
        retirement_date: chrono::NaiveDate,
        proceeds: &str,
        removal_cost: &str,
        reference_number: Option<&str>,
        buyer_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetRetirement> {
        let asset = self.repository.get_asset(asset_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Fixed asset {} not found", asset_id)
            ))?;

        if asset.status != "in_service" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot retire asset in '{}' status. Must be 'in_service'.", asset.status)
            ));
        }

        if !VALID_RETIREMENT_TYPES.contains(&retirement_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid retirement type '{}'. Must be one of: {}",
                retirement_type, VALID_RETIREMENT_TYPES.join(", ")
            )));
        }

        let proceeds_val: f64 = proceeds.parse().map_err(|_| AtlasError::ValidationFailed(
            "Proceeds must be a valid number".to_string(),
        ))?;
        let removal_val: f64 = removal_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Removal cost must be a valid number".to_string(),
        ))?;

        let nbv: f64 = asset.net_book_value.parse().unwrap_or(0.0);
        let accum: f64 = asset.accumulated_depreciation.parse().unwrap_or(0.0);

        // Gain/Loss = Proceeds - NBV - Removal Cost
        let gain_loss = proceeds_val - nbv - removal_val;
        let gain_loss_type = if gain_loss > 0.0 {
            "gain"
        } else if gain_loss < 0.0 {
            "loss"
        } else {
            "none"
        };

        let retirement_number = format!("ARE-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating asset retirement {} for asset {} ({})", retirement_number, asset.asset_number, retirement_type);

        self.repository.create_retirement(
            org_id, &retirement_number, asset_id,
            retirement_type, retirement_date,
            proceeds, removal_cost,
            &format!("{:.2}", nbv),
            &format!("{:.2}", accum),
            &format!("{:.2}", gain_loss.abs()),
            Some(gain_loss_type),
            asset.gain_loss_account_code.as_deref(), // gain account
            asset.gain_loss_account_code.as_deref(), // loss account
            None,  // cash account
            asset.asset_account_code.as_deref(),
            asset.accum_depr_account_code.as_deref(),
            reference_number, buyer_name, notes,
            created_by,
        ).await
    }

    /// Approve an asset retirement
    pub async fn approve_retirement(&self, retirement_id: Uuid, approved_by: Uuid) -> AtlasResult<AssetRetirement> {
        let retirement = self.repository.get_retirement(retirement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Asset retirement {} not found", retirement_id)
            ))?;

        if retirement.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve retirement in '{}' status. Must be 'pending'.", retirement.status)
            ));
        }

        // Update retirement status
        let retirement = self.repository.update_retirement_status(retirement_id, "approved", Some(approved_by)).await?;

        // Update asset status to retired
        let today = chrono::Utc::now().date_naive();
        self.repository.update_asset_status(retirement.asset_id, "retired", None, Some(today), Some(today)).await?;

        // Mark retirement as completed
        let retirement = self.repository.update_retirement_status(retirement_id, "completed", None).await?;

        info!("Approved and completed asset retirement {}", retirement_id);
        Ok(retirement)
    }

    /// List asset retirements
    pub async fn list_retirements(&self, org_id: Uuid, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetRetirement>> {
        self.repository.list_retirements(org_id, asset_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_depreciation_methods() {
        assert!(VALID_DEPRECIATION_METHODS.contains(&"straight_line"));
        assert!(VALID_DEPRECIATION_METHODS.contains(&"declining_balance"));
        assert!(VALID_DEPRECIATION_METHODS.contains(&"sum_of_years_digits"));
    }

    #[test]
    fn test_valid_asset_types() {
        assert!(VALID_ASSET_TYPES.contains(&"tangible"));
        assert!(VALID_ASSET_TYPES.contains(&"intangible"));
        assert!(VALID_ASSET_TYPES.contains(&"leased"));
        assert!(VALID_ASSET_TYPES.contains(&"cipc"));
    }

    #[test]
    fn test_valid_asset_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"acquired"));
        assert!(VALID_STATUSES.contains(&"in_service"));
        assert!(VALID_STATUSES.contains(&"retired"));
    }

    #[test]
    fn test_valid_retirement_types() {
        assert!(VALID_RETIREMENT_TYPES.contains(&"sale"));
        assert!(VALID_RETIREMENT_TYPES.contains(&"scrap"));
        assert!(VALID_RETIREMENT_TYPES.contains(&"donation"));
        assert!(VALID_RETIREMENT_TYPES.contains(&"write_off"));
        assert!(VALID_RETIREMENT_TYPES.contains(&"casualty"));
    }

    #[test]
    fn test_straight_line_depreciation() {
        let engine = FixedAssetEngine::new(Arc::new(crate::MockFixedAssetRepository));
        // Asset: 10000 cost, 1000 salvage, 60 months => (10000-1000)/60 = 150/month
        let dep = engine.calculate_straight_line(9000.0, 60);
        assert!((dep - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_straight_line_depreciation_zero_life() {
        let engine = FixedAssetEngine::new(Arc::new(crate::MockFixedAssetRepository));
        let dep = engine.calculate_straight_line(9000.0, 0);
        assert_eq!(dep, 0.0);
    }

    #[test]
    fn test_declining_balance_depreciation() {
        let engine = FixedAssetEngine::new(Arc::new(crate::MockFixedAssetRepository));
        // NBV=9000, rate=2.0 (double declining), life=60 months
        // Straight-line rate = 1/60, DB rate = 2/60 = 1/30
        // Depreciation = 9000 * (1/30) = 300
        let dep = engine.calculate_declining_balance(9000.0, 2.0, 60, 0);
        assert!((dep - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_sum_of_years_digits_depreciation() {
        let engine = FixedAssetEngine::new(Arc::new(crate::MockFixedAssetRepository));
        // Basis=9000, life=60 months, period 0
        // SOYD = 60*61/2 = 1830
        // Remaining = 60
        // Dep = 9000 * (60/1830) = 295.08
        let dep = engine.calculate_sum_of_years_digits(9000.0, 60, 0);
        assert!((dep - 295.08).abs() < 1.0);
    }

    #[test]
    fn test_sum_of_years_digits_fully_depreciated() {
        let engine = FixedAssetEngine::new(Arc::new(crate::MockFixedAssetRepository));
        let dep = engine.calculate_sum_of_years_digits(9000.0, 60, 60);
        assert_eq!(dep, 0.0);
    }
}
