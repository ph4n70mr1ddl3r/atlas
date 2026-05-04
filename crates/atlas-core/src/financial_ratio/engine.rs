//! Financial Ratio Analysis Engine
//! Oracle Fusion: Financial Reporting Center > Ratio Analysis

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_CATEGORIES: &[&str] = &[
    "liquidity", "profitability", "leverage", "efficiency", "market", "coverage",
];
const VALID_UNITS: &[&str] = &[
    "ratio", "percent", "times", "days", "currency",
];
const VALID_SNAPSHOT_STATUSES: &[&str] = &[
    "draft", "calculated", "approved", "archived",
];
#[allow(dead_code)]
const VALID_TREND_DIRECTIONS: &[&str] = &[
    "improving", "declining", "stable",
];
#[allow(dead_code)]
const VALID_STATUS_FLAGS: &[&str] = &[
    "above_benchmark", "below_benchmark", "at_benchmark", "no_benchmark", "critical", "warning", "normal",
];

pub struct FinancialRatioEngine {
    repository: Arc<dyn FinancialRatioRepository>,
}

impl FinancialRatioEngine {
    pub fn new(r: Arc<dyn FinancialRatioRepository>) -> Self { Self { repository: r } }

    // ========================================================================
    // Ratio Definitions
    // ========================================================================

    pub async fn create_definition(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        category: &str, formula: &str, numerator_accounts: serde_json::Value,
        denominator_accounts: serde_json::Value, unit: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<RatioDefinition> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Code and name are required".into()));
        }
        if !VALID_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid category '{}'. Must be one of: {}", category, VALID_CATEGORIES.join(", ")
            )));
        }
        if formula.is_empty() {
            return Err(AtlasError::ValidationFailed("Formula is required".into()));
        }
        if !VALID_UNITS.contains(&unit) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid unit '{}'. Must be one of: {}", unit, VALID_UNITS.join(", ")
            )));
        }
        if self.repository.get_definition(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Ratio '{}' already exists", code)));
        }
        info!("Creating financial ratio definition {} for org {}", code, org_id);
        self.repository.create_definition(org_id, code, name, description, category, formula, numerator_accounts, denominator_accounts, unit, created_by).await
    }

    pub async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RatioDefinition>> {
        self.repository.get_definition(org_id, code).await
    }

    pub async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<RatioDefinition>> {
        self.repository.get_definition_by_id(id).await
    }

    pub async fn list_definitions(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<RatioDefinition>> {
        if let Some(c) = category {
            if !VALID_CATEGORIES.contains(&c) {
                return Err(AtlasError::ValidationFailed(format!("Invalid category '{}'", c)));
            }
        }
        self.repository.list_definitions(org_id, category).await
    }

    pub async fn delete_definition(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.get_definition_by_id(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Definition {} not found", id)))?;
        self.repository.delete_definition(id).await
    }

    // ========================================================================
    // Ratio Computation
    // ========================================================================

    /// Compute a ratio from numerator and denominator values
    pub fn compute_ratio(&self, numerator: f64, denominator: f64, unit: &str) -> String {
        if denominator.abs() < 0.0001 {
            return "N/A".to_string();
        }
        let result = numerator / denominator;
        match unit {
            "percent" => format!("{:.2}", result * 100.0),
            "times" => format!("{:.2}x", result),
            "days" => format!("{:.0}", result),
            _ => format!("{:.4}", result),
        }
    }

    /// Determine trend direction based on current vs previous value
    pub fn determine_trend(&self, current: &str, previous: &str) -> Option<String> {
        let cur: f64 = current.parse().ok()?;
        let prev: f64 = previous.parse().ok()?;
        let diff = (cur - prev).abs();
        if diff < 0.001 {
            Some("stable".to_string())
        } else if cur > prev {
            Some("improving".to_string())
        } else {
            Some("declining".to_string())
        }
    }

    /// Compute change amount and percent between current and previous values
    pub fn compute_change(&self, current: &str, previous: &str) -> (Option<String>, Option<String>) {
        let cur: f64 = match current.parse() {
            Ok(v) => v,
            Err(_) => return (None, None),
        };
        let prev: f64 = match previous.parse() {
            Ok(v) => v,
            Err(_) => return (None, None),
        };
        let change = cur - prev;
        let change_pct = if prev.abs() > 0.0001 {
            Some(format!("{:.2}", change / prev * 100.0))
        } else {
            None
        };
        (Some(format!("{:.4}", change)), change_pct)
    }

    /// Determine status flag by comparing result to benchmark thresholds
    pub fn evaluate_status(&self, result: &str, min_acceptable: Option<&str>, max_acceptable: Option<&str>) -> String {
        let val: f64 = match result.parse() {
            Ok(v) => v,
            Err(_) => return "no_benchmark".to_string(),
        };
        match (min_acceptable, max_acceptable) {
            (Some(min), Some(max)) => {
                let min_v: f64 = min.parse().unwrap_or(f64::MIN);
                let max_v: f64 = max.parse().unwrap_or(f64::MAX);
                if val < min_v {
                    "below_benchmark".to_string()
                } else if val > max_v {
                    "above_benchmark".to_string()
                } else {
                    "at_benchmark".to_string()
                }
            }
            (Some(min), None) => {
                let min_v: f64 = min.parse().unwrap_or(f64::MIN);
                if val < min_v { "below_benchmark".to_string() } else { "at_benchmark".to_string() }
            }
            (None, Some(max)) => {
                let max_v: f64 = max.parse().unwrap_or(f64::MAX);
                if val > max_v { "above_benchmark".to_string() } else { "at_benchmark".to_string() }
            }
            (None, None) => "no_benchmark".to_string(),
        }
    }

    // ========================================================================
    // Snapshots
    // ========================================================================

    pub async fn create_snapshot(&self, org_id: Uuid, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate, currency_code: &str, created_by: Option<Uuid>) -> AtlasResult<RatioSnapshot> {
        if period_end < period_start {
            return Err(AtlasError::ValidationFailed("Period end must be after start".into()));
        }
        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        info!("Creating ratio snapshot for org {} period {} to {}", org_id, period_start, period_end);
        self.repository.create_snapshot(org_id, period_start, period_end, currency_code, created_by).await
    }

    pub async fn get_snapshot(&self, id: Uuid) -> AtlasResult<Option<RatioSnapshot>> {
        self.repository.get_snapshot(id).await
    }

    pub async fn list_snapshots(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RatioSnapshot>> {
        if let Some(s) = status {
            if !VALID_SNAPSHOT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s)));
            }
        }
        self.repository.list_snapshots(org_id, status).await
    }

    pub async fn add_ratio_result(
        &self, org_id: Uuid, snapshot_id: Uuid, ratio_id: Uuid,
        ratio_code: &str, ratio_name: &str, category: &str,
        numerator_value: &str, denominator_value: &str, result_value: &str,
        unit: &str, previous_value: Option<&str>, benchmark_value: Option<&str>,
        status_flag: Option<&str>,
    ) -> AtlasResult<RatioResult> {
        if ratio_code.is_empty() || ratio_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Ratio code and name are required".into()));
        }
        if !VALID_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!("Invalid category '{}'", category)));
        }
        let num: f64 = numerator_value.parse().map_err(|_| AtlasError::ValidationFailed("Invalid numerator".into()))?;
        let _denom: f64 = denominator_value.parse().map_err(|_| AtlasError::ValidationFailed("Invalid denominator".into()))?;
        if num < 0.0 {
            return Err(AtlasError::ValidationFailed("Numerator must be non-negative".into()));
        }

        let (change_amount, change_percent) = if let Some(prev) = previous_value {
            self.compute_change(result_value, prev)
        } else {
            (None, None)
        };

        let trend = if let Some(prev) = previous_value {
            self.determine_trend(result_value, prev)
        } else {
            None
        };

        let flag = status_flag.unwrap_or("normal").to_string();

        self.repository.create_ratio_result(
            org_id, snapshot_id, ratio_id, ratio_code, ratio_name, category,
            numerator_value, denominator_value, result_value, unit,
            previous_value, change_amount.as_deref(), change_percent.as_deref(),
            trend.as_deref(), benchmark_value, Some(&flag),
        ).await
    }

    pub async fn list_ratio_results(&self, snapshot_id: Uuid) -> AtlasResult<Vec<RatioResult>> {
        self.repository.list_ratio_results(snapshot_id).await
    }

    pub async fn list_ratio_results_by_category(&self, snapshot_id: Uuid, category: &str) -> AtlasResult<Vec<RatioResult>> {
        if !VALID_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!("Invalid category '{}'", category)));
        }
        self.repository.list_ratio_results_by_category(snapshot_id, category).await
    }

    // ========================================================================
    // Benchmarks
    // ========================================================================

    pub async fn create_benchmark(
        &self, org_id: Uuid, ratio_id: Uuid, name: &str, value: &str,
        min_acceptable: Option<&str>, max_acceptable: Option<&str>,
        industry: Option<&str>, effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<RatioBenchmark> {
        self.repository.get_definition_by_id(ratio_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Ratio {} not found", ratio_id)))?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Benchmark name is required".into()));
        }
        let v: f64 = value.parse().map_err(|_| AtlasError::ValidationFailed("Invalid benchmark value".into()))?;
        if v < 0.0 {
            return Err(AtlasError::ValidationFailed("Benchmark value must be non-negative".into()));
        }
        if let Some(to) = effective_to {
            if to < effective_from {
                return Err(AtlasError::ValidationFailed("Effective to must be after from".into()));
            }
        }
        info!("Creating benchmark '{}' for ratio {}", name, ratio_id);
        self.repository.create_benchmark(org_id, ratio_id, name, value, min_acceptable, max_acceptable, industry, effective_from, effective_to).await
    }

    pub async fn list_benchmarks(&self, org_id: Uuid, ratio_id: Option<Uuid>) -> AtlasResult<Vec<RatioBenchmark>> {
        self.repository.list_benchmarks(org_id, ratio_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RatioDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        definitions: std::sync::Mutex<Vec<RatioDefinition>>,
        snapshots: std::sync::Mutex<Vec<RatioSnapshot>>,
        results: std::sync::Mutex<Vec<RatioResult>>,
        benchmarks: std::sync::Mutex<Vec<RatioBenchmark>>,
    }
    impl MockRepo {
        fn new() -> Self {
            Self {
                definitions: std::sync::Mutex::new(vec![]),
                snapshots: std::sync::Mutex::new(vec![]),
                results: std::sync::Mutex::new(vec![]),
                benchmarks: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl FinancialRatioRepository for MockRepo {
        async fn create_definition(&self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>, cat: &str, formula: &str, num_acc: serde_json::Value, den_acc: serde_json::Value, unit: &str, cb: Option<Uuid>) -> AtlasResult<RatioDefinition> {
            let d = RatioDefinition {
                id: Uuid::new_v4(), organization_id: org_id, ratio_code: code.into(),
                name: name.into(), description: desc.map(Into::into), category: cat.into(),
                formula: formula.into(), numerator_accounts: num_acc, denominator_accounts: den_acc,
                unit: unit.into(), is_active: true, metadata: serde_json::json!({}),
                created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.definitions.lock().unwrap().push(d.clone());
            Ok(d)
        }
        async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RatioDefinition>> {
            Ok(self.definitions.lock().unwrap().iter().find(|d| d.organization_id == org_id && d.ratio_code == code).cloned())
        }
        async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<RatioDefinition>> {
            Ok(self.definitions.lock().unwrap().iter().find(|d| d.id == id).cloned())
        }
        async fn list_definitions(&self, org_id: Uuid, cat: Option<&str>) -> AtlasResult<Vec<RatioDefinition>> {
            Ok(self.definitions.lock().unwrap().iter().filter(|d| d.organization_id == org_id && (cat.is_none() || d.category == cat.unwrap())).cloned().collect())
        }
        async fn delete_definition(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
        async fn create_snapshot(&self, org_id: Uuid, ps: chrono::NaiveDate, pe: chrono::NaiveDate, cc: &str, cb: Option<Uuid>) -> AtlasResult<RatioSnapshot> {
            let s = RatioSnapshot {
                id: Uuid::new_v4(), organization_id: org_id, snapshot_date: chrono::Utc::now().date_naive(),
                period_start: ps, period_end: pe, currency_code: cc.into(), status: "calculated".into(),
                total_ratios: 0, metadata: serde_json::json!({}), created_by: cb,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.snapshots.lock().unwrap().push(s.clone());
            Ok(s)
        }
        async fn get_snapshot(&self, id: Uuid) -> AtlasResult<Option<RatioSnapshot>> {
            Ok(self.snapshots.lock().unwrap().iter().find(|s| s.id == id).cloned())
        }
        async fn list_snapshots(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RatioSnapshot>> {
            Ok(self.snapshots.lock().unwrap().iter().filter(|s| s.organization_id == org_id && (status.is_none() || s.status == status.unwrap())).cloned().collect())
        }
        async fn update_snapshot_status(&self, id: Uuid, status: &str, total: i32) -> AtlasResult<RatioSnapshot> {
            let mut ss = self.snapshots.lock().unwrap();
            let s = ss.iter_mut().find(|s| s.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            s.status = status.into();
            s.total_ratios = total;
            Ok(s.clone())
        }
        async fn create_ratio_result(&self, org_id: Uuid, sid: Uuid, rid: Uuid, rc: &str, rn: &str, cat: &str, nv: &str, dv: &str, rv: &str, unit: &str, pv: Option<&str>, ca: Option<&str>, cp: Option<&str>, td: Option<&str>, bv: Option<&str>, sf: Option<&str>) -> AtlasResult<RatioResult> {
            let r = RatioResult {
                id: Uuid::new_v4(), organization_id: org_id, snapshot_id: sid, ratio_id: rid,
                ratio_code: rc.into(), ratio_name: rn.into(), category: cat.into(),
                numerator_value: nv.into(), denominator_value: dv.into(), result_value: rv.into(),
                unit: unit.into(), previous_value: pv.map(Into::into), change_amount: ca.map(Into::into),
                change_percent: cp.map(Into::into), trend_direction: td.map(Into::into),
                benchmark_value: bv.map(Into::into), status_flag: sf.map(Into::into),
                metadata: serde_json::json!({}), created_at: chrono::Utc::now(),
            };
            self.results.lock().unwrap().push(r.clone());
            Ok(r)
        }
        async fn list_ratio_results(&self, sid: Uuid) -> AtlasResult<Vec<RatioResult>> {
            Ok(self.results.lock().unwrap().iter().filter(|r| r.snapshot_id == sid).cloned().collect())
        }
        async fn list_ratio_results_by_category(&self, sid: Uuid, cat: &str) -> AtlasResult<Vec<RatioResult>> {
            Ok(self.results.lock().unwrap().iter().filter(|r| r.snapshot_id == sid && r.category == cat).cloned().collect())
        }
        async fn create_benchmark(&self, org_id: Uuid, rid: Uuid, name: &str, val: &str, min: Option<&str>, max: Option<&str>, ind: Option<&str>, ef: chrono::NaiveDate, et: Option<chrono::NaiveDate>) -> AtlasResult<RatioBenchmark> {
            let b = RatioBenchmark {
                id: Uuid::new_v4(), organization_id: org_id, ratio_id: rid, name: name.into(),
                benchmark_value: val.into(), min_acceptable: min.map(Into::into),
                max_acceptable: max.map(Into::into), industry: ind.map(Into::into),
                effective_from: ef, effective_to: et, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.benchmarks.lock().unwrap().push(b.clone());
            Ok(b)
        }
        async fn list_benchmarks(&self, org_id: Uuid, rid: Option<Uuid>) -> AtlasResult<Vec<RatioBenchmark>> {
            Ok(self.benchmarks.lock().unwrap().iter().filter(|b| b.organization_id == org_id && (rid.is_none() || b.ratio_id == rid.unwrap())).cloned().collect())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RatioDashboard> {
            let defs = self.definitions.lock().unwrap();
            let snaps = self.snapshots.lock().unwrap();
            Ok(RatioDashboard {
                total_definitions: defs.iter().filter(|d| d.organization_id == org_id).count() as i32,
                total_snapshots: snaps.iter().filter(|s| s.organization_id == org_id).count() as i32,
                liquidity_score: None, profitability_score: None,
                leverage_score: None, efficiency_score: None,
                category_summaries: vec![],
            })
        }
    }

    fn eng() -> FinancialRatioEngine { FinancialRatioEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_CATEGORIES.len(), 6);
        assert_eq!(VALID_UNITS.len(), 5);
        assert_eq!(VALID_SNAPSHOT_STATUSES.len(), 4);
        assert_eq!(VALID_TREND_DIRECTIONS.len(), 3);
        assert_eq!(VALID_STATUS_FLAGS.len(), 7);
    }

    #[test]
    fn test_compute_ratio_basic() {
        let e = eng();
        assert_eq!(e.compute_ratio(1000.0, 500.0, "ratio"), "2.0000");
    }

    #[test]
    fn test_compute_ratio_percent() {
        let e = eng();
        assert_eq!(e.compute_ratio(250.0, 1000.0, "percent"), "25.00");
    }

    #[test]
    fn test_compute_ratio_times() {
        let e = eng();
        assert_eq!(e.compute_ratio(5000.0, 1000.0, "times"), "5.00x");
    }

    #[test]
    fn test_compute_ratio_days() {
        let e = eng();
        assert_eq!(e.compute_ratio(36500.0, 100.0, "days"), "365");
    }

    #[test]
    fn test_compute_ratio_zero_denominator() {
        let e = eng();
        assert_eq!(e.compute_ratio(1000.0, 0.0, "ratio"), "N/A");
    }

    #[test]
    fn test_determine_trend_improving() {
        assert_eq!(eng().determine_trend("2.5", "2.0"), Some("improving".to_string()));
    }

    #[test]
    fn test_determine_trend_declining() {
        assert_eq!(eng().determine_trend("1.5", "2.0"), Some("declining".to_string()));
    }

    #[test]
    fn test_determine_trend_stable() {
        assert_eq!(eng().determine_trend("2.0", "2.0"), Some("stable".to_string()));
    }

    #[test]
    fn test_determine_trend_invalid() {
        assert!(eng().determine_trend("abc", "2.0").is_none());
    }

    #[test]
    fn test_compute_change_positive() {
        let (amt, pct) = eng().compute_change("2.5", "2.0");
        assert_eq!(amt, Some("0.5000".to_string()));
        assert_eq!(pct, Some("25.00".to_string()));
    }

    #[test]
    fn test_compute_change_negative() {
        let (amt, pct) = eng().compute_change("1.5", "2.0");
        assert_eq!(amt, Some("-0.5000".to_string()));
        assert_eq!(pct, Some("-25.00".to_string()));
    }

    #[test]
    fn test_compute_change_zero_prev() {
        let (amt, pct) = eng().compute_change("2.5", "0");
        assert_eq!(amt, Some("2.5000".to_string()));
        assert!(pct.is_none());
    }

    #[test]
    fn test_compute_change_invalid() {
        let (amt, pct) = eng().compute_change("abc", "2.0");
        assert!(amt.is_none());
        assert!(pct.is_none());
    }

    #[test]
    fn test_evaluate_status_at_benchmark() {
        let e = eng();
        assert_eq!(e.evaluate_status("2.5", Some("1.0"), Some("3.0")), "at_benchmark");
    }

    #[test]
    fn test_evaluate_status_below_benchmark() {
        let e = eng();
        assert_eq!(e.evaluate_status("0.5", Some("1.0"), Some("3.0")), "below_benchmark");
    }

    #[test]
    fn test_evaluate_status_above_benchmark() {
        let e = eng();
        assert_eq!(e.evaluate_status("4.0", Some("1.0"), Some("3.0")), "above_benchmark");
    }

    #[test]
    fn test_evaluate_status_min_only() {
        assert_eq!(eng().evaluate_status("1.5", Some("1.0"), None), "at_benchmark");
        assert_eq!(eng().evaluate_status("0.5", Some("1.0"), None), "below_benchmark");
    }

    #[test]
    fn test_evaluate_status_max_only() {
        assert_eq!(eng().evaluate_status("2.5", None, Some("3.0")), "at_benchmark");
        assert_eq!(eng().evaluate_status("4.0", None, Some("3.0")), "above_benchmark");
    }

    #[test]
    fn test_evaluate_status_no_benchmark() {
        assert_eq!(eng().evaluate_status("2.5", None, None), "no_benchmark");
    }

    #[test]
    fn test_evaluate_status_invalid_value() {
        assert_eq!(eng().evaluate_status("abc", Some("1.0"), Some("3.0")), "no_benchmark");
    }

    #[tokio::test]
    async fn test_create_definition_valid() {
        let d = eng().create_definition(
            Uuid::new_v4(), "CURRENT_RATIO", "Current Ratio", Some("Liquidity metric"),
            "liquidity", "current_assets / current_liabilities",
            serde_json::json!(["1000", "1100"]), serde_json::json!(["2000", "2100"]),
            "ratio", None,
        ).await.unwrap();
        assert_eq!(d.ratio_code, "CURRENT_RATIO");
        assert_eq!(d.category, "liquidity");
        assert!(d.is_active);
    }

    #[tokio::test]
    async fn test_create_definition_empty_code() {
        assert!(eng().create_definition(
            Uuid::new_v4(), "", "Name", None, "liquidity", "a/b",
            serde_json::json!([]), serde_json::json!([]), "ratio", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_empty_name() {
        assert!(eng().create_definition(
            Uuid::new_v4(), "CODE", "", None, "liquidity", "a/b",
            serde_json::json!([]), serde_json::json!([]), "ratio", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_invalid_category() {
        assert!(eng().create_definition(
            Uuid::new_v4(), "CODE", "Name", None, "bad_cat", "a/b",
            serde_json::json!([]), serde_json::json!([]), "ratio", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_empty_formula() {
        assert!(eng().create_definition(
            Uuid::new_v4(), "CODE", "Name", None, "liquidity", "",
            serde_json::json!([]), serde_json::json!([]), "ratio", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_invalid_unit() {
        assert!(eng().create_definition(
            Uuid::new_v4(), "CODE", "Name", None, "liquidity", "a/b",
            serde_json::json!([]), serde_json::json!([]), "bad_unit", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_duplicate() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_definition(org, "DUP", "N1", None, "liquidity", "a/b", serde_json::json!([]), serde_json::json!([]), "ratio", None).await;
        assert!(e.create_definition(org, "DUP", "N2", None, "liquidity", "a/b", serde_json::json!([]), serde_json::json!([]), "ratio", None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_definitions_invalid_category() {
        assert!(eng().list_definitions(Uuid::new_v4(), Some("bad")).await.is_err());
    }

    #[tokio::test]
    async fn test_list_definitions_valid() {
        let r = eng().list_definitions(Uuid::new_v4(), Some("liquidity")).await;
        assert!(r.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_delete_definition_not_found() {
        assert!(eng().delete_definition(Uuid::new_v4()).await.is_err());
    }

    #[tokio::test]
    async fn test_create_snapshot_valid() {
        let s = eng().create_snapshot(
            Uuid::new_v4(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            "USD", None,
        ).await.unwrap();
        assert_eq!(s.status, "calculated");
    }

    #[tokio::test]
    async fn test_create_snapshot_end_before_start() {
        assert!(eng().create_snapshot(
            Uuid::new_v4(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            "USD", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_snapshot_bad_currency() {
        assert!(eng().create_snapshot(
            Uuid::new_v4(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            "US", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_list_snapshots_invalid_status() {
        assert!(eng().list_snapshots(Uuid::new_v4(), Some("bad")).await.is_err());
    }

    #[tokio::test]
    async fn test_add_ratio_result_valid() {
        let e = eng();
        let org = Uuid::new_v4();
        let s = e.create_snapshot(org, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(), "USD", None).await.unwrap();
        let r = e.add_ratio_result(org, s.id, Uuid::new_v4(), "CR", "Current Ratio", "liquidity",
            "500000.00", "250000.00", "2.0000", "ratio", Some("1.8000"), Some("2.0"), None).await.unwrap();
        assert_eq!(r.ratio_code, "CR");
        assert_eq!(r.result_value, "2.0000");
        assert_eq!(r.trend_direction, Some("improving".to_string()));
    }

    #[tokio::test]
    async fn test_add_ratio_result_empty_code() {
        let e = eng();
        let org = Uuid::new_v4();
        let s = e.create_snapshot(org, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(), "USD", None).await.unwrap();
        assert!(e.add_ratio_result(org, s.id, Uuid::new_v4(), "", "Name", "liquidity",
            "500.0", "250.0", "2.0", "ratio", None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_add_ratio_result_invalid_category() {
        let e = eng();
        let org = Uuid::new_v4();
        let s = e.create_snapshot(org, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(), "USD", None).await.unwrap();
        assert!(e.add_ratio_result(org, s.id, Uuid::new_v4(), "CR", "Name", "bad",
            "500.0", "250.0", "2.0", "ratio", None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_add_ratio_result_negative_numerator() {
        let e = eng();
        let org = Uuid::new_v4();
        let s = e.create_snapshot(org, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(), "USD", None).await.unwrap();
        assert!(e.add_ratio_result(org, s.id, Uuid::new_v4(), "CR", "Name", "liquidity",
            "-500.0", "250.0", "2.0", "ratio", None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_benchmark_valid() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "CR", "Current Ratio", None, "liquidity", "a/b", serde_json::json!([]), serde_json::json!([]), "ratio", None).await.unwrap();
        let b = e.create_benchmark(org, d.id, "Industry Avg", "2.0", Some("1.5"), Some("2.5"), Some("Manufacturing"),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None).await.unwrap();
        assert_eq!(b.name, "Industry Avg");
        assert_eq!(b.benchmark_value, "2.0");
    }

    #[tokio::test]
    async fn test_create_benchmark_ratio_not_found() {
        assert!(eng().create_benchmark(Uuid::new_v4(), Uuid::new_v4(), "Bench", "2.0", None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_benchmark_empty_name() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "CR", "Current Ratio", None, "liquidity", "a/b", serde_json::json!([]), serde_json::json!([]), "ratio", None).await.unwrap();
        assert!(e.create_benchmark(org, d.id, "", "2.0", None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_benchmark_negative_value() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "CR", "Current Ratio", None, "liquidity", "a/b", serde_json::json!([]), serde_json::json!([]), "ratio", None).await.unwrap();
        assert!(e.create_benchmark(org, d.id, "Bench", "-2.0", None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_benchmark_invalid_value() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "CR", "Current Ratio", None, "liquidity", "a/b", serde_json::json!([]), serde_json::json!([]), "ratio", None).await.unwrap();
        assert!(e.create_benchmark(org, d.id, "Bench", "abc", None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_benchmark_effective_to_before_from() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "CR", "Current Ratio", None, "liquidity", "a/b", serde_json::json!([]), serde_json::json!([]), "ratio", None).await.unwrap();
        assert!(e.create_benchmark(org, d.id, "Bench", "2.0", None, None, None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())).await.is_err());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create_definition(org, "CR", "Current Ratio", None, "liquidity", "a/b", serde_json::json!([]), serde_json::json!([]), "ratio", None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_definitions, 1);
        assert_eq!(dash.total_snapshots, 0);
    }

    #[tokio::test]
    async fn test_list_ratio_results_by_category_invalid() {
        assert!(eng().list_ratio_results_by_category(Uuid::new_v4(), "bad").await.is_err());
    }
}
