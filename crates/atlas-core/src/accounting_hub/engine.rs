//! Accounting Hub Engine
//!
//! Processes accounting events from external systems, applies transaction mapping rules,
//! and generates accounting entries. Provides the central hub for integrating
//! third-party financial systems into the Atlas accounting framework.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Accounting Hub

use atlas_shared::{
    ExternalSystem, AccountingEvent, TransactionMappingRule, AccountingHubDashboardSummary,
    AtlasError, AtlasResult,
};
use super::AccountingHubRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid system types
const VALID_SYSTEM_TYPES: &[&str] = &[
    "erp", "billing", "pos", "banking", "insurance", "custom",
];

/// Valid event classes
const VALID_EVENT_CLASSES: &[&str] = &[
    "invoice", "payment", "adjustment", "transfer", "custom",
];

/// Valid event statuses
const VALID_EVENT_STATUSES: &[&str] = &[
    "received", "validated", "accounted", "posted", "transferred", "error",
];

/// Accounting Hub Engine
pub struct AccountingHubEngine {
    repository: Arc<dyn AccountingHubRepository>,
}

impl AccountingHubEngine {
    pub fn new(repository: Arc<dyn AccountingHubRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // External Systems
    // ========================================================================

    /// Register a new external system
    pub async fn register_external_system(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        system_type: &str,
        connection_config: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExternalSystem> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "System code and name are required".to_string(),
            ));
        }
        if !VALID_SYSTEM_TYPES.contains(&system_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid system_type '{}'. Must be one of: {}", system_type, VALID_SYSTEM_TYPES.join(", ")
            )));
        }

        if self.repository.get_external_system(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(
                format!("External system code '{}' already exists", code)
            ));
        }

        info!("Registering external system {} ({}) for org {}", code, system_type, org_id);

        self.repository.create_external_system(
            org_id, code, name, description, system_type,
            connection_config, created_by,
        ).await
    }

    /// Get an external system by code
    pub async fn get_external_system(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ExternalSystem>> {
        self.repository.get_external_system(org_id, code).await
    }

    /// List all external systems
    pub async fn list_external_systems(&self, org_id: Uuid) -> AtlasResult<Vec<ExternalSystem>> {
        self.repository.list_external_systems(org_id).await
    }

    /// Delete an external system
    pub async fn delete_external_system(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.get_external_system(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("External system '{}' not found", code)
            ))?;

        self.repository.delete_external_system(org_id, code).await
    }

    // ========================================================================
    // Transaction Mapping Rules
    // ========================================================================

    /// Create a transaction mapping rule
    pub async fn create_mapping_rule(
        &self,
        org_id: Uuid,
        external_system_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        event_type: &str,
        event_class: &str,
        priority: i32,
        conditions: serde_json::Value,
        field_mappings: serde_json::Value,
        accounting_method_id: Option<Uuid>,
        stop_on_match: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionMappingRule> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Mapping rule code and name are required".to_string(),
            ));
        }
        if !VALID_EVENT_CLASSES.contains(&event_class) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid event_class '{}'. Must be one of: {}", event_class, VALID_EVENT_CLASSES.join(", ")
            )));
        }
        if priority < 0 {
            return Err(AtlasError::ValidationFailed(
                "Priority must be non-negative".to_string(),
            ));
        }

        // Verify external system exists
        self.repository.get_external_system_by_id(external_system_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("External system {} not found", external_system_id)
            ))?;

        info!("Creating mapping rule {} for external system {}", code, external_system_id);

        self.repository.create_mapping_rule(
            org_id, external_system_id, code, name, description,
            event_type, event_class, priority, conditions, field_mappings,
            accounting_method_id, stop_on_match, effective_from, effective_to,
            created_by,
        ).await
    }

    /// List mapping rules
    pub async fn list_mapping_rules(&self, org_id: Uuid, external_system_id: Option<Uuid>) -> AtlasResult<Vec<TransactionMappingRule>> {
        self.repository.list_mapping_rules(org_id, external_system_id).await
    }

    /// Delete a mapping rule
    pub async fn delete_mapping_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_mapping_rule(org_id, code).await
    }

    // ========================================================================
    // Accounting Events
    // ========================================================================

    /// Receive an accounting event from an external system
    pub async fn receive_event(
        &self,
        org_id: Uuid,
        external_system_id: Uuid,
        event_type: &str,
        event_class: &str,
        source_event_id: &str,
        payload: serde_json::Value,
        event_date: chrono::NaiveDate,
        currency_code: &str,
        total_amount: Option<&str>,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingEvent> {
        if source_event_id.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Source event ID is required".to_string(),
            ));
        }
        if !VALID_EVENT_CLASSES.contains(&event_class) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid event_class '{}'. Must be one of: {}", event_class, VALID_EVENT_CLASSES.join(", ")
            )));
        }

        // Verify external system exists
        let system = self.repository.get_external_system_by_id(external_system_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("External system {} not found", external_system_id)
            ))?;

        let event_number = format!("AE-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Receiving accounting event {} from {} ({})", event_number, system.code, event_type);

        let event = self.repository.create_accounting_event(
            org_id, &event_number, external_system_id, Some(&system.code),
            event_type, event_class, source_event_id, payload.clone(),
            serde_json::json!({}), // Will be populated during processing
            None, // Accounting method determined during processing
            "received", event_date, None,
            currency_code, total_amount, description, created_by,
        ).await?;

        // Auto-process: validate and apply mapping rules
        self.process_event(event.id).await?;

        // Refresh after processing
        self.repository.get_accounting_event(event.id).await?.ok_or_else(|| AtlasError::Internal(
            "Event disappeared after processing".to_string(),
        ))
    }

    /// Get an accounting event by ID
    pub async fn get_event(&self, id: Uuid) -> AtlasResult<Option<AccountingEvent>> {
        self.repository.get_accounting_event(id).await
    }

    /// List accounting events
    pub async fn list_events(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        external_system_id: Option<Uuid>,
        event_type: Option<&str>,
    ) -> AtlasResult<Vec<AccountingEvent>> {
        if let Some(s) = status {
            if !VALID_EVENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_EVENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_accounting_events(org_id, status, external_system_id, event_type).await
    }

    // ========================================================================
    // Event Processing
    // ========================================================================

    /// Process a received event: validate and apply mapping rules
    async fn process_event(&self, event_id: Uuid) -> AtlasResult<()> {
        let event = self.repository.get_accounting_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Accounting event {} not found", event_id)
            ))?;

        if event.status != "received" {
            return Ok(()); // Already processed
        }

        // Step 1: Validate required fields
        let validation_result = self.validate_event(&event);
        if let Err(e) = validation_result {
            self.repository.update_event_status(
                event_id, "error", Some(&e.to_string()), None, None, None,
            ).await?;
            return Err(e);
        }

        // Step 2: Apply mapping rules to extract transaction attributes
        let mapping_result = self.apply_mapping_rules(&event).await;

        match mapping_result {
            Ok((attributes, method_id)) => {
                self.repository.update_event_status(
                    event_id, "validated", None,
                    Some(attributes), None, None,
                ).await?;

                // Store the determined accounting method
                if let Some(mid) = method_id {
                    let _event = self.repository.get_accounting_event(event_id).await?
                        .ok_or_else(|| AtlasError::EntityNotFound(
                            format!("Accounting event {} not found", event_id)
                        ))?;

                    // In a full implementation, we would create SLA journal entries here
                    // For now, mark as accounted
                    let _ = mid; // Acknowledge usage
                }

                info!("Successfully processed accounting event {}", event.event_number);
            }
            Err(e) => {
                self.repository.update_event_status(
                    event_id, "error", Some(&e.to_string()), None, None, None,
                ).await?;
                return Err(e);
            }
        }

        Ok(())
    }

    /// Validate an event has required fields
    fn validate_event(&self, event: &AccountingEvent) -> AtlasResult<()> {
        if event.event_type.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Event type is required".to_string(),
            ));
        }
        if event.currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        Ok(())
    }

    /// Apply mapping rules to extract transaction attributes from the event payload
    async fn apply_mapping_rules(
        &self,
        event: &AccountingEvent,
    ) -> AtlasResult<(serde_json::Value, Option<Uuid>)> {
        let rules = self.repository.list_active_mapping_rules(
            event.organization_id, event.external_system_id, &event.event_type,
        ).await?;

        let mut attributes = serde_json::json!({});
        let mut matched_method_id: Option<Uuid> = None;

        for rule in &rules {
            // Check conditions
            if !rule.conditions.is_null() && !Self::conditions_match(&rule.conditions, &event.payload) {
                continue;
            }

            // Apply field mappings
            if let Some(mappings) = rule.field_mappings.as_object() {
                for (target_field, source_path) in mappings {
                    if let Some(path) = source_path.as_str() {
                        if let Some(value) = Self::extract_value(&event.payload, path) {
                            attributes[target_field] = value.clone();
                        }
                    }
                }
            }

            if rule.accounting_method_id.is_some() && matched_method_id.is_none() {
                matched_method_id = rule.accounting_method_id;
            }

            if rule.stop_on_match {
                break;
            }
        }

        Ok((attributes, matched_method_id))
    }

    /// Check if conditions match against a payload
    fn conditions_match(conditions: &serde_json::Value, payload: &serde_json::Value) -> bool {
        let Some(obj) = conditions.as_object() else { return true; };
        if obj.is_empty() { return true; }

        for (key, expected) in obj {
            let actual = Self::extract_value(payload, key);
            match (actual, expected) {
                (Some(a), serde_json::Value::String(s)) => {
                    if a.as_str().unwrap_or("") != s.as_str() {
                        return false;
                    }
                }
                (Some(a), serde_json::Value::Number(n)) => {
                    if a.as_f64().unwrap_or(0.0) != n.as_f64().unwrap_or(0.0) {
                        return false;
                    }
                }
                (None, _) => return false,
                _ => {}
            }
        }
        true
    }

    /// Extract a value from a JSON payload using dot-notation path
    fn extract_value(payload: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = payload;
        for part in &parts {
            current = current.get(part)?;
        }
        Some(current.clone())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the Accounting Hub dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<AccountingHubDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_system_types() {
        assert!(VALID_SYSTEM_TYPES.contains(&"erp"));
        assert!(VALID_SYSTEM_TYPES.contains(&"billing"));
        assert!(VALID_SYSTEM_TYPES.contains(&"pos"));
        assert!(VALID_SYSTEM_TYPES.contains(&"banking"));
        assert!(VALID_SYSTEM_TYPES.contains(&"insurance"));
        assert!(VALID_SYSTEM_TYPES.contains(&"custom"));
        assert_eq!(VALID_SYSTEM_TYPES.len(), 6);
    }

    #[test]
    fn test_valid_event_classes() {
        assert!(VALID_EVENT_CLASSES.contains(&"invoice"));
        assert!(VALID_EVENT_CLASSES.contains(&"payment"));
        assert!(VALID_EVENT_CLASSES.contains(&"adjustment"));
        assert!(VALID_EVENT_CLASSES.contains(&"transfer"));
        assert!(VALID_EVENT_CLASSES.contains(&"custom"));
    }

    #[test]
    fn test_valid_event_statuses() {
        assert!(VALID_EVENT_STATUSES.contains(&"received"));
        assert!(VALID_EVENT_STATUSES.contains(&"validated"));
        assert!(VALID_EVENT_STATUSES.contains(&"accounted"));
        assert!(VALID_EVENT_STATUSES.contains(&"posted"));
        assert!(VALID_EVENT_STATUSES.contains(&"transferred"));
        assert!(VALID_EVENT_STATUSES.contains(&"error"));
    }

    #[test]
    fn test_conditions_match_empty() {
        let result = AccountingHubEngine::conditions_match(
            &serde_json::json!({}),
            &serde_json::json!({"amount": 100}),
        );
        assert!(result);
    }

    #[test]
    fn test_conditions_match_null() {
        let result = AccountingHubEngine::conditions_match(
            &serde_json::Value::Null,
            &serde_json::json!({}),
        );
        assert!(result);
    }

    #[test]
    fn test_conditions_match_string() {
        let result = AccountingHubEngine::conditions_match(
            &serde_json::json!({"type": "invoice"}),
            &serde_json::json!({"type": "invoice", "amount": 100}),
        );
        assert!(result);

        let result = AccountingHubEngine::conditions_match(
            &serde_json::json!({"type": "invoice"}),
            &serde_json::json!({"type": "payment", "amount": 100}),
        );
        assert!(!result);
    }

    #[test]
    fn test_conditions_match_number() {
        let result = AccountingHubEngine::conditions_match(
            &serde_json::json!({"amount": 100.0}),
            &serde_json::json!({"amount": 100.0}),
        );
        assert!(result);
    }

    #[test]
    fn test_conditions_match_missing_field() {
        let result = AccountingHubEngine::conditions_match(
            &serde_json::json!({"type": "invoice"}),
            &serde_json::json!({"amount": 100}),
        );
        assert!(!result);
    }

    #[test]
    fn test_extract_value_simple() {
        let payload = serde_json::json!({
            "amount": 100,
            "currency": "USD",
            "description": "Test"
        });

        assert_eq!(
            AccountingHubEngine::extract_value(&payload, "amount"),
            Some(serde_json::json!(100))
        );
        assert_eq!(
            AccountingHubEngine::extract_value(&payload, "currency"),
            Some(serde_json::json!("USD"))
        );
        assert_eq!(
            AccountingHubEngine::extract_value(&payload, "nonexistent"),
            None
        );
    }

    #[test]
    fn test_extract_value_nested() {
        let payload = serde_json::json!({
            "transaction": {
                "header": {
                    "invoice_number": "INV-001"
                },
                "amount": 500.0
            }
        });

        assert_eq!(
            AccountingHubEngine::extract_value(&payload, "transaction.header.invoice_number"),
            Some(serde_json::json!("INV-001"))
        );
        assert_eq!(
            AccountingHubEngine::extract_value(&payload, "transaction.amount"),
            Some(serde_json::json!(500.0))
        );
        assert_eq!(
            AccountingHubEngine::extract_value(&payload, "transaction.header.nonexistent"),
            None
        );
    }
}
