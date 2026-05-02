//! Customer Deposit Engine
//!
//! Manages customer advance deposits with full lifecycle:
//! - Create draft deposit
//! - Record receipt of deposit
//! - Apply deposit to AR invoices
//! - Unapply or reverse applications
//! - Refund unapplied deposits
//! - Dashboard and reporting
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Accounts Receivable > Customer Deposits

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_STATUSES: &[&str] = &[
    "draft", "received", "partially_applied", "fully_applied", "refunded", "cancelled",
];

#[allow(dead_code)]
const VALID_APPLICATION_STATUSES: &[&str] = &[
    "applied", "unapplied", "reversed",
];

pub struct CustomerDepositEngine {
    repository: Arc<dyn CustomerDepositRepository>,
}

impl CustomerDepositEngine {
    pub fn new(repository: Arc<dyn CustomerDepositRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Deposit CRUD
    // ========================================================================

    /// Create a new customer deposit (in draft status)
    pub async fn create_deposit(
        &self,
        org_id: Uuid,
        deposit_number: &str,
        customer_id: Uuid,
        customer_name: &str,
        customer_site_id: Option<Uuid>,
        description: Option<&str>,
        currency_code: &str,
        deposit_amount: &str,
        exchange_rate: Option<&str>,
        deposit_account_code: Option<&str>,
        receivable_account_code: Option<&str>,
        deposit_date: chrono::NaiveDate,
        expiration_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CustomerDeposit> {
        if deposit_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Deposit number is required".into()));
        }
        if customer_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Customer name is required".into()));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".into()));
        }
        let amount: f64 = deposit_amount.parse().unwrap_or(-1.0);
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed("Deposit amount must be positive".into()));
        }
        if let Some(exp) = expiration_date {
            if exp < deposit_date {
                return Err(AtlasError::ValidationFailed("Expiration date must be after deposit date".into()));
            }
        }
        if self.repository.get_deposit_by_number(org_id, deposit_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Deposit number '{}' already exists", deposit_number)));
        }

        info!("Creating customer deposit {} for customer {} ({})", deposit_number, customer_name, customer_id);

        self.repository.create_deposit(
            org_id, deposit_number, customer_id, customer_name, customer_site_id,
            description, currency_code, deposit_amount, exchange_rate,
            deposit_account_code, receivable_account_code, deposit_date, expiration_date, created_by,
        ).await
    }

    /// Get deposit by ID
    pub async fn get_deposit(&self, id: Uuid) -> AtlasResult<Option<CustomerDeposit>> {
        self.repository.get_deposit(id).await
    }

    /// Get deposit by number
    pub async fn get_deposit_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<CustomerDeposit>> {
        self.repository.get_deposit_by_number(org_id, number).await
    }

    /// List deposits with optional filters
    pub async fn list_deposits(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<Vec<CustomerDeposit>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_deposits(org_id, status, customer_id).await
    }

    // ========================================================================
    // Workflow: Receive, Refund, Cancel
    // ========================================================================

    /// Record receipt of a draft deposit
    pub async fn receive_deposit(&self, deposit_id: Uuid, receipt_reference: Option<&str>, received_by: Option<Uuid>) -> AtlasResult<CustomerDeposit> {
        let dep = self.repository.get_deposit(deposit_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Deposit {} not found", deposit_id)))?;

        if dep.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot receive deposit in '{}' status. Must be 'draft'.", dep.status)
            ));
        }

        info!("Recording receipt for customer deposit {}", dep.deposit_number);
        self.repository.update_receipt_info(deposit_id, receipt_reference, received_by).await?;
        self.repository.update_deposit_status(deposit_id, "received").await
    }

    /// Refund an unapplied deposit
    pub async fn refund_deposit(&self, deposit_id: Uuid, refund_reference: Option<&str>, refunded_by: Option<Uuid>) -> AtlasResult<CustomerDeposit> {
        let dep = self.repository.get_deposit(deposit_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Deposit {} not found", deposit_id)))?;

        if dep.status != "received" && dep.status != "partially_applied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot refund deposit in '{}' status.", dep.status)
            ));
        }

        let unapplied: f64 = dep.unapplied_amount.parse().unwrap_or(0.0);
        if unapplied <= 0.01 {
            return Err(AtlasError::ValidationFailed("No unapplied amount to refund".into()));
        }

        info!("Refunding customer deposit {}", dep.deposit_number);
        self.repository.update_refund_info(deposit_id, refund_reference, refunded_by).await?;
        self.repository.update_deposit_status(deposit_id, "refunded").await
    }

    /// Cancel a draft deposit
    pub async fn cancel_deposit(&self, deposit_id: Uuid, _reason: Option<&str>) -> AtlasResult<CustomerDeposit> {
        let dep = self.repository.get_deposit(deposit_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Deposit {} not found", deposit_id)))?;

        if dep.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel deposit in '{}' status. Only 'draft' allowed.", dep.status)
            ));
        }

        info!("Cancelling customer deposit {}", dep.deposit_number);
        self.repository.update_deposit_status(deposit_id, "cancelled").await
    }

    // ========================================================================
    // Application to Invoices
    // ========================================================================

    /// Apply a deposit to an AR invoice
    pub async fn apply_to_invoice(
        &self,
        org_id: Uuid,
        deposit_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        applied_amount: &str,
        application_date: chrono::NaiveDate,
        gl_account_code: Option<&str>,
        applied_by: Option<Uuid>,
    ) -> AtlasResult<DepositApplication> {
        let dep = self.repository.get_deposit(deposit_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Deposit {} not found", deposit_id)))?;

        if dep.status != "received" && dep.status != "partially_applied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot apply deposit in '{}' status. Must be 'received' or 'partially_applied'.", dep.status)
            ));
        }

        let amount: f64 = applied_amount.parse().unwrap_or(-1.0);
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed("Applied amount must be positive".into()));
        }

        let unapplied: f64 = dep.unapplied_amount.parse().unwrap_or(0.0);
        if amount > unapplied + 0.01 {
            return Err(AtlasError::ValidationFailed(
                format!("Applied amount {} exceeds unapplied amount {}", amount, unapplied)
            ));
        }

        info!("Applying deposit {} to invoice (amount: {})", dep.deposit_number, applied_amount);

        let application = self.repository.create_application(
            org_id, deposit_id, Some(&dep.deposit_number),
            invoice_id, invoice_number, applied_amount, application_date,
            gl_account_code, applied_by,
        ).await?;

        // Recalculate deposit amounts
        let applications = self.repository.list_applications_by_deposit(deposit_id).await?;
        let total_applied: f64 = applications.iter()
            .filter(|a| a.status == "applied")
            .map(|a| a.applied_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let deposit_amt: f64 = dep.deposit_amount.parse().unwrap_or(0.0);
        let new_unapplied = deposit_amt - total_applied;
        let new_status = if new_unapplied.abs() < 0.01 { "fully_applied" } else { "partially_applied" };

        self.repository.update_deposit_amounts(
            deposit_id,
            &format!("{:.2}", total_applied),
            &format!("{:.2}", new_unapplied),
            new_status,
        ).await?;

        Ok(application)
    }

    /// Unapply (reverse) a deposit application
    pub async fn unapply_application(&self, application_id: Uuid, reversed_by: Option<Uuid>, reason: Option<&str>) -> AtlasResult<DepositApplication> {
        let app = self.repository.get_application(application_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", application_id)))?;

        if app.status != "applied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot unapply application in '{}' status. Must be 'applied'.", app.status)
            ));
        }

        info!("Unapplying deposit application for deposit {}", app.deposit_number.as_deref().unwrap_or("?"));

        let updated = self.repository.update_application_status(application_id, "reversed", reversed_by, reason).await?;

        // Recalculate deposit amounts
        let applications = self.repository.list_applications_by_deposit(app.deposit_id).await?;
        let total_applied: f64 = applications.iter()
            .filter(|a| a.status == "applied")
            .map(|a| a.applied_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        if let Some(dep) = self.repository.get_deposit(app.deposit_id).await? {
            let deposit_amt: f64 = dep.deposit_amount.parse().unwrap_or(0.0);
            let new_unapplied = deposit_amt - total_applied;
            let new_status = if total_applied.abs() < 0.01 { "received" } else { "partially_applied" };
            self.repository.update_deposit_amounts(
                app.deposit_id,
                &format!("{:.2}", total_applied),
                &format!("{:.2}", new_unapplied),
                new_status,
            ).await?;
        }

        Ok(updated)
    }

    /// List applications for a deposit
    pub async fn list_applications_by_deposit(&self, deposit_id: Uuid) -> AtlasResult<Vec<DepositApplication>> {
        self.repository.list_applications_by_deposit(deposit_id).await
    }

    /// List applications for an invoice
    pub async fn list_applications_by_invoice(&self, invoice_id: Uuid) -> AtlasResult<Vec<DepositApplication>> {
        self.repository.list_applications_by_invoice(invoice_id).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CustomerDepositDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        deposits: std::sync::Mutex<Vec<CustomerDeposit>>,
        applications: std::sync::Mutex<Vec<DepositApplication>>,
    }

    impl MockRepo { fn new() -> Self { MockRepo { deposits: std::sync::Mutex::new(vec![]), applications: std::sync::Mutex::new(vec![]) } } }

    #[async_trait::async_trait]
    impl CustomerDepositRepository for MockRepo {
        async fn create_deposit(&self, org_id: Uuid, dn: &str, cid: Uuid, cn: &str, csi: Option<Uuid>, desc: Option<&str>, cc: &str, amt: &str, er: Option<&str>, dac: Option<&str>, rac: Option<&str>, dd: chrono::NaiveDate, ed: Option<chrono::NaiveDate>, cb: Option<Uuid>) -> AtlasResult<CustomerDeposit> {
            let d = CustomerDeposit {
                id: Uuid::new_v4(), organization_id: org_id, deposit_number: dn.into(),
                customer_id: cid, customer_name: cn.into(), customer_site_id: csi,
                description: desc.map(Into::into), status: "draft".into(),
                currency_code: cc.into(), deposit_amount: amt.into(),
                applied_amount: "0".into(), unapplied_amount: amt.into(),
                exchange_rate: er.map(Into::into), deposit_account_code: dac.map(Into::into),
                receivable_account_code: rac.map(Into::into), deposit_date: dd,
                receipt_date: None, receipt_reference: None, expiration_date: ed,
                received_by: None, received_at: None, refund_reference: None,
                refunded_by: None, refunded_at: None, cancelled_reason: None,
                metadata: serde_json::json!({}), created_by: cb,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.deposits.lock().unwrap().push(d.clone());
            Ok(d)
        }
        async fn get_deposit(&self, id: Uuid) -> AtlasResult<Option<CustomerDeposit>> {
            Ok(self.deposits.lock().unwrap().iter().find(|d| d.id == id).cloned())
        }
        async fn get_deposit_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<CustomerDeposit>> {
            Ok(self.deposits.lock().unwrap().iter().find(|d| d.organization_id == org_id && d.deposit_number == num).cloned())
        }
        async fn list_deposits(&self, org_id: Uuid, status: Option<&str>, _customer_id: Option<Uuid>) -> AtlasResult<Vec<CustomerDeposit>> {
            Ok(self.deposits.lock().unwrap().iter()
                .filter(|d| d.organization_id == org_id)
                .filter(|d| status.map_or(true, |s| d.status == s))
                .cloned().collect())
        }
        async fn update_deposit_status(&self, id: Uuid, status: &str) -> AtlasResult<CustomerDeposit> {
            let mut deps = self.deposits.lock().unwrap();
            let d = deps.iter_mut().find(|d| d.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Deposit {} not found", id)))?;
            d.status = status.into();
            d.updated_at = chrono::Utc::now();
            Ok(d.clone())
        }
        async fn update_deposit_amounts(&self, id: Uuid, applied: &str, unapplied: &str, status: &str) -> AtlasResult<()> {
            let mut deps = self.deposits.lock().unwrap();
            if let Some(d) = deps.iter_mut().find(|d| d.id == id) {
                d.applied_amount = applied.into();
                d.unapplied_amount = unapplied.into();
                d.status = status.into();
                d.updated_at = chrono::Utc::now();
            }
            Ok(())
        }
        async fn update_receipt_info(&self, id: Uuid, rr: Option<&str>, rb: Option<Uuid>) -> AtlasResult<CustomerDeposit> {
            let mut deps = self.deposits.lock().unwrap();
            let d = deps.iter_mut().find(|d| d.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Deposit {} not found", id)))?;
            d.receipt_reference = rr.map(Into::into);
            d.received_by = rb;
            d.receipt_date = Some(chrono::Utc::now().date_naive());
            d.updated_at = chrono::Utc::now();
            Ok(d.clone())
        }
        async fn update_refund_info(&self, id: Uuid, rr: Option<&str>, rb: Option<Uuid>) -> AtlasResult<CustomerDeposit> {
            let mut deps = self.deposits.lock().unwrap();
            let d = deps.iter_mut().find(|d| d.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Deposit {} not found", id)))?;
            d.refund_reference = rr.map(Into::into);
            d.refunded_by = rb;
            d.refunded_at = Some(chrono::Utc::now());
            d.updated_at = chrono::Utc::now();
            Ok(d.clone())
        }
        async fn create_application(&self, org_id: Uuid, did: Uuid, dn: Option<&str>, iid: Uuid, in_: Option<&str>, amt: &str, ad: chrono::NaiveDate, glac: Option<&str>, ab: Option<Uuid>) -> AtlasResult<DepositApplication> {
            let app = DepositApplication {
                id: Uuid::new_v4(), organization_id: org_id, deposit_id: did,
                deposit_number: dn.map(Into::into), invoice_id: iid,
                invoice_number: in_.map(Into::into), applied_amount: amt.into(),
                application_date: ad, status: "applied".into(),
                gl_account_code: glac.map(Into::into), reversed_at: None,
                reversed_by: None, reversal_reason: None, metadata: serde_json::json!({}),
                applied_by: ab, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.applications.lock().unwrap().push(app.clone());
            Ok(app)
        }
        async fn get_application(&self, id: Uuid) -> AtlasResult<Option<DepositApplication>> {
            Ok(self.applications.lock().unwrap().iter().find(|a| a.id == id).cloned())
        }
        async fn list_applications_by_deposit(&self, deposit_id: Uuid) -> AtlasResult<Vec<DepositApplication>> {
            Ok(self.applications.lock().unwrap().iter().filter(|a| a.deposit_id == deposit_id).cloned().collect())
        }
        async fn list_applications_by_invoice(&self, invoice_id: Uuid) -> AtlasResult<Vec<DepositApplication>> {
            Ok(self.applications.lock().unwrap().iter().filter(|a| a.invoice_id == invoice_id).cloned().collect())
        }
        async fn update_application_status(&self, id: Uuid, status: &str, rb: Option<Uuid>, rr: Option<&str>) -> AtlasResult<DepositApplication> {
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
        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<CustomerDepositDashboard> {
            Ok(CustomerDepositDashboard { total_deposits: 0, draft_deposits: 0, open_deposits: 0, total_deposit_amount: "0".into(), total_applied_amount: "0".into(), total_unapplied_amount: "0".into(), total_refunded_amount: "0".into(), deposits_by_customer: serde_json::json!([]), aging_buckets: serde_json::json!([]) })
        }
    }

    fn eng() -> CustomerDepositEngine { CustomerDepositEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_STATUSES.len(), 6);
        assert_eq!(VALID_APPLICATION_STATUSES.len(), 3);
    }

    #[tokio::test]
    async fn test_create_deposit_valid() {
        let d = eng().create_deposit(
            Uuid::new_v4(), "DEP-001", Uuid::new_v4(), "BigCorp", None,
            Some("Advance for Q2 order"), "USD", "100000.00", None,
            None, None, chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()), None,
        ).await.unwrap();
        assert_eq!(d.deposit_number, "DEP-001");
        assert_eq!(d.status, "draft");
        assert_eq!(d.deposit_amount, "100000.00");
        assert_eq!(d.unapplied_amount, "100000.00");
    }

    #[tokio::test]
    async fn test_create_deposit_empty_number() {
        let r = eng().create_deposit(Uuid::new_v4(), "", Uuid::new_v4(), "Cust", None, None, "USD", "1000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_deposit_empty_customer() {
        let r = eng().create_deposit(Uuid::new_v4(), "DEP-1", Uuid::new_v4(), "", None, None, "USD", "1000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_deposit_empty_currency() {
        let r = eng().create_deposit(Uuid::new_v4(), "DEP-1", Uuid::new_v4(), "Cust", None, None, "", "1000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_deposit_zero_amount() {
        let r = eng().create_deposit(Uuid::new_v4(), "DEP-1", Uuid::new_v4(), "Cust", None, None, "USD", "0.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_deposit_negative_amount() {
        let r = eng().create_deposit(Uuid::new_v4(), "DEP-1", Uuid::new_v4(), "Cust", None, None, "USD", "-500.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_deposit_expiry_before_date() {
        let r = eng().create_deposit(Uuid::new_v4(), "DEP-1", Uuid::new_v4(), "Cust", None, None, "USD", "1000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,6,1).unwrap(), Some(chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap()), None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_deposit_duplicate_number() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_deposit(org, "DEP-DUP", Uuid::new_v4(), "C1", None, None, "USD", "1000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        let r = e.create_deposit(org, "DEP-DUP", Uuid::new_v4(), "C2", None, None, "USD", "2000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_receive_deposit_valid() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-RCV1", Uuid::new_v4(), "Cust", None, None, "USD", "50000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        let received = e.receive_deposit(d.id, Some("RCT-001"), None).await.unwrap();
        assert_eq!(received.status, "received");
    }

    #[tokio::test]
    async fn test_receive_non_draft_fails() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-RCV2", Uuid::new_v4(), "Cust", None, None, "USD", "50000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        e.receive_deposit(d.id, None, None).await.unwrap();
        let r = e.receive_deposit(d.id, None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_refund_deposit_valid() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-REF1", Uuid::new_v4(), "Cust", None, None, "USD", "50000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        e.receive_deposit(d.id, None, None).await.unwrap();
        let refunded = e.refund_deposit(d.id, Some("RF-001"), None).await.unwrap();
        assert_eq!(refunded.status, "refunded");
    }

    #[tokio::test]
    async fn test_refund_draft_fails() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-REF2", Uuid::new_v4(), "Cust", None, None, "USD", "50000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        let r = e.refund_deposit(d.id, None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_cancel_draft_valid() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-CAN1", Uuid::new_v4(), "Cust", None, None, "USD", "50000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        let cancelled = e.cancel_deposit(d.id, Some("Order cancelled")).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_received_fails() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-CAN2", Uuid::new_v4(), "Cust", None, None, "USD", "50000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        e.receive_deposit(d.id, None, None).await.unwrap();
        let r = e.cancel_deposit(d.id, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_list_deposits_invalid_status() {
        let r = eng().list_deposits(Uuid::new_v4(), Some("bad"), None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_list_deposits_valid() {
        let r = eng().list_deposits(Uuid::new_v4(), Some("draft"), None).await;
        assert!(r.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_apply_to_invoice_valid() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-APP1", Uuid::new_v4(), "Cust", None, None, "USD", "100000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        e.receive_deposit(d.id, None, None).await.unwrap();
        let app = e.apply_to_invoice(d.organization_id, d.id, Uuid::new_v4(), Some("INV-001"), "60000.00", chrono::NaiveDate::from_ymd_opt(2026,2,1).unwrap(), None, None).await.unwrap();
        assert_eq!(app.status, "applied");
        assert_eq!(app.applied_amount, "60000.00");
    }

    #[tokio::test]
    async fn test_apply_to_invoice_not_received_fails() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-APP2", Uuid::new_v4(), "Cust", None, None, "USD", "50000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        let r = e.apply_to_invoice(d.organization_id, d.id, Uuid::new_v4(), None, "10000.00", chrono::NaiveDate::from_ymd_opt(2026,2,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_apply_zero_amount_fails() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-APP3", Uuid::new_v4(), "Cust", None, None, "USD", "50000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        e.receive_deposit(d.id, None, None).await.unwrap();
        let r = e.apply_to_invoice(d.organization_id, d.id, Uuid::new_v4(), None, "0.00", chrono::NaiveDate::from_ymd_opt(2026,2,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_apply_exceeds_unapplied_fails() {
        let e = eng();
        let d = e.create_deposit(Uuid::new_v4(), "DEP-APP4", Uuid::new_v4(), "Cust", None, None, "USD", "10000.00", None, None, None, chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        e.receive_deposit(d.id, None, None).await.unwrap();
        let r = e.apply_to_invoice(d.organization_id, d.id, Uuid::new_v4(), None, "99999.00", chrono::NaiveDate::from_ymd_opt(2026,2,1).unwrap(), None, None).await;
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
        assert_eq!(d.total_deposits, 0);
    }
}
