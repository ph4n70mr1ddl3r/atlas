//! Cash Position Engine
//!
//! Manages real-time cash positions across bank accounts:
//! - Calculate position per account from inflows/outflows
//! - Aggregate positions by currency
//! - Track position trends over time
//! - Dashboard with multi-currency overview
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Treasury > Cash Position

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct CashPositionEngine {
    repository: Arc<dyn CashPositionRepository>,
}

impl CashPositionEngine {
    pub fn new(repository: Arc<dyn CashPositionRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Position Management
    // ========================================================================

    /// Create/update a cash position for a bank account on a given date
    pub async fn record_position(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        bank_account_number: Option<&str>,
        bank_account_name: Option<&str>,
        currency_code: &str,
        opening_balance: &str,
        total_inflows: &str,
        total_outflows: &str,
        closing_balance: &str,
        ledger_balance: &str,
        available_balance: &str,
        hold_amount: &str,
        position_date: chrono::NaiveDate,
        source_breakdown: serde_json::Value,
    ) -> AtlasResult<CashPosition> {
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".into()));
        }

        let opening: f64 = opening_balance.parse().unwrap_or(0.0);
        let inflows: f64 = total_inflows.parse().unwrap_or(0.0);
        let outflows: f64 = total_outflows.parse().unwrap_or(0.0);
        let closing: f64 = closing_balance.parse().unwrap_or(0.0);
        let ledger: f64 = ledger_balance.parse().unwrap_or(0.0);
        let available: f64 = available_balance.parse().unwrap_or(0.0);
        let hold: f64 = hold_amount.parse().unwrap_or(0.0);

        if hold < 0.0 {
            return Err(AtlasError::ValidationFailed("Hold amount must be non-negative".into()));
        }

        // Verify closing = opening + inflows - outflows (with tolerance)
        let expected_closing = opening + inflows - outflows;
        if (closing - expected_closing).abs() > 0.01 {
            return Err(AtlasError::ValidationFailed(
                format!("Closing balance {} doesn't match opening({}) + inflows({}) - outflows({}) = {}",
                    closing, opening, inflows, outflows, expected_closing)
            ));
        }

        // Verify available = ledger - hold
        if (available - (ledger - hold)).abs() > 0.01 {
            return Err(AtlasError::ValidationFailed(
                format!("Available balance {} doesn't match ledger({}) - hold({}) = {}",
                    available, ledger, hold, ledger - hold)
            ));
        }

        info!("Recording cash position for account {:?} on {} (closing: {})",
            bank_account_number, position_date, closing_balance);

        self.repository.create_position(
            org_id, bank_account_id, bank_account_number, bank_account_name,
            currency_code, opening_balance, total_inflows, total_outflows,
            closing_balance, ledger_balance, available_balance, hold_amount,
            position_date, source_breakdown,
        ).await
    }

    /// Get a specific position by ID
    pub async fn get_position(&self, id: Uuid) -> AtlasResult<Option<CashPosition>> {
        self.repository.get_position(id).await
    }

    /// Get the latest position for a bank account
    pub async fn get_latest_position(&self, org_id: Uuid, bank_account_id: Uuid) -> AtlasResult<Option<CashPosition>> {
        self.repository.get_latest_position(org_id, bank_account_id).await
    }

    /// List positions with optional filters
    pub async fn list_positions(
        &self,
        org_id: Uuid,
        position_date: Option<chrono::NaiveDate>,
        currency_code: Option<&str>,
    ) -> AtlasResult<Vec<CashPosition>> {
        self.repository.list_positions(org_id, position_date, currency_code).await
    }

    // ========================================================================
    // Aggregation
    // ========================================================================

    /// Create an aggregated summary across accounts for a single currency
    pub async fn create_summary(
        &self,
        org_id: Uuid,
        currency_code: &str,
        position_date: chrono::NaiveDate,
        positions: &[CashPosition],
    ) -> AtlasResult<CashPositionSummary> {
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".into()));
        }

        let currency_positions: Vec<&CashPosition> = positions.iter()
            .filter(|p| p.currency_code == currency_code)
            .collect();

        if currency_positions.is_empty() {
            return Err(AtlasError::ValidationFailed(
                format!("No positions found for currency {}", currency_code)
            ));
        }

        let total_opening: f64 = currency_positions.iter()
            .map(|p| p.opening_balance.parse::<f64>().unwrap_or(0.0)).sum();
        let total_inflows: f64 = currency_positions.iter()
            .map(|p| p.total_inflows.parse::<f64>().unwrap_or(0.0)).sum();
        let total_outflows: f64 = currency_positions.iter()
            .map(|p| p.total_outflows.parse::<f64>().unwrap_or(0.0)).sum();
        let total_closing: f64 = currency_positions.iter()
            .map(|p| p.closing_balance.parse::<f64>().unwrap_or(0.0)).sum();
        let total_ledger: f64 = currency_positions.iter()
            .map(|p| p.ledger_balance.parse::<f64>().unwrap_or(0.0)).sum();
        let total_available: f64 = currency_positions.iter()
            .map(|p| p.available_balance.parse::<f64>().unwrap_or(0.0)).sum();
        let total_hold: f64 = currency_positions.iter()
            .map(|p| p.hold_amount.parse::<f64>().unwrap_or(0.0)).sum();

        let accounts: Vec<serde_json::Value> = currency_positions.iter().map(|p| {
            serde_json::json!({
                "bank_account_id": p.bank_account_id,
                "bank_account_name": p.bank_account_name,
                "closing_balance": p.closing_balance,
            })
        }).collect();

        info!("Creating cash position summary for {} on {} ({} accounts, total: {})",
            currency_code, position_date, currency_positions.len(), total_closing);

        self.repository.create_summary(
            org_id, currency_code, position_date,
            &format!("{:.2}", total_opening),
            &format!("{:.2}", total_inflows),
            &format!("{:.2}", total_outflows),
            &format!("{:.2}", total_closing),
            &format!("{:.2}", total_ledger),
            &format!("{:.2}", total_available),
            &format!("{:.2}", total_hold),
            currency_positions.len() as i32,
            serde_json::json!(accounts),
        ).await
    }

    /// Get latest summary for a currency
    pub async fn get_latest_summary(&self, org_id: Uuid, currency_code: &str) -> AtlasResult<Option<CashPositionSummary>> {
        self.repository.get_latest_summary(org_id, currency_code).await
    }

    /// List summaries
    pub async fn list_summaries(&self, org_id: Uuid, position_date: Option<chrono::NaiveDate>) -> AtlasResult<Vec<CashPositionSummary>> {
        self.repository.list_summaries(org_id, position_date).await
    }

    /// Get the cash position dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashPositionDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        positions: std::sync::Mutex<Vec<CashPosition>>,
    }

    impl MockRepo { fn new() -> Self { MockRepo { positions: std::sync::Mutex::new(vec![]) } } }

    #[async_trait::async_trait]
    impl CashPositionRepository for MockRepo {
        async fn create_position(&self, org_id: Uuid, bai: Uuid, ban: Option<&str>, bnam: Option<&str>, cc: &str, ob: &str, ti: &str, to_: &str, cb: &str, lb: &str, ab: &str, ha: &str, pd: chrono::NaiveDate, sb: serde_json::Value) -> AtlasResult<CashPosition> {
            let p = CashPosition {
                id: Uuid::new_v4(), organization_id: org_id, bank_account_id: bai,
                bank_account_number: ban.map(Into::into), bank_account_name: bnam.map(Into::into),
                currency_code: cc.into(), opening_balance: ob.into(), total_inflows: ti.into(),
                total_outflows: to_.into(), closing_balance: cb.into(), ledger_balance: lb.into(),
                available_balance: ab.into(), hold_amount: ha.into(), position_date: pd,
                source_breakdown: sb, metadata: serde_json::json!({}),
                calculated_at: chrono::Utc::now(), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.positions.lock().unwrap().push(p.clone());
            Ok(p)
        }
        async fn get_position(&self, id: Uuid) -> AtlasResult<Option<CashPosition>> {
            Ok(self.positions.lock().unwrap().iter().find(|p| p.id == id).cloned())
        }
        async fn get_latest_position(&self, org_id: Uuid, bai: Uuid) -> AtlasResult<Option<CashPosition>> {
            Ok(self.positions.lock().unwrap().iter()
                .filter(|p| p.organization_id == org_id && p.bank_account_id == bai)
                .max_by_key(|p| p.position_date).cloned())
        }
        async fn list_positions(&self, org_id: Uuid, pd: Option<chrono::NaiveDate>, _cc: Option<&str>) -> AtlasResult<Vec<CashPosition>> {
            Ok(self.positions.lock().unwrap().iter()
                .filter(|p| p.organization_id == org_id)
                .filter(|p| pd.map_or(true, |d| p.position_date == d))
                .cloned().collect())
        }
        async fn create_summary(&self, org_id: Uuid, cc: &str, pd: chrono::NaiveDate, ob: &str, ti: &str, to_: &str, cb: &str, lb: &str, ab: &str, ha: &str, ac: i32, accs: serde_json::Value) -> AtlasResult<CashPositionSummary> {
            Ok(CashPositionSummary {
                id: Uuid::new_v4(), organization_id: org_id, currency_code: cc.into(), position_date: pd,
                total_opening_balance: ob.into(), total_inflows: ti.into(), total_outflows: to_.into(),
                total_closing_balance: cb.into(), total_ledger_balance: lb.into(),
                total_available_balance: ab.into(), total_hold_amount: ha.into(),
                account_count: ac, accounts: accs, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn get_latest_summary(&self, _: Uuid, _: &str) -> AtlasResult<Option<CashPositionSummary>> { Ok(None) }
        async fn list_summaries(&self, _: Uuid, _: Option<chrono::NaiveDate>) -> AtlasResult<Vec<CashPositionSummary>> { Ok(vec![]) }
        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<CashPositionDashboard> {
            Ok(CashPositionDashboard {
                total_cash_position: "0".into(), base_currency_code: "USD".into(),
                position_by_currency: serde_json::json!([]), position_by_account: serde_json::json!([]),
                largest_account: None, accounts_with_deficit: 0, total_accounts: 0, latest_position_date: None,
            })
        }
    }

    fn eng() -> CashPositionEngine { CashPositionEngine::new(Arc::new(MockRepo::new())) }

    #[tokio::test]
    async fn test_record_position_valid() {
        let p = eng().record_position(
            Uuid::new_v4(), Uuid::new_v4(), Some("ACC-001"), Some("Operating"), "USD",
            "100000.00", "50000.00", "30000.00", "120000.00",
            "120000.00", "120000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            serde_json::json!({"ar_receipts": "50000", "ap_payments": "30000"}),
        ).await.unwrap();
        assert_eq!(p.currency_code, "USD");
        assert_eq!(p.closing_balance, "120000.00");
    }

    #[tokio::test]
    async fn test_record_position_empty_currency() {
        let r = eng().record_position(
            Uuid::new_v4(), Uuid::new_v4(), None, None, "",
            "100000.00", "0.00", "0.00", "100000.00",
            "100000.00", "100000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            serde_json::json!({}),
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_record_position_closing_mismatch() {
        let r = eng().record_position(
            Uuid::new_v4(), Uuid::new_v4(), None, None, "USD",
            "100000.00", "50000.00", "30000.00", "99999.00",
            "99999.00", "99999.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            serde_json::json!({}),
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_record_position_available_mismatch() {
        let r = eng().record_position(
            Uuid::new_v4(), Uuid::new_v4(), None, None, "USD",
            "100000.00", "0.00", "0.00", "100000.00",
            "100000.00", "90000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            serde_json::json!({}),
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_record_position_with_hold() {
        let p = eng().record_position(
            Uuid::new_v4(), Uuid::new_v4(), None, None, "USD",
            "100000.00", "0.00", "0.00", "100000.00",
            "100000.00", "90000.00", "10000.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            serde_json::json!({}),
        ).await.unwrap();
        assert_eq!(p.ledger_balance, "100000.00");
        assert_eq!(p.available_balance, "90000.00");
        assert_eq!(p.hold_amount, "10000.00");
    }

    #[tokio::test]
    async fn test_record_position_negative_hold() {
        let r = eng().record_position(
            Uuid::new_v4(), Uuid::new_v4(), None, None, "USD",
            "100000.00", "0.00", "0.00", "100000.00",
            "100000.00", "110000.00", "-10000.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            serde_json::json!({}),
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_summary_valid() {
        let e = eng();
        let org = Uuid::new_v4();
        let acc1 = Uuid::new_v4();
        let acc2 = Uuid::new_v4();

        let p1 = e.record_position(
            org, acc1, Some("A1"), Some("Checking"), "USD",
            "50000.00", "10000.00", "5000.00", "55000.00",
            "55000.00", "55000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            serde_json::json!({}),
        ).await.unwrap();

        let p2 = e.record_position(
            org, acc2, Some("A2"), Some("Savings"), "USD",
            "30000.00", "5000.00", "2000.00", "33000.00",
            "33000.00", "33000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            serde_json::json!({}),
        ).await.unwrap();

        let summary = e.create_summary(
            org, "USD", chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            &[p1, p2],
        ).await.unwrap();

        assert_eq!(summary.currency_code, "USD");
        assert_eq!(summary.account_count, 2);
        assert_eq!(summary.total_opening_balance, "80000.00");
        assert_eq!(summary.total_closing_balance, "88000.00");
        assert_eq!(summary.total_inflows, "15000.00");
        assert_eq!(summary.total_outflows, "7000.00");
    }

    #[tokio::test]
    async fn test_create_summary_no_positions() {
        let r = eng().create_summary(
            Uuid::new_v4(), "EUR", chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            &[],
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_summary_empty_currency() {
        let r = eng().create_summary(
            Uuid::new_v4(), "", chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            &[],
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_get_position_not_found() {
        let r = eng().get_position(Uuid::new_v4()).await.unwrap();
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn test_get_latest_position_not_found() {
        let r = eng().get_latest_position(Uuid::new_v4(), Uuid::new_v4()).await.unwrap();
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn test_list_positions_empty() {
        let r = eng().list_positions(Uuid::new_v4(), None, None).await.unwrap();
        assert!(r.is_empty());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let d = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(d.total_cash_position, "0");
        assert_eq!(d.accounts_with_deficit, 0);
    }

    #[tokio::test]
    async fn test_get_latest_summary_empty() {
        let r = eng().get_latest_summary(Uuid::new_v4(), "USD").await.unwrap();
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn test_list_summaries_empty() {
        let r = eng().list_summaries(Uuid::new_v4(), None).await.unwrap();
        assert!(r.is_empty());
    }

    #[tokio::test]
    async fn test_record_position_inflows_and_outflows() {
        let p = eng().record_position(
            Uuid::new_v4(), Uuid::new_v4(), Some("OPS"), Some("Operating"), "USD",
            "250000.00", "75000.00", "125000.00", "200000.00",
            "200000.00", "195000.00", "5000.00",
            chrono::NaiveDate::from_ymd_opt(2026, 4, 15).unwrap(),
            serde_json::json!({"ar": "75000", "ap": "125000"}),
        ).await.unwrap();
        assert_eq!(p.opening_balance, "250000.00");
        assert_eq!(p.total_inflows, "75000.00");
        assert_eq!(p.total_outflows, "125000.00");
        assert_eq!(p.closing_balance, "200000.00");
    }
}
