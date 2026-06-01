use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPostCriteriaSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPostCriteria {
    pub id: Uuid,
    pub criteria_set_id: Uuid,
    pub ledger_id: Option<Uuid>,
    pub journal_source_id: Option<Uuid>,
    pub journal_category_id: Option<Uuid>,
    pub num_days_before: i32,
    pub num_days_after: i32,
    pub is_active: bool,
}

pub struct AutoPostCriteriaService {
    sets: Arc<RwLock<Vec<AutoPostCriteriaSet>>>,
    criteria: Arc<RwLock<Vec<AutoPostCriteria>>>,
}

impl Default for AutoPostCriteriaService {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoPostCriteriaService {
    pub fn new() -> Self {
        Self {
            sets: Arc::new(RwLock::new(Vec::new())),
            criteria: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_criteria_set(
        &self,
        organization_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<AutoPostCriteriaSet, String> {
        let set = AutoPostCriteriaSet {
            id: Uuid::new_v4(),
            organization_id,
            name: name.clone(),
            description,
            is_active: true,
        };

        let mut sets = self.sets.write().unwrap();
        if sets
            .iter()
            .any(|s| s.organization_id == organization_id && s.name == name)
        {
            return Err("AutoPost criteria set with this name already exists".to_string());
        }

        sets.push(set.clone());
        Ok(set)
    }

    pub fn add_criteria(
        &self,
        criteria_set_id: Uuid,
        ledger_id: Option<Uuid>,
        journal_source_id: Option<Uuid>,
        journal_category_id: Option<Uuid>,
        num_days_before: i32,
        num_days_after: i32,
    ) -> Result<AutoPostCriteria, String> {
        // Validate set exists
        {
            let sets = self.sets.read().unwrap();
            if !sets.iter().any(|s| s.id == criteria_set_id) {
                return Err("Criteria set not found".to_string());
            }
        }

        let crit = AutoPostCriteria {
            id: Uuid::new_v4(),
            criteria_set_id,
            ledger_id,
            journal_source_id,
            journal_category_id,
            num_days_before,
            num_days_after,
            is_active: true,
        };

        let mut criteria = self.criteria.write().unwrap();
        criteria.push(crit.clone());
        Ok(crit)
    }

    pub fn evaluate_journal(
        &self,
        organization_id: Uuid,
        ledger_id: Uuid,
        journal_source_id: Uuid,
        journal_category_id: Uuid,
        accounting_date: NaiveDate,
        current_date: NaiveDate,
    ) -> bool {
        let sets = self.sets.read().unwrap();
        let active_sets: Vec<Uuid> = sets
            .iter()
            .filter(|s| s.organization_id == organization_id && s.is_active)
            .map(|s| s.id)
            .collect();

        if active_sets.is_empty() {
            return false;
        }

        let criteria_list = self.criteria.read().unwrap();

        for crit in criteria_list.iter() {
            if crit.is_active && active_sets.contains(&crit.criteria_set_id) {
                // Match dimensions (None means "All")
                let match_ledger = crit.ledger_id.is_none_or(|id| id == ledger_id);
                let match_source = crit
                    .journal_source_id
                    .is_none_or(|id| id == journal_source_id);
                let match_category = crit
                    .journal_category_id
                    .is_none_or(|id| id == journal_category_id);

                if match_ledger && match_source && match_category {
                    // Check dates
                    let diff_days = (accounting_date - current_date).num_days() as i32;
                    if diff_days >= -crit.num_days_before && diff_days <= crit.num_days_after {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_criteria_set() {
        let service = AutoPostCriteriaService::new();
        let org_id = Uuid::new_v4();

        let result = service.create_criteria_set(org_id, "Daily End of Day".to_string(), None);

        assert!(result.is_ok());
        let set = result.unwrap();
        assert_eq!(set.name, "Daily End of Day");
    }

    #[test]
    fn test_create_duplicate_criteria_set() {
        let service = AutoPostCriteriaService::new();
        let org_id = Uuid::new_v4();

        service
            .create_criteria_set(org_id, "Daily Post".to_string(), None)
            .unwrap();

        let result = service.create_criteria_set(org_id, "Daily Post".to_string(), None);

        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_journal_matches() {
        let service = AutoPostCriteriaService::new();
        let org_id = Uuid::new_v4();
        let ledger_id = Uuid::new_v4();
        let source_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();

        let set = service
            .create_criteria_set(org_id, "Monthly Post".to_string(), None)
            .unwrap();

        // Add criteria matching specific ledger and source, any category, allowed within 5 days past and 2 days future
        service
            .add_criteria(set.id, Some(ledger_id), Some(source_id), None, 5, 2)
            .unwrap();

        let current_date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();

        // Match: exactly current date
        assert!(service.evaluate_journal(
            org_id,
            ledger_id,
            source_id,
            category_id,
            current_date,
            current_date
        ));

        // Match: 3 days ago (<= 5 days before)
        let past_date = NaiveDate::from_ymd_opt(2026, 5, 28).unwrap();
        assert!(service.evaluate_journal(
            org_id,
            ledger_id,
            source_id,
            category_id,
            past_date,
            current_date
        ));

        // No Match: 6 days ago (> 5 days before)
        let too_old = NaiveDate::from_ymd_opt(2026, 5, 25).unwrap();
        assert!(!service.evaluate_journal(
            org_id,
            ledger_id,
            source_id,
            category_id,
            too_old,
            current_date
        ));

        // No Match: different ledger
        assert!(!service.evaluate_journal(
            org_id,
            Uuid::new_v4(),
            source_id,
            category_id,
            current_date,
            current_date
        ));
    }
}
