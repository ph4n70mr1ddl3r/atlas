//! Withholding Tax Engine Implementation
//!
//! Manages withholding tax codes, tax groups, supplier assignments,
//! automatic withholding computation during payment, thresholds,
//! exemptions, and certificate management.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Withholding Tax

use atlas_shared::{
    WithholdingTaxCode, WithholdingTaxGroup, SupplierWithholdingAssignment,
    WithholdingCertificate, WithholdingTaxLine,
    WithholdingComputationResult, WithholdingComputedLine, WithholdingSummary,
    AtlasError, AtlasResult,
};
use super::WithholdingTaxRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid tax types for withholding
#[allow(dead_code)]
const VALID_TAX_TYPES: &[&str] = &[
    "income_tax", "vat", "service_tax", "contract_tax",
    "royalty", "dividend", "interest", "other",
];

/// Valid withholding line statuses
#[allow(dead_code)]
const VALID_LINE_STATUSES: &[&str] = &[
    "pending", "withheld", "remitted", "refunded",
];

/// Valid certificate statuses
#[allow(dead_code)]
const VALID_CERTIFICATE_STATUSES: &[&str] = &[
    "draft", "issued", "acknowledged", "cancelled",
];

/// Withholding tax engine for managing tax codes, groups, and computations
pub struct WithholdingTaxEngine {
    repository: Arc<dyn WithholdingTaxRepository>,
}

impl WithholdingTaxEngine {
    pub fn new(repository: Arc<dyn WithholdingTaxRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Withholding Tax Code Management
    // ========================================================================

    /// Create a new withholding tax code
    pub async fn create_tax_code(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_type: &str,
        rate_percentage: &str,
        threshold_amount: &str,
        threshold_is_cumulative: bool,
        withholding_account_code: Option<&str>,
        expense_account_code: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingTaxCode> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Withholding tax code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Withholding tax code name is required".to_string(),
            ));
        }
        if !VALID_TAX_TYPES.contains(&tax_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid tax_type '{}'. Must be one of: {}", tax_type, VALID_TAX_TYPES.join(", ")
            )));
        }

        let rate: f64 = rate_percentage.parse().map_err(|_| AtlasError::ValidationFailed(
            "rate_percentage must be a valid number".to_string(),
        ))?;
        if rate < 0.0 || rate > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "rate_percentage must be between 0 and 100".to_string(),
            ));
        }

        let threshold: f64 = threshold_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "threshold_amount must be a valid number".to_string(),
        ))?;
        if threshold < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "threshold_amount must be non-negative".to_string(),
            ));
        }

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        info!("Creating withholding tax code '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_tax_code(
            org_id, &code_upper, name, description, tax_type,
            rate_percentage, threshold_amount, threshold_is_cumulative,
            withholding_account_code, expense_account_code,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a withholding tax code by code
    pub async fn get_tax_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WithholdingTaxCode>> {
        self.repository.get_tax_code(org_id, &code.to_uppercase()).await
    }

    /// Get a withholding tax code by ID
    pub async fn get_tax_code_by_id(&self, id: Uuid) -> AtlasResult<Option<WithholdingTaxCode>> {
        self.repository.get_tax_code_by_id(id).await
    }

    /// List withholding tax codes, optionally filtered by tax type
    pub async fn list_tax_codes(&self, org_id: Uuid, tax_type: Option<&str>) -> AtlasResult<Vec<WithholdingTaxCode>> {
        self.repository.list_tax_codes(org_id, tax_type).await
    }

    /// Deactivate a withholding tax code
    pub async fn delete_tax_code(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating withholding tax code '{}' for org {}", code, org_id);
        self.repository.delete_tax_code(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Withholding Tax Group Management
    // ========================================================================

    /// Create a new withholding tax group
    pub async fn create_tax_group(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_code_ids: &[Uuid],
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingTaxGroup> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Tax group code is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Tax group name is required".to_string(),
            ));
        }
        if tax_code_ids.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Tax group must have at least one tax code".to_string(),
            ));
        }

        // Validate all tax codes exist
        for tc_id in tax_code_ids {
            let tc = self.repository.get_tax_code_by_id(*tc_id).await?;
            if tc.is_none() {
                return Err(AtlasError::EntityNotFound(
                    format!("Withholding tax code {} not found", tc_id)
                ));
            }
        }

        info!("Creating withholding tax group '{}' ({}) for org {}", code_upper, name, org_id);

        let group = self.repository.create_tax_group(
            org_id, &code_upper, name, description, created_by,
        ).await?;

        // Add members
        for (i, tc_id) in tax_code_ids.iter().enumerate() {
            self.repository.add_group_member(
                group.id, *tc_id, None, (i + 1) as i32,
            ).await?;
        }

        // Reload with members
        self.repository.get_tax_group_by_id(group.id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                "Tax group not found after creation".to_string()
            ))
    }

    /// Get a withholding tax group by code
    pub async fn get_tax_group(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WithholdingTaxGroup>> {
        self.repository.get_tax_group(org_id, &code.to_uppercase()).await
    }

    /// List all tax groups
    pub async fn list_tax_groups(&self, org_id: Uuid) -> AtlasResult<Vec<WithholdingTaxGroup>> {
        self.repository.list_tax_groups(org_id).await
    }

    /// Deactivate a tax group
    pub async fn delete_tax_group(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating tax group '{}' for org {}", code, org_id);
        self.repository.delete_tax_group(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Supplier Withholding Tax Assignment
    // ========================================================================

    /// Assign a withholding tax group to a supplier
    pub async fn assign_supplier(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        tax_group_code: &str,
        is_exempt: bool,
        exemption_reason: Option<&str>,
        exemption_certificate: Option<&str>,
        exemption_valid_until: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierWithholdingAssignment> {
        // Validate tax group exists
        let group = self.get_tax_group(org_id, tax_group_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Withholding tax group '{}' not found", tax_group_code)
            ))?;

        // Validate exemption
        if is_exempt {
            if exemption_reason.is_none() || exemption_reason.unwrap().is_empty() {
                return Err(AtlasError::ValidationFailed(
                    "Exemption reason is required when supplier is exempt".to_string(),
                ));
            }
            if let Some(valid_until) = exemption_valid_until {
                if valid_until < chrono::Utc::now().date_naive() {
                    return Err(AtlasError::ValidationFailed(
                        "Exemption valid_until date must be in the future".to_string(),
                    ));
                }
            }
        }

        info!("Assigning withholding tax group '{}' to supplier {} for org {}", tax_group_code, supplier_id, org_id);

        self.repository.create_supplier_assignment(
            org_id, supplier_id, supplier_number, supplier_name,
            group.id, is_exempt, exemption_reason,
            exemption_certificate, exemption_valid_until, created_by,
        ).await
    }

    /// Get the withholding tax assignment for a supplier
    pub async fn get_supplier_assignment(&self, org_id: Uuid, supplier_id: Uuid) -> AtlasResult<Option<SupplierWithholdingAssignment>> {
        self.repository.get_supplier_assignment(org_id, supplier_id).await
    }

    /// List all supplier assignments
    pub async fn list_supplier_assignments(&self, org_id: Uuid) -> AtlasResult<Vec<SupplierWithholdingAssignment>> {
        self.repository.list_supplier_assignments(org_id).await
    }

    /// Remove a supplier assignment
    pub async fn remove_supplier_assignment(&self, id: Uuid) -> AtlasResult<()> {
        info!("Removing supplier withholding assignment {}", id);
        self.repository.delete_supplier_assignment(id).await
    }

    // ========================================================================
    // Withholding Tax Computation
    // ========================================================================

    /// Compute withholding tax for a payment to a supplier
    ///
    /// This is the core computation function that:
    /// 1. Looks up the supplier's withholding tax assignment
    /// 2. If exempt, returns empty result
    /// 3. Loads the tax group and its member tax codes
    /// 4. For each tax code, checks threshold and computes withholding
    /// 5. Returns the total withholding amount and individual lines
    ///
    /// Oracle Fusion equivalent: Automatic withholding during payment processing
    pub async fn compute_withholding(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        invoice_amount: f64,
        _invoice_id: Uuid,
    ) -> AtlasResult<WithholdingComputationResult> {
        // Look up supplier assignment
        let assignment = self.repository.get_supplier_assignment(org_id, supplier_id).await?;

        let (is_exempt, group) = match assignment {
            Some(a) => {
                if a.is_exempt {
                    // Check if exemption has expired
                    if let Some(valid_until) = a.exemption_valid_until {
                        if valid_until < chrono::Utc::now().date_naive() {
                            info!("Supplier {} exemption expired on {}, proceeding with withholding", supplier_id, valid_until);
                            // Exemption expired, load the group
                            let group = self.repository.get_tax_group_by_id(a.tax_group_id).await?
                                .ok_or_else(|| AtlasError::EntityNotFound(
                                    "Tax group not found".to_string()
                                ))?;
                            (false, group)
                        } else {
                            (true, WithholdingTaxGroup {
                                id: a.tax_group_id,
                                organization_id: org_id,
                                code: a.tax_group_code.clone(),
                                name: a.tax_group_name.clone(),
                                description: None,
                                tax_codes: vec![],
                                is_active: true,
                                created_by: None,
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                            })
                        }
                    } else {
                        (true, WithholdingTaxGroup {
                            id: a.tax_group_id,
                            organization_id: org_id,
                            code: a.tax_group_code.clone(),
                            name: a.tax_group_name.clone(),
                            description: None,
                            tax_codes: vec![],
                            is_active: true,
                            created_by: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        })
                    }
                } else {
                    let group = self.repository.get_tax_group_by_id(a.tax_group_id).await?
                        .ok_or_else(|| AtlasError::EntityNotFound(
                            "Tax group not found".to_string()
                        ))?;
                    (false, group)
                }
            }
            None => {
                // No assignment - no withholding
                return Ok(WithholdingComputationResult {
                    tax_group_code: None,
                    is_exempt: false,
                    lines: vec![],
                    total_taxable_amount: "0.00".to_string(),
                    total_withheld_amount: "0.00".to_string(),
                    net_payment_amount: format!("{:.2}", invoice_amount),
                });
            }
        };

        if is_exempt {
            return Ok(WithholdingComputationResult {
                tax_group_code: Some(group.code),
                is_exempt: true,
                lines: vec![],
                total_taxable_amount: "0.00".to_string(),
                total_withheld_amount: "0.00".to_string(),
                net_payment_amount: format!("{:.2}", invoice_amount),
            });
        }

        // Compute withholding for each tax code in the group
        let mut computed_lines: Vec<WithholdingComputedLine> = Vec::new();
        let mut total_taxable = 0.0_f64;
        let mut total_withheld = 0.0_f64;

        for member in &group.tax_codes {
            if !member.is_active {
                continue;
            }

            // Load the tax code details
            let tax_code = self.repository.get_tax_code_by_id(member.tax_code_id).await?;
            let tax_code = match tax_code {
                Some(tc) if tc.is_active => tc,
                _ => continue,
            };

            // Check effective dates
            let today = chrono::Utc::now().date_naive();
            if let Some(from) = tax_code.effective_from {
                if today < from { continue; }
            }
            if let Some(to) = tax_code.effective_to {
                if today > to { continue; }
            }

            // Get the rate (use override if provided, otherwise use tax code default)
            let rate: f64 = member.rate_override
                .as_ref()
                .and_then(|r| r.parse().ok())
                .unwrap_or_else(|| tax_code.rate_percentage.parse().unwrap_or(0.0));

            // Check threshold
            let threshold: f64 = tax_code.threshold_amount.parse().unwrap_or(0.0);
            let threshold_applied = invoice_amount < threshold;

            let (taxable_amount, withheld_amount) = if threshold_applied {
                (invoice_amount, 0.0) // Below threshold, no withholding
            } else {
                let withheld = invoice_amount * rate / 100.0;
                (invoice_amount, withheld)
            };

            total_taxable += taxable_amount;
            total_withheld += withheld_amount;

            computed_lines.push(WithholdingComputedLine {
                tax_code_id: tax_code.id,
                tax_code: tax_code.code,
                tax_type: tax_code.tax_type,
                rate_percentage: format!("{:.2}", rate),
                threshold_amount: tax_code.threshold_amount.clone(),
                taxable_amount: format!("{:.2}", taxable_amount),
                withheld_amount: format!("{:.2}", withheld_amount),
                withholding_account_code: tax_code.withholding_account_code.clone(),
                threshold_applied,
            });
        }

        let net_payment = invoice_amount - total_withheld;

        Ok(WithholdingComputationResult {
            tax_group_code: Some(group.code),
            is_exempt: false,
            lines: computed_lines,
            total_taxable_amount: format!("{:.2}", total_taxable),
            total_withheld_amount: format!("{:.2}", total_withheld),
            net_payment_amount: format!("{:.2}", net_payment),
        })
    }

    /// Record withholding tax lines for a payment (persist after computation)
    pub async fn record_withholding(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        payment_number: Option<&str>,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        computation: &WithholdingComputationResult,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Vec<WithholdingTaxLine>> {
        let mut lines = Vec::new();

        for computed in &computation.lines {
            let tax_code = self.repository.get_tax_code_by_id(computed.tax_code_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Tax code {} not found", computed.tax_code_id)
                ))?;

            info!("Recording withholding tax line: {} {} {} for payment {}",
                computed.tax_code, computed.rate_percentage, computed.withheld_amount, payment_id);

            let line = self.repository.create_withholding_line(
                org_id, payment_id, payment_number,
                invoice_id, invoice_number,
                supplier_id, supplier_name,
                computed.tax_code_id, &computed.tax_code,
                Some(&tax_code.name), &computed.tax_type,
                &computed.rate_percentage,
                &computed.taxable_amount, &computed.withheld_amount,
                computed.withholding_account_code.as_deref(),
                created_by,
            ).await?;

            lines.push(line);
        }

        Ok(lines)
    }

    // ========================================================================
    // Withholding Tax Line Management
    // ========================================================================

    /// Get withholding lines for a payment
    pub async fn get_withholding_lines_by_payment(&self, payment_id: Uuid) -> AtlasResult<Vec<WithholdingTaxLine>> {
        self.repository.get_withholding_lines_by_payment(payment_id).await
    }

    /// Get withholding lines for a supplier in a date range
    pub async fn get_withholding_lines_by_supplier(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        from_date: Option<chrono::NaiveDate>,
        to_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<WithholdingTaxLine>> {
        self.repository.get_withholding_lines_by_supplier(org_id, supplier_id, from_date, to_date).await
    }

    /// Mark withholding lines as remitted (paid to tax authority)
    pub async fn remit_withholding(
        &self,
        line_ids: &[Uuid],
        remittance_date: chrono::NaiveDate,
        remittance_reference: Option<&str>,
    ) -> AtlasResult<Vec<WithholdingTaxLine>> {
        let mut lines = Vec::new();
        for id in line_ids {
            info!("Remitting withholding tax line {}", id);
            let line = self.repository.update_withholding_line_status(
                *id, "remitted", Some(remittance_date), remittance_reference,
            ).await?;
            lines.push(line);
        }
        Ok(lines)
    }

    // ========================================================================
    // Certificate Management
    // ========================================================================

    /// Generate a withholding tax certificate for a supplier and period
    pub async fn generate_certificate(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        tax_code_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingCertificate> {
        if period_start > period_end {
            return Err(AtlasError::ValidationFailed(
                "period_start must be before period_end".to_string(),
            ));
        }

        // Get the tax code
        let tax_code = self.repository.get_tax_code_by_id(tax_code_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Withholding tax code {} not found", tax_code_id)
            ))?;

        // Get withholding lines for this supplier, tax code, and period
        let all_lines = self.repository.get_withholding_lines_by_supplier(
            org_id, supplier_id, Some(period_start), Some(period_end),
        ).await?;

        // Filter to lines matching the tax code
        let matching_lines: Vec<_> = all_lines.iter()
            .filter(|l| l.tax_code_id == tax_code_id)
            .collect();

        let total_invoice: f64 = matching_lines.iter()
            .map(|l| l.taxable_amount.parse().unwrap_or(0.0))
            .sum();
        let total_withheld: f64 = matching_lines.iter()
            .map(|l| l.withheld_amount.parse().unwrap_or(0.0))
            .sum();

        let payment_ids: Vec<Uuid> = matching_lines.iter()
            .map(|l| l.payment_id)
            .collect();

        // Generate certificate number
        let cert_number = format!("WHT-CERT-{}-{}-{}",
            supplier_id,
            period_start.format("%Y%m"),
            chrono::Utc::now().timestamp() % 10000,
        );

        info!("Generating withholding certificate {} for supplier {} period {} to {}",
            cert_number, supplier_id, period_start, period_end);

        self.repository.create_certificate(
            org_id, &cert_number,
            supplier_id, supplier_number, supplier_name,
            &tax_code.tax_type, tax_code_id, &tax_code.code,
            period_start, period_end,
            &format!("{:.2}", total_invoice),
            &format!("{:.2}", total_withheld),
            &tax_code.rate_percentage,
            serde_json::json!(payment_ids),
            created_by,
        ).await
    }

    /// Get a certificate by ID
    pub async fn get_certificate(&self, id: Uuid) -> AtlasResult<Option<WithholdingCertificate>> {
        self.repository.get_certificate(id).await
    }

    /// Get a certificate by number
    pub async fn get_certificate_by_number(&self, org_id: Uuid, certificate_number: &str) -> AtlasResult<Option<WithholdingCertificate>> {
        self.repository.get_certificate_by_number(org_id, certificate_number).await
    }

    /// List certificates, optionally filtered by supplier
    pub async fn list_certificates(&self, org_id: Uuid, supplier_id: Option<Uuid>) -> AtlasResult<Vec<WithholdingCertificate>> {
        self.repository.list_certificates(org_id, supplier_id).await
    }

    /// Issue a certificate (change status from draft to issued)
    pub async fn issue_certificate(&self, id: Uuid) -> AtlasResult<WithholdingCertificate> {
        let cert = self.repository.get_certificate(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Certificate {} not found", id)
            ))?;

        if cert.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot issue certificate in '{}' status. Must be 'draft'.", cert.status)
            ));
        }

        info!("Issuing withholding certificate {}", id);
        self.repository.update_certificate_status(id, "issued").await
    }

    /// Cancel a certificate
    pub async fn cancel_certificate(&self, id: Uuid) -> AtlasResult<WithholdingCertificate> {
        let cert = self.repository.get_certificate(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Certificate {} not found", id)
            ))?;

        if cert.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                "Certificate is already cancelled".to_string(),
            ));
        }

        info!("Cancelling withholding certificate {}", id);
        self.repository.update_certificate_status(id, "cancelled").await
    }

    // ========================================================================
    // Dashboard Summary
    // ========================================================================

    /// Get a withholding tax dashboard summary
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<WithholdingSummary> {
        let tax_codes = self.repository.list_tax_codes(org_id, None).await?;
        let groups = self.repository.list_tax_groups(org_id).await?;
        let assignments = self.repository.list_supplier_assignments(org_id).await?;
        let certificates = self.repository.list_certificates(org_id, None).await?;

        let exempt_count = assignments.iter().filter(|a| a.is_exempt).count();

        Ok(WithholdingSummary {
            active_tax_code_count: tax_codes.len() as i32,
            tax_group_count: groups.len() as i32,
            assigned_supplier_count: assignments.len() as i32,
            exempt_supplier_count: exempt_count as i32,
            total_withheld_amount: "0.00".to_string(),
            total_remitted_amount: "0.00".to_string(),
            total_pending_remittance: "0.00".to_string(),
            by_tax_type: serde_json::json!({}),
            by_supplier: serde_json::json!({}),
            certificates_issued: certificates.iter().filter(|c| c.status == "issued").count() as i32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tax_types() {
        assert!(VALID_TAX_TYPES.contains(&"income_tax"));
        assert!(VALID_TAX_TYPES.contains(&"vat"));
        assert!(VALID_TAX_TYPES.contains(&"service_tax"));
        assert!(VALID_TAX_TYPES.contains(&"contract_tax"));
        assert!(VALID_TAX_TYPES.contains(&"royalty"));
        assert!(VALID_TAX_TYPES.contains(&"dividend"));
        assert!(VALID_TAX_TYPES.contains(&"interest"));
        assert!(VALID_TAX_TYPES.contains(&"other"));
    }

    #[test]
    fn test_valid_line_statuses() {
        assert!(VALID_LINE_STATUSES.contains(&"pending"));
        assert!(VALID_LINE_STATUSES.contains(&"withheld"));
        assert!(VALID_LINE_STATUSES.contains(&"remitted"));
        assert!(VALID_LINE_STATUSES.contains(&"refunded"));
    }

    #[test]
    fn test_valid_certificate_statuses() {
        assert!(VALID_CERTIFICATE_STATUSES.contains(&"draft"));
        assert!(VALID_CERTIFICATE_STATUSES.contains(&"issued"));
        assert!(VALID_CERTIFICATE_STATUSES.contains(&"acknowledged"));
        assert!(VALID_CERTIFICATE_STATUSES.contains(&"cancelled"));
    }
}
