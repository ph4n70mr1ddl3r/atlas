use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub chart_of_accounts_id: String,
    pub accounting_calendar: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSetAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ledger_set_id: Uuid,
    pub ledger_id: Uuid,
    pub is_active: bool,
}

pub struct LedgerSetService {
    sets: Arc<RwLock<Vec<LedgerSet>>>,
    assignments: Arc<RwLock<Vec<LedgerSetAssignment>>>,
}

impl Default for LedgerSetService {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgerSetService {
    pub fn new() -> Self {
        Self {
            sets: Arc::new(RwLock::new(Vec::new())),
            assignments: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_ledger_set(
        &self,
        organization_id: Uuid,
        name: String,
        description: Option<String>,
        chart_of_accounts_id: String,
        accounting_calendar: String,
    ) -> Result<LedgerSet, String> {
        let id = Uuid::new_v4();
        let set = LedgerSet {
            id,
            organization_id,
            name: name.clone(),
            description,
            chart_of_accounts_id,
            accounting_calendar,
            is_active: true,
        };

        let mut sets = self.sets.write().unwrap();
        // Check uniqueness
        if sets.iter().any(|s| s.organization_id == organization_id && s.name == name) {
            return Err("Ledger set with this name already exists for the organization".to_string());
        }

        sets.push(set.clone());
        Ok(set)
    }

    pub fn assign_ledger(
        &self,
        organization_id: Uuid,
        ledger_set_id: Uuid,
        ledger_id: Uuid,
    ) -> Result<LedgerSetAssignment, String> {
        // Assume ledger has matching COA and Calendar (validation omitted for brevity/mock)
        let assignment = LedgerSetAssignment {
            id: Uuid::new_v4(),
            organization_id,
            ledger_set_id,
            ledger_id,
            is_active: true,
        };

        let mut assignments = self.assignments.write().unwrap();
        if assignments.iter().any(|a| a.ledger_set_id == ledger_set_id && a.ledger_id == ledger_id) {
            return Err("Ledger is already assigned to this ledger set".to_string());
        }

        assignments.push(assignment.clone());
        Ok(assignment)
    }

    pub fn get_assigned_ledgers(&self, ledger_set_id: Uuid) -> Vec<Uuid> {
        let assignments = self.assignments.read().unwrap();
        assignments
            .iter()
            .filter(|a| a.ledger_set_id == ledger_set_id && a.is_active)
            .map(|a| a.ledger_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ledger_set() {
        let service = LedgerSetService::new();
        let org_id = Uuid::new_v4();
        
        let result = service.create_ledger_set(
            org_id,
            "US Ledgers".to_string(),
            Some("All US Ledgers".to_string()),
            "US_COA".to_string(),
            "Standard_Monthly".to_string(),
        );

        assert!(result.is_ok());
        let set = result.unwrap();
        assert_eq!(set.name, "US Ledgers");
    }

    #[test]
    fn test_create_duplicate_ledger_set() {
        let service = LedgerSetService::new();
        let org_id = Uuid::new_v4();
        
        service.create_ledger_set(
            org_id,
            "US Ledgers".to_string(),
            None,
            "US_COA".to_string(),
            "Standard_Monthly".to_string(),
        ).unwrap();

        let result = service.create_ledger_set(
            org_id,
            "US Ledgers".to_string(),
            None,
            "US_COA".to_string(),
            "Standard_Monthly".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_assign_ledger() {
        let service = LedgerSetService::new();
        let org_id = Uuid::new_v4();
        
        let set = service.create_ledger_set(
            org_id,
            "EU Ledgers".to_string(),
            None,
            "EU_COA".to_string(),
            "Standard_Monthly".to_string(),
        ).unwrap();

        let ledger_id = Uuid::new_v4();
        let assign_result = service.assign_ledger(org_id, set.id, ledger_id);
        
        assert!(assign_result.is_ok());

        let assigned = service.get_assigned_ledgers(set.id);
        assert_eq!(assigned.len(), 1);
        assert_eq!(assigned[0], ledger_id);
    }
}
