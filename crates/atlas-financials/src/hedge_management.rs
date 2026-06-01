use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivativeInstrument {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub instrument_number: String,
    pub instrument_type: String, // forward, swap, option
    pub underlying_type: String, // fx, interest_rate, commodity
    pub notional_amount: Decimal,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgeRelationship {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hedge_id: String,
    pub hedge_type: String, // fair_value, cash_flow, net_investment
    pub derivative_id: Uuid,
    pub hedged_risk: String,          // fx_risk, interest_rate_risk
    pub effectiveness_method: String, // dollar_offset, regression
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessTest {
    pub id: Uuid,
    pub hedge_relationship_id: Uuid,
    pub test_date: NaiveDate,
    pub derivative_fair_value_change: Decimal,
    pub hedged_item_fair_value_change: Decimal,
    pub effectiveness_result: String, // effective, ineffective
}

pub struct HedgeManagementService {
    instruments: Arc<RwLock<Vec<DerivativeInstrument>>>,
    relationships: Arc<RwLock<Vec<HedgeRelationship>>>,
    tests: Arc<RwLock<Vec<EffectivenessTest>>>,
}

impl Default for HedgeManagementService {
    fn default() -> Self {
        Self {
            instruments: Arc::new(RwLock::new(Vec::new())),
            relationships: Arc::new(RwLock::new(Vec::new())),
            tests: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl HedgeManagementService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_instrument(
        &self,
        organization_id: Uuid,
        number: String,
        instrument_type: String,
        underlying: String,
        amount: Decimal,
    ) -> Result<DerivativeInstrument, String> {
        let mut instruments = self.instruments.write().unwrap();
        if instruments
            .iter()
            .any(|i| i.organization_id == organization_id && i.instrument_number == number)
        {
            return Err("Instrument with this number already exists".to_string());
        }

        let inst = DerivativeInstrument {
            id: Uuid::new_v4(),
            organization_id,
            instrument_number: number,
            instrument_type,
            underlying_type: underlying,
            notional_amount: amount,
            status: "active".to_string(),
        };

        instruments.push(inst.clone());
        Ok(inst)
    }

    pub fn create_relationship(
        &self,
        organization_id: Uuid,
        hedge_id: String,
        hedge_type: String,
        derivative_id: Uuid,
        hedged_risk: String,
    ) -> Result<HedgeRelationship, String> {
        let instruments = self.instruments.read().unwrap();
        if !instruments.iter().any(|i| i.id == derivative_id) {
            return Err("Derivative instrument not found".to_string());
        }

        let mut relationships = self.relationships.write().unwrap();
        if relationships
            .iter()
            .any(|r| r.organization_id == organization_id && r.hedge_id == hedge_id)
        {
            return Err("Hedge relationship with this ID already exists".to_string());
        }

        let rel = HedgeRelationship {
            id: Uuid::new_v4(),
            organization_id,
            hedge_id,
            hedge_type,
            derivative_id,
            hedged_risk,
            effectiveness_method: "dollar_offset".to_string(),
            status: "active".to_string(),
        };

        relationships.push(rel.clone());
        Ok(rel)
    }

    pub fn perform_dollar_offset_test(
        &self,
        relationship_id: Uuid,
        date: NaiveDate,
        deriv_change: Decimal,
        item_change: Decimal,
    ) -> Result<EffectivenessTest, String> {
        let relationships = self.relationships.read().unwrap();
        if !relationships.iter().any(|r| r.id == relationship_id) {
            return Err("Relationship not found".to_string());
        }

        // Dollar Offset Ratio = (Change in Derivative FV) / (Change in Hedged Item FV)
        // Highly effective if between 80% and 125% (0.8 to 1.25)

        let mut result = "ineffective".to_string();
        if !item_change.is_zero() {
            let ratio = (deriv_change / item_change).abs();
            if ratio >= Decimal::new(8, 1) && ratio <= Decimal::new(125, 2) {
                result = "effective".to_string();
            }
        }

        let test = EffectivenessTest {
            id: Uuid::new_v4(),
            hedge_relationship_id: relationship_id,
            test_date: date,
            derivative_fair_value_change: deriv_change,
            hedged_item_fair_value_change: item_change,
            effectiveness_result: result,
        };

        let mut tests = self.tests.write().unwrap();
        tests.push(test.clone());
        Ok(test)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_hedge_effectiveness() {
        let service = HedgeManagementService::new();
        let org_id = Uuid::new_v4();

        let inst = service
            .create_instrument(
                org_id,
                "Fwd-001".to_string(),
                "forward".to_string(),
                "fx".to_string(),
                dec!(1000000),
            )
            .unwrap();
        let rel = service
            .create_relationship(
                org_id,
                "Hedge-1".to_string(),
                "cash_flow".to_string(),
                inst.id,
                "fx_risk".to_string(),
            )
            .unwrap();

        let today = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();

        // Effective case: 100% offset (ratio 1.0)
        let t1 = service
            .perform_dollar_offset_test(rel.id, today, dec!(-10000), dec!(10000))
            .unwrap();
        assert_eq!(t1.effectiveness_result, "effective");

        // Effective case: 90% offset (ratio 0.9)
        let t2 = service
            .perform_dollar_offset_test(rel.id, today, dec!(-9000), dec!(10000))
            .unwrap();
        assert_eq!(t2.effectiveness_result, "effective");

        // Ineffective case: 50% offset (ratio 0.5)
        let t3 = service
            .perform_dollar_offset_test(rel.id, today, dec!(-5000), dec!(10000))
            .unwrap();
        assert_eq!(t3.effectiveness_result, "ineffective");
    }
}
