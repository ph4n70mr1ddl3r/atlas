use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevaluationEvent {
    pub id: Uuid,
    pub book_type_code: String,
    pub description: Option<String>,
    pub revaluation_date: chrono::NaiveDate,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevaluationLine {
    pub id: Uuid,
    pub revaluation_id: Uuid,
    pub asset_id: Uuid,
    pub revaluation_rate: Option<Decimal>,
    pub fair_value: Option<Decimal>,
    pub old_cost: Decimal,
    pub new_cost: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRevaluationRequest {
    pub book_type_code: String,
    pub description: Option<String>,
    pub revaluation_date: chrono::NaiveDate,
    pub asset_ids: Vec<Uuid>,
    pub rate: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRevaluationResult {
    pub revaluation_id: Uuid,
    pub status: String,
    pub processed_assets: usize,
    pub message: String,
}

pub struct AssetRevaluationService {
    // In a real app, this would be a database pool.
    // We mock it for the sake of tests.
    events: Arc<RwLock<Vec<RevaluationEvent>>>,
    lines: Arc<RwLock<Vec<RevaluationLine>>>,
}

impl Default for AssetRevaluationService {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetRevaluationService {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            lines: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_revaluation(
        &self,
        request: AssetRevaluationRequest,
    ) -> Result<AssetRevaluationResult, String> {
        if request.asset_ids.is_empty() {
            return Err("At least one asset must be specified".to_string());
        }

        if request.rate.is_none() {
            return Err("Revaluation rate must be provided".to_string());
        }

        let revaluation_id = Uuid::new_v4();
        let rate = request.rate.unwrap();

        let event = RevaluationEvent {
            id: revaluation_id,
            book_type_code: request.book_type_code,
            description: request.description,
            revaluation_date: request.revaluation_date,
            status: "COMPLETED".to_string(),
        };

        let mut new_lines = Vec::new();
        let processed_assets = request.asset_ids.len();

        for asset_id in request.asset_ids {
            // Mocking old cost retrieval
            let old_cost = Decimal::new(10000, 2); // 100.00
            let new_cost = old_cost * rate;

            new_lines.push(RevaluationLine {
                id: Uuid::new_v4(),
                revaluation_id,
                asset_id,
                revaluation_rate: Some(rate),
                fair_value: None,
                old_cost,
                new_cost,
            });
        }

        let mut events_lock = self.events.write().unwrap();
        events_lock.push(event);

        let mut lines_lock = self.lines.write().unwrap();
        lines_lock.extend(new_lines);

        Ok(AssetRevaluationResult {
            revaluation_id,
            status: "COMPLETED".to_string(),
            processed_assets,
            message: "Revaluation completed successfully".to_string(),
        })
    }

    pub fn get_revaluation(&self, id: Uuid) -> Option<RevaluationEvent> {
        let events = self.events.read().unwrap();
        events.iter().find(|e| e.id == id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_asset_revaluation() {
        let service = AssetRevaluationService::new();
        
        let req = AssetRevaluationRequest {
            book_type_code: "CORP_BOOK".to_string(),
            description: Some("Year end revaluation".to_string()),
            revaluation_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            asset_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            rate: Some(dec!(1.05)),
        };

        let result = service.create_revaluation(req);
        assert!(result.is_ok());
        
        let res = result.unwrap();
        assert_eq!(res.status, "COMPLETED");
        assert_eq!(res.processed_assets, 2);

        let event = service.get_revaluation(res.revaluation_id);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.book_type_code, "CORP_BOOK");
    }

    #[test]
    fn test_asset_revaluation_missing_rate() {
        let service = AssetRevaluationService::new();
        
        let req = AssetRevaluationRequest {
            book_type_code: "CORP_BOOK".to_string(),
            description: Some("Invalid revaluation".to_string()),
            revaluation_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            asset_ids: vec![Uuid::new_v4()],
            rate: None,
        };

        let result = service.create_revaluation(req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Revaluation rate must be provided");
    }
}
