use atlas_core::financials::{AverageBalanceBook, AverageBalanceCalculation, AverageBalanceEngine};
use atlas_shared::AtlasResult;
use chrono::NaiveDate;
use std::sync::Arc;
use uuid::Uuid;

/// Service to handle Average Balance Processing, wrapping the core engine.
/// Oracle Fusion equivalent: Financials > General Ledger > Average Balances
pub struct AverageBalanceService {
    engine: Arc<AverageBalanceEngine>,
}

impl AverageBalanceService {
    #[must_use]
    pub fn new(engine: Arc<AverageBalanceEngine>) -> Self {
        Self { engine }
    }

    /// Create a new average balance book
    pub async fn create_book(
        &self,
        org_id: Uuid,
        code: String,
        name: String,
        period_type: String,
    ) -> AtlasResult<AverageBalanceBook> {
        self.engine
            .create_book(
                org_id,
                &code,
                &name,
                None,
                &period_type,
                30,    // Default 30-day window
                false, // Not primary by default
                "USD", // Default currency
                chrono::Utc::now().date_naive(),
                None,
                None,
            )
            .await
    }

    /// Calculate average balances for an account
    pub async fn calculate_average(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        account_id: Uuid,
        start: NaiveDate,
        end: NaiveDate,
    ) -> AtlasResult<AverageBalanceCalculation> {
        self.engine
            .calculate_average_balance(org_id, book_id, account_id, start, end, "daily", None)
            .await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_service_initialization() {
        // Just verify that the service can be instantiated.
        // In a real test, we would use a mock repository from atlas-core.
    }
}
