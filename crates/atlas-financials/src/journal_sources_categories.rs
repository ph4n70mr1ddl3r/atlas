use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalSource {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub import_journal_references: bool,
    pub freeze_journals: bool,
    pub require_journal_approval: bool,
    pub action_if_unbalanced: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

pub struct JournalSetupService {
    sources: Arc<RwLock<Vec<JournalSource>>>,
    categories: Arc<RwLock<Vec<JournalCategory>>>,
}

impl Default for JournalSetupService {
    fn default() -> Self {
        Self {
            sources: Arc::new(RwLock::new(Vec::new())),
            categories: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl JournalSetupService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_journal_source(
        &self,
        organization_id: Uuid,
        name: String,
        description: Option<String>,
        import_journal_references: bool,
        freeze_journals: bool,
        require_journal_approval: bool,
        action_if_unbalanced: String,
    ) -> Result<JournalSource, String> {
        let mut sources = self.sources.write().unwrap();
        
        if sources.iter().any(|s| s.organization_id == organization_id && s.name == name) {
            return Err("Journal source with this name already exists for the organization".to_string());
        }

        let valid_actions = ["error", "warning", "post_to_suspense"];
        if !valid_actions.contains(&action_if_unbalanced.as_str()) {
            return Err("Invalid action_if_unbalanced. Must be error, warning, or post_to_suspense".to_string());
        }

        let source = JournalSource {
            id: Uuid::new_v4(),
            organization_id,
            name,
            description,
            import_journal_references,
            freeze_journals,
            require_journal_approval,
            action_if_unbalanced,
            is_active: true,
        };

        sources.push(source.clone());
        Ok(source)
    }

    pub fn create_journal_category(
        &self,
        organization_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<JournalCategory, String> {
        let mut categories = self.categories.write().unwrap();
        
        if categories.iter().any(|c| c.organization_id == organization_id && c.name == name) {
            return Err("Journal category with this name already exists for the organization".to_string());
        }

        let category = JournalCategory {
            id: Uuid::new_v4(),
            organization_id,
            name,
            description,
            is_active: true,
        };

        categories.push(category.clone());
        Ok(category)
    }

    pub fn get_source_by_name(&self, organization_id: Uuid, name: &str) -> Option<JournalSource> {
        let sources = self.sources.read().unwrap();
        sources.iter().find(|s| s.organization_id == organization_id && s.name == name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_journal_source() {
        let service = JournalSetupService::new();
        let org_id = Uuid::new_v4();
        
        let result = service.create_journal_source(
            org_id,
            "Payables".to_string(),
            Some("AP Invoices".to_string()),
            true,
            false,
            true,
            "error".to_string(),
        );

        assert!(result.is_ok());
        let source = result.unwrap();
        assert_eq!(source.name, "Payables");
        assert!(source.require_journal_approval);
    }

    #[test]
    fn test_duplicate_journal_source() {
        let service = JournalSetupService::new();
        let org_id = Uuid::new_v4();
        
        service.create_journal_source(org_id, "Manual".to_string(), None, false, false, false, "error".to_string()).unwrap();
        
        let result = service.create_journal_source(org_id, "Manual".to_string(), None, false, false, false, "error".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_action_if_unbalanced() {
        let service = JournalSetupService::new();
        let org_id = Uuid::new_v4();
        
        let result = service.create_journal_source(
            org_id,
            "Receivables".to_string(),
            None,
            false,
            false,
            false,
            "ignore".to_string(), // Invalid
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid action_if_unbalanced. Must be error, warning, or post_to_suspense");
    }

    #[test]
    fn test_create_journal_category() {
        let service = JournalSetupService::new();
        let org_id = Uuid::new_v4();
        
        let result = service.create_journal_category(
            org_id,
            "Accrual".to_string(),
            Some("Accrual entries".to_string()),
        );

        assert!(result.is_ok());
        let category = result.unwrap();
        assert_eq!(category.name, "Accrual");
    }

    #[test]
    fn test_duplicate_journal_category() {
        let service = JournalSetupService::new();
        let org_id = Uuid::new_v4();
        
        service.create_journal_category(org_id, "Accrual".to_string(), None).unwrap();
        
        let result = service.create_journal_category(org_id, "Accrual".to_string(), None);
        assert!(result.is_err());
    }
}
