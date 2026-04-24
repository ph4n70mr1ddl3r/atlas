//! Approval Authority Engine
//!
//! Business logic for managing approval authority limits and checking
//! whether a user/role is authorised to approve a given transaction amount.

use atlas_shared::{
    ApprovalAuthorityLimit, AuthorityCheckAudit,
    ApprovalAuthorityDashboard,
    CreateApprovalAuthorityLimitRequest,
    AtlasError, AtlasResult,
};
use super::ApprovalAuthorityRepository;
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

/// Valid owner types
const VALID_OWNER_TYPES: &[&str] = &["user", "role"];

/// Valid document types
const VALID_DOCUMENT_TYPES: &[&str] = &[
    "purchase_order",
    "purchase_requisition",
    "expense_report",
    "journal_entry",
    "journal_batch",
    "payment",
    "receipt",
    "invoice",
    "credit_memo",
    "debit_memo",
    "sales_order",
    "quote",
    "sales_contract",
    "procurement_contract",
    "intercompany_batch",
    "budget_transfer",
    "requisition",
    "work_order",
    "custom",
];

/// Valid statuses for a limit
const _VALID_STATUSES: &[&str] = &["active", "inactive"];

/// Approval Authority Engine
pub struct ApprovalAuthorityEngine {
    repository: Arc<dyn ApprovalAuthorityRepository>,
}

impl ApprovalAuthorityEngine {
    pub fn new(repository: Arc<dyn ApprovalAuthorityRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Limit CRUD
    // ========================================================================

    /// Create a new approval authority limit.
    pub async fn create_limit(
        &self,
        org_id: Uuid,
        request: CreateApprovalAuthorityLimitRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApprovalAuthorityLimit> {
        // --- validations ---
        if request.limit_code.is_empty() {
            return Err(AtlasError::ValidationFailed("limit_code is required".into()));
        }
        if request.name.is_empty() {
            return Err(AtlasError::ValidationFailed("name is required".into()));
        }
        if !VALID_OWNER_TYPES.contains(&request.owner_type.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid owner_type '{}'. Must be one of: {}",
                request.owner_type,
                VALID_OWNER_TYPES.join(", ")
            )));
        }
        if request.owner_type == "user" && request.user_id.is_none() {
            return Err(AtlasError::ValidationFailed(
                "user_id is required when owner_type is 'user'".into(),
            ));
        }
        if request.owner_type == "role" && request.role_name.is_none() {
            return Err(AtlasError::ValidationFailed(
                "role_name is required when owner_type is 'role'".into(),
            ));
        }
        if !VALID_DOCUMENT_TYPES.contains(&request.document_type.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid document_type '{}'. Must be one of: {}",
                request.document_type,
                VALID_DOCUMENT_TYPES.join(", ")
            )));
        }

        let limit_amount = request.approval_limit_amount.parse::<f64>()
            .map_err(|_| AtlasError::ValidationFailed(
                "approval_limit_amount must be a valid number".into(),
            ))?;
        if limit_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "approval_limit_amount must be greater than 0".into(),
            ));
        }

        if request.currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("currency_code is required".into()));
        }

        if let (Some(from), Some(to)) = (request.effective_from, request.effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "effective_to must be after effective_from".into(),
                ));
            }
        }

        // Uniqueness check
        if self.repository.get_limit_by_code(org_id, &request.limit_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Approval authority limit with code '{}' already exists", request.limit_code
            )));
        }

        let user_id = request.user_id.as_ref()
            .map(|s| Uuid::parse_str(s))
            .transpose()
            .map_err(|_| AtlasError::ValidationFailed("user_id must be a valid UUID".into()))?;

        let business_unit_id = request.business_unit_id.as_ref()
            .map(|s| Uuid::parse_str(s))
            .transpose()
            .map_err(|_| AtlasError::ValidationFailed("business_unit_id must be a valid UUID".into()))?;

        info!(
            "Creating approval authority limit {} ({} {} for {} {})",
            request.limit_code, request.owner_type,
            request.user_id.as_deref().unwrap_or(request.role_name.as_deref().unwrap_or("")),
            request.document_type, request.approval_limit_amount
        );

        self.repository.create_limit(
            org_id,
            &request.limit_code,
            &request.name,
            request.description.as_deref(),
            &request.owner_type,
            user_id,
            request.role_name.as_deref(),
            &request.document_type,
            &request.approval_limit_amount,
            &request.currency_code,
            business_unit_id,
            request.cost_center.as_deref(),
            request.effective_from,
            request.effective_to,
            created_by,
        ).await
    }

    /// Get a limit by ID.
    pub async fn get_limit(&self, id: Uuid) -> AtlasResult<Option<ApprovalAuthorityLimit>> {
        self.repository.get_limit(id).await
    }

    /// Get a limit by code.
    pub async fn get_limit_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ApprovalAuthorityLimit>> {
        self.repository.get_limit_by_code(org_id, code).await
    }

    /// List limits with optional filters.
    pub async fn list_limits(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        owner_type: Option<&str>,
        document_type: Option<&str>,
        user_id: Option<Uuid>,
        role_name: Option<&str>,
    ) -> AtlasResult<Vec<ApprovalAuthorityLimit>> {
        self.repository.list_limits(org_id, status, owner_type, document_type, user_id, role_name).await
    }

    /// Activate a limit.
    pub async fn activate_limit(&self, id: Uuid) -> AtlasResult<ApprovalAuthorityLimit> {
        let limit = self.get_limit(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Limit {} not found", id)))?;

        if limit.status == "active" {
            return Err(AtlasError::WorkflowError("Limit is already active".into()));
        }

        info!("Activated approval authority limit {}", limit.limit_code);
        self.repository.update_limit_status(id, "active").await
    }

    /// Deactivate a limit.
    pub async fn deactivate_limit(&self, id: Uuid) -> AtlasResult<ApprovalAuthorityLimit> {
        let limit = self.get_limit(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Limit {} not found", id)))?;

        if limit.status == "inactive" {
            return Err(AtlasError::WorkflowError("Limit is already inactive".into()));
        }

        info!("Deactivated approval authority limit {}", limit.limit_code);
        self.repository.update_limit_status(id, "inactive").await
    }

    /// Delete a limit.
    pub async fn delete_limit(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted approval authority limit {}", id);
        self.repository.delete_limit(id).await
    }

    // ========================================================================
    // Authority Checking
    // ========================================================================

    /// Check whether a user is authorized to approve a transaction of the
    /// given document type and amount.  Returns the applicable limit on
    /// success or an error on denial.
    ///
    /// Resolution order:
    /// 1. User-specific limit matching document type + business unit
    /// 2. User-specific limit matching document type (no BU)
    /// 3. Role-based limit matching document type + business unit
    /// 4. Role-based limit matching document type (no BU)
    ///
    /// Each tier considers only *active* and *currently effective* limits,
    /// and picks the one with the highest amount.
    pub async fn check_authority(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        user_roles: &[String],
        document_type: &str,
        amount: &str,
        business_unit_id: Option<Uuid>,
        cost_center: Option<&str>,
        document_id: Option<Uuid>,
    ) -> AtlasResult<AuthorityCheckAudit> {
        let requested_amount = amount.parse::<f64>()
            .map_err(|_| AtlasError::ValidationFailed("amount must be a valid number".into()))?;

        if requested_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed("amount must be greater than 0".into()));
        }

        // Try user-specific limits first (with BU, then without)
        let user_limit = self.resolve_limit(
            org_id, Some(user_id), None, document_type,
            business_unit_id, cost_center,
        ).await?;

        // Then role-based limits
        let role_limit = if user_limit.is_none() {
            let mut best: Option<ApprovalAuthorityLimit> = None;
            for role in user_roles {
                if let Some(lim) = self.resolve_limit(
                    org_id, None, Some(role.as_str()), document_type,
                    business_unit_id, cost_center,
                ).await? {
                    let lim_amount: f64 = lim.approval_limit_amount.parse().unwrap_or(0.0);
                    let best_amount: f64 = best.as_ref()
                        .map(|b| b.approval_limit_amount.parse().unwrap_or(0.0))
                        .unwrap_or(0.0);
                    if lim_amount > best_amount {
                        best = Some(lim);
                    }
                }
            }
            best
        } else {
            None
        };

        let applicable = user_limit.or(role_limit);

        let (result, reason, limit_id, applicable_limit_str): (String, Option<String>, Option<Uuid>, String) = match &applicable {
            Some(lim) => {
                let limit_amount: f64 = lim.approval_limit_amount.parse().unwrap_or(0.0);
                if requested_amount <= limit_amount {
                    ("approved".to_string(),
                     Some(format!("Within {} limit of {}", lim.owner_type, lim.approval_limit_amount)),
                     Some(lim.id),
                     lim.approval_limit_amount.clone())
                } else {
                    ("denied".to_string(),
                     Some(format!("Requested {} exceeds {} limit of {}",
                                  requested_amount, lim.owner_type, lim.approval_limit_amount)),
                     Some(lim.id),
                     lim.approval_limit_amount.clone())
                }
            }
            None => {
                ("denied".to_string(),
                 Some("No applicable approval authority limit found".to_string()),
                 None,
                 "0".to_string())
            }
        };

        info!(
            "Authority check: user={} doc={} amount={} => {} ({})",
            user_id, document_type, amount, result,
            reason.as_deref().unwrap_or("")
        );

        self.repository.create_check_audit(
            org_id,
            limit_id,
            user_id,
            user_roles.first().map(|s| s.as_str()),
            document_type,
            document_id,
            amount,
            &applicable_limit_str,
            &result,
            reason.as_deref(),
        ).await
    }

    /// List check audit entries.
    pub async fn list_check_audits(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        document_type: Option<&str>,
        result: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<AuthorityCheckAudit>> {
        self.repository.list_check_audits(org_id, user_id, document_type, result, limit).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard summary.
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ApprovalAuthorityDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Resolve the best matching active & effective limit.
    async fn resolve_limit(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        role_name: Option<&str>,
        document_type: &str,
        business_unit_id: Option<Uuid>,
        cost_center: Option<&str>,
    ) -> AtlasResult<Option<ApprovalAuthorityLimit>> {
        let owner_type = if user_id.is_some() { "user" } else { "role" };

        let limits = self.repository.find_applicable_limits(
            org_id, owner_type, user_id, role_name,
            document_type, business_unit_id, cost_center,
        ).await?;

        // Filter to currently effective and pick the one with the highest amount
        let today = chrono::Utc::now().date_naive();
        let mut best: Option<ApprovalAuthorityLimit> = None;
        for lim in limits {
            // Status check
            if lim.status != "active" {
                continue;
            }
            // Effective date check
            if let Some(from) = lim.effective_from {
                if today < from { continue; }
            }
            if let Some(to) = lim.effective_to {
                if today > to { continue; }
            }

            let lim_amount: f64 = lim.approval_limit_amount.parse().unwrap_or(0.0);
            let best_amount: f64 = best.as_ref()
                .map(|b| b.approval_limit_amount.parse().unwrap_or(0.0))
                .unwrap_or(0.0);

            // Prefer BU-scoped limits over global ones when amounts are equal
            let lim_bu_score = if lim.business_unit_id.is_some() { 1 } else { 0 };
            let best_bu_score = best.as_ref()
                .map(|b| if b.business_unit_id.is_some() { 1 } else { 0 })
                .unwrap_or(0);

            if lim_amount > best_amount || (lim_amount == best_amount && lim_bu_score > best_bu_score) {
                best = Some(lim);
            }
        }

        Ok(best)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_owner_types() {
        assert!(VALID_OWNER_TYPES.contains(&"user"));
        assert!(VALID_OWNER_TYPES.contains(&"role"));
        assert_eq!(VALID_OWNER_TYPES.len(), 2);
    }

    #[test]
    fn test_valid_document_types() {
        assert!(VALID_DOCUMENT_TYPES.contains(&"purchase_order"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"expense_report"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"journal_entry"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"invoice"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"payment"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"sales_order"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"custom"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(_VALID_STATUSES.contains(&"active"));
        assert!(_VALID_STATUSES.contains(&"inactive"));
    }
}
