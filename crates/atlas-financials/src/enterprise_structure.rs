use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub legal_entity_identifier: Option<String>,
    pub registration_number: Option<String>,
    pub inception_date: Option<NaiveDate>,
    pub registration_date: Option<NaiveDate>,
    pub place_of_registration: Option<String>,
    pub is_primary_legal_entity: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessUnit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub code: String,
    pub manager_id: Option<Uuid>,
    pub default_legal_entity_id: Option<Uuid>,
    pub default_ledger_id: Option<Uuid>,
    pub is_active: bool,
}

pub struct EnterpriseStructureService {
    legal_entities: Arc<RwLock<Vec<LegalEntity>>>,
    business_units: Arc<RwLock<Vec<BusinessUnit>>>,
}

impl EnterpriseStructureService {
    pub fn new() -> Self {
        Self {
            legal_entities: Arc::new(RwLock::new(Vec::new())),
            business_units: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_legal_entity(
        &self,
        organization_id: Uuid,
        name: String,
        legal_entity_identifier: Option<String>,
        registration_number: Option<String>,
        is_primary_legal_entity: bool,
    ) -> Result<LegalEntity, String> {
        let mut entities = self.legal_entities.write().unwrap();
        
        if entities.iter().any(|e| e.organization_id == organization_id && e.name == name) {
            return Err("Legal entity with this name already exists for the organization".to_string());
        }

        if is_primary_legal_entity {
            if let Some(existing_primary) = entities.iter_mut().find(|e| e.organization_id == organization_id && e.is_primary_legal_entity) {
                existing_primary.is_primary_legal_entity = false;
            }
        }

        let entity = LegalEntity {
            id: Uuid::new_v4(),
            organization_id,
            name,
            legal_entity_identifier,
            registration_number,
            inception_date: None,
            registration_date: None,
            place_of_registration: None,
            is_primary_legal_entity,
            is_active: true,
        };

        entities.push(entity.clone());
        Ok(entity)
    }

    pub fn create_business_unit(
        &self,
        organization_id: Uuid,
        name: String,
        code: String,
        manager_id: Option<Uuid>,
        default_legal_entity_id: Option<Uuid>,
        default_ledger_id: Option<Uuid>,
    ) -> Result<BusinessUnit, String> {
        let mut units = self.business_units.write().unwrap();
        
        if units.iter().any(|u| u.organization_id == organization_id && u.code == code) {
            return Err("Business unit with this code already exists for the organization".to_string());
        }

        if let Some(le_id) = default_legal_entity_id {
            let entities = self.legal_entities.read().unwrap();
            if !entities.iter().any(|e| e.id == le_id) {
                return Err("Default legal entity not found".to_string());
            }
        }

        let unit = BusinessUnit {
            id: Uuid::new_v4(),
            organization_id,
            name,
            code,
            manager_id,
            default_legal_entity_id,
            default_ledger_id,
            is_active: true,
        };

        units.push(unit.clone());
        Ok(unit)
    }

    pub fn get_business_units_for_legal_entity(&self, legal_entity_id: Uuid) -> Vec<BusinessUnit> {
        let units = self.business_units.read().unwrap();
        units.iter()
            .filter(|u| u.default_legal_entity_id == Some(legal_entity_id) && u.is_active)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_legal_entity() {
        let service = EnterpriseStructureService::new();
        let org_id = Uuid::new_v4();
        
        let result = service.create_legal_entity(
            org_id,
            "Global Corp US".to_string(),
            Some("LEI12345".to_string()),
            Some("REG6789".to_string()),
            true,
        );

        assert!(result.is_ok());
        let entity = result.unwrap();
        assert_eq!(entity.name, "Global Corp US");
        assert!(entity.is_primary_legal_entity);
    }

    #[test]
    fn test_duplicate_legal_entity_name() {
        let service = EnterpriseStructureService::new();
        let org_id = Uuid::new_v4();
        
        service.create_legal_entity(org_id, "Corp".to_string(), None, None, false).unwrap();
        
        let result = service.create_legal_entity(org_id, "Corp".to_string(), None, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_primary_legal_entity_replacement() {
        let service = EnterpriseStructureService::new();
        let org_id = Uuid::new_v4();
        
        let first = service.create_legal_entity(org_id, "First".to_string(), None, None, true).unwrap();
        let second = service.create_legal_entity(org_id, "Second".to_string(), None, None, true).unwrap();

        let entities = service.legal_entities.read().unwrap();
        let updated_first = entities.iter().find(|e| e.id == first.id).unwrap();
        
        assert!(!updated_first.is_primary_legal_entity);
        assert!(second.is_primary_legal_entity);
    }

    #[test]
    fn test_create_business_unit() {
        let service = EnterpriseStructureService::new();
        let org_id = Uuid::new_v4();
        
        let le = service.create_legal_entity(org_id, "US Entity".to_string(), None, None, true).unwrap();

        let result = service.create_business_unit(
            org_id,
            "US Sales".to_string(),
            "US_SALES".to_string(),
            None,
            Some(le.id),
            None,
        );

        assert!(result.is_ok());
        let unit = result.unwrap();
        assert_eq!(unit.code, "US_SALES");
    }

    #[test]
    fn test_create_business_unit_invalid_le() {
        let service = EnterpriseStructureService::new();
        let org_id = Uuid::new_v4();
        
        let result = service.create_business_unit(
            org_id,
            "US Sales".to_string(),
            "US_SALES".to_string(),
            None,
            Some(Uuid::new_v4()), // Non-existent LE
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_business_units_for_legal_entity() {
        let service = EnterpriseStructureService::new();
        let org_id = Uuid::new_v4();
        
        let le = service.create_legal_entity(org_id, "UK Entity".to_string(), None, None, true).unwrap();

        service.create_business_unit(org_id, "UK Sales".to_string(), "UK_SALES".to_string(), None, Some(le.id), None).unwrap();
        service.create_business_unit(org_id, "UK Marketing".to_string(), "UK_MKTG".to_string(), None, Some(le.id), None).unwrap();
        service.create_business_unit(org_id, "US Sales".to_string(), "US_SALES".to_string(), None, None, None).unwrap();

        let units = service.get_business_units_for_legal_entity(le.id);
        assert_eq!(units.len(), 2);
    }
}
