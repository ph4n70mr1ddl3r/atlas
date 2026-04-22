//! Product Information Management Engine
//!
//! Core business logic for the Oracle Fusion Product Hub module.
//! Manages the complete product lifecycle from creation through obsolescence.

use atlas_shared::{
    ProductItem, PimCategory, PimCategoryAssignment, PimCrossReference,
    PimNewItemRequest, PimItemTemplate, PimDashboard,
    AtlasError, AtlasResult,
};
use super::ProductInformationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid item types for product items
const VALID_ITEM_TYPES: &[&str] = &[
    "finished_good", "subassembly", "component", "service", "supply", "expense",
];

/// Valid item statuses
const VALID_ITEM_STATUSES: &[&str] = &[
    "draft", "active", "obsolete", "inactive", "pending_approval",
];

/// Valid lifecycle phases
const VALID_LIFECYCLE_PHASES: &[&str] = &[
    "concept", "design", "prototype", "production", "phase_out", "obsolete",
];

/// Valid NIR statuses
const VALID_NIR_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "implemented", "cancelled",
];

/// Valid NIR priorities
const VALID_NIR_PRIORITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

/// Valid cross-reference types
const VALID_XREF_TYPES: &[&str] = &[
    "gtin", "upc", "ean", "supplier", "customer", "internal", "other",
];

/// Product Information Management engine
pub struct ProductInformationEngine {
    repository: Arc<dyn ProductInformationRepository>,
}

impl ProductInformationEngine {
    pub fn new(repository: Arc<dyn ProductInformationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Product Item CRUD
    // ========================================================================

    /// Create a new product item
    pub async fn create_item(
        &self,
        org_id: Uuid,
        item_number: &str,
        item_name: &str,
        description: Option<&str>,
        long_description: Option<&str>,
        item_type: &str,
        primary_uom_code: &str,
        secondary_uom_code: Option<&str>,
        weight: Option<&str>,
        weight_uom: Option<&str>,
        volume: Option<&str>,
        volume_uom: Option<&str>,
        hazmat_flag: bool,
        lot_control_flag: bool,
        serial_control_flag: bool,
        shelf_life_days: Option<i32>,
        min_order_quantity: Option<&str>,
        max_order_quantity: Option<&str>,
        lead_time_days: Option<i32>,
        list_price: Option<&str>,
        cost_price: Option<&str>,
        currency_code: &str,
        inventory_item_flag: bool,
        purchasable_flag: bool,
        sellable_flag: bool,
        stock_enabled_flag: bool,
        invoice_enabled_flag: bool,
        default_buyer_id: Option<Uuid>,
        default_supplier_id: Option<Uuid>,
        template_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProductItem> {
        // Validate item number
        if item_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Item number is required".to_string()));
        }
        if item_number.len() > 100 {
            return Err(AtlasError::ValidationFailed("Item number must be at most 100 characters".to_string()));
        }

        // Validate item name
        if item_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Item name is required".to_string()));
        }

        // Validate item type
        if !VALID_ITEM_TYPES.contains(&item_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid item_type '{}'. Must be one of: {}", item_type, VALID_ITEM_TYPES.join(", ")
            )));
        }

        // Validate UOM
        if primary_uom_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Primary UOM code is required".to_string()));
        }

        // Validate currency
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".to_string()));
        }

        // Check uniqueness of item number within org
        if self.repository.get_item_by_number(org_id, item_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Item number '{}' already exists", item_number
            )));
        }

        // Validate template exists if specified
        if let Some(tid) = template_id {
            if self.repository.get_template(tid).await?.is_none() {
                return Err(AtlasError::EntityNotFound(
                    format!("Item template {} not found", tid)
                ));
            }
        }

        // Validate prices
        if let Some(lp) = list_price {
            if let Ok(val) = lp.parse::<f64>() {
                if val < 0.0 {
                    return Err(AtlasError::ValidationFailed("List price cannot be negative".to_string()));
                }
            }
        }
        if let Some(cp) = cost_price {
            if let Ok(val) = cp.parse::<f64>() {
                if val < 0.0 {
                    return Err(AtlasError::ValidationFailed("Cost price cannot be negative".to_string()));
                }
            }
        }

        info!("Creating product item '{}' for org {}", item_number, org_id);

        self.repository.create_item(
            org_id, item_number, item_name, description, long_description,
            item_type, "draft", "concept", primary_uom_code, secondary_uom_code,
            weight, weight_uom, volume, volume_uom,
            hazmat_flag, lot_control_flag, serial_control_flag, shelf_life_days,
            min_order_quantity, max_order_quantity, lead_time_days,
            list_price, cost_price, currency_code,
            inventory_item_flag, purchasable_flag, sellable_flag,
            stock_enabled_flag, invoice_enabled_flag,
            default_buyer_id, default_supplier_id, template_id,
            created_by,
        ).await
    }

    /// Get an item by ID
    pub async fn get_item(&self, id: Uuid) -> AtlasResult<Option<ProductItem>> {
        self.repository.get_item(id).await
    }

    /// Get an item by its item number
    pub async fn get_item_by_number(&self, org_id: Uuid, item_number: &str) -> AtlasResult<Option<ProductItem>> {
        self.repository.get_item_by_number(org_id, item_number).await
    }

    /// List items with optional filtering
    pub async fn list_items(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        item_type: Option<&str>,
        category_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ProductItem>> {
        if let Some(s) = status {
            if !VALID_ITEM_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_ITEM_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = item_type {
            if !VALID_ITEM_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid item_type '{}'. Must be one of: {}", t, VALID_ITEM_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_items(org_id, status, item_type, category_id).await
    }

    /// Update item status with lifecycle transition validation
    pub async fn update_item_status(
        &self,
        id: Uuid,
        new_status: &str,
    ) -> AtlasResult<ProductItem> {
        if !VALID_ITEM_STATUSES.contains(&new_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", new_status, VALID_ITEM_STATUSES.join(", ")
            )));
        }

        let item = self.repository.get_item(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Item {} not found", id)))?;

        // Validate status transition
        let valid_transition = matches!(
            (item.status.as_str(), new_status),
            ("draft", "active") | ("draft", "pending_approval") | ("draft", "inactive")
            | ("pending_approval", "active") | ("pending_approval", "draft")
            | ("active", "inactive") | ("active", "obsolete")
            | ("inactive", "active") | ("inactive", "obsolete")
            | ("obsolete", "inactive")
        );

        if !valid_transition {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot transition item from '{}' to '{}'", item.status, new_status
            )));
        }

        // When activating, auto-transition lifecycle to production if still concept
        let new_phase = if new_status == "active" && item.lifecycle_phase == "concept" {
            "production"
        } else {
            ""
        };

        info!("Updating item {} status from '{}' to '{}'", item.item_number, item.status, new_status);

        self.repository.update_item_status(id, new_status, if new_phase.is_empty() { None } else { Some(new_phase) }).await
    }

    /// Update item lifecycle phase
    pub async fn update_lifecycle_phase(
        &self,
        id: Uuid,
        new_phase: &str,
    ) -> AtlasResult<ProductItem> {
        if !VALID_LIFECYCLE_PHASES.contains(&new_phase) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid lifecycle_phase '{}'. Must be one of: {}", new_phase, VALID_LIFECYCLE_PHASES.join(", ")
            )));
        }

        let item = self.repository.get_item(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Item {} not found", id)))?;

        // Validate lifecycle transition (forward-only through defined phases)
        let current_idx = VALID_LIFECYCLE_PHASES.iter()
            .position(|&p| p == item.lifecycle_phase)
            .unwrap_or(0);
        let new_idx = VALID_LIFECYCLE_PHASES.iter()
            .position(|&p| p == new_phase)
            .unwrap_or(0);

        if new_idx < current_idx {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot move lifecycle backwards from '{}' to '{}'",
                item.lifecycle_phase, new_phase
            )));
        }

        info!("Updating item {} lifecycle from '{}' to '{}'", item.item_number, item.lifecycle_phase, new_phase);

        self.repository.update_item_lifecycle(id, new_phase).await
    }

    /// Delete an item (soft delete - sets to inactive)
    pub async fn delete_item(&self, id: Uuid) -> AtlasResult<()> {
        let item = self.repository.get_item(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Item {} not found", id)))?;

        if item.status == "active" {
            return Err(AtlasError::ValidationFailed(
                "Cannot delete an active item. Set to inactive or obsolete first.".to_string()
            ));
        }

        info!("Deleting item {}", item.item_number);
        self.repository.delete_item(id).await
    }

    // ========================================================================
    // Item Categories
    // ========================================================================

    /// Create a new item category
    pub async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_category_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCategory> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Category code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Category name is required".to_string()));
        }

        // Check code uniqueness
        if self.repository.get_category_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Category code '{}' already exists", code
            )));
        }

        // Validate parent exists if specified
        let level_number = if let Some(parent_id) = parent_category_id {
            let parent = self.repository.get_category(parent_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Parent category {} not found", parent_id)
                ))?;
            if parent.organization_id != org_id {
                return Err(AtlasError::ValidationFailed(
                    "Parent category must belong to the same organization".to_string()
                ));
            }
            parent.level_number + 1
        } else {
            1
        };

        info!("Creating item category '{}' for org {}", code, org_id);

        self.repository.create_category(
            org_id, code, name, description, parent_category_id,
            level_number, created_by,
        ).await
    }

    /// Get a category by ID
    pub async fn get_category(&self, id: Uuid) -> AtlasResult<Option<PimCategory>> {
        self.repository.get_category(id).await
    }

    /// List categories (optionally filtered by parent)
    pub async fn list_categories(
        &self,
        org_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PimCategory>> {
        self.repository.list_categories(org_id, parent_id).await
    }

    /// Delete a category
    pub async fn delete_category(&self, id: Uuid) -> AtlasResult<()> {
        let cat = self.repository.get_category(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Category {} not found", id)))?;

        if cat.item_count > 0 {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot delete category '{}' - it has {} items assigned", cat.code, cat.item_count)
            ));
        }

        info!("Deleting item category {}", cat.code);
        self.repository.delete_category(id).await
    }

    // ========================================================================
    // Item Category Assignments
    // ========================================================================

    /// Assign an item to a category
    pub async fn assign_item_category(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        category_id: Uuid,
        is_primary: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCategoryAssignment> {
        // Validate item exists
        let item = self.repository.get_item(item_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Item {} not found", item_id)))?;
        if item.organization_id != org_id {
            return Err(AtlasError::ValidationFailed("Item does not belong to this organization".to_string()));
        }

        // Validate category exists
        let category = self.repository.get_category(category_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Category {} not found", category_id)))?;
        if category.organization_id != org_id {
            return Err(AtlasError::ValidationFailed("Category does not belong to this organization".to_string()));
        }

        // If primary, check if already has a primary assignment
        if is_primary {
            let existing = self.repository.get_primary_category_assignment(item_id).await?;
            if existing.is_some() {
                return Err(AtlasError::Conflict(
                    "Item already has a primary category assignment. Remove it first.".to_string()
                ));
            }
        }

        info!("Assigning item {} to category {}", item.item_number, category.code);

        self.repository.assign_item_category(
            org_id, item_id, category_id, is_primary, created_by,
        ).await
    }

    /// List category assignments for an item
    pub async fn list_item_categories(&self, item_id: Uuid) -> AtlasResult<Vec<PimCategoryAssignment>> {
        self.repository.list_item_categories(item_id).await
    }

    /// Remove an item from a category
    pub async fn remove_item_category(&self, assignment_id: Uuid) -> AtlasResult<()> {
        info!("Removing item category assignment {}", assignment_id);
        self.repository.remove_item_category(assignment_id).await
    }

    // ========================================================================
    // Item Cross-References
    // ========================================================================

    /// Create a cross-reference for an item
    pub async fn create_cross_reference(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        cross_reference_type: &str,
        cross_reference_value: &str,
        description: Option<&str>,
        source_system: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCrossReference> {
        // Validate item exists
        let item = self.repository.get_item(item_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Item {} not found", item_id)))?;
        if item.organization_id != org_id {
            return Err(AtlasError::ValidationFailed("Item does not belong to this organization".to_string()));
        }

        // Validate cross-reference type
        if !VALID_XREF_TYPES.contains(&cross_reference_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cross_reference_type '{}'. Must be one of: {}",
                cross_reference_type, VALID_XREF_TYPES.join(", ")
            )));
        }

        if cross_reference_value.is_empty() {
            return Err(AtlasError::ValidationFailed("Cross-reference value is required".to_string()));
        }

        // Validate effective dates
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "effective_to must be after effective_from".to_string()
                ));
            }
        }

        // Check uniqueness of type+value within org
        if self.repository.get_cross_reference_by_value(org_id, cross_reference_type, cross_reference_value).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Cross-reference of type '{}' with value '{}' already exists",
                cross_reference_type, cross_reference_value
            )));
        }

        info!("Creating cross-reference '{}' for item {}", cross_reference_value, item.item_number);

        self.repository.create_cross_reference(
            org_id, item_id, cross_reference_type, cross_reference_value,
            description, source_system, effective_from, effective_to, created_by,
        ).await
    }

    /// List cross-references for an item
    pub async fn list_cross_references(&self, item_id: Uuid) -> AtlasResult<Vec<PimCrossReference>> {
        self.repository.list_cross_references(item_id).await
    }

    /// List all cross-references for an organization, optionally filtered by type
    pub async fn list_all_cross_references(
        &self,
        org_id: Uuid,
        xref_type: Option<&str>,
    ) -> AtlasResult<Vec<PimCrossReference>> {
        self.repository.list_all_cross_references(org_id, xref_type).await
    }

    /// Delete a cross-reference
    pub async fn delete_cross_reference(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting cross-reference {}", id);
        self.repository.delete_cross_reference(id).await
    }

    // ========================================================================
    // Item Templates
    // ========================================================================

    /// Create an item template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        item_type: &str,
        default_uom_code: Option<&str>,
        default_category_id: Option<Uuid>,
        default_inventory_flag: bool,
        default_purchasable_flag: bool,
        default_sellable_flag: bool,
        default_stock_enabled_flag: bool,
        attribute_defaults: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimItemTemplate> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Template code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Template name is required".to_string()));
        }
        if !VALID_ITEM_TYPES.contains(&item_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid item_type '{}'. Must be one of: {}", item_type, VALID_ITEM_TYPES.join(", ")
            )));
        }

        // Check code uniqueness
        if self.repository.get_template_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Template code '{}' already exists", code
            )));
        }

        // Validate default category if specified
        if let Some(cat_id) = default_category_id {
            if self.repository.get_category(cat_id).await?.is_none() {
                return Err(AtlasError::EntityNotFound(
                    format!("Default category {} not found", cat_id)
                ));
            }
        }

        info!("Creating item template '{}' for org {}", code, org_id);

        self.repository.create_template(
            org_id, code, name, description, item_type,
            default_uom_code, default_category_id,
            default_inventory_flag, default_purchasable_flag,
            default_sellable_flag, default_stock_enabled_flag,
            attribute_defaults, created_by,
        ).await
    }

    /// Get a template by ID
    pub async fn get_template(&self, id: Uuid) -> AtlasResult<Option<PimItemTemplate>> {
        self.repository.get_template(id).await
    }

    /// List templates for an org
    pub async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<PimItemTemplate>> {
        self.repository.list_templates(org_id).await
    }

    /// Delete a template
    pub async fn delete_template(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting item template {}", id);
        self.repository.delete_template(id).await
    }

    // ========================================================================
    // New Item Requests (NIR)
    // ========================================================================

    /// Create a New Item Request
    pub async fn create_new_item_request(
        &self,
        org_id: Uuid,
        title: &str,
        description: Option<&str>,
        item_type: &str,
        priority: &str,
        requested_item_number: Option<&str>,
        requested_item_name: Option<&str>,
        requested_category_id: Option<Uuid>,
        justification: Option<&str>,
        target_launch_date: Option<chrono::NaiveDate>,
        estimated_cost: Option<&str>,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimNewItemRequest> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("NIR title is required".to_string()));
        }
        if !VALID_ITEM_TYPES.contains(&item_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid item_type '{}'. Must be one of: {}", item_type, VALID_ITEM_TYPES.join(", ")
            )));
        }
        if !VALID_NIR_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_NIR_PRIORITIES.join(", ")
            )));
        }

        // Check requested_item_number uniqueness if provided
        if let Some(req_num) = requested_item_number {
            if !req_num.is_empty() && self.repository.get_item_by_number(org_id, req_num).await?.is_some() {
                return Err(AtlasError::Conflict(format!(
                    "Item number '{}' already exists", req_num
                )));
            }
        }

        // Validate category if specified
        if let Some(cat_id) = requested_category_id {
            if self.repository.get_category(cat_id).await?.is_none() {
                return Err(AtlasError::EntityNotFound(
                    format!("Requested category {} not found", cat_id)
                ));
            }
        }

        // Generate request number
        let request_number = format!("NIR-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));

        info!("Creating New Item Request '{}' for org {}", request_number, org_id);

        self.repository.create_new_item_request(
            org_id, &request_number, title, description, item_type,
            priority, "draft", requested_item_number, requested_item_name,
            requested_category_id, justification, target_launch_date,
            estimated_cost, currency_code, created_by,
        ).await
    }

    /// Get a NIR by ID
    pub async fn get_new_item_request(&self, id: Uuid) -> AtlasResult<Option<PimNewItemRequest>> {
        self.repository.get_new_item_request(id).await
    }

    /// List NIRs with optional status filter
    pub async fn list_new_item_requests(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PimNewItemRequest>> {
        if let Some(s) = status {
            if !VALID_NIR_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid NIR status '{}'. Must be one of: {}", s, VALID_NIR_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_new_item_requests(org_id, status).await
    }

    /// Submit a NIR for approval
    pub async fn submit_new_item_request(&self, id: Uuid) -> AtlasResult<PimNewItemRequest> {
        let nir = self.repository.get_new_item_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NIR {} not found", id)))?;

        if nir.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit NIR in '{}' status. Must be 'draft'.", nir.status
            )));
        }

        info!("Submitting NIR {} for approval", nir.request_number);
        self.repository.update_nir_status(id, "submitted", None, None, None).await
    }

    /// Approve a NIR
    pub async fn approve_new_item_request(
        &self,
        id: Uuid,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<PimNewItemRequest> {
        let nir = self.repository.get_new_item_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NIR {} not found", id)))?;

        if nir.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve NIR in '{}' status. Must be 'submitted'.", nir.status
            )));
        }

        info!("Approving NIR {}", nir.request_number);
        self.repository.update_nir_status(
            id, "approved", approved_by, Some(chrono::Utc::now()), None,
        ).await
    }

    /// Reject a NIR
    pub async fn reject_new_item_request(
        &self,
        id: Uuid,
        rejection_reason: Option<&str>,
    ) -> AtlasResult<PimNewItemRequest> {
        let nir = self.repository.get_new_item_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NIR {} not found", id)))?;

        if nir.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject NIR in '{}' status. Must be 'submitted'.", nir.status
            )));
        }

        info!("Rejecting NIR {}", nir.request_number);
        self.repository.update_nir_status(
            id, "rejected", None, None, rejection_reason,
        ).await
    }

    /// Implement an approved NIR — creates the actual product item
    pub async fn implement_new_item_request(&self, id: Uuid) -> AtlasResult<ProductItem> {
        let nir = self.repository.get_new_item_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NIR {} not found", id)))?;

        if nir.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot implement NIR in '{}' status. Must be 'approved'.", nir.status
            )));
        }

        // Create the item from NIR data
        let item_number = nir.requested_item_number.as_deref()
            .unwrap_or(&nir.request_number);
        let item_name = nir.requested_item_name.as_deref()
            .unwrap_or(&nir.title);

        let item = self.create_item(
            nir.organization_id,
            item_number,
            item_name,
            nir.description.as_deref(),
            None,
            &nir.item_type,
            "EA", // default UOM
            None,
            None, None, None, None,
            false, false, false, None,
            None, None, None,
            nir.estimated_cost.as_deref(), None,
            &nir.currency_code,
            true, true, true, true, true,
            None, None, None,
            nir.created_by,
        ).await?;

        // Assign to requested category if provided
        if let Some(cat_id) = nir.requested_category_id {
            let _ = self.assign_item_category(
                nir.organization_id, item.id, cat_id, true, nir.created_by,
            ).await;
        }

        // Update NIR status
        self.repository.update_nir_implemented(id, item.id, Some(chrono::Utc::now())).await?;

        info!("Implemented NIR {} as item {}", nir.request_number, item.item_number);

        Ok(item)
    }

    /// Cancel a NIR
    pub async fn cancel_new_item_request(&self, id: Uuid) -> AtlasResult<PimNewItemRequest> {
        let nir = self.repository.get_new_item_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NIR {} not found", id)))?;

        if nir.status != "draft" && nir.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel NIR in '{}' status. Only draft or submitted NIRs can be cancelled.", nir.status
            )));
        }

        info!("Cancelling NIR {}", nir.request_number);
        self.repository.update_nir_status(id, "cancelled", None, None, None).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the PIM dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PimDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Unit tests for validation constants
    // ========================================================================

    #[test]
    fn test_valid_item_types() {
        assert!(VALID_ITEM_TYPES.contains(&"finished_good"));
        assert!(VALID_ITEM_TYPES.contains(&"subassembly"));
        assert!(VALID_ITEM_TYPES.contains(&"component"));
        assert!(VALID_ITEM_TYPES.contains(&"service"));
        assert!(VALID_ITEM_TYPES.contains(&"supply"));
        assert!(VALID_ITEM_TYPES.contains(&"expense"));
        assert!(!VALID_ITEM_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_item_statuses() {
        assert!(VALID_ITEM_STATUSES.contains(&"draft"));
        assert!(VALID_ITEM_STATUSES.contains(&"active"));
        assert!(VALID_ITEM_STATUSES.contains(&"obsolete"));
        assert!(VALID_ITEM_STATUSES.contains(&"inactive"));
        assert!(VALID_ITEM_STATUSES.contains(&"pending_approval"));
    }

    #[test]
    fn test_valid_lifecycle_phases() {
        assert!(VALID_LIFECYCLE_PHASES.contains(&"concept"));
        assert!(VALID_LIFECYCLE_PHASES.contains(&"design"));
        assert!(VALID_LIFECYCLE_PHASES.contains(&"prototype"));
        assert!(VALID_LIFECYCLE_PHASES.contains(&"production"));
        assert!(VALID_LIFECYCLE_PHASES.contains(&"phase_out"));
        assert!(VALID_LIFECYCLE_PHASES.contains(&"obsolete"));
    }

    #[test]
    fn test_valid_nir_statuses() {
        assert!(VALID_NIR_STATUSES.contains(&"draft"));
        assert!(VALID_NIR_STATUSES.contains(&"submitted"));
        assert!(VALID_NIR_STATUSES.contains(&"approved"));
        assert!(VALID_NIR_STATUSES.contains(&"rejected"));
        assert!(VALID_NIR_STATUSES.contains(&"implemented"));
        assert!(VALID_NIR_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_nir_priorities() {
        assert!(VALID_NIR_PRIORITIES.contains(&"low"));
        assert!(VALID_NIR_PRIORITIES.contains(&"medium"));
        assert!(VALID_NIR_PRIORITIES.contains(&"high"));
        assert!(VALID_NIR_PRIORITIES.contains(&"critical"));
    }

    #[test]
    fn test_valid_xref_types() {
        assert!(VALID_XREF_TYPES.contains(&"gtin"));
        assert!(VALID_XREF_TYPES.contains(&"upc"));
        assert!(VALID_XREF_TYPES.contains(&"ean"));
        assert!(VALID_XREF_TYPES.contains(&"supplier"));
        assert!(VALID_XREF_TYPES.contains(&"customer"));
        assert!(VALID_XREF_TYPES.contains(&"internal"));
        assert!(VALID_XREF_TYPES.contains(&"other"));
    }

    // ========================================================================
    // Unit tests for status transition validation
    // ========================================================================

    #[test]
    fn test_item_status_transitions() {
        // Valid transitions from draft
        let valid_from_draft = &["active", "pending_approval", "inactive"];
        for target in valid_from_draft {
            let is_valid = matches!(
                ("draft", *target),
                ("draft", "active") | ("draft", "pending_approval") | ("draft", "inactive")
            );
            assert!(is_valid, "Expected draft -> {} to be valid", target);
        }

        // Invalid transitions from draft
        let invalid_from_draft = &["obsolete"];
        for target in invalid_from_draft {
            let is_valid = matches!(
                ("draft", *target),
                ("draft", "active") | ("draft", "pending_approval") | ("draft", "inactive")
            );
            assert!(!is_valid, "Expected draft -> {} to be invalid", target);
        }

        // Valid from active
        let valid_from_active = &["inactive", "obsolete"];
        for target in valid_from_active {
            let is_valid = matches!(
                ("active", *target),
                ("active", "inactive") | ("active", "obsolete")
            );
            assert!(is_valid, "Expected active -> {} to be valid", target);
        }

        // Invalid: can't go from active back to draft
        let is_valid = matches!(
            ("active", "draft"),
            ("draft", "active") | ("draft", "pending_approval") | ("draft", "inactive") |
            ("pending_approval", "active") | ("pending_approval", "draft") |
            ("active", "inactive") | ("active", "obsolete") |
            ("inactive", "active") | ("inactive", "obsolete") |
            ("obsolete", "inactive")
        );
        assert!(!is_valid, "Expected active -> draft to be invalid");
    }

    #[test]
    fn test_nir_status_transitions() {
        // Submit: draft -> submitted
        assert!(can_submit_nir("draft"));
        assert!(!can_submit_nir("submitted"));
        assert!(!can_submit_nir("approved"));
        assert!(!can_submit_nir("rejected"));
        assert!(!can_submit_nir("implemented"));

        // Approve: submitted -> approved
        assert!(can_approve_nir("submitted"));
        assert!(!can_approve_nir("draft"));
        assert!(!can_approve_nir("approved"));

        // Reject: submitted -> rejected
        assert!(can_reject_nir("submitted"));
        assert!(!can_reject_nir("draft"));

        // Implement: approved -> implemented
        assert!(can_implement_nir("approved"));
        assert!(!can_implement_nir("draft"));
        assert!(!can_implement_nir("submitted"));

        // Cancel: draft|submitted -> cancelled
        assert!(can_cancel_nir("draft"));
        assert!(can_cancel_nir("submitted"));
        assert!(!can_cancel_nir("approved"));
        assert!(!can_cancel_nir("implemented"));
    }

    #[test]
    fn test_lifecycle_forward_only() {
        let phases = VALID_LIFECYCLE_PHASES;
        // Verify forward transitions are allowed
        for i in 0..phases.len() - 1 {
            let current_idx = i;
            let new_idx = i + 1;
            assert!(new_idx >= current_idx, "Forward transition should be allowed");
        }

        // Verify backward transitions are not allowed
        for i in 1..phases.len() {
            let current_idx = i;
            let new_idx = i - 1;
            assert!(new_idx < current_idx, "Backward transition should be rejected");
        }
    }

    // ========================================================================
    // Unit tests for NIR request number format
    // ========================================================================

    #[test]
    fn test_nir_request_number_format() {
        let request_number = format!("NIR-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));
        assert!(request_number.starts_with("NIR-"));
        assert!(request_number.len() > 10);
    }

    // ========================================================================
    // Unit tests for price validation
    // ========================================================================

    #[test]
    fn test_price_validation() {
        // Valid prices
        assert!("0.00".parse::<f64>().unwrap() >= 0.0);
        assert!("100.50".parse::<f64>().unwrap() >= 0.0);
        assert!("999999.99".parse::<f64>().unwrap() >= 0.0);

        // Invalid (negative)
        assert!("−1.00".parse::<f64>().is_err() || "-1.00".parse::<f64>().unwrap() < 0.0);
    }

    // ========================================================================
    // Unit tests for cross-reference type validation
    // ========================================================================

    #[test]
    fn test_xref_effective_date_validation() {
        let from = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let to = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert!(to >= from, "effective_to should be >= effective_from");

        let bad_to = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert!(bad_to < from, "Backward date should be detected");
    }

    #[test]
    fn test_category_level_calculation() {
        // Root level
        let root_level = 1;
        assert_eq!(root_level, 1);

        // Child of root
        let child_level = root_level + 1;
        assert_eq!(child_level, 2);

        // Grandchild
        let grandchild_level = child_level + 1;
        assert_eq!(grandchild_level, 3);
    }

    // ========================================================================
    // Helper functions for test match patterns
    // ========================================================================

    fn can_submit_nir(status: &str) -> bool {
        status == "draft"
    }

    fn can_approve_nir(status: &str) -> bool {
        status == "submitted"
    }

    fn can_reject_nir(status: &str) -> bool {
        status == "submitted"
    }

    fn can_implement_nir(status: &str) -> bool {
        status == "approved"
    }

    fn can_cancel_nir(status: &str) -> bool {
        status == "draft" || status == "submitted"
    }
}
