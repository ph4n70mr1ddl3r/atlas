use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub chart_of_accounts_id: String,
    pub accounting_calendar: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessLevel {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessSetDetail {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub data_access_set_id: Uuid,
    pub ledger_id: Option<Uuid>,
    pub ledger_set_id: Option<Uuid>,
    pub access_level: AccessLevel,
    pub all_segment_values: bool,
    pub specific_segment_value: Option<String>,
    pub is_active: bool,
}

pub struct DataAccessSetService {
    sets: Arc<RwLock<Vec<DataAccessSet>>>,
    details: Arc<RwLock<Vec<DataAccessSetDetail>>>,
}

impl Default for DataAccessSetService {
    fn default() -> Self {
        Self {
            sets: Arc::new(RwLock::new(Vec::new())),
            details: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl DataAccessSetService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_data_access_set(
        &self,
        organization_id: Uuid,
        name: String,
        description: Option<String>,
        chart_of_accounts_id: String,
        accounting_calendar: String,
    ) -> Result<DataAccessSet, String> {
        let mut sets = self.sets.write().unwrap();

        if sets
            .iter()
            .any(|s| s.organization_id == organization_id && s.name == name)
        {
            return Err(
                "Data access set with this name already exists for the organization".to_string(),
            );
        }

        let set = DataAccessSet {
            id: Uuid::new_v4(),
            organization_id,
            name,
            description,
            chart_of_accounts_id,
            accounting_calendar,
            is_active: true,
        };

        sets.push(set.clone());
        Ok(set)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_access_detail(
        &self,
        organization_id: Uuid,
        data_access_set_id: Uuid,
        ledger_id: Option<Uuid>,
        ledger_set_id: Option<Uuid>,
        access_level: AccessLevel,
        all_segment_values: bool,
        specific_segment_value: Option<String>,
    ) -> Result<DataAccessSetDetail, String> {
        if ledger_id.is_none() && ledger_set_id.is_none() {
            return Err("Must specify either ledger_id or ledger_set_id".to_string());
        }

        if !all_segment_values && specific_segment_value.is_none() {
            return Err(
                "Must provide specific_segment_value when all_segment_values is false".to_string(),
            );
        }

        let sets = self.sets.read().unwrap();
        if !sets.iter().any(|s| s.id == data_access_set_id) {
            return Err("Data access set not found".to_string());
        }

        let mut details = self.details.write().unwrap();
        let detail = DataAccessSetDetail {
            id: Uuid::new_v4(),
            organization_id,
            data_access_set_id,
            ledger_id,
            ledger_set_id,
            access_level,
            all_segment_values,
            specific_segment_value,
            is_active: true,
        };

        details.push(detail.clone());
        Ok(detail)
    }

    pub fn has_ledger_access(
        &self,
        data_access_set_id: Uuid,
        target_ledger_id: Uuid,
        target_segment_value: Option<&str>,
        require_write: bool,
    ) -> bool {
        let details = self.details.read().unwrap();
        for detail in details
            .iter()
            .filter(|d| d.data_access_set_id == data_access_set_id && d.is_active)
        {
            // Note: For full accuracy, ledger_set_id would need to resolve to ledger_ids.
            // In this implementation, we simply check ledger_id direct matches.
            if detail.ledger_id == Some(target_ledger_id) {
                let level_ok = match (&detail.access_level, require_write) {
                    (AccessLevel::ReadWrite, _) => true,
                    (AccessLevel::ReadOnly, false) => true,
                    (AccessLevel::ReadOnly, true) => false,
                };

                let value_ok = if detail.all_segment_values {
                    true
                } else {
                    target_segment_value.is_some()
                        && detail.specific_segment_value.as_deref() == target_segment_value
                };

                if level_ok && value_ok {
                    return true;
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
    fn test_create_data_access_set() {
        let service = DataAccessSetService::new();
        let org_id = Uuid::new_v4();

        let result = service.create_data_access_set(
            org_id,
            "US Primary Set".to_string(),
            None,
            "US_COA".to_string(),
            "Standard_Monthly".to_string(),
        );

        assert!(result.is_ok());
        let set = result.unwrap();
        assert_eq!(set.name, "US Primary Set");
    }

    #[test]
    fn test_duplicate_data_access_set() {
        let service = DataAccessSetService::new();
        let org_id = Uuid::new_v4();

        service
            .create_data_access_set(
                org_id,
                "Set A".to_string(),
                None,
                "COA".to_string(),
                "CAL".to_string(),
            )
            .unwrap();

        let result = service.create_data_access_set(
            org_id,
            "Set A".to_string(),
            None,
            "COA".to_string(),
            "CAL".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_add_access_detail_validation() {
        let service = DataAccessSetService::new();
        let org_id = Uuid::new_v4();

        let set = service
            .create_data_access_set(
                org_id,
                "Set".to_string(),
                None,
                "COA".to_string(),
                "CAL".to_string(),
            )
            .unwrap();

        // Error: neither ledger nor ledger_set
        let result = service.add_access_detail(
            org_id,
            set.id,
            None,
            None,
            AccessLevel::ReadOnly,
            true,
            None,
        );
        assert!(result.is_err());

        // Error: specific segment value missing
        let result = service.add_access_detail(
            org_id,
            set.id,
            Some(Uuid::new_v4()),
            None,
            AccessLevel::ReadOnly,
            false,
            None,
        );
        assert!(result.is_err());

        // Success
        let result = service.add_access_detail(
            org_id,
            set.id,
            Some(Uuid::new_v4()),
            None,
            AccessLevel::ReadOnly,
            false,
            Some("101".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_ledger_access() {
        let service = DataAccessSetService::new();
        let org_id = Uuid::new_v4();
        let ledger_id = Uuid::new_v4();

        let set = service
            .create_data_access_set(
                org_id,
                "Set".to_string(),
                None,
                "COA".to_string(),
                "CAL".to_string(),
            )
            .unwrap();

        service
            .add_access_detail(
                org_id,
                set.id,
                Some(ledger_id),
                None,
                AccessLevel::ReadOnly, // Only read
                false,
                Some("101".to_string()),
            )
            .unwrap();

        // Read access to correct segment -> true
        assert!(service.has_ledger_access(set.id, ledger_id, Some("101"), false));

        // Write access to correct segment -> false (it is read only)
        assert!(!service.has_ledger_access(set.id, ledger_id, Some("101"), true));

        // Read access to wrong segment -> false
        assert!(!service.has_ledger_access(set.id, ledger_id, Some("102"), false));

        // Read access to wrong ledger -> false
        assert!(!service.has_ledger_access(set.id, Uuid::new_v4(), Some("101"), false));
    }
}
