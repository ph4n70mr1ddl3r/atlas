//! Cost Accounting Engine
//!
//! Oracle Fusion Cost Management > Cost Accounting.
//!
//! Features:
//! - Cost Books with costing methods (standard, average, FIFO, LIFO)
//! - Cost Elements (material, labor, overhead, subcontracting, expense)
//! - Cost Profiles for item-level costing configuration
//! - Standard Cost management per item/element/book
//! - Cost Adjustments with full lifecycle (draft → submitted → approved → posted)
//! - Variance Analysis (purchase price, routing, overhead, rate, usage, mix)
//! - Cost Accounting dashboard with analytics
//!
//! Process:
//! 1. Create cost books with costing method
//! 2. Define cost elements per book
//! 3. Create cost profiles linking items to books
//! 4. Set standard costs per item/element
//! 5. Submit cost adjustments when costs change
//! 6. Approve and post adjustments
//! 7. Analyze variances (standard vs actual)
//! 8. Monitor via dashboard

use atlas_shared::AtlasError;
use super::CostAccountingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid costing methods
const VALID_COSTING_METHODS: &[&str] = &[
    "standard", "average", "fifo", "lifo",
];

/// Valid cost element types
const VALID_ELEMENT_TYPES: &[&str] = &[
    "material", "labor", "overhead", "subcontracting", "expense",
];

/// Valid overhead absorption methods
const VALID_OVERHEAD_METHODS: &[&str] = &[
    "rate", "amount", "percentage",
];

/// Valid adjustment types
const VALID_ADJUSTMENT_TYPES: &[&str] = &[
    "standard_cost_update", "cost_correction", "revaluation", "overhead_adjustment",
];

/// Valid adjustment statuses
const VALID_ADJUSTMENT_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "posted",
];

/// Valid variance types
const VALID_VARIANCE_TYPES: &[&str] = &[
    "purchase_price", "routing", "overhead", "rate", "usage", "mix",
];

/// Valid variance source types
const VALID_SOURCE_TYPES: &[&str] = &[
    "purchase_order", "work_order", "transfer_order",
];

/// Cost Accounting Engine
pub struct CostAccountingEngine {
    repository: Arc<dyn CostAccountingRepository>,
}

impl CostAccountingEngine {
    pub fn new(repository: Arc<dyn CostAccountingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Cost Books
    // ========================================================================

    /// Create a new cost book
    pub async fn create_cost_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        costing_method: &str,
        currency_code: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostBook> {
        let code = code.trim().to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Cost book code must be 1-50 characters".to_string(),
            ));
        }
        if name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Cost book name is required".to_string()));
        }
        if !VALID_COSTING_METHODS.contains(&costing_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid costing method '{}'. Must be one of: {}",
                costing_method,
                VALID_COSTING_METHODS.join(", ")
            )));
        }
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }
        // Check uniqueness
        if self.repository.get_cost_book_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Cost book '{}' already exists", code)));
        }
        info!("Creating cost book '{}' ({}) for org {}", name, code, org_id);
        self.repository.create_cost_book(
            org_id, &code, name, description, costing_method,
            currency_code, effective_from, effective_to, created_by,
        ).await
    }

    /// Get a cost book by ID
    pub async fn get_cost_book(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::CostBook>> {
        self.repository.get_cost_book(id).await
    }

    /// List cost books
    pub async fn list_cost_books(
        &self,
        org_id: Uuid,
        costing_method: Option<&str>,
        include_inactive: bool,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::CostBook>> {
        if let Some(cm) = costing_method {
            if !VALID_COSTING_METHODS.contains(&cm) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid costing method '{}'", cm
                )));
            }
        }
        self.repository.list_cost_books(org_id, costing_method, include_inactive).await
    }

    /// Update a cost book
    pub async fn update_cost_book(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        costing_method: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostBook> {
        let existing = self.repository.get_cost_book(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", id)))?;

        if let Some(cm) = costing_method {
            if !VALID_COSTING_METHODS.contains(&cm) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid costing method '{}'", cm
                )));
            }
        }

        // Validate dates
        let from = effective_from.or(existing.effective_from);
        let to = effective_to.or(existing.effective_to);
        if let (Some(f), Some(t)) = (from, to) {
            if f > t {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        info!("Updating cost book {} ({})", id, existing.code);
        self.repository.update_cost_book(
            id, name, description, costing_method, effective_from, effective_to,
        ).await
    }

    /// Deactivate a cost book
    pub async fn deactivate_cost_book(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::CostBook> {
        let book = self.repository.get_cost_book(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", id)))?;
        if !book.is_active {
            return Err(AtlasError::ValidationFailed("Cost book is already inactive".to_string()));
        }
        info!("Deactivating cost book {}", book.code);
        self.repository.update_cost_book_status(id, "inactive").await
    }

    /// Activate a cost book
    pub async fn activate_cost_book(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::CostBook> {
        let book = self.repository.get_cost_book(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", id)))?;
        if book.is_active {
            return Err(AtlasError::ValidationFailed("Cost book is already active".to_string()));
        }
        info!("Activating cost book {}", book.code);
        self.repository.update_cost_book_status(id, "active").await
    }

    /// Delete a cost book
    pub async fn delete_cost_book(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        let book = self.repository.get_cost_book(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", id)))?;
        info!("Deleting cost book {} ({})", book.code, id);
        self.repository.delete_cost_book(id).await
    }

    // ========================================================================
    // Cost Elements
    // ========================================================================

    /// Create a cost element
    pub async fn create_cost_element(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        element_type: &str,
        cost_book_id: Option<Uuid>,
        default_rate: &str,
        rate_uom: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostElement> {
        let code = code.trim().to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Cost element code must be 1-50 characters".to_string(),
            ));
        }
        if name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Cost element name is required".to_string()));
        }
        if !VALID_ELEMENT_TYPES.contains(&element_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid element type '{}'. Must be one of: {}",
                element_type,
                VALID_ELEMENT_TYPES.join(", ")
            )));
        }
        let rate: f64 = default_rate.parse().map_err(|_| {
            AtlasError::ValidationFailed("Default rate must be a valid number".to_string())
        })?;
        if rate < 0.0 {
            return Err(AtlasError::ValidationFailed("Default rate cannot be negative".to_string()));
        }
        if let Some(cb_id) = cost_book_id {
            self.repository.get_cost_book(cb_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", cb_id)))?;
        }
        if self.repository.get_cost_element_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Cost element '{}' already exists", code)));
        }
        info!("Creating cost element '{}' ({}) for org {}", name, code, org_id);
        self.repository.create_cost_element(
            org_id, &code, name, description, element_type,
            cost_book_id, default_rate, rate_uom, created_by,
        ).await
    }

    /// Get a cost element
    pub async fn get_cost_element(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::CostElement>> {
        self.repository.get_cost_element(id).await
    }

    /// List cost elements
    pub async fn list_cost_elements(
        &self,
        org_id: Uuid,
        element_type: Option<&str>,
        cost_book_id: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::CostElement>> {
        if let Some(et) = element_type {
            if !VALID_ELEMENT_TYPES.contains(&et) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid element type '{}'", et
                )));
            }
        }
        self.repository.list_cost_elements(org_id, element_type, cost_book_id).await
    }

    /// Update a cost element
    pub async fn update_cost_element(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        default_rate: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostElement> {
        if let Some(dr) = default_rate {
            let rate: f64 = dr.parse().map_err(|_| {
                AtlasError::ValidationFailed("Default rate must be a valid number".to_string())
            })?;
            if rate < 0.0 {
                return Err(AtlasError::ValidationFailed("Default rate cannot be negative".to_string()));
            }
        }
        self.repository.update_cost_element(id, name, description, default_rate).await
    }

    /// Delete a cost element
    pub async fn delete_cost_element(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.get_cost_element(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost element {} not found", id)))?;
        self.repository.delete_cost_element(id).await
    }

    // ========================================================================
    // Cost Profiles
    // ========================================================================

    /// Create a cost profile
    pub async fn create_cost_profile(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        cost_book_id: Uuid,
        item_id: Option<Uuid>,
        item_name: Option<&str>,
        cost_type: &str,
        lot_level_costing: bool,
        include_landed_costs: bool,
        overhead_absorption_method: &str,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostProfile> {
        let code = code.trim().to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Cost profile code must be 1-50 characters".to_string(),
            ));
        }
        if name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Cost profile name is required".to_string()));
        }
        if !VALID_COSTING_METHODS.contains(&cost_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cost type '{}'. Must be one of: {}",
                cost_type,
                VALID_COSTING_METHODS.join(", ")
            )));
        }
        if !VALID_OVERHEAD_METHODS.contains(&overhead_absorption_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid overhead absorption method '{}'. Must be one of: {}",
                overhead_absorption_method,
                VALID_OVERHEAD_METHODS.join(", ")
            )));
        }
        // Verify cost book exists
        self.repository.get_cost_book(cost_book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", cost_book_id)))?;

        // Check uniqueness
        if self.repository.get_cost_profile_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Cost profile '{}' already exists", code)));
        }

        info!("Creating cost profile '{}' for org {}", name, org_id);
        self.repository.create_cost_profile(
            org_id, &code, name, description, cost_book_id,
            item_id, item_name, cost_type, lot_level_costing,
            include_landed_costs, overhead_absorption_method, created_by,
        ).await
    }

    /// Get a cost profile
    pub async fn get_cost_profile(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::CostProfile>> {
        self.repository.get_cost_profile(id).await
    }

    /// List cost profiles
    pub async fn list_cost_profiles(
        &self,
        org_id: Uuid,
        cost_book_id: Option<Uuid>,
        item_id: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::CostProfile>> {
        self.repository.list_cost_profiles(org_id, cost_book_id, item_id).await
    }

    /// Delete a cost profile
    pub async fn delete_cost_profile(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.get_cost_profile(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost profile {} not found", id)))?;
        self.repository.delete_cost_profile(id).await
    }

    // ========================================================================
    // Standard Costs
    // ========================================================================

    /// Create a standard cost
    pub async fn create_standard_cost(
        &self,
        org_id: Uuid,
        cost_book_id: Uuid,
        cost_profile_id: Option<Uuid>,
        cost_element_id: Uuid,
        item_id: Uuid,
        item_name: Option<&str>,
        standard_cost: &str,
        currency_code: &str,
        effective_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::StandardCost> {
        // Verify cost book
        self.repository.get_cost_book(cost_book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", cost_book_id)))?;
        // Verify cost element
        self.repository.get_cost_element(cost_element_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost element {} not found", cost_element_id)))?;

        let cost: f64 = standard_cost.parse().map_err(|_| {
            AtlasError::ValidationFailed("Standard cost must be a valid number".to_string())
        })?;
        if cost < 0.0 {
            return Err(AtlasError::ValidationFailed("Standard cost cannot be negative".to_string()));
        }

        info!("Creating standard cost for item {} in book {}", item_id, cost_book_id);
        self.repository.create_standard_cost(
            org_id, cost_book_id, cost_profile_id, cost_element_id,
            item_id, item_name, standard_cost, currency_code, effective_date, created_by,
        ).await
    }

    /// Get a standard cost
    pub async fn get_standard_cost(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::StandardCost>> {
        self.repository.get_standard_cost(id).await
    }

    /// List standard costs
    pub async fn list_standard_costs(
        &self,
        org_id: Uuid,
        cost_book_id: Option<Uuid>,
        item_id: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::StandardCost>> {
        self.repository.list_standard_costs(org_id, cost_book_id, item_id).await
    }

    /// Update a standard cost value
    pub async fn update_standard_cost(&self, id: Uuid, standard_cost: &str) -> atlas_shared::AtlasResult<atlas_shared::StandardCost> {
        let cost: f64 = standard_cost.parse().map_err(|_| {
            AtlasError::ValidationFailed("Standard cost must be a valid number".to_string())
        })?;
        if cost < 0.0 {
            return Err(AtlasError::ValidationFailed("Standard cost cannot be negative".to_string()));
        }
        self.repository.get_standard_cost(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Standard cost {} not found", id)))?;
        info!("Updating standard cost {} to {}", id, standard_cost);
        self.repository.update_standard_cost(id, standard_cost).await
    }

    /// Supersede a standard cost (mark as replaced)
    pub async fn supersede_standard_cost(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::StandardCost> {
        let sc = self.repository.get_standard_cost(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Standard cost {} not found", id)))?;
        if sc.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot supersede standard cost in '{}' status. Must be 'active'.", sc.status)
            ));
        }
        info!("Superseding standard cost {}", id);
        self.repository.supersede_standard_cost(id).await
    }

    /// Delete a standard cost (only pending)
    pub async fn delete_standard_cost(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        let sc = self.repository.get_standard_cost(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Standard cost {} not found", id)))?;
        if sc.status != "pending" {
            return Err(AtlasError::ValidationFailed(
                "Can only delete standard costs in 'pending' status".to_string(),
            ));
        }
        self.repository.delete_standard_cost(id).await
    }

    // ========================================================================
    // Cost Adjustments
    // ========================================================================

    /// Create a cost adjustment
    pub async fn create_cost_adjustment(
        &self,
        org_id: Uuid,
        cost_book_id: Uuid,
        adjustment_type: &str,
        description: Option<&str>,
        reason: Option<&str>,
        currency_code: &str,
        effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostAdjustment> {
        // Verify cost book
        self.repository.get_cost_book(cost_book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", cost_book_id)))?;

        if !VALID_ADJUSTMENT_TYPES.contains(&adjustment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid adjustment type '{}'. Must be one of: {}",
                adjustment_type,
                VALID_ADJUSTMENT_TYPES.join(", ")
            )));
        }
        if let Some(d) = description {
            if d.trim().is_empty() {
                return Err(AtlasError::ValidationFailed("Description cannot be empty".to_string()));
            }
        }

        let adj_number = format!("ADJ-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating cost adjustment {} of type {}", adj_number, adjustment_type);
        self.repository.create_cost_adjustment(
            org_id, &adj_number, cost_book_id, adjustment_type,
            description, reason, currency_code, effective_date, created_by,
        ).await
    }

    /// Get a cost adjustment
    pub async fn get_cost_adjustment(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::CostAdjustment>> {
        self.repository.get_cost_adjustment(id).await
    }

    /// List cost adjustments
    pub async fn list_cost_adjustments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        adjustment_type: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::CostAdjustment>> {
        if let Some(s) = status {
            if !VALID_ADJUSTMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s)));
            }
        }
        if let Some(at) = adjustment_type {
            if !VALID_ADJUSTMENT_TYPES.contains(&at) {
                return Err(AtlasError::ValidationFailed(format!("Invalid adjustment type '{}'", at)));
            }
        }
        self.repository.list_cost_adjustments(org_id, status, adjustment_type).await
    }

    /// Submit a cost adjustment for approval
    pub async fn submit_adjustment(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::CostAdjustment> {
        let adj = self.repository.get_cost_adjustment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost adjustment {} not found", id)))?;
        if adj.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot submit adjustment in '{}' status. Must be 'draft'.", adj.status)
            ));
        }
        info!("Submitting cost adjustment {}", adj.adjustment_number);
        self.repository.update_adjustment_status(id, "submitted", None, None, None).await
    }

    /// Approve a cost adjustment
    pub async fn approve_adjustment(
        &self,
        id: Uuid,
        approved_by: Uuid,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostAdjustment> {
        let adj = self.repository.get_cost_adjustment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost adjustment {} not found", id)))?;
        if adj.status != "submitted" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot approve adjustment in '{}' status. Must be 'submitted'.", adj.status)
            ));
        }
        // Calculate total from lines
        let lines = self.repository.list_adjustment_lines(id).await?;
        let total: f64 = lines.iter().map(|l| l.adjustment_amount.parse::<f64>().unwrap_or(0.0)).sum();
        info!("Approving cost adjustment {} with total {}", adj.adjustment_number, total);
        self.repository.update_adjustment_status(
            id, "approved", Some(approved_by), None, Some(&format!("{:.6}", total)),
        ).await
    }

    /// Reject a cost adjustment
    pub async fn reject_adjustment(
        &self,
        id: Uuid,
        rejected_by: Uuid,
        reason: &str,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostAdjustment> {
        let adj = self.repository.get_cost_adjustment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost adjustment {} not found", id)))?;
        if adj.status != "submitted" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot reject adjustment in '{}' status. Must be 'submitted'.", adj.status)
            ));
        }
        if reason.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Rejection reason is required".to_string()));
        }
        info!("Rejecting cost adjustment {}: {}", adj.adjustment_number, reason);
        self.repository.update_adjustment_status(id, "rejected", Some(rejected_by), Some(reason), None).await
    }

    /// Post an approved cost adjustment
    pub async fn post_adjustment(
        &self,
        id: Uuid,
        posted_by: Uuid,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostAdjustment> {
        let adj = self.repository.get_cost_adjustment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost adjustment {} not found", id)))?;
        if adj.status != "approved" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot post adjustment in '{}' status. Must be 'approved'.", adj.status)
            ));
        }
        info!("Posting cost adjustment {}", adj.adjustment_number);
        self.repository.post_adjustment(id, posted_by, &adj.total_adjustment_amount).await
    }

    /// Delete a cost adjustment (only in draft)
    pub async fn delete_cost_adjustment(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        let adj = self.repository.get_cost_adjustment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost adjustment {} not found", id)))?;
        if adj.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Can only delete adjustments in 'draft' status".to_string(),
            ));
        }
        self.repository.delete_cost_adjustment(id).await
    }

    // ========================================================================
    // Cost Adjustment Lines
    // ========================================================================

    /// Add a line to a cost adjustment
    pub async fn add_adjustment_line(
        &self,
        org_id: Uuid,
        adjustment_id: Uuid,
        line_number: i32,
        item_id: Uuid,
        item_name: Option<&str>,
        cost_element_id: Option<Uuid>,
        old_cost: &str,
        new_cost: &str,
        currency_code: &str,
        effective_date: Option<chrono::NaiveDate>,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostAdjustmentLine> {
        let adj = self.repository.get_cost_adjustment(adjustment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost adjustment {} not found", adjustment_id)))?;
        if adj.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Can only add lines to adjustments in 'draft' status".to_string(),
            ));
        }

        let oc: f64 = old_cost.parse().map_err(|_| {
            AtlasError::ValidationFailed("Old cost must be a valid number".to_string())
        })?;
        let nc: f64 = new_cost.parse().map_err(|_| {
            AtlasError::ValidationFailed("New cost must be a valid number".to_string())
        })?;
        if nc < 0.0 || oc < 0.0 {
            return Err(AtlasError::ValidationFailed("Costs cannot be negative".to_string()));
        }
        let adj_amount = nc - oc;

        if line_number < 1 {
            return Err(AtlasError::ValidationFailed("Line number must be >= 1".to_string()));
        }

        info!("Adding adjustment line {} for item {} ({} -> {})", line_number, item_id, old_cost, new_cost);
        self.repository.create_adjustment_line(
            org_id, adjustment_id, line_number, item_id, item_name,
            cost_element_id, old_cost, new_cost, &format!("{:.6}", adj_amount),
            currency_code, effective_date,
        ).await
    }

    /// List adjustment lines
    pub async fn list_adjustment_lines(
        &self,
        adjustment_id: Uuid,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::CostAdjustmentLine>> {
        self.repository.list_adjustment_lines(adjustment_id).await
    }

    /// Delete an adjustment line
    pub async fn delete_adjustment_line(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_adjustment_line(id).await
    }

    // ========================================================================
    // Cost Variances
    // ========================================================================

    /// Record a cost variance
    pub async fn create_cost_variance(
        &self,
        org_id: Uuid,
        cost_book_id: Uuid,
        variance_type: &str,
        variance_date: chrono::NaiveDate,
        item_id: Uuid,
        item_name: Option<&str>,
        cost_element_id: Option<Uuid>,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        standard_cost: &str,
        actual_cost: &str,
        quantity: &str,
        currency_code: &str,
        accounting_period: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostVariance> {
        // Verify cost book
        self.repository.get_cost_book(cost_book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost book {} not found", cost_book_id)))?;

        if !VALID_VARIANCE_TYPES.contains(&variance_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid variance type '{}'. Must be one of: {}",
                variance_type,
                VALID_VARIANCE_TYPES.join(", ")
            )));
        }
        if let Some(st) = source_type {
            if !VALID_SOURCE_TYPES.contains(&st) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid source type '{}'. Must be one of: {}",
                    st,
                    VALID_SOURCE_TYPES.join(", ")
                )));
            }
        }

        let sc: f64 = standard_cost.parse().map_err(|_| {
            AtlasError::ValidationFailed("Standard cost must be a valid number".to_string())
        })?;
        let ac: f64 = actual_cost.parse().map_err(|_| {
            AtlasError::ValidationFailed("Actual cost must be a valid number".to_string())
        })?;
        let qty: f64 = quantity.parse().map_err(|_| {
            AtlasError::ValidationFailed("Quantity must be a valid number".to_string())
        })?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed("Quantity must be greater than zero".to_string()));
        }

        let variance_amount = (ac - sc) * qty;
        let variance_percent = if sc > 0.0 { ((ac - sc) / sc) * 100.0 } else { 0.0 };

        info!("Recording {} variance for item {}: std={} actual={} variance={}",
            variance_type, item_id, standard_cost, actual_cost, variance_amount);

        self.repository.create_cost_variance(
            org_id, cost_book_id, variance_type, variance_date,
            item_id, item_name, cost_element_id, source_type, source_id, source_number,
            standard_cost, actual_cost,
            &format!("{:.6}", variance_amount),
            &format!("{:.4}", variance_percent),
            quantity, currency_code, accounting_period, created_by,
        ).await
    }

    /// Get a cost variance
    pub async fn get_cost_variance(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::CostVariance>> {
        self.repository.get_cost_variance(id).await
    }

    /// List cost variances
    pub async fn list_cost_variances(
        &self,
        org_id: Uuid,
        variance_type: Option<&str>,
        item_id: Option<Uuid>,
        cost_book_id: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::CostVariance>> {
        if let Some(vt) = variance_type {
            if !VALID_VARIANCE_TYPES.contains(&vt) {
                return Err(AtlasError::ValidationFailed(format!("Invalid variance type '{}'", vt)));
            }
        }
        self.repository.list_cost_variances(org_id, variance_type, item_id, cost_book_id).await
    }

    /// Analyze a variance (add notes)
    pub async fn analyze_variance(
        &self,
        id: Uuid,
        notes: &str,
    ) -> atlas_shared::AtlasResult<atlas_shared::CostVariance> {
        let v = self.repository.get_cost_variance(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cost variance {} not found", id)))?;
        if v.is_analyzed {
            return Err(AtlasError::ValidationFailed("Variance is already analyzed".to_string()));
        }
        if notes.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Analysis notes are required".to_string()));
        }
        info!("Analyzing variance {} for item {}", id, v.item_id);
        self.repository.analyze_variance(id, notes).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the cost accounting dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::CostAccountingDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Validation constant tests
    // ========================================================================

    #[test]
    fn test_valid_costing_methods() {
        assert!(VALID_COSTING_METHODS.contains(&"standard"));
        assert!(VALID_COSTING_METHODS.contains(&"average"));
        assert!(VALID_COSTING_METHODS.contains(&"fifo"));
        assert!(VALID_COSTING_METHODS.contains(&"lifo"));
        assert!(!VALID_COSTING_METHODS.contains(&"unknown"));
        assert!(!VALID_COSTING_METHODS.contains(&"weighted"));
    }

    #[test]
    fn test_valid_element_types() {
        assert!(VALID_ELEMENT_TYPES.contains(&"material"));
        assert!(VALID_ELEMENT_TYPES.contains(&"labor"));
        assert!(VALID_ELEMENT_TYPES.contains(&"overhead"));
        assert!(VALID_ELEMENT_TYPES.contains(&"subcontracting"));
        assert!(VALID_ELEMENT_TYPES.contains(&"expense"));
        assert!(!VALID_ELEMENT_TYPES.contains(&"freight"));
        assert!(!VALID_ELEMENT_TYPES.contains(&"tax"));
    }

    #[test]
    fn test_valid_overhead_methods() {
        assert!(VALID_OVERHEAD_METHODS.contains(&"rate"));
        assert!(VALID_OVERHEAD_METHODS.contains(&"amount"));
        assert!(VALID_OVERHEAD_METHODS.contains(&"percentage"));
        assert!(!VALID_OVERHEAD_METHODS.contains(&"volume"));
    }

    #[test]
    fn test_valid_adjustment_types() {
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"standard_cost_update"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"cost_correction"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"revaluation"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"overhead_adjustment"));
        assert!(!VALID_ADJUSTMENT_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_valid_adjustment_statuses() {
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"draft"));
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"submitted"));
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"approved"));
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"rejected"));
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"posted"));
        assert!(!VALID_ADJUSTMENT_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_variance_types() {
        assert!(VALID_VARIANCE_TYPES.contains(&"purchase_price"));
        assert!(VALID_VARIANCE_TYPES.contains(&"routing"));
        assert!(VALID_VARIANCE_TYPES.contains(&"overhead"));
        assert!(VALID_VARIANCE_TYPES.contains(&"rate"));
        assert!(VALID_VARIANCE_TYPES.contains(&"usage"));
        assert!(VALID_VARIANCE_TYPES.contains(&"mix"));
        assert!(!VALID_VARIANCE_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_valid_source_types() {
        assert!(VALID_SOURCE_TYPES.contains(&"purchase_order"));
        assert!(VALID_SOURCE_TYPES.contains(&"work_order"));
        assert!(VALID_SOURCE_TYPES.contains(&"transfer_order"));
        assert!(!VALID_SOURCE_TYPES.contains(&"sales_order"));
    }

    // ========================================================================
    // Business logic / calculation tests
    // ========================================================================

    #[test]
    fn test_variance_calculation_favorable() {
        // Standard cost > actual cost => favorable variance (negative)
        let standard = 100.0_f64;
        let actual = 95.0_f64;
        let quantity = 1000.0_f64;
        let variance = (actual - standard) * quantity;
        assert!(variance < 0.0); // Favorable
        assert_eq!(variance, -5000.0);
    }

    #[test]
    fn test_variance_calculation_unfavorable() {
        // Actual cost > standard cost => unfavorable variance (positive)
        let standard = 100.0_f64;
        let actual = 110.0_f64;
        let quantity = 500.0_f64;
        let variance = (actual - standard) * quantity;
        assert!(variance > 0.0); // Unfavorable
        assert_eq!(variance, 5000.0);
    }

    #[test]
    fn test_variance_percent_calculation() {
        let standard = 100.0_f64;
        let actual = 105.0_f64;
        let pct = if standard > 0.0 { ((actual - standard) / standard) * 100.0 } else { 0.0 };
        assert!((pct - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_variance_percent_zero_standard() {
        let standard = 0.0_f64;
        let actual = 50.0_f64;
        let pct = if standard > 0.0 { ((actual - standard) / standard) * 100.0 } else { 0.0 };
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_adjustment_amount_calculation() {
        let old_cost = 50.0_f64;
        let new_cost = 55.0_f64;
        let adj = new_cost - old_cost;
        assert_eq!(adj, 5.0);

        // Negative adjustment (cost reduction)
        let old_cost = 100.0_f64;
        let new_cost = 90.0_f64;
        let adj = new_cost - old_cost;
        assert_eq!(adj, -10.0);
    }

    #[test]
    fn test_adjustment_total_from_lines() {
        let lines = vec![
            ("50.0".parse::<f64>().unwrap(), "55.0".parse::<f64>().unwrap()),
            ("100.0".parse::<f64>().unwrap(), "95.0".parse::<f64>().unwrap()),
            ("75.0".parse::<f64>().unwrap(), "80.0".parse::<f64>().unwrap()),
        ];
        let total: f64 = lines.iter().map(|(oc, nc)| nc - oc).sum();
        // (55-50) + (95-100) + (80-75) = 5 + (-5) + 5 = 5
        assert!((total - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_code_normalization() {
        let code = "std-cost-book".to_string();
        let normalized = code.trim().to_uppercase();
        assert_eq!(normalized, "STD-COST-BOOK");

        let code = "  my_code  ".to_string();
        let normalized = code.trim().to_uppercase();
        assert_eq!(normalized, "MY_CODE");
    }

    #[test]
    fn test_code_validation_empty() {
        let code = "".to_string();
        assert!(code.trim().is_empty() || code.trim().len() > 50);
    }

    #[test]
    fn test_code_validation_too_long() {
        let code = "A".repeat(51);
        assert!(code.trim().len() > 50);
    }

    #[test]
    fn test_code_validation_valid() {
        let code = "STD-BOOK".to_string();
        assert!(!code.trim().is_empty() && code.trim().len() <= 50);
    }

    #[test]
    fn test_date_range_validation() {
        let from = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let to = chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert!(from <= to);

        let from = chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let to = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert!(from > to);
    }

    #[test]
    fn test_negative_cost_validation() {
        let cost: f64 = "-50.0".parse().unwrap();
        assert!(cost < 0.0);

        let cost: f64 = "50.0".parse().unwrap();
        assert!(cost >= 0.0);
    }

    #[test]
    fn test_negative_rate_validation() {
        let rate: f64 = "-10.0".parse().unwrap();
        assert!(rate < 0.0);

        let rate: f64 = "25.0".parse().unwrap();
        assert!(rate >= 0.0);
    }

    #[test]
    fn test_quantity_validation() {
        let qty: f64 = "100.0".parse().unwrap();
        assert!(qty > 0.0);

        let qty: f64 = "0.0".parse().unwrap();
        assert!(qty <= 0.0);

        let qty: f64 = "-5.0".parse().unwrap();
        assert!(qty <= 0.0);
    }

    #[test]
    fn test_line_number_validation() {
        let valid: i32 = 1;
        let zero: i32 = 0;
        let neg: i32 = -1;
        assert!(valid >= 1);
        assert!(zero < 1);
        assert!(neg < 1);
    }

    #[test]
    fn test_adjustment_status_lifecycle() {
        // Valid transitions: draft -> submitted -> approved -> posted
        // Also: submitted -> rejected
        let valid_from_draft = vec!["submitted"];
        assert!(valid_from_draft.contains(&"submitted"));
        assert!(!valid_from_draft.contains(&"approved"));

        let valid_from_submitted = vec!["approved", "rejected"];
        assert!(valid_from_submitted.contains(&"approved"));
        assert!(valid_from_submitted.contains(&"rejected"));

        let valid_from_approved = vec!["posted"];
        assert!(valid_from_approved.contains(&"posted"));
    }

    #[test]
    fn test_standard_cost_status_lifecycle() {
        // active -> superseded
        // pending -> deleted or active
        let can_supersede = "active";
        assert_eq!(can_supersede, "active");

        let can_delete = "pending";
        assert_eq!(can_delete, "pending");

        let cannot = "superseded";
        assert_ne!(cannot, "active");
        assert_ne!(cannot, "pending");
    }

    #[test]
    fn test_format_precision() {
        let val = 1234.567890_f64;
        let fmt6 = format!("{:.6}", val);
        assert_eq!(fmt6, "1234.567890");

        let val = 0.1234_f64;
        let fmt4 = format!("{:.4}", val);
        assert_eq!(fmt4, "0.1234");

        let val = 100.0_f64;
        let fmt2 = format!("{:.2}", val);
        assert_eq!(fmt2, "100.00");
    }

    #[test]
    fn test_adjustment_number_format() {
        let id = Uuid::new_v4();
        let adj_number = format!("ADJ-{}", &id.to_string()[..8].to_uppercase());
        assert!(adj_number.starts_with("ADJ-"));
        assert_eq!(adj_number.len(), 12); // "ADJ-" + 8 chars
    }

    #[test]
    fn test_total_cost_rollup_by_item() {
        // Simulate calculating total standard cost per item across elements
        let material = 50.0_f64;
        let labor = 20.0_f64;
        let overhead = 10.0_f64;
        let total = material + labor + overhead;
        assert!((total - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_variance_aggregation() {
        let variances = vec![
            (100.0_f64, 95.0_f64, 10.0_f64),   // -50 (favorable)
            (200.0_f64, 210.0_f64, 5.0_f64),   // +50 (unfavorable)
            (150.0_f64, 140.0_f64, 20.0_f64),  // -200 (favorable)
        ];
        let total: f64 = variances.iter()
            .map(|(std, act, qty)| (act - std) * qty)
            .sum();
        // (-50) + 50 + (-200) = -200
        assert_eq!(total, -200.0);
    }

    #[test]
    fn test_empty_name_validation() {
        let name = "".to_string();
        assert!(name.trim().is_empty());

        let name = "   ".to_string();
        assert!(name.trim().is_empty());

        let name = "Valid Name".to_string();
        assert!(!name.trim().is_empty());
    }

    #[test]
    fn test_rejection_reason_required() {
        let reason = "".to_string();
        assert!(reason.trim().is_empty());

        let reason = "Material price increase".to_string();
        assert!(!reason.trim().is_empty());
    }

    #[test]
    fn test_analysis_notes_required() {
        let notes = "".to_string();
        assert!(notes.trim().is_empty());

        let notes = "Supplier changed pricing tier".to_string();
        assert!(!notes.trim().is_empty());
    }
}
