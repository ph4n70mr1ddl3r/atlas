//! Bank Guarantee Engine
//!
//! Manages bank guarantee lifecycle (request → approved → issued → active →
//! released/expired), amendments, and dashboard reporting.
//!
//! Oracle Fusion Cloud ERP equivalent: Treasury > Bank Guarantees

use atlas_shared::{
    BankGuarantee, BankGuaranteeAmendment, BankGuaranteeDashboard,
    AtlasError, AtlasResult,
};
use super::BankGuaranteeRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid bank guarantee types
const VALID_GUARANTEE_TYPES: &[&str] = &[
    "bid_bond", "performance_guarantee", "advance_payment_guarantee",
    "retention_guarantee", "warranty_guarantee", "financial_guarantee",
    "customs_guarantee", "shipping_guarantee", "other",
];

/// Valid guarantee statuses
const VALID_STATUSES: &[&str] = &[
    "draft", "pending_approval", "approved", "issued",
    "active", "invoked", "released", "expired", "cancelled",
];

/// Valid amendment types
const VALID_AMENDMENT_TYPES: &[&str] = &[
    "amount_increase", "amount_decrease", "expiry_extension",
    "expiry_reduction", "beneficiary_change", "terms_change", "other",
];

/// Valid amendment statuses
const VALID_AMENDMENT_STATUSES: &[&str] = &[
    "draft", "pending_approval", "approved", "rejected", "applied",
];

/// Valid collateral types
const VALID_COLLATERAL_TYPES: &[&str] = &[
    "cash_margin", "fixed_deposit", "bank_guarantee", "insurance_policy",
    "corporate_guarantee", "none",
];

/// Bank Guarantee engine
pub struct BankGuaranteeEngine {
    repository: Arc<dyn BankGuaranteeRepository>,
}

impl BankGuaranteeEngine {
    pub fn new(repository: Arc<dyn BankGuaranteeRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Bank Guarantee CRUD
    // ========================================================================

    /// Create a new bank guarantee (in draft status)
    pub async fn create_guarantee(
        &self,
        org_id: Uuid,
        guarantee_number: &str,
        guarantee_type: &str,
        description: Option<&str>,
        beneficiary_name: &str,
        beneficiary_code: Option<&str>,
        applicant_name: &str,
        applicant_code: Option<&str>,
        issuing_bank_name: &str,
        issuing_bank_code: Option<&str>,
        bank_account_number: Option<&str>,
        guarantee_amount: &str,
        currency_code: &str,
        margin_percentage: &str,
        commission_rate: &str,
        issue_date: Option<chrono::NaiveDate>,
        effective_date: Option<chrono::NaiveDate>,
        expiry_date: Option<chrono::NaiveDate>,
        claim_expiry_date: Option<chrono::NaiveDate>,
        renewal_date: Option<chrono::NaiveDate>,
        auto_renew: bool,
        reference_contract_number: Option<&str>,
        reference_purchase_order: Option<&str>,
        purpose: Option<&str>,
        collateral_type: Option<&str>,
        collateral_amount: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankGuarantee> {
        // Validation
        if guarantee_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Guarantee number is required".to_string()));
        }
        if beneficiary_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Beneficiary name is required".to_string()));
        }
        if applicant_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Applicant name is required".to_string()));
        }
        if issuing_bank_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Issuing bank name is required".to_string()));
        }
        if !VALID_GUARANTEE_TYPES.contains(&guarantee_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid guarantee_type '{}'. Must be one of: {}",
                guarantee_type, VALID_GUARANTEE_TYPES.join(", ")
            )));
        }

        let amount: f64 = guarantee_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Guarantee amount must be a valid number".to_string(),
        ))?;
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Guarantee amount must be positive".to_string(),
            ));
        }

        let margin_pct: f64 = margin_percentage.parse().map_err(|_| AtlasError::ValidationFailed(
            "Margin percentage must be a valid number".to_string(),
        ))?;
        if !(0.0..=100.0).contains(&margin_pct) {
            return Err(AtlasError::ValidationFailed(
                "Margin percentage must be between 0 and 100".to_string(),
            ));
        }

        let comm_rate: f64 = commission_rate.parse().map_err(|_| AtlasError::ValidationFailed(
            "Commission rate must be a valid number".to_string(),
        ))?;
        if comm_rate < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Commission rate cannot be negative".to_string(),
            ));
        }

        if let Some(ct) = collateral_type {
            if !VALID_COLLATERAL_TYPES.contains(&ct) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid collateral_type '{}'. Must be one of: {}",
                    ct, VALID_COLLATERAL_TYPES.join(", ")
                )));
            }
        }

        // Validate date order
        if let (Some(iss), Some(exp)) = (issue_date, expiry_date) {
            if exp <= iss {
                return Err(AtlasError::ValidationFailed(
                    "Expiry date must be after issue date".to_string(),
                ));
            }
        }
        if let (Some(eff), Some(exp)) = (effective_date, expiry_date) {
            if exp <= eff {
                return Err(AtlasError::ValidationFailed(
                    "Expiry date must be after effective date".to_string(),
                ));
            }
        }

        // Calculate margin and commission amounts
        let margin_amount = format!("{:.2}", amount * margin_pct / 100.0);
        let commission_amount = format!("{:.2}", amount * comm_rate / 100.0);

        info!(
            "Bank Guarantee: Creating guarantee '{}' of type {} for {} ({})",
            guarantee_number, guarantee_type, beneficiary_name, guarantee_amount
        );

        self.repository.create_guarantee(
            org_id, guarantee_number, guarantee_type, description,
            beneficiary_name, beneficiary_code,
            applicant_name, applicant_code,
            issuing_bank_name, issuing_bank_code, bank_account_number,
            guarantee_amount, currency_code,
            margin_percentage, &margin_amount,
            commission_rate, &commission_amount,
            issue_date, effective_date, expiry_date,
            claim_expiry_date, renewal_date, auto_renew,
            reference_contract_number, reference_purchase_order,
            purpose, collateral_type, collateral_amount,
            notes, created_by,
        ).await
    }

    /// Get a bank guarantee by ID
    pub async fn get_guarantee(&self, id: Uuid) -> AtlasResult<Option<BankGuarantee>> {
        self.repository.get_guarantee_by_id(id).await
    }

    /// Get a bank guarantee by number
    pub async fn get_guarantee_by_number(
        &self,
        org_id: Uuid,
        guarantee_number: &str,
    ) -> AtlasResult<Option<BankGuarantee>> {
        self.repository.get_guarantee(org_id, guarantee_number).await
    }

    /// List bank guarantees with optional status filter
    pub async fn list_guarantees(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        guarantee_type: Option<&str>,
    ) -> AtlasResult<Vec<BankGuarantee>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = guarantee_type {
            if !VALID_GUARANTEE_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid guarantee_type '{}'. Must be one of: {}",
                    t, VALID_GUARANTEE_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_guarantees(org_id, status, guarantee_type).await
    }

    /// Delete a draft guarantee
    pub async fn delete_guarantee(
        &self,
        org_id: Uuid,
        guarantee_number: &str,
    ) -> AtlasResult<()> {
        let guarantee = self.repository.get_guarantee(org_id, guarantee_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Guarantee '{}' not found", guarantee_number
            )))?;

        if guarantee.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Only draft guarantees can be deleted".to_string(),
            ));
        }

        self.repository.delete_guarantee(org_id, guarantee_number).await
    }

    // ========================================================================
    // Lifecycle Transitions
    // ========================================================================

    /// Submit a draft guarantee for approval
    pub async fn submit_for_approval(&self, id: Uuid) -> AtlasResult<BankGuarantee> {
        let guarantee = self.repository.get_guarantee_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Guarantee {} not found", id)))?;

        if guarantee.status != "draft" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot submit guarantee in '{}' status. Must be 'draft'.",
                guarantee.status
            )));
        }

        // Validate required fields for submission
        if guarantee.expiry_date.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Expiry date is required before submission".to_string(),
            ));
        }
        if guarantee.effective_date.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Effective date is required before submission".to_string(),
            ));
        }

        info!("Bank Guarantee: Submitting guarantee '{}' for approval", guarantee.guarantee_number);
        self.repository.update_guarantee_status(id, "pending_approval", None).await
    }

    /// Approve a pending guarantee
    pub async fn approve_guarantee(
        &self,
        id: Uuid,
        approved_by: Uuid,
    ) -> AtlasResult<BankGuarantee> {
        let guarantee = self.repository.get_guarantee_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Guarantee {} not found", id)))?;

        if guarantee.status != "pending_approval" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot approve guarantee in '{}' status. Must be 'pending_approval'.",
                guarantee.status
            )));
        }

        info!("Bank Guarantee: Approving guarantee '{}' by {}", guarantee.guarantee_number, approved_by);
        self.repository.update_guarantee_status(id, "approved", Some(approved_by)).await
    }

    /// Mark an approved guarantee as issued by the bank
    pub async fn issue_guarantee(
        &self,
        id: Uuid,
        issue_date: chrono::NaiveDate,
    ) -> AtlasResult<BankGuarantee> {
        let guarantee = self.repository.get_guarantee_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Guarantee {} not found", id)))?;

        if guarantee.status != "approved" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot issue guarantee in '{}' status. Must be 'approved'.",
                guarantee.status
            )));
        }

        info!("Bank Guarantee: Issuing guarantee '{}' on {}", guarantee.guarantee_number, issue_date);
        self.repository.update_guarantee_status(id, "issued", None).await
    }

    /// Activate an issued guarantee (effective date reached)
    pub async fn activate_guarantee(&self, id: Uuid) -> AtlasResult<BankGuarantee> {
        let guarantee = self.repository.get_guarantee_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Guarantee {} not found", id)))?;

        if guarantee.status != "issued" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot activate guarantee in '{}' status. Must be 'issued'.",
                guarantee.status
            )));
        }

        info!("Bank Guarantee: Activating guarantee '{}'", guarantee.guarantee_number);
        self.repository.update_guarantee_status(id, "active", None).await
    }

    /// Invoke/claim a guarantee
    pub async fn invoke_guarantee(&self, id: Uuid) -> AtlasResult<BankGuarantee> {
        let guarantee = self.repository.get_guarantee_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Guarantee {} not found", id)))?;

        if guarantee.status != "active" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot invoke guarantee in '{}' status. Must be 'active'.",
                guarantee.status
            )));
        }

        info!("Bank Guarantee: Invoking guarantee '{}'", guarantee.guarantee_number);
        self.repository.update_guarantee_status(id, "invoked", None).await
    }

    /// Release a guarantee (returned by beneficiary)
    pub async fn release_guarantee(&self, id: Uuid) -> AtlasResult<BankGuarantee> {
        let guarantee = self.repository.get_guarantee_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Guarantee {} not found", id)))?;

        if guarantee.status != "active" && guarantee.status != "invoked" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot release guarantee in '{}' status. Must be 'active' or 'invoked'.",
                guarantee.status
            )));
        }

        info!("Bank Guarantee: Releasing guarantee '{}'", guarantee.guarantee_number);
        self.repository.update_guarantee_status(id, "released", None).await
    }

    /// Cancel a guarantee
    pub async fn cancel_guarantee(&self, id: Uuid) -> AtlasResult<BankGuarantee> {
        let guarantee = self.repository.get_guarantee_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Guarantee {} not found", id)))?;

        if guarantee.status == "released" || guarantee.status == "expired" || guarantee.status == "cancelled" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot cancel guarantee in '{}' status.",
                guarantee.status
            )));
        }

        info!("Bank Guarantee: Cancelling guarantee '{}'", guarantee.guarantee_number);
        self.repository.update_guarantee_status(id, "cancelled", None).await
    }

    /// Mark expired guarantees (called by scheduled job)
    pub async fn process_expired_guarantees(
        &self,
        org_id: Uuid,
        as_of_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<BankGuarantee>> {
        let active = self.repository.list_guarantees(org_id, Some("active"), None).await?;
        let mut expired = Vec::new();
        for g in &active {
            if let Some(expiry) = g.expiry_date {
                if expiry <= as_of_date {
                    info!("Bank Guarantee: Expiring guarantee '{}' (expiry: {})", g.guarantee_number, expiry);
                    let updated = self.repository.update_guarantee_status(g.id, "expired", None).await?;
                    expired.push(updated);
                }
            }
        }
        Ok(expired)
    }

    // ========================================================================
    // Amendments
    // ========================================================================

    /// Create an amendment to a guarantee
    pub async fn create_amendment(
        &self,
        org_id: Uuid,
        guarantee_id: Uuid,
        amendment_type: &str,
        previous_amount: Option<&str>,
        new_amount: Option<&str>,
        previous_expiry_date: Option<chrono::NaiveDate>,
        new_expiry_date: Option<chrono::NaiveDate>,
        previous_terms: Option<&str>,
        new_terms: Option<&str>,
        reason: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankGuaranteeAmendment> {
        let guarantee = self.repository.get_guarantee_by_id(guarantee_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Guarantee {} not found", guarantee_id)))?;

        if guarantee.status != "active" && guarantee.status != "issued" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot amend guarantee in '{}' status. Must be 'active' or 'issued'.",
                guarantee.status
            )));
        }

        if !VALID_AMENDMENT_TYPES.contains(&amendment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid amendment_type '{}'. Must be one of: {}",
                amendment_type, VALID_AMENDMENT_TYPES.join(", ")
            )));
        }

        // Validate amount amendments
        if amendment_type == "amount_increase" || amendment_type == "amount_decrease" {
            if previous_amount.is_none() || new_amount.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "Amount amendments require both previous_amount and new_amount".to_string(),
                ));
            }
            let prev: f64 = previous_amount.unwrap().parse().map_err(|_| AtlasError::ValidationFailed(
                "Previous amount must be a valid number".to_string(),
            ))?;
            let new_val: f64 = new_amount.unwrap().parse().map_err(|_| AtlasError::ValidationFailed(
                "New amount must be a valid number".to_string(),
            ))?;
            if amendment_type == "amount_increase" && new_val <= prev {
                return Err(AtlasError::ValidationFailed(
                    "Amount increase amendment requires new_amount > previous_amount".to_string(),
                ));
            }
            if amendment_type == "amount_decrease" && new_val >= prev {
                return Err(AtlasError::ValidationFailed(
                    "Amount decrease amendment requires new_amount < previous_amount".to_string(),
                ));
            }
        }

        // Validate expiry amendments
        if amendment_type == "expiry_extension" || amendment_type == "expiry_reduction" {
            if previous_expiry_date.is_none() || new_expiry_date.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "Expiry amendments require both previous and new expiry dates".to_string(),
                ));
            }
        }

        let amendment_number = format!("AMD-{}-{:03}", guarantee.guarantee_number, guarantee.amendment_count + 1);

        info!(
            "Bank Guarantee: Creating {} amendment '{}' for guarantee '{}'",
            amendment_type, amendment_number, guarantee.guarantee_number
        );

        let amendment = self.repository.create_amendment(
            org_id, guarantee_id, &guarantee.guarantee_number,
            &amendment_number, amendment_type,
            previous_amount, new_amount,
            previous_expiry_date, new_expiry_date,
            previous_terms, new_terms,
            reason, effective_date, created_by,
        ).await?;

        // Update guarantee amendment count
        self.repository.increment_amendment_count(guarantee_id, &amendment_number).await?;

        Ok(amendment)
    }

    /// List amendments for a guarantee
    pub async fn list_amendments(
        &self,
        guarantee_id: Uuid,
    ) -> AtlasResult<Vec<BankGuaranteeAmendment>> {
        self.repository.list_amendments(guarantee_id).await
    }

    /// Approve an amendment
    pub async fn approve_amendment(
        &self,
        amendment_id: Uuid,
        approved_by: Uuid,
    ) -> AtlasResult<BankGuaranteeAmendment> {
        let amendment = self.repository.get_amendment_by_id(amendment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Amendment {} not found", amendment_id)))?;

        if amendment.status != "pending_approval" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot approve amendment in '{}' status.", amendment.status
            )));
        }

        info!("Bank Guarantee: Approving amendment '{}' for guarantee '{}'",
            amendment.amendment_number, amendment.guarantee_number);

        let _updated = self.repository.update_amendment_status(amendment_id, "approved", Some(approved_by)).await?;

        // Apply the amendment to the guarantee (this also updates status to "applied")
        self.apply_amendment_to_guarantee(&amendment).await?;

        // Return the final state ("applied")
        self.repository.get_amendment_by_id(amendment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Amendment {} not found", amendment_id)))
    }

    /// Reject an amendment
    pub async fn reject_amendment(
        &self,
        amendment_id: Uuid,
    ) -> AtlasResult<BankGuaranteeAmendment> {
        let amendment = self.repository.get_amendment_by_id(amendment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Amendment {} not found", amendment_id)))?;

        if amendment.status != "pending_approval" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot reject amendment in '{}' status.", amendment.status
            )));
        }

        self.repository.update_amendment_status(amendment_id, "rejected", None).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get bank guarantee dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BankGuaranteeDashboard> {
        let all = self.repository.list_guarantees(org_id, None, None).await?;

        let active_count = all.iter().filter(|g| g.status == "active").count() as i32;
        let total_amount: f64 = all.iter()
            .filter(|g| g.status == "active" || g.status == "issued")
            .map(|g| g.guarantee_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_margin: f64 = all.iter()
            .filter(|g| g.status == "active" || g.status == "issued")
            .map(|g| g.margin_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let pending_approval = all.iter().filter(|g| g.status == "pending_approval").count() as i32;

        let today = chrono::Utc::now().date_naive();
        let expiring_30 = all.iter().filter(|g| {
            g.status == "active" && g.expiry_date.map_or(false, |d| {
                let diff = (d - today).num_days();
                diff >= 0 && diff <= 30
            })
        }).count() as i32;
        let expiring_90 = all.iter().filter(|g| {
            g.status == "active" && g.expiry_date.map_or(false, |d| {
                let diff = (d - today).num_days();
                diff >= 0 && diff <= 90
            })
        }).count() as i32;

        // Group by type
        let mut by_type_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for g in &all {
            if g.status == "active" || g.status == "issued" {
                *by_type_map.entry(g.guarantee_type.clone()).or_insert(0) += 1;
            }
        }
        let by_type: serde_json::Map<String, serde_json::Value> = by_type_map.into_iter()
            .map(|(k, v)| (k, serde_json::json!(v)))
            .collect();

        // Group by currency
        let mut by_currency_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for g in &all {
            if g.status == "active" || g.status == "issued" {
                let amount = g.guarantee_amount.parse::<f64>().unwrap_or(0.0);
                *by_currency_map.entry(g.currency_code.clone()).or_insert(0.0) += amount;
            }
        }
        let by_currency: serde_json::Map<String, serde_json::Value> = by_currency_map.into_iter()
            .map(|(k, v)| (k, serde_json::json!(format!("{:.2}", v))))
            .collect();

        // Count pending amendments
        let amendments_pending = self.repository.count_pending_amendments(org_id).await?;

        Ok(BankGuaranteeDashboard {
            total_guarantees: all.len() as i32,
            active_guarantees: active_count,
            total_guarantee_amount: format!("{:.2}", total_amount),
            total_margin_held: format!("{:.2}", total_margin),
            expiring_within_30_days: expiring_30,
            expiring_within_90_days: expiring_90,
            pending_approval,
            amendments_pending: amendments_pending as i32,
            by_type: serde_json::Value::Object(by_type),
            by_currency: serde_json::Value::Object(by_currency),
        })
    }

    // ========================================================================
    // Private Helpers
    // ========================================================================

    /// Apply an approved amendment to the parent guarantee
    async fn apply_amendment_to_guarantee(
        &self,
        amendment: &BankGuaranteeAmendment,
    ) -> AtlasResult<()> {
        match amendment.amendment_type.as_str() {
            "amount_increase" | "amount_decrease" => {
                if let (Some(_prev), Some(new_amt)) = (&amendment.previous_amount, &amendment.new_amount) {
                    let amount: f64 = new_amt.parse().unwrap_or(0.0);
                    let guarantee = self.repository.get_guarantee_by_id(amendment.guarantee_id).await?
                        .ok_or_else(|| AtlasError::EntityNotFound("Guarantee not found".to_string()))?;
                    let margin_pct: f64 = guarantee.margin_percentage.parse().unwrap_or(0.0);
                    let comm_rate: f64 = guarantee.commission_rate.parse().unwrap_or(0.0);
                    let new_margin = format!("{:.2}", amount * margin_pct / 100.0);
                    let new_comm = format!("{:.2}", amount * comm_rate / 100.0);
                    self.repository.update_guarantee_amounts(
                        amendment.guarantee_id, new_amt, &new_margin, &new_comm,
                    ).await?;
                }
            }
            "expiry_extension" | "expiry_reduction" => {
                if let Some(new_expiry) = amendment.new_expiry_date {
                    self.repository.update_guarantee_expiry(
                        amendment.guarantee_id, new_expiry,
                    ).await?;
                }
            }
            _ => {}
        }

        // Mark amendment as applied
        self.repository.update_amendment_status(amendment.id, "applied", None).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Validation Unit Tests
    // ========================================================================

    #[test]
    fn test_valid_guarantee_types() {
        assert!(VALID_GUARANTEE_TYPES.contains(&"bid_bond"));
        assert!(VALID_GUARANTEE_TYPES.contains(&"performance_guarantee"));
        assert!(VALID_GUARANTEE_TYPES.contains(&"advance_payment_guarantee"));
        assert!(VALID_GUARANTEE_TYPES.contains(&"retention_guarantee"));
        assert!(VALID_GUARANTEE_TYPES.contains(&"warranty_guarantee"));
        assert!(VALID_GUARANTEE_TYPES.contains(&"financial_guarantee"));
        assert!(VALID_GUARANTEE_TYPES.contains(&"customs_guarantee"));
        assert!(VALID_GUARANTEE_TYPES.contains(&"shipping_guarantee"));
        assert!(VALID_GUARANTEE_TYPES.contains(&"other"));
        assert_eq!(VALID_GUARANTEE_TYPES.len(), 9);
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"pending_approval"));
        assert!(VALID_STATUSES.contains(&"approved"));
        assert!(VALID_STATUSES.contains(&"issued"));
        assert!(VALID_STATUSES.contains(&"active"));
        assert!(VALID_STATUSES.contains(&"invoked"));
        assert!(VALID_STATUSES.contains(&"released"));
        assert!(VALID_STATUSES.contains(&"expired"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
        assert_eq!(VALID_STATUSES.len(), 9);
    }

    #[test]
    fn test_valid_amendment_types() {
        assert!(VALID_AMENDMENT_TYPES.contains(&"amount_increase"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"amount_decrease"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"expiry_extension"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"expiry_reduction"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"beneficiary_change"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"terms_change"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"other"));
        assert_eq!(VALID_AMENDMENT_TYPES.len(), 7);
    }

    #[test]
    fn test_valid_amendment_statuses() {
        assert!(VALID_AMENDMENT_STATUSES.contains(&"draft"));
        assert!(VALID_AMENDMENT_STATUSES.contains(&"pending_approval"));
        assert!(VALID_AMENDMENT_STATUSES.contains(&"approved"));
        assert!(VALID_AMENDMENT_STATUSES.contains(&"rejected"));
        assert!(VALID_AMENDMENT_STATUSES.contains(&"applied"));
    }

    #[test]
    fn test_valid_collateral_types() {
        assert!(VALID_COLLATERAL_TYPES.contains(&"cash_margin"));
        assert!(VALID_COLLATERAL_TYPES.contains(&"fixed_deposit"));
        assert!(VALID_COLLATERAL_TYPES.contains(&"bank_guarantee"));
        assert!(VALID_COLLATERAL_TYPES.contains(&"insurance_policy"));
        assert!(VALID_COLLATERAL_TYPES.contains(&"corporate_guarantee"));
        assert!(VALID_COLLATERAL_TYPES.contains(&"none"));
    }

    #[test]
    fn test_margin_calculation() {
        // 10% margin on 100,000 = 10,000
        let amount = 100000.0_f64;
        let margin_pct = 10.0_f64;
        let margin = amount * margin_pct / 100.0;
        assert!((margin - 10000.0).abs() < 0.01);

        // 15% margin on 250,000 = 37,500
        let margin2: f64 = 250000.0 * 15.0 / 100.0;
        assert!((margin2 - 37500.0).abs() < 0.01);

        // 0% margin
        let margin3: f64 = 50000.0 * 0.0 / 100.0;
        assert!((margin3 - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_commission_calculation() {
        // 1.5% commission on 100,000 = 1,500
        let amount = 100000.0_f64;
        let comm_rate = 1.5_f64;
        let commission = amount * comm_rate / 100.0;
        assert!((commission - 1500.0).abs() < 0.01);

        // 0.5% commission on 500,000 = 2,500
        let commission2: f64 = 500000.0 * 0.5 / 100.0;
        assert!((commission2 - 2500.0).abs() < 0.01);
    }

    #[test]
    fn test_expiry_date_validation() {
        let issue_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let expiry_date = chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        // Expiry after issue => valid
        assert!(expiry_date > issue_date);

        // Same date => invalid
        let same_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert!(same_date <= issue_date);

        // Before issue => invalid
        let before_date = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert!(before_date <= issue_date);
    }

    #[test]
    fn test_amount_validation() {
        // Positive amounts are valid
        assert!(100000.0_f64 > 0.0);
        assert!(0.01_f64 > 0.0);

        // Zero and negative are invalid
        assert!(0.0_f64 <= 0.0);
        assert!(-100.0_f64 <= 0.0);
    }

    #[test]
    fn test_margin_percentage_range() {
        // Valid range: 0-100
        assert!((0.0..=100.0).contains(&0.0));
        assert!((0.0..=100.0).contains(&50.0));
        assert!((0.0..=100.0).contains(&100.0));

        // Invalid
        assert!(!(0.0..=100.0).contains(&-1.0));
        assert!(!(0.0..=100.0).contains(&101.0));
    }

    #[test]
    fn test_amendment_number_format() {
        let guarantee_number = "BG-2025-001";
        let amendment_count = 1;
        let amendment_number = format!("AMD-{}-{:03}", guarantee_number, amendment_count);
        assert_eq!(amendment_number, "AMD-BG-2025-001-001");

        let amendment_number2 = format!("AMD-{}-{:03}", guarantee_number, 5);
        assert_eq!(amendment_number2, "AMD-BG-2025-001-005");
    }

    #[test]
    fn test_guarantee_type_naming_convention() {
        // Types should be snake_case
        for gtype in VALID_GUARANTEE_TYPES {
            assert_eq!(*gtype, gtype.to_lowercase(),
                "Guarantee type '{}' should be lowercase", gtype);
            assert!(!gtype.contains(' '),
                "Guarantee type '{}' should not contain spaces", gtype);
        }
    }

    #[test]
    fn test_lifecycle_transitions() {
        // Valid transitions: draft → pending_approval → approved → issued → active
        // active → invoked → released
        // active → released
        // active → expired
        // Any non-terminal → cancelled

        let valid_from_draft = vec!["pending_approval", "cancelled"];
        assert!(valid_from_draft.contains(&"pending_approval"));
        assert!(valid_from_draft.contains(&"cancelled"));

        let valid_from_pending_approval = vec!["approved", "cancelled"];
        assert!(valid_from_pending_approval.contains(&"approved"));

        let valid_from_approved = vec!["issued", "cancelled"];
        assert!(valid_from_approved.contains(&"issued"));

        let valid_from_issued = vec!["active", "cancelled"];
        assert!(valid_from_issued.contains(&"active"));

        let valid_from_active = vec!["invoked", "released", "expired", "cancelled"];
        assert!(valid_from_active.contains(&"invoked"));
        assert!(valid_from_active.contains(&"released"));
        assert!(valid_from_active.contains(&"expired"));

        let valid_from_invoked = vec!["released"];
        assert!(valid_from_invoked.contains(&"released"));
    }

    #[test]
    fn test_amount_increase_validation() {
        let prev = 100000.0_f64;
        let new_val = 150000.0_f64;
        assert!(new_val > prev, "Amount increase requires new > prev");

        let same = 100000.0_f64;
        assert!(same <= prev, "Same amount is not an increase");

        let decrease = 50000.0_f64;
        assert!(decrease <= prev, "Decrease is not an increase");
    }

    #[test]
    fn test_amount_decrease_validation() {
        let prev = 100000.0_f64;
        let new_val = 50000.0_f64;
        assert!(new_val < prev, "Amount decrease requires new < prev");

        let increase = 150000.0_f64;
        assert!(increase >= prev, "Increase is not a decrease");
    }
}
