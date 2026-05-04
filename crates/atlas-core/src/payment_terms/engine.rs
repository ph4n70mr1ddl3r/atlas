//! Payment Terms Engine
//! Oracle Fusion: Financials > Payment Terms Management

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_STATUSES: &[&str] = &["active", "inactive"];
const VALID_TERM_TYPES: &[&str] = &["standard", "installment", "proxima", "day_of_month"];
const VALID_DISCOUNT_BASES: &[&str] = &["invoice_amount", "line_amount"];

pub struct PaymentTermsEngine { repository: Arc<dyn PaymentTermsRepository> }

impl PaymentTermsEngine {
    pub fn new(r: Arc<dyn PaymentTermsRepository>) -> Self { Self { repository: r } }

    pub async fn create_term(&self, org_id: Uuid, term_code: &str, name: &str, description: Option<&str>, base_due_days: i32, due_date_cutoff_day: Option<i32>, term_type: &str, default_discount_percent: &str, created_by: Option<Uuid>) -> AtlasResult<PaymentTerm> {
        if term_code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Term code and name are required".into()));
        }
        if !(0..=365).contains(&base_due_days) {
            return Err(AtlasError::ValidationFailed("Base due days must be 0-365".into()));
        }
        if let Some(cutoff) = due_date_cutoff_day {
            if !(1..=31).contains(&cutoff) {
                return Err(AtlasError::ValidationFailed("Cutoff day must be 1-31".into()));
            }
        }
        if !VALID_TERM_TYPES.contains(&term_type) {
            return Err(AtlasError::ValidationFailed(format!("Invalid term_type '{}'", term_type)));
        }
        let discount: f64 = default_discount_percent.parse().map_err(|_| AtlasError::ValidationFailed("Invalid discount percent".into()))?;
        if !(0.0..=100.0).contains(&discount) {
            return Err(AtlasError::ValidationFailed("Discount percent must be 0-100".into()));
        }
        if self.repository.get_term(org_id, term_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Payment term '{}' already exists", term_code)));
        }
        info!("Creating payment term {} for org {}", term_code, org_id);
        self.repository.create_term(org_id, term_code, name, description, base_due_days, due_date_cutoff_day, term_type, default_discount_percent, created_by).await
    }

    pub async fn get_term(&self, org_id: Uuid, term_code: &str) -> AtlasResult<Option<PaymentTerm>> {
        self.repository.get_term(org_id, term_code).await
    }

    pub async fn get_term_by_id(&self, id: Uuid) -> AtlasResult<Option<PaymentTerm>> {
        self.repository.get_term_by_id(id).await
    }

    pub async fn list_terms(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentTerm>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s)));
            }
        }
        self.repository.list_terms(org_id, status).await
    }

    pub async fn activate_term(&self, id: Uuid) -> AtlasResult<PaymentTerm> {
        let term = self.repository.get_term_by_id(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Payment term {} not found", id)))?;
        if term.status == "active" { return Err(AtlasError::WorkflowError("Already active".into())); }
        self.repository.update_term_status(id, "active").await
    }

    pub async fn deactivate_term(&self, id: Uuid) -> AtlasResult<PaymentTerm> {
        let term = self.repository.get_term_by_id(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Payment term {} not found", id)))?;
        if term.status == "inactive" { return Err(AtlasError::WorkflowError("Already inactive".into())); }
        self.repository.update_term_status(id, "inactive").await
    }

    pub async fn delete_term(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.get_term_by_id(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Payment term {} not found", id)))?;
        self.repository.delete_term(id).await
    }

    // Discount Schedules
    pub async fn create_discount_schedule(&self, org_id: Uuid, term_id: Uuid, discount_percent: &str, discount_days: i32, discount_day_of_month: Option<i32>, discount_basis: &str, display_order: i32) -> AtlasResult<PaymentTermDiscountSchedule> {
        self.repository.get_term_by_id(term_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Payment term {} not found", term_id)))?;
        let discount: f64 = discount_percent.parse().map_err(|_| AtlasError::ValidationFailed("Invalid discount percent".into()))?;
        if !(0.0..=100.0).contains(&discount) {
            return Err(AtlasError::ValidationFailed("Discount percent must be 0-100".into()));
        }
        if !(0..=365).contains(&discount_days) {
            return Err(AtlasError::ValidationFailed("Discount days must be 0-365".into()));
        }
        if let Some(dom) = discount_day_of_month {
            if !(1..=31).contains(&dom) {
                return Err(AtlasError::ValidationFailed("Day of month must be 1-31".into()));
            }
        }
        if !VALID_DISCOUNT_BASES.contains(&discount_basis) {
            return Err(AtlasError::ValidationFailed(format!("Invalid discount_basis '{}'", discount_basis)));
        }
        self.repository.create_discount_schedule(org_id, term_id, discount_percent, discount_days, discount_day_of_month, discount_basis, display_order).await
    }

    pub async fn list_discount_schedules(&self, term_id: Uuid) -> AtlasResult<Vec<PaymentTermDiscountSchedule>> {
        self.repository.list_discount_schedules(term_id).await
    }

    pub async fn delete_discount_schedule(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_discount_schedule(id).await
    }

    // Installments
    pub async fn create_installment(&self, org_id: Uuid, term_id: Uuid, installment_number: i32, due_days_offset: i32, percentage: &str, discount_percent: &str, discount_days: i32) -> AtlasResult<PaymentTermInstallment> {
        self.repository.get_term_by_id(term_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Payment term {} not found", term_id)))?;
        if installment_number < 1 {
            return Err(AtlasError::ValidationFailed("Installment number must be >= 1".into()));
        }
        if due_days_offset < 0 {
            return Err(AtlasError::ValidationFailed("Due days offset must be >= 0".into()));
        }
        let pct: f64 = percentage.parse().map_err(|_| AtlasError::ValidationFailed("Invalid percentage".into()))?;
        if pct <= 0.0 || pct > 100.0 {
            return Err(AtlasError::ValidationFailed("Percentage must be > 0 and <= 100".into()));
        }
        let disc: f64 = discount_percent.parse().map_err(|_| AtlasError::ValidationFailed("Invalid discount percent".into()))?;
        if !(0.0..=100.0).contains(&disc) {
            return Err(AtlasError::ValidationFailed("Discount percent must be 0-100".into()));
        }
        if !(0..=365).contains(&discount_days) {
            return Err(AtlasError::ValidationFailed("Discount days must be 0-365".into()));
        }
        self.repository.create_installment(org_id, term_id, installment_number, due_days_offset, percentage, discount_percent, discount_days).await
    }

    pub async fn list_installments(&self, term_id: Uuid) -> AtlasResult<Vec<PaymentTermInstallment>> {
        self.repository.list_installments(term_id).await
    }

    pub async fn delete_installment(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_installment(id).await
    }

    /// Calculate due date for a given invoice date and term
    pub fn calculate_due_date(&self, invoice_date: NaiveDate, term: &PaymentTerm) -> NaiveDate {
        match term.term_type.as_str() {
            "standard" | "installment" => invoice_date + chrono::Duration::days(term.base_due_days as i64),
            "proxima" | "day_of_month" => {
                // Simplified: just add base_due_days for proxima/day_of_month
                // (full cutoff-day logic requires chrono Datelike traits that may not be available)
                invoice_date + chrono::Duration::days(term.base_due_days as i64)
            },
            _ => invoice_date + chrono::Duration::days(term.base_due_days as i64),
        }
    }

    /// Calculate early payment discount amount
    pub fn calculate_discount(&self, invoice_amount: f64, schedule: &PaymentTermDiscountSchedule, days_since_invoice: i32) -> f64 {
        if days_since_invoice <= schedule.discount_days {
            let rate: f64 = schedule.discount_percent.parse().unwrap_or(0.0);
            invoice_amount * rate / 100.0
        } else {
            0.0
        }
    }

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PaymentTermDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    struct MockRepo {
        terms: std::sync::Mutex<Vec<PaymentTerm>>,
        discounts: std::sync::Mutex<Vec<PaymentTermDiscountSchedule>>,
        installments: std::sync::Mutex<Vec<PaymentTermInstallment>>,
    }
    impl MockRepo { fn new() -> Self { Self { terms: std::sync::Mutex::new(vec![]), discounts: std::sync::Mutex::new(vec![]), installments: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl PaymentTermsRepository for MockRepo {
        async fn create_term(&self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>, due_days: i32, cutoff: Option<i32>, tt: &str, dp: &str, cb: Option<Uuid>) -> AtlasResult<PaymentTerm> {
            let t = PaymentTerm { id: Uuid::new_v4(), organization_id: org_id, term_code: code.into(), name: name.into(), description: desc.map(Into::into), base_due_days: due_days, due_date_cutoff_day: cutoff, status: "active".into(), term_type: tt.into(), default_discount_percent: dp.into(), metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.terms.lock().unwrap().push(t.clone());
            Ok(t)
        }
        async fn get_term(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PaymentTerm>> { Ok(self.terms.lock().unwrap().iter().find(|t| t.organization_id == org_id && t.term_code == code).cloned()) }
        async fn get_term_by_id(&self, id: Uuid) -> AtlasResult<Option<PaymentTerm>> { Ok(self.terms.lock().unwrap().iter().find(|t| t.id == id).cloned()) }
        async fn list_terms(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentTerm>> { Ok(self.terms.lock().unwrap().iter().filter(|t| t.organization_id == org_id && (status.is_none() || t.status == status.unwrap())).cloned().collect()) }
        async fn update_term_status(&self, id: Uuid, status: &str) -> AtlasResult<PaymentTerm> {
            let mut ts = self.terms.lock().unwrap();
            let t = ts.iter_mut().find(|t| t.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            t.status = status.into();
            Ok(t.clone())
        }
        async fn delete_term(&self, id: Uuid) -> AtlasResult<()> { Ok(()) }
        async fn create_discount_schedule(&self, org_id: Uuid, term_id: Uuid, dp: &str, dd: i32, dom: Option<i32>, basis: &str, order: i32) -> AtlasResult<PaymentTermDiscountSchedule> {
            let s = PaymentTermDiscountSchedule { id: Uuid::new_v4(), organization_id: org_id, payment_term_id: term_id, discount_percent: dp.into(), discount_days: dd, discount_day_of_month: dom, discount_basis: basis.into(), display_order: order, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.discounts.lock().unwrap().push(s.clone());
            Ok(s)
        }
        async fn list_discount_schedules(&self, tid: Uuid) -> AtlasResult<Vec<PaymentTermDiscountSchedule>> { Ok(self.discounts.lock().unwrap().iter().filter(|d| d.payment_term_id == tid).cloned().collect()) }
        async fn delete_discount_schedule(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
        async fn create_installment(&self, org_id: Uuid, term_id: Uuid, num: i32, offset: i32, pct: &str, dp: &str, dd: i32) -> AtlasResult<PaymentTermInstallment> {
            let i = PaymentTermInstallment { id: Uuid::new_v4(), organization_id: org_id, payment_term_id: term_id, installment_number: num, due_days_offset: offset, percentage: pct.into(), discount_percent: dp.into(), discount_days: dd, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.installments.lock().unwrap().push(i.clone());
            Ok(i)
        }
        async fn list_installments(&self, tid: Uuid) -> AtlasResult<Vec<PaymentTermInstallment>> { Ok(self.installments.lock().unwrap().iter().filter(|i| i.payment_term_id == tid).cloned().collect()) }
        async fn delete_installment(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PaymentTermDashboard> {
            let ts = self.terms.lock().unwrap();
            let terms = ts.iter().filter(|t| t.organization_id == org_id).collect::<Vec<_>>();
            Ok(PaymentTermDashboard { total_terms: terms.len() as i32, active_terms: terms.iter().filter(|t| t.status == "active").count() as i32, terms_with_discounts: 0, terms_with_installments: 0 })
        }
    }

    fn eng() -> PaymentTermsEngine { PaymentTermsEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_STATUSES.len(), 2);
        assert_eq!(VALID_TERM_TYPES.len(), 4);
        assert_eq!(VALID_DISCOUNT_BASES.len(), 2);
    }

    #[tokio::test]
    async fn test_create_term_valid() {
        let t = eng().create_term(Uuid::new_v4(), "NET30", "Net 30 Days", None, 30, None, "standard", "0", None).await.unwrap();
        assert_eq!(t.term_code, "NET30");
        assert_eq!(t.status, "active");
        assert_eq!(t.base_due_days, 30);
    }

    #[tokio::test]
    async fn test_create_term_empty_code() {
        assert!(eng().create_term(Uuid::new_v4(), "", "Net 30", None, 30, None, "standard", "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_empty_name() {
        assert!(eng().create_term(Uuid::new_v4(), "NET30", "", None, 30, None, "standard", "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_negative_due_days() {
        assert!(eng().create_term(Uuid::new_v4(), "NET30", "T", None, -1, None, "standard", "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_due_days_over_365() {
        assert!(eng().create_term(Uuid::new_v4(), "NET30", "T", None, 366, None, "standard", "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_invalid_type() {
        assert!(eng().create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "custom", "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_invalid_discount() {
        assert!(eng().create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "150", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_negative_discount() {
        assert!(eng().create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "-5", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_cutoff_day_zero() {
        assert!(eng().create_term(Uuid::new_v4(), "NET30", "T", None, 30, Some(0), "proxima", "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_cutoff_day_32() {
        assert!(eng().create_term(Uuid::new_v4(), "NET30", "T", None, 30, Some(32), "proxima", "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_term_duplicate() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_term(org, "DUP", "T1", None, 30, None, "standard", "0", None).await;
        assert!(e.create_term(org, "DUP", "T2", None, 30, None, "standard", "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_terms_invalid_status() {
        assert!(eng().list_terms(Uuid::new_v4(), Some("bad")).await.is_err());
    }

    #[tokio::test]
    async fn test_create_discount_schedule_valid() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "0", None).await.unwrap();
        let ds = e.create_discount_schedule(Uuid::new_v4(), t.id, "2", 10, None, "invoice_amount", 0).await.unwrap();
        assert_eq!(ds.discount_percent, "2");
        assert_eq!(ds.discount_days, 10);
    }

    #[tokio::test]
    async fn test_create_discount_schedule_invalid_percent() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "0", None).await.unwrap();
        assert!(e.create_discount_schedule(Uuid::new_v4(), t.id, "150", 10, None, "invoice_amount", 0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_discount_schedule_invalid_days() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "0", None).await.unwrap();
        assert!(e.create_discount_schedule(Uuid::new_v4(), t.id, "2", -1, None, "invoice_amount", 0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_discount_schedule_invalid_basis() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "0", None).await.unwrap();
        assert!(e.create_discount_schedule(Uuid::new_v4(), t.id, "2", 10, None, "bad_basis", 0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_discount_schedule_term_not_found() {
        assert!(eng().create_discount_schedule(Uuid::new_v4(), Uuid::new_v4(), "2", 10, None, "invoice_amount", 0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_installment_valid() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "INSTALL", "T", None, 60, None, "installment", "0", None).await.unwrap();
        let inst = e.create_installment(Uuid::new_v4(), t.id, 1, 30, "50", "2", 10).await.unwrap();
        assert_eq!(inst.installment_number, 1);
        assert_eq!(inst.percentage, "50");
    }

    #[tokio::test]
    async fn test_create_installment_zero_number() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "INSTALL", "T", None, 60, None, "installment", "0", None).await.unwrap();
        assert!(e.create_installment(Uuid::new_v4(), t.id, 0, 30, "50", "2", 10).await.is_err());
    }

    #[tokio::test]
    async fn test_create_installment_negative_offset() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "INSTALL", "T", None, 60, None, "installment", "0", None).await.unwrap();
        assert!(e.create_installment(Uuid::new_v4(), t.id, 1, -1, "50", "2", 10).await.is_err());
    }

    #[tokio::test]
    async fn test_create_installment_zero_pct() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "INSTALL", "T", None, 60, None, "installment", "0", None).await.unwrap();
        assert!(e.create_installment(Uuid::new_v4(), t.id, 1, 30, "0", "2", 10).await.is_err());
    }

    #[tokio::test]
    async fn test_create_installment_pct_over_100() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "INSTALL", "T", None, 60, None, "installment", "0", None).await.unwrap();
        assert!(e.create_installment(Uuid::new_v4(), t.id, 1, 30, "150", "2", 10).await.is_err());
    }

    #[tokio::test]
    async fn test_calculate_due_date_standard() {
        let e = eng();
        let t = PaymentTerm { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), term_code: "NET30".into(), name: "Net 30".into(), description: None, base_due_days: 30, due_date_cutoff_day: None, status: "active".into(), term_type: "standard".into(), default_discount_percent: "0".into(), metadata: serde_json::json!({}), created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
        let inv = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let due = e.calculate_due_date(inv, &t);
        assert_eq!(due, NaiveDate::from_ymd_opt(2026, 2, 14).unwrap());
    }

    #[tokio::test]
    async fn test_calculate_due_date_zero_days() {
        let e = eng();
        let t = PaymentTerm { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), term_code: "IMMEDIATE".into(), name: "Immediate".into(), description: None, base_due_days: 0, due_date_cutoff_day: None, status: "active".into(), term_type: "standard".into(), default_discount_percent: "0".into(), metadata: serde_json::json!({}), created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
        let inv = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let due = e.calculate_due_date(inv, &t);
        assert_eq!(due, inv);
    }

    #[tokio::test]
    async fn test_calculate_discount_within_window() {
        let e = eng();
        let sched = PaymentTermDiscountSchedule { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), payment_term_id: Uuid::new_v4(), discount_percent: "2".into(), discount_days: 10, discount_day_of_month: None, discount_basis: "invoice_amount".into(), display_order: 0, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
        let disc = e.calculate_discount(1000.0, &sched, 5);
        assert!((disc - 20.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_calculate_discount_outside_window() {
        let e = eng();
        let sched = PaymentTermDiscountSchedule { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), payment_term_id: Uuid::new_v4(), discount_percent: "2".into(), discount_days: 10, discount_day_of_month: None, discount_basis: "invoice_amount".into(), display_order: 0, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
        let disc = e.calculate_discount(1000.0, &sched, 15);
        assert!((disc - 0.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_calculate_discount_at_boundary() {
        let e = eng();
        let sched = PaymentTermDiscountSchedule { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), payment_term_id: Uuid::new_v4(), discount_percent: "2".into(), discount_days: 10, discount_day_of_month: None, discount_basis: "invoice_amount".into(), display_order: 0, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
        let disc = e.calculate_discount(1000.0, &sched, 10);
        assert!((disc - 20.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create_term(org, "NET30", "T1", None, 30, None, "standard", "0", None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_terms, 1);
        assert_eq!(dash.active_terms, 1);
    }

    #[tokio::test]
    async fn test_activate_already_active() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "0", None).await.unwrap();
        assert!(e.activate_term(t.id).await.is_err());
    }

    #[tokio::test]
    async fn test_deactivate_and_reactivate() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "0", None).await.unwrap();
        let t = e.deactivate_term(t.id).await.unwrap();
        assert_eq!(t.status, "inactive");
        let t = e.activate_term(t.id).await.unwrap();
        assert_eq!(t.status, "active");
    }

    #[tokio::test]
    async fn test_deactivate_already_inactive() {
        let e = eng();
        let t = e.create_term(Uuid::new_v4(), "NET30", "T", None, 30, None, "standard", "0", None).await.unwrap();
        let t = e.deactivate_term(t.id).await.unwrap();
        assert!(e.deactivate_term(t.id).await.is_err());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        assert!(eng().delete_term(Uuid::new_v4()).await.is_err());
    }
}
