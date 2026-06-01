use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub set_code: String,
    pub set_name: String,
    pub description: Option<String>,
    pub distribution_type: String, // percentage, amount
    pub total_percentage: Decimal,
    pub total_amount: Option<Decimal>,
    pub status: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSetLine {
    pub id: Uuid,
    pub distribution_set_id: Uuid,
    pub line_number: i32,
    pub account_combination: String,
    pub percentage: Decimal,
    pub amount: Option<Decimal>,
    pub is_active: bool,
}

pub struct DistributionSetService {
    sets: Arc<RwLock<Vec<DistributionSet>>>,
    lines: Arc<RwLock<Vec<DistributionSetLine>>>,
}

impl Default for DistributionSetService {
    fn default() -> Self {
        Self::new()
    }
}

impl DistributionSetService {
    pub fn new() -> Self {
        Self {
            sets: Arc::new(RwLock::new(Vec::new())),
            lines: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_distribution_set(
        &self,
        organization_id: Uuid,
        set_code: String,
        set_name: String,
        description: Option<String>,
        distribution_type: String,
    ) -> Result<DistributionSet, String> {
        if distribution_type != "percentage" && distribution_type != "amount" {
            return Err("Invalid distribution type. Must be percentage or amount".to_string());
        }

        let mut sets = self.sets.write().unwrap();
        if sets
            .iter()
            .any(|s| s.organization_id == organization_id && s.set_code == set_code)
        {
            return Err("Distribution set with this code already exists".to_string());
        }

        let set = DistributionSet {
            id: Uuid::new_v4(),
            organization_id,
            set_code,
            set_name,
            description,
            distribution_type,
            total_percentage: Decimal::ZERO,
            total_amount: None,
            status: "active".to_string(),
            is_default: false,
        };

        sets.push(set.clone());
        Ok(set)
    }

    pub fn add_line(
        &self,
        distribution_set_id: Uuid,
        account_combination: String,
        percentage: Decimal,
        amount: Option<Decimal>,
    ) -> Result<DistributionSetLine, String> {
        let mut sets = self.sets.write().unwrap();
        let set = sets
            .iter_mut()
            .find(|s| s.id == distribution_set_id)
            .ok_or_else(|| "Distribution set not found".to_string())?;

        let mut lines = self.lines.write().unwrap();
        let line_number = (lines
            .iter()
            .filter(|l| l.distribution_set_id == distribution_set_id)
            .count() as i32)
            + 1;

        let line = DistributionSetLine {
            id: Uuid::new_v4(),
            distribution_set_id,
            line_number,
            account_combination,
            percentage,
            amount,
            is_active: true,
        };

        if set.distribution_type == "percentage" {
            set.total_percentage += percentage;
            if set.total_percentage > Decimal::new(100, 0) {
                // In some cases Oracle allows > 100% for over-distributions, but we'll just track it
            }
        } else if let Some(amt) = amount {
            set.total_amount = Some(set.total_amount.unwrap_or(Decimal::ZERO) + amt);
        }

        lines.push(line.clone());
        Ok(line)
    }

    pub fn apply_to_amount(
        &self,
        set_id: Uuid,
        total_amount: Decimal,
    ) -> Result<Vec<(String, Decimal)>, String> {
        let sets = self.sets.read().unwrap();
        let set = sets
            .iter()
            .find(|s| s.id == set_id)
            .ok_or_else(|| "Distribution set not found".to_string())?;

        if set.status != "active" {
            return Err("Distribution set is not active".to_string());
        }

        let lines = self.lines.read().unwrap();
        let set_lines: Vec<&DistributionSetLine> = lines
            .iter()
            .filter(|l| l.distribution_set_id == set_id && l.is_active)
            .collect();

        let mut distributions = Vec::new();
        if set.distribution_type == "percentage" {
            for line in set_lines {
                let amount = (total_amount * line.percentage) / Decimal::new(100, 0);
                distributions.push((line.account_combination.clone(), amount));
            }
        } else {
            for line in set_lines {
                if let Some(amt) = line.amount {
                    distributions.push((line.account_combination.clone(), amt));
                }
            }
        }

        Ok(distributions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_distribution_set() {
        let service = DistributionSetService::new();
        let org_id = Uuid::new_v4();

        let result = service.create_distribution_set(
            org_id,
            "RENT_DIST".to_string(),
            "Rent Distribution".to_string(),
            None,
            "percentage".to_string(),
        );

        assert!(result.is_ok());
        let set = result.unwrap();
        assert_eq!(set.set_code, "RENT_DIST");
    }

    #[test]
    fn test_apply_percentage_set() {
        let service = DistributionSetService::new();
        let org_id = Uuid::new_v4();

        let set = service
            .create_distribution_set(
                org_id,
                "CORP_EXP".to_string(),
                "Corp Expenses".to_string(),
                None,
                "percentage".to_string(),
            )
            .unwrap();

        service
            .add_line(set.id, "6010-001".to_string(), dec!(60), None)
            .unwrap();
        service
            .add_line(set.id, "6010-002".to_string(), dec!(40), None)
            .unwrap();

        let total = dec!(1000);
        let dists = service.apply_to_amount(set.id, total).unwrap();

        assert_eq!(dists.len(), 2);
        assert_eq!(dists[0], ("6010-001".to_string(), dec!(600)));
        assert_eq!(dists[1], ("6010-002".to_string(), dec!(400)));
    }

    #[test]
    fn test_duplicate_code() {
        let service = DistributionSetService::new();
        let org_id = Uuid::new_v4();

        service
            .create_distribution_set(
                org_id,
                "S1".to_string(),
                "N".to_string(),
                None,
                "percentage".to_string(),
            )
            .unwrap();
        let result = service.create_distribution_set(
            org_id,
            "S1".to_string(),
            "N2".to_string(),
            None,
            "percentage".to_string(),
        );

        assert!(result.is_err());
    }
}
