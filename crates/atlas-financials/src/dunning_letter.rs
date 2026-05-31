use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningLetterSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub set_name: String,
    pub status: String,
    pub minimum_overdue_days: i32,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningLetterSetLine {
    pub id: Uuid,
    pub set_id: Uuid,
    pub level_number: i32,
    pub level_name: String,
    pub min_days_overdue: i32,
    pub max_days_overdue: Option<i32>,
    pub minimum_amount: Decimal,
    pub delivery_method: String, // print, email, both
    pub apply_credit_hold: bool,
}

pub struct DunningLetterSetupService {
    sets: Arc<RwLock<Vec<DunningLetterSet>>>,
    lines: Arc<RwLock<Vec<DunningLetterSetLine>>>,
}

impl DunningLetterSetupService {
    pub fn new() -> Self {
        Self {
            sets: Arc::new(RwLock::new(Vec::new())),
            lines: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_set(
        &self,
        organization_id: Uuid,
        name: String,
        min_days: i32,
        currency: String,
    ) -> Result<DunningLetterSet, String> {
        let mut sets = self.sets.write().unwrap();
        if sets.iter().any(|s| s.organization_id == organization_id && s.set_name == name) {
            return Err("Dunning letter set with this name already exists".to_string());
        }

        let set = DunningLetterSet {
            id: Uuid::new_v4(),
            organization_id,
            set_name: name,
            status: "active".to_string(),
            minimum_overdue_days: min_days,
            currency_code: currency,
        };

        sets.push(set.clone());
        Ok(set)
    }

    pub fn add_level(
        &self,
        set_id: Uuid,
        level_number: i32,
        level_name: String,
        min_days: i32,
        max_days: Option<i32>,
        min_amount: Decimal,
    ) -> Result<DunningLetterSetLine, String> {
        let mut lines = self.lines.write().unwrap();
        if lines.iter().any(|l| l.set_id == set_id && l.level_number == level_number) {
            return Err("Level number already exists in this set".to_string());
        }

        let line = DunningLetterSetLine {
            id: Uuid::new_v4(),
            set_id,
            level_number,
            level_name,
            min_days_overdue: min_days,
            max_days_overdue: max_days,
            minimum_amount: min_amount,
            delivery_method: "email".to_string(),
            apply_credit_hold: false,
        };

        lines.push(line.clone());
        Ok(line)
    }

    pub fn determine_dunning_level(&self, set_id: Uuid, days_overdue: i32, overdue_amount: Decimal) -> Option<DunningLetterSetLine> {
        let lines = self.lines.read().unwrap();
        let mut applicable_lines: Vec<&DunningLetterSetLine> = lines.iter()
            .filter(|l| l.set_id == set_id)
            .filter(|l| days_overdue >= l.min_days_overdue)
            .filter(|l| l.max_days_overdue.map_or(true, |max| days_overdue <= max))
            .filter(|l| overdue_amount >= l.minimum_amount)
            .collect();
            
        // Return highest level applicable
        applicable_lines.sort_by_key(|l| std::cmp::Reverse(l.level_number));
        applicable_lines.first().cloned().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_dunning_set() {
        let service = DunningLetterSetupService::new();
        let org_id = Uuid::new_v4();
        let set = service.create_set(org_id, "Standard".to_string(), 5, "USD".to_string()).unwrap();
        assert_eq!(set.set_name, "Standard");
    }

    #[test]
    fn test_determine_dunning_level() {
        let service = DunningLetterSetupService::new();
        let org_id = Uuid::new_v4();
        let set = service.create_set(org_id, "S".to_string(), 1, "USD".to_string()).unwrap();

        service.add_level(set.id, 1, "Friendly".to_string(), 1, Some(30), dec!(100)).unwrap();
        service.add_level(set.id, 2, "Urgent".to_string(), 31, Some(60), dec!(100)).unwrap();
        service.add_level(set.id, 3, "Final".to_string(), 61, None, dec!(500)).unwrap();

        // Level 1: 15 days, $200
        let l1 = service.determine_dunning_level(set.id, 15, dec!(200)).unwrap();
        assert_eq!(l1.level_number, 1);

        // Level 2: 45 days, $200
        let l2 = service.determine_dunning_level(set.id, 45, dec!(200)).unwrap();
        assert_eq!(l2.level_number, 2);

        // Level 3: 70 days, $1000
        let l3 = service.determine_dunning_level(set.id, 70, dec!(1000)).unwrap();
        assert_eq!(l3.level_number, 3);

        // None: 70 days but only $100 (below min for level 3, and doesn't match level 2 range)
        let none = service.determine_dunning_level(set.id, 70, dec!(100));
        assert!(none.is_none());
    }
}
