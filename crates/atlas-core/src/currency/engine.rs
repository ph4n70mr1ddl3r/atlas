//! Currency Engine Implementation
//!
//! Manages currency definitions, exchange rates, currency conversion
//! with triangulation support, and unrealized gain/loss calculation.
//!
//! Oracle Fusion Cloud ERP equivalent: General Ledger > Currency Rates Manager

use atlas_shared::{
    CurrencyDefinition, ExchangeRate, CurrencyConversionResult,
    UnrealizedGainLoss,
    AtlasError, AtlasResult,
};
use super::CurrencyRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Supported rate types
const VALID_RATE_TYPES: &[&str] = &[
    "daily", "spot", "corporate", "period_average", "period_end", "user", "fixed",
];

/// Currency engine for managing multi-currency operations
pub struct CurrencyEngine {
    repository: Arc<dyn CurrencyRepository>,
}

impl CurrencyEngine {
    pub fn new(repository: Arc<dyn CurrencyRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Currency Definition Management
    // ========================================================================

    /// Define a new currency or update an existing one
    pub async fn create_currency(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        symbol: Option<&str>,
        precision: i32,
        is_base_currency: bool,
    ) -> AtlasResult<CurrencyDefinition> {
        // Validate currency code (ISO 4217: 3 uppercase letters)
        let code_upper = code.to_uppercase();
        if code_upper.len() != 3 || !code_upper.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(AtlasError::ValidationFailed(
                "Currency code must be exactly 3 alphabetic characters (ISO 4217)".to_string(),
            ));
        }

        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency name is required".to_string(),
            ));
        }

        if !(0..=6).contains(&precision) {
            return Err(AtlasError::ValidationFailed(
                "Currency precision must be between 0 and 6".to_string(),
            ));
        }

        info!("Creating currency '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository
            .create_currency(org_id, &code_upper, name, symbol, precision, is_base_currency)
            .await
    }

    /// Get a currency by code
    pub async fn get_currency(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CurrencyDefinition>> {
        self.repository.get_currency(org_id, &code.to_uppercase()).await
    }

    /// List all active currencies for an organization
    pub async fn list_currencies(&self, org_id: Uuid) -> AtlasResult<Vec<CurrencyDefinition>> {
        self.repository.list_currencies(org_id).await
    }

    /// Get the organization's base (functional) currency
    pub async fn get_base_currency(&self, org_id: Uuid) -> AtlasResult<CurrencyDefinition> {
        self.repository
            .get_base_currency(org_id)
            .await?
            .ok_or_else(|| AtlasError::ConfigError(
                "No base currency configured for this organization".to_string(),
            ))
    }

    /// Deactivate a currency
    pub async fn delete_currency(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        // Don't allow deleting the base currency
        if let Some(currency) = self.repository.get_currency(org_id, code).await? {
            if currency.is_base_currency {
                return Err(AtlasError::ValidationFailed(
                    "Cannot deactivate the base currency".to_string(),
                ));
            }
        }
        info!("Deactivating currency '{}' for org {}", code, org_id);
        self.repository.delete_currency(org_id, code).await
    }

    // ========================================================================
    // Exchange Rate Management
    // ========================================================================

    /// Set an exchange rate (upserts if one already exists for the same date/type/pair)
    pub async fn set_exchange_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        rate: &str,
        effective_date: chrono::NaiveDate,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExchangeRate> {
        let from_upper = from_currency.to_uppercase();
        let to_upper = to_currency.to_uppercase();

        // Validate inputs
        if from_upper == to_upper {
            return Err(AtlasError::ValidationFailed(
                "from_currency and to_currency must be different".to_string(),
            ));
        }

        if !VALID_RATE_TYPES.contains(&rate_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rate_type '{}'. Must be one of: {}",
                rate_type,
                VALID_RATE_TYPES.join(", ")
            )));
        }

        let rate_value: f64 = rate.parse().map_err(|_| AtlasError::ValidationFailed(
            "Rate must be a valid number".to_string(),
        ))?;

        if rate_value <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Exchange rate must be positive".to_string(),
            ));
        }

        // Compute inverse rate
        let inverse = 1.0 / rate_value;
        // Format to 10 decimal places to preserve precision
        let inverse_str = format!("{:.10}", inverse);

        info!(
            "Setting exchange rate {} -> {} = {} (type: {}, date: {})",
            from_upper, to_upper, rate, rate_type, effective_date
        );

        self.repository
            .upsert_exchange_rate(
                org_id,
                &from_upper,
                &to_upper,
                rate_type,
                rate,
                effective_date,
                Some(&inverse_str),
                source,
                created_by,
            )
            .await
    }

    /// Get exchange rate for a specific date
    pub async fn get_exchange_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<Option<ExchangeRate>> {
        let from_upper = from_currency.to_uppercase();
        let to_upper = to_currency.to_uppercase();

        if from_upper == to_upper {
            // Same currency: rate is always 1
            return Ok(Some(ExchangeRate {
                id: Uuid::nil(),
                organization_id: org_id,
                from_currency: from_upper.clone(),
                to_currency: to_upper,
                rate_type: rate_type.to_string(),
                rate: "1".to_string(),
                effective_date,
                inverse_rate: Some("1".to_string()),
                source: Some("identity".to_string()),
                created_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }));
        }

        // Try forward lookup
        if let Some(rate) = self.repository.get_exchange_rate(
            org_id, &from_upper, &to_upper, rate_type, effective_date,
        ).await? {
            return Ok(Some(rate));
        }

        // Try reverse lookup (use inverse rate)
        if let Some(rate) = self.repository.get_exchange_rate(
            org_id, &to_upper, &from_upper, rate_type, effective_date,
        ).await? {
            // Swap and use inverse
            return Ok(Some(ExchangeRate {
                id: rate.id,
                organization_id: rate.organization_id,
                from_currency: rate.to_currency.clone(),
                to_currency: rate.from_currency.clone(),
                rate_type: rate.rate_type,
                rate: rate.inverse_rate.clone().unwrap_or_else(|| {
                    let r: f64 = rate.rate.parse().unwrap_or(1.0);
                    format!("{:.10}", 1.0 / r)
                }),
                effective_date: rate.effective_date,
                inverse_rate: Some(rate.rate.clone()),
                source: rate.source,
                created_by: rate.created_by,
                created_at: rate.created_at,
                updated_at: rate.updated_at,
            }));
        }

        Ok(None)
    }

    /// Get the most recent exchange rate on or before a given date
    pub async fn get_latest_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        on_or_before: chrono::NaiveDate,
    ) -> AtlasResult<Option<ExchangeRate>> {
        let from_upper = from_currency.to_uppercase();
        let to_upper = to_currency.to_uppercase();

        if from_upper == to_upper {
            return Ok(Some(ExchangeRate {
                id: Uuid::nil(),
                organization_id: org_id,
                from_currency: from_upper.clone(),
                to_currency: to_upper,
                rate_type: rate_type.to_string(),
                rate: "1".to_string(),
                effective_date: on_or_before,
                inverse_rate: Some("1".to_string()),
                source: Some("identity".to_string()),
                created_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }));
        }

        // Try forward
        if let Some(rate) = self.repository.get_latest_rate(
            org_id, &from_upper, &to_upper, rate_type, on_or_before,
        ).await? {
            return Ok(Some(rate));
        }

        // Try reverse
        if let Some(rate) = self.repository.get_latest_rate(
            org_id, &to_upper, &from_upper, rate_type, on_or_before,
        ).await? {
            return Ok(Some(ExchangeRate {
                id: rate.id,
                organization_id: rate.organization_id,
                from_currency: rate.to_currency.clone(),
                to_currency: rate.from_currency.clone(),
                rate_type: rate.rate_type,
                rate: rate.inverse_rate.clone().unwrap_or_else(|| {
                    let r: f64 = rate.rate.parse().unwrap_or(1.0);
                    format!("{:.10}", 1.0 / r)
                }),
                effective_date: rate.effective_date,
                inverse_rate: Some(rate.rate.clone()),
                source: rate.source,
                created_by: rate.created_by,
                created_at: rate.created_at,
                updated_at: rate.updated_at,
            }));
        }

        // Triangulation: try through the base currency
        self.triangulate_rate(org_id, &from_upper, &to_upper, rate_type, on_or_before).await
    }

    /// List exchange rates with optional filters
    pub async fn list_rates(
        &self,
        org_id: Uuid,
        from_currency: Option<&str>,
        to_currency: Option<&str>,
        rate_type: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<ExchangeRate>> {
        self.repository.list_rates(
            org_id,
            from_currency.map(|s| s.to_uppercase().leak() as &str),
            to_currency.map(|s| s.to_uppercase().leak() as &str),
            rate_type,
            effective_date,
            limit.clamp(1, 200),
            offset.max(0),
        ).await
    }

    /// Delete an exchange rate
    pub async fn delete_exchange_rate(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting exchange rate {}", id);
        self.repository.delete_exchange_rate(id).await
    }

    // ========================================================================
    // Currency Conversion
    // ========================================================================

    /// Convert an amount from one currency to another
    ///
    /// Uses the specified rate type and date. Falls back to triangulation
    /// through the base currency if no direct rate is available.
    pub async fn convert(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        amount: &str,
        rate_type: &str,
        effective_date: chrono::NaiveDate,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CurrencyConversionResult> {
        let from_upper = from_currency.to_uppercase();
        let to_upper = to_currency.to_uppercase();

        let amount_value: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;

        if amount_value < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount must be non-negative".to_string(),
            ));
        }

        // Same currency: no conversion needed
        if from_upper == to_upper {
            return Ok(CurrencyConversionResult {
                from_currency: from_upper,
                to_currency: to_upper,
                from_amount: amount.to_string(),
                to_amount: amount.to_string(),
                exchange_rate: "1".to_string(),
                rate_type: rate_type.to_string(),
                effective_date,
                gain_loss: None,
            });
        }

        // Get the exchange rate (tries forward, reverse, then triangulation)
        let rate = self.get_latest_rate(org_id, &from_upper, &to_upper, rate_type, effective_date)
            .await?
            .ok_or_else(|| AtlasError::ValidationFailed(format!(
                "No exchange rate found for {} -> {} (type: {}) on or before {}",
                from_upper, to_upper, rate_type, effective_date
            )))?;

        let rate_value: f64 = rate.rate.parse()
            .map_err(|_| AtlasError::Internal("Invalid rate stored in database".to_string()))?;

        let to_amount = amount_value * rate_value;
        let to_amount_str = format!("{:.2}", to_amount);

        // Record the conversion
        self.repository.record_conversion(
            org_id,
            entity_type,
            entity_id,
            &from_upper,
            &to_upper,
            amount,
            &to_amount_str,
            &rate.rate,
            rate_type,
            effective_date,
            None,
            None,
            None,
            created_by,
        ).await?;

        info!(
            "Converted {} {} -> {} {} (rate: {})",
            amount, from_upper, to_amount_str, to_upper, rate.rate
        );

        Ok(CurrencyConversionResult {
            from_currency: from_upper,
            to_currency: to_upper,
            from_amount: amount.to_string(),
            to_amount: to_amount_str,
            exchange_rate: rate.rate,
            rate_type: rate_type.to_string(),
            effective_date,
            gain_loss: None,
        })
    }

    // ========================================================================
    // Revaluation / Unrealized Gain-Loss
    // ========================================================================

    /// Calculate unrealized gain/loss for a foreign-currency-denominated balance
    ///
    /// Compares the original transaction rate with the current rate to
    /// determine the unrealized gain or loss if the balance were settled today.
    pub async fn calculate_unrealized_gain_loss(
        &self,
        org_id: Uuid,
        currency: &str,
        original_amount: &str,
        original_rate: &str,
        revaluation_date: chrono::NaiveDate,
        rate_type: &str,
    ) -> AtlasResult<UnrealizedGainLoss> {
        let currency_upper = currency.to_uppercase();

        // Get base currency
        let base = self.get_base_currency(org_id).await?;

        if currency_upper == base.code {
            return Err(AtlasError::ValidationFailed(
                "Cannot calculate gain/loss for base currency against itself".to_string(),
            ));
        }

        let original_amount_val: f64 = original_amount.parse()
            .map_err(|_| AtlasError::ValidationFailed("Invalid original_amount".to_string()))?;
        let original_rate_val: f64 = original_rate.parse()
            .map_err(|_| AtlasError::ValidationFailed("Invalid original_rate".to_string()))?;

        // Get the current rate
        let current_rate = self.get_latest_rate(
            org_id, &currency_upper, &base.code, rate_type, revaluation_date,
        ).await?
        .ok_or_else(|| AtlasError::ValidationFailed(format!(
            "No exchange rate found for {} -> {} on {}",
            currency_upper, base.code, revaluation_date
        )))?;

        let current_rate_val: f64 = current_rate.rate.parse()
            .map_err(|_| AtlasError::Internal("Invalid rate value".to_string()))?;

        // Calculate original base amount and revalued amount
        let original_base = original_amount_val * original_rate_val;
        let revalued_base = original_amount_val * current_rate_val;

        let gain_loss = revalued_base - original_base;
        let gain_loss_type = if gain_loss > 0.0 {
            "gain"
        } else if gain_loss < 0.0 {
            "loss"
        } else {
            "none"
        };

        Ok(UnrealizedGainLoss {
            currency: currency_upper,
            original_amount: original_amount.to_string(),
            original_rate: original_rate.to_string(),
            revalued_amount: format!("{:.2}", revalued_base),
            current_rate: current_rate.rate,
            gain_loss_amount: format!("{:.2}", gain_loss.abs()),
            gain_loss_type: gain_loss_type.to_string(),
        })
    }

    // ========================================================================
    // Bulk Rate Import
    // ========================================================================

    /// Import multiple exchange rates at once (e.g., from an external feed)
    pub async fn import_rates(
        &self,
        org_id: Uuid,
        rates: Vec<ExchangeRateImport>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ImportRatesResult> {
        let mut imported = 0i32;
        let mut failed = 0i32;
        let mut errors = Vec::new();

        for rate in &rates {
            match self.set_exchange_rate(
                org_id,
                &rate.from_currency,
                &rate.to_currency,
                &rate.rate_type,
                &rate.rate,
                rate.effective_date,
                rate.source.as_deref(),
                created_by,
            ).await {
                Ok(_) => imported += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!(
                        "{} -> {}: {}",
                        rate.from_currency, rate.to_currency, e
                    ));
                }
            }
        }

        info!("Imported {} exchange rates ({} failed) for org {}", imported, failed, org_id);

        Ok(ImportRatesResult {
            total: rates.len() as i32,
            imported,
            failed,
            errors,
        })
    }

    // ========================================================================
    // Triangulation (indirect conversion through base currency)
    // ========================================================================

    /// Try to find a rate by triangulating through the base currency
    /// e.g., GBP -> EUR when only GBP -> USD and EUR -> USD exist
    async fn triangulate_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        on_or_before: chrono::NaiveDate,
    ) -> AtlasResult<Option<ExchangeRate>> {
        // Get base currency
        let base = match self.get_base_currency(org_id).await {
            Ok(b) => b,
            Err(_) => return Ok(None), // No base currency configured, can't triangulate
        };

        // Don't triangulate if either currency IS the base
        if from_currency == base.code || to_currency == base.code {
            return Ok(None);
        }

        // Try: from -> base, then base -> to
        let from_to_base = self.repository.get_latest_rate(
            org_id, from_currency, &base.code, rate_type, on_or_before,
        ).await?;

        let base_to_to = self.repository.get_latest_rate(
            org_id, &base.code, to_currency, rate_type, on_or_before,
        ).await?;

        match (from_to_base, base_to_to) {
            (Some(r1), Some(r2)) => {
                let rate1: f64 = r1.rate.parse().unwrap_or(0.0);
                let rate2: f64 = r2.rate.parse().unwrap_or(0.0);

                if rate1 <= 0.0 || rate2 <= 0.0 {
                    return Ok(None);
                }

                // Cross rate: from -> to = from -> base * base -> to
                let cross_rate = rate1 * rate2;
                let cross_rate_str = format!("{:.10}", cross_rate);

                info!(
                    "Triangulated rate {} -> {} via {}: {} ({} * {})",
                    from_currency, to_currency, base.code, cross_rate, rate1, rate2
                );

                Ok(Some(ExchangeRate {
                    id: Uuid::nil(),
                    organization_id: org_id,
                    from_currency: from_currency.to_string(),
                    to_currency: to_currency.to_string(),
                    rate_type: rate_type.to_string(),
                    rate: cross_rate_str,
                    effective_date: on_or_before,
                    inverse_rate: Some(format!("{:.10}", 1.0 / cross_rate)),
                    source: Some(format!("triangulation:{}", base.code)),
                    created_by: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            }
            // Try reverse paths
            _ => {
                // Try from -> base via inverse of base -> from
                let base_to_from = self.repository.get_latest_rate(
                    org_id, &base.code, from_currency, rate_type, on_or_before,
                ).await.ok().flatten();

                let to_to_base = self.repository.get_latest_rate(
                    org_id, to_currency, &base.code, rate_type, on_or_before,
                ).await.ok().flatten();

                match (base_to_from, to_to_base) {
                    (Some(r1_inv), Some(r2_inv)) => {
                        let rate1_inv: f64 = r1_inv.rate.parse().unwrap_or(0.0);
                        let rate2_inv: f64 = r2_inv.rate.parse().unwrap_or(0.0);

                        if rate1_inv <= 0.0 || rate2_inv <= 0.0 {
                            return Ok(None);
                        }

                        // from -> base = 1/(base -> from), then base -> to = 1/(to -> base)
                        // cross = (1/rate1_inv) * (1/rate2_inv)
                        let cross_rate = (1.0 / rate1_inv) * (1.0 / rate2_inv);
                        let cross_rate_str = format!("{:.10}", cross_rate);

                        Ok(Some(ExchangeRate {
                            id: Uuid::nil(),
                            organization_id: org_id,
                            from_currency: from_currency.to_string(),
                            to_currency: to_currency.to_string(),
                            rate_type: rate_type.to_string(),
                            rate: cross_rate_str,
                            effective_date: on_or_before,
                            inverse_rate: Some(format!("{:.10}", 1.0 / cross_rate)),
                            source: Some(format!("triangulation:{}", base.code)),
                            created_by: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        }))
                    }
                    _ => Ok(None),
                }
            }
        }
    }
}

/// Import entry for bulk rate import
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExchangeRateImport {
    pub from_currency: String,
    pub to_currency: String,
    pub rate_type: String,
    pub rate: String,
    pub effective_date: chrono::NaiveDate,
    pub source: Option<String>,
}

/// Result of bulk rate import
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportRatesResult {
    pub total: i32,
    pub imported: i32,
    pub failed: i32,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rate_types() {
        assert!(VALID_RATE_TYPES.contains(&"daily"));
        assert!(VALID_RATE_TYPES.contains(&"spot"));
        assert!(VALID_RATE_TYPES.contains(&"corporate"));
        assert!(VALID_RATE_TYPES.contains(&"period_average"));
        assert!(VALID_RATE_TYPES.contains(&"period_end"));
        assert!(VALID_RATE_TYPES.contains(&"user"));
        assert!(VALID_RATE_TYPES.contains(&"fixed"));
    }

    #[test]
    fn test_rate_type_count() {
        assert_eq!(VALID_RATE_TYPES.len(), 7);
    }
}
