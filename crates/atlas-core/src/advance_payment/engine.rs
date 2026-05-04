//! Advance Payment Engine
//!
//! Manages supplier advance payments (prepayments) with full lifecycle:
//! - Create draft advance
//! - Approve and pay advance
//! - Apply advance to supplier invoices
//! - Unapply or reverse applications
//! - Dashboard and reporting
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Accounts Payable > Advance Payments

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_STATUSES: &[&str] = &[
    "draft", "approved", "paid", "partially_applied", "fully_applied", "cancelled",
];

#[allow(dead_code)]
const VALID_APPLICATION_STATUSES: &[&str] = &[
    "applied", "unapplied", "reversed",
];

#[allow(dead_code)]
const VALID_PAYMENT_METHODS: &[&str] = &[
    "check", "electronic", "wire", "ach", "manual",
];

pub struct AdvancePaymentEngine {
    repository: Arc<dyn AdvancePaymentRepository>,
}

impl AdvancePaymentEngine {
    pub fn new(repository: Arc<dyn AdvancePaymentRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Advance Payment CRUD
    // ========================================================================

    /// Create a new advance payment (in draft status)
    pub async fn create_advance(
        &self,
        org_id: Uuid,
        advance_number: &str,
        supplier_id: Uuid,
        supplier_name: &str,
        supplier_site_id: Option<Uuid>,
        description: Option<&str>,
        currency_code: &str,
        advance_amount: &str,
        exchange_rate: Option<&str>,
        payment_method: Option<&str>,
        prepayment_account_code: Option<&str>,
        liability_account_code: Option<&str>,
        advance_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        expiration_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AdvancePayment> {
        if advance_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Advance number is required".into()));
        }
        if supplier_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Supplier name is required".into()));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".into()));
        }
        let amount: f64 = advance_amount.parse().unwrap_or(-1.0);
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed("Advance amount must be positive".into()));
        }
        if let Some(pm) = payment_method {
            if !VALID_PAYMENT_METHODS.contains(&pm) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid payment_method '{}'. Must be one of: {}", pm, VALID_PAYMENT_METHODS.join(", ")
                )));
            }
        }
        if let Some(exp) = expiration_date {
            if exp < advance_date {
                return Err(AtlasError::ValidationFailed("Expiration date must be after advance date".into()));
            }
        }
        if let Some(due) = due_date {
            if due < advance_date {
                return Err(AtlasError::ValidationFailed("Due date must be on or after advance date".into()));
            }
        }
        if self.repository.get_advance_by_number(org_id, advance_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Advance number '{}' already exists", advance_number)));
        }

        info!("Creating advance payment {} for supplier {} ({})", advance_number, supplier_name, supplier_id);

        self.repository.create_advance(
            org_id, advance_number, supplier_id, supplier_name, supplier_site_id,
            description, currency_code, advance_amount, exchange_rate, payment_method,
            prepayment_account_code, liability_account_code, advance_date, due_date,
            expiration_date, created_by,
        ).await
    }

    /// Get advance by ID
    pub async fn get_advance(&self, id: Uuid) -> AtlasResult<Option<AdvancePayment>> {
        self.repository.get_advance(id).await
    }

    /// Get advance by number
    pub async fn get_advance_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AdvancePayment>> {
        self.repository.get_advance_by_number(org_id, number).await
    }

    /// List advances with optional filters
    pub async fn list_advances(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        supplier_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AdvancePayment>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_advances(org_id, status, supplier_id).await
    }

    // ========================================================================
    // Workflow: Approve, Pay, Cancel
    // ========================================================================

    /// Approve a draft advance payment
    pub async fn approve_advance(&self, advance_id: Uuid, _approved_by: Uuid) -> AtlasResult<AdvancePayment> {
        let adv = self.repository.get_advance(advance_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Advance {} not found", advance_id)))?;

        if adv.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve advance in '{}' status. Must be 'draft'.", adv.status)
            ));
        }

        info!("Approving advance payment {}", adv.advance_number);
        self.repository.update_advance_status(advance_id, "approved").await
    }

    /// Record payment for an approved advance
    pub async fn pay_advance(&self, advance_id: Uuid, payment_reference: Option<&str>, paid_by: Option<Uuid>) -> AtlasResult<AdvancePayment> {
        let adv = self.repository.get_advance(advance_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Advance {} not found", advance_id)))?;

        if adv.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot pay advance in '{}' status. Must be 'approved'.", adv.status)
            ));
        }

        info!("Recording payment for advance {}", adv.advance_number);
        let _updated = self.repository.update_payment_info(advance_id, payment_reference, paid_by).await?;
        self.repository.update_advance_status(advance_id, "paid").await
    }

    /// Cancel a draft or approved advance
    pub async fn cancel_advance(&self, advance_id: Uuid, _reason: Option<&str>) -> AtlasResult<AdvancePayment> {
        let adv = self.repository.get_advance(advance_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Advance {} not found", advance_id)))?;

        if adv.status != "draft" && adv.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel advance in '{}' status. Only 'draft' or 'approved' allowed.", adv.status)
            ));
        }

        info!("Cancelling advance payment {}", adv.advance_number);
        self.repository.update_advance_status(advance_id, "cancelled").await
    }

    // ========================================================================
    // Application to Invoices
    // ========================================================================

    /// Apply an advance payment to a supplier invoice
    pub async fn apply_to_invoice(
        &self,
        org_id: Uuid,
        advance_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        applied_amount: &str,
        application_date: chrono::NaiveDate,
        gl_account_code: Option<&str>,
        applied_by: Option<Uuid>,
    ) -> AtlasResult<AdvanceApplication> {
        let adv = self.repository.get_advance(advance_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Advance {} not found", advance_id)))?;

        if adv.status != "paid" && adv.status != "partially_applied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot apply advance in '{}' status. Must be 'paid' or 'partially_applied'.", adv.status)
            ));
        }

        let amount: f64 = applied_amount.parse().unwrap_or(-1.0);
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed("Applied amount must be positive".into()));
        }

        let unapplied: f64 = adv.unapplied_amount.parse().unwrap_or(0.0);
        if amount > unapplied + 0.01 {
            return Err(AtlasError::ValidationFailed(
                format!("Applied amount {} exceeds unapplied amount {}", amount, unapplied)
            ));
        }

        info!("Applying advance {} to invoice (amount: {})", adv.advance_number, applied_amount);

        let application = self.repository.create_application(
            org_id, advance_id, Some(&adv.advance_number),
            invoice_id, invoice_number, applied_amount, application_date,
            gl_account_code, applied_by,
        ).await?;

        // Recalculate advance amounts
        let applications = self.repository.list_applications_by_advance(advance_id).await?;
        let active_apps: Vec<&AdvanceApplication> = applications.iter()
            .filter(|a| a.status == "applied")
            .collect();
        let total_applied: f64 = active_apps.iter()
            .map(|a| a.applied_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let advance_amt: f64 = adv.advance_amount.parse().unwrap_or(0.0);
        let new_unapplied = advance_amt - total_applied;
        let new_status = if (new_unapplied).abs() < 0.01 { "fully_applied" } else { "partially_applied" };

        self.repository.update_advance_amounts(
            advance_id,
            &format!("{:.2}", total_applied),
            &format!("{:.2}", new_unapplied),
            new_status,
        ).await?;

        Ok(application)
    }

    /// Unapply (reverse) an advance application
    pub async fn unapply_application(&self, application_id: Uuid, reversed_by: Option<Uuid>, reason: Option<&str>) -> AtlasResult<AdvanceApplication> {
        let app = self.repository.get_application(application_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", application_id)))?;

        if app.status != "applied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot unapply application in '{}' status. Must be 'applied'.", app.status)
            ));
        }

        info!("Unapplying advance application for advance {}", app.advance_number.as_deref().unwrap_or("?"));

        let updated = self.repository.update_application_status(application_id, "reversed", reversed_by, reason).await?;

        // Recalculate advance amounts
        let applications = self.repository.list_applications_by_advance(app.advance_id).await?;
        let total_applied: f64 = applications.iter()
            .filter(|a| a.status == "applied")
            .map(|a| a.applied_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        // Get the advance to know the total
        if let Some(adv) = self.repository.get_advance(app.advance_id).await? {
            let advance_amt: f64 = adv.advance_amount.parse().unwrap_or(0.0);
            let new_unapplied = advance_amt - total_applied;
            let new_status = if total_applied.abs() < 0.01 { "paid" } else { "partially_applied" };
            self.repository.update_advance_amounts(
                app.advance_id,
                &format!("{:.2}", total_applied),
                &format!("{:.2}", new_unapplied),
                new_status,
            ).await?;
        }

        Ok(updated)
    }

    /// List applications for an advance
    pub async fn list_applications_by_advance(&self, advance_id: Uuid) -> AtlasResult<Vec<AdvanceApplication>> {
        self.repository.list_applications_by_advance(advance_id).await
    }

    /// List applications for an invoice
    pub async fn list_applications_by_invoice(&self, invoice_id: Uuid) -> AtlasResult<Vec<AdvanceApplication>> {
        self.repository.list_applications_by_invoice(invoice_id).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<AdvancePaymentDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        advances: std::sync::Mutex<Vec<AdvancePayment>>,
        applications: std::sync::Mutex<Vec<AdvanceApplication>>,
    }

    impl MockRepo {
        fn new() -> Self {
            MockRepo {
                advances: std::sync::Mutex::new(vec![]),
                applications: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait::async_trait]
    impl AdvancePaymentRepository for MockRepo {
        async fn create_advance(&self, org_id: Uuid, an: &str, sid: Uuid, sn: &str, ssi: Option<Uuid>, desc: Option<&str>, cc: &str, amt: &str, er: Option<&str>, pm: Option<&str>, pac: Option<&str>, lac: Option<&str>, ad: chrono::NaiveDate, dd: Option<chrono::NaiveDate>, ed: Option<chrono::NaiveDate>, cb: Option<Uuid>) -> AtlasResult<AdvancePayment> {
            let a = AdvancePayment {
                id: Uuid::new_v4(), organization_id: org_id, advance_number: an.into(),
                supplier_id: sid, supplier_name: sn.into(), supplier_site_id: ssi,
                description: desc.map(Into::into), status: "draft".into(),
                currency_code: cc.into(), advance_amount: amt.into(),
                applied_amount: "0".into(), unapplied_amount: amt.into(),
                exchange_rate: er.map(Into::into), payment_method: pm.map(Into::into),
                payment_reference: None, prepayment_account_code: pac.map(Into::into),
                liability_account_code: lac.map(Into::into), advance_date: ad,
                payment_date: None, due_date: dd, expiration_date: ed,
                approved_by: None, approved_at: None, paid_by: None, paid_at: None,
                cancelled_reason: None, metadata: serde_json::json!({}),
                created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.advances.lock().unwrap().push(a.clone());
            Ok(a)
        }
        async fn get_advance(&self, id: Uuid) -> AtlasResult<Option<AdvancePayment>> {
            Ok(self.advances.lock().unwrap().iter().find(|a| a.id == id).cloned())
        }
        async fn get_advance_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<AdvancePayment>> {
            Ok(self.advances.lock().unwrap().iter().find(|a| a.organization_id == org_id && a.advance_number == num).cloned())
        }
        async fn list_advances(&self, org_id: Uuid, status: Option<&str>, _supplier_id: Option<Uuid>) -> AtlasResult<Vec<AdvancePayment>> {
            Ok(self.advances.lock().unwrap().iter()
                .filter(|a| a.organization_id == org_id)
                .filter(|a| status.map_or(true, |s| a.status == s))
                .cloned().collect())
        }
        async fn update_advance_status(&self, id: Uuid, status: &str) -> AtlasResult<AdvancePayment> {
            let mut advs = self.advances.lock().unwrap();
            let adv = advs.iter_mut().find(|a| a.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Advance {} not found", id)))?;
            adv.status = status.into();
            adv.updated_at = chrono::Utc::now();
            Ok(adv.clone())
        }
        async fn update_advance_amounts(&self, id: Uuid, applied: &str, unapplied: &str, status: &str) -> AtlasResult<()> {
            let mut advs = self.advances.lock().unwrap();
            if let Some(adv) = advs.iter_mut().find(|a| a.id == id) {
                adv.applied_amount = applied.into();
                adv.unapplied_amount = unapplied.into();
                adv.status = status.into();
                adv.updated_at = chrono::Utc::now();
            }
            Ok(())
        }
        async fn update_payment_info(&self, id: Uuid, pref: Option<&str>, pb: Option<Uuid>) -> AtlasResult<AdvancePayment> {
            let mut advs = self.advances.lock().unwrap();
            let adv = advs.iter_mut().find(|a| a.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Advance {} not found", id)))?;
            adv.payment_reference = pref.map(Into::into);
            adv.paid_by = pb;
            adv.updated_at = chrono::Utc::now();
            Ok(adv.clone())
        }
        async fn create_application(&self, org_id: Uuid, aid: Uuid, an: Option<&str>, iid: Uuid, in_: Option<&str>, amt: &str, ad: chrono::NaiveDate, glac: Option<&str>, ab: Option<Uuid>) -> AtlasResult<AdvanceApplication> {
            let app = AdvanceApplication {
                id: Uuid::new_v4(), organization_id: org_id, advance_id: aid,
                advance_number: an.map(Into::into), invoice_id: iid,
                invoice_number: in_.map(Into::into), applied_amount: amt.into(),
                application_date: ad, status: "applied".into(),
                gl_account_code: glac.map(Into::into), reversed_at: None,
                reversed_by: None, reversal_reason: None, metadata: serde_json::json!({}),
                applied_by: ab, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.applications.lock().unwrap().push(app.clone());
            Ok(app)
        }
        async fn get_application(&self, id: Uuid) -> AtlasResult<Option<AdvanceApplication>> {
            Ok(self.applications.lock().unwrap().iter().find(|a| a.id == id).cloned())
        }
        async fn list_applications_by_advance(&self, advance_id: Uuid) -> AtlasResult<Vec<AdvanceApplication>> {
            Ok(self.applications.lock().unwrap().iter().filter(|a| a.advance_id == advance_id).cloned().collect())
        }
        async fn list_applications_by_invoice(&self, invoice_id: Uuid) -> AtlasResult<Vec<AdvanceApplication>> {
            Ok(self.applications.lock().unwrap().iter().filter(|a| a.invoice_id == invoice_id).cloned().collect())
        }
        async fn update_application_status(&self, id: Uuid, status: &str, rb: Option<Uuid>, rr: Option<&str>) -> AtlasResult<AdvanceApplication> {
            let mut apps = self.applications.lock().unwrap();
            let app = apps.iter_mut().find(|a| a.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", id)))?;
            app.status = status.into();
            app.reversed_by = rb;
            app.reversal_reason = rr.map(Into::into);
            app.reversed_at = Some(chrono::Utc::now());
            app.updated_at = chrono::Utc::now();
            Ok(app.clone())
        }
        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<AdvancePaymentDashboard> {
            Ok(AdvancePaymentDashboard {
                total_advances: 0, draft_advances: 0, open_advances: 0,
                total_advance_amount: "0".into(), total_applied_amount: "0".into(),
                total_unapplied_amount: "0".into(), advances_by_supplier: serde_json::json!([]),
                aging_buckets: serde_json::json!([]),
            })
        }
    }

    fn eng() -> AdvancePaymentEngine {
        AdvancePaymentEngine::new(Arc::new(MockRepo::new()))
    }

    // ========================================================================
    // Tests
    // ========================================================================

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_STATUSES.len(), 6);
        assert_eq!(VALID_APPLICATION_STATUSES.len(), 3);
        assert_eq!(VALID_PAYMENT_METHODS.len(), 5);
    }

    #[tokio::test]
    async fn test_create_advance_valid() {
        let a = eng().create_advance(
            Uuid::new_v4(), "ADV-001", Uuid::new_v4(), "Acme Corp", None,
            Some("Q2 prepayment"), "USD", "50000.00", None, Some("electronic"),
            None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()),
            Some(chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()), None,
        ).await.unwrap();
        assert_eq!(a.advance_number, "ADV-001");
        assert_eq!(a.status, "draft");
        assert_eq!(a.advance_amount, "50000.00");
        assert_eq!(a.unapplied_amount, "50000.00");
        assert_eq!(a.applied_amount, "0");
    }

    #[tokio::test]
    async fn test_create_advance_empty_number() {
        let r = eng().create_advance(
            Uuid::new_v4(), "", Uuid::new_v4(), "Supplier", None, None,
            "USD", "1000.00", None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_advance_empty_supplier() {
        let r = eng().create_advance(
            Uuid::new_v4(), "ADV-1", Uuid::new_v4(), "", None, None,
            "USD", "1000.00", None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_advance_empty_currency() {
        let r = eng().create_advance(
            Uuid::new_v4(), "ADV-1", Uuid::new_v4(), "Supplier", None, None,
            "", "1000.00", None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_advance_zero_amount() {
        let r = eng().create_advance(
            Uuid::new_v4(), "ADV-1", Uuid::new_v4(), "Supplier", None, None,
            "USD", "0.00", None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_advance_negative_amount() {
        let r = eng().create_advance(
            Uuid::new_v4(), "ADV-1", Uuid::new_v4(), "Supplier", None, None,
            "USD", "-500.00", None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_advance_invalid_payment_method() {
        let r = eng().create_advance(
            Uuid::new_v4(), "ADV-1", Uuid::new_v4(), "Supplier", None, None,
            "USD", "1000.00", None, Some("crypto"), None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_advance_expiry_before_advance() {
        let r = eng().create_advance(
            Uuid::new_v4(), "ADV-1", Uuid::new_v4(), "Supplier", None, None,
            "USD", "1000.00", None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(), None,
            Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()), None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_advance_due_before_advance() {
        let r = eng().create_advance(
            Uuid::new_v4(), "ADV-1", Uuid::new_v4(), "Supplier", None, None,
            "USD", "1000.00", None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()), None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_advance_duplicate_number() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_advance(org, "ADV-DUP", Uuid::new_v4(), "S1", None, None, "USD", "1000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await;
        let r = e.create_advance(org, "ADV-DUP", Uuid::new_v4(), "S2", None, None, "USD", "2000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_approve_advance_valid() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-AP1", Uuid::new_v4(), "S", None, None, "USD", "10000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        let approved = e.approve_advance(a.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(approved.status, "approved");
    }

    #[tokio::test]
    async fn test_approve_non_draft_fails() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-AP2", Uuid::new_v4(), "S", None, None, "USD", "10000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        e.approve_advance(a.id, Uuid::new_v4()).await.unwrap();
        let r = e.approve_advance(a.id, Uuid::new_v4()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_pay_advance_valid() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-PAY1", Uuid::new_v4(), "S", None, None, "USD", "10000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        e.approve_advance(a.id, Uuid::new_v4()).await.unwrap();
        let paid = e.pay_advance(a.id, Some("CHK-12345"), None).await.unwrap();
        assert_eq!(paid.status, "paid");
    }

    #[tokio::test]
    async fn test_pay_unapproved_fails() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-PAY2", Uuid::new_v4(), "S", None, None, "USD", "10000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        let r = e.pay_advance(a.id, None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_cancel_draft_valid() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-CAN1", Uuid::new_v4(), "S", None, None, "USD", "10000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        let cancelled = e.cancel_advance(a.id, Some("No longer needed")).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_paid_fails() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-CAN2", Uuid::new_v4(), "S", None, None, "USD", "10000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        e.approve_advance(a.id, Uuid::new_v4()).await.unwrap();
        e.pay_advance(a.id, None, None).await.unwrap();
        let r = e.cancel_advance(a.id, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_list_advances_invalid_status() {
        let r = eng().list_advances(Uuid::new_v4(), Some("bad"), None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_list_advances_valid_status() {
        let r = eng().list_advances(Uuid::new_v4(), Some("draft"), None).await;
        assert!(r.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_apply_to_invoice_valid() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-APPLY1", Uuid::new_v4(), "S", None, None, "USD", "50000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        e.approve_advance(a.id, Uuid::new_v4()).await.unwrap();
        e.pay_advance(a.id, None, None).await.unwrap();
        let app = e.apply_to_invoice(
            a.organization_id, a.id, Uuid::new_v4(), Some("INV-001"), "30000.00",
            chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), None, None,
        ).await.unwrap();
        assert_eq!(app.status, "applied");
        assert_eq!(app.applied_amount, "30000.00");
    }

    #[tokio::test]
    async fn test_apply_to_invoice_not_paid_fails() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-APPLY2", Uuid::new_v4(), "S", None, None, "USD", "50000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        let r = e.apply_to_invoice(a.organization_id, a.id, Uuid::new_v4(), None, "10000.00", chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_apply_zero_amount_fails() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-APPLY3", Uuid::new_v4(), "S", None, None, "USD", "50000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        e.approve_advance(a.id, Uuid::new_v4()).await.unwrap();
        e.pay_advance(a.id, None, None).await.unwrap();
        let r = e.apply_to_invoice(a.organization_id, a.id, Uuid::new_v4(), None, "0.00", chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_apply_exceeds_unapplied_fails() {
        let e = eng();
        let a = e.create_advance(Uuid::new_v4(), "ADV-APPLY4", Uuid::new_v4(), "S", None, None, "USD", "10000.00", None, None, None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None, None, None).await.unwrap();
        e.approve_advance(a.id, Uuid::new_v4()).await.unwrap();
        e.pay_advance(a.id, None, None).await.unwrap();
        let r = e.apply_to_invoice(a.organization_id, a.id, Uuid::new_v4(), None, "99999.00", chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_unapply_not_found() {
        let r = eng().unapply_application(Uuid::new_v4(), None, None).await;
        assert!(matches!(r, Err(AtlasError::EntityNotFound(_))));
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let d = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(d.total_advances, 0);
    }
}
