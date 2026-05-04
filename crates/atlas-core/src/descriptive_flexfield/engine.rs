//! Descriptive Flexfield Engine
//!
//! Manages flexfield lifecycle, value set validation, context resolution,
//! segment validation, and flexfield data CRUD.
//!
//! Oracle Fusion equivalent: Application Extensions > Flexfields > Descriptive

use atlas_shared::{
    DescriptiveFlexfield, FlexfieldContext, FlexfieldSegment,
    FlexfieldValueSet, FlexfieldValueSetEntry, FlexfieldData,
    AtlasError, AtlasResult,
};
use super::DescriptiveFlexfieldRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid validation types for value sets
const VALID_VALIDATION_TYPES: &[&str] = &["none", "independent", "dependent", "table", "format_only"];

/// Valid data types for segments and value sets
const VALID_DATA_TYPES: &[&str] = &["string", "number", "date", "datetime"];

/// Maximum number of attribute columns (Oracle Fusion convention: typically 15-30)
const MAX_SEGMENTS_PER_CONTEXT: usize = 30;

/// Descriptive Flexfield engine
pub struct DescriptiveFlexfieldEngine {
    repository: Arc<dyn DescriptiveFlexfieldRepository>,
}

impl DescriptiveFlexfieldEngine {
    pub fn new(repository: Arc<dyn DescriptiveFlexfieldRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Flexfield Management
    // ========================================================================

    /// Create a new descriptive flexfield on an entity
    pub async fn create_flexfield(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        entity_name: &str,
        context_column: Option<&str>,
        default_context_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DescriptiveFlexfield> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Flexfield code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Flexfield name is required".to_string()));
        }
        if entity_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Entity name is required".to_string()));
        }

        // Check uniqueness by code
        if self.repository.get_flexfield(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Flexfield with code '{}' already exists", code
            )));
        }

        // Check uniqueness by entity (one DFF per entity)
        if self.repository.get_flexfield_by_entity(org_id, entity_name).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "An active flexfield already exists for entity '{}'", entity_name
            )));
        }

        let ctx_col = context_column.unwrap_or("dff_context");

        info!("Creating flexfield {} on entity {}", code, entity_name);

        self.repository.create_flexfield(
            org_id, code, name, description, entity_name,
            ctx_col, default_context_code, created_by,
        ).await
    }

    /// Get a flexfield by code
    pub async fn get_flexfield(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DescriptiveFlexfield>> {
        self.repository.get_flexfield(org_id, code).await
    }

    /// Get the flexfield for a given entity
    pub async fn get_flexfield_by_entity(&self, org_id: Uuid, entity_name: &str) -> AtlasResult<Option<DescriptiveFlexfield>> {
        self.repository.get_flexfield_by_entity(org_id, entity_name).await
    }

    /// List all flexfields
    pub async fn list_flexfields(&self, org_id: Uuid) -> AtlasResult<Vec<DescriptiveFlexfield>> {
        self.repository.list_flexfields(org_id).await
    }

    /// Activate a flexfield
    pub async fn activate_flexfield(&self, id: Uuid) -> AtlasResult<DescriptiveFlexfield> {
        let ff = self.repository.get_flexfield_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Flexfield {} not found", id)))?;

        if ff.is_active {
            return Err(AtlasError::WorkflowError("Flexfield is already active".to_string()));
        }

        info!("Activated flexfield {}", ff.code);
        self.repository.update_flexfield_active(id, true).await
    }

    /// Deactivate a flexfield
    pub async fn deactivate_flexfield(&self, id: Uuid) -> AtlasResult<DescriptiveFlexfield> {
        let ff = self.repository.get_flexfield_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Flexfield {} not found", id)))?;

        if !ff.is_active {
            return Err(AtlasError::WorkflowError("Flexfield is already inactive".to_string()));
        }

        info!("Deactivated flexfield {}", ff.code);
        self.repository.update_flexfield_active(id, false).await
    }

    /// Delete a flexfield and all its contexts, segments, and data
    pub async fn delete_flexfield(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleted flexfield {}", code);
        self.repository.delete_flexfield(org_id, code).await
    }

    // ========================================================================
    // Context Management
    // ========================================================================

    /// Create a new context within a flexfield
    pub async fn create_context(
        &self,
        org_id: Uuid,
        flexfield_code: &str,
        code: &str,
        name: &str,
        description: Option<&str>,
        is_global: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldContext> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Context code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Context name is required".to_string()));
        }

        let flexfield = self.repository.get_flexfield(org_id, flexfield_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Flexfield '{}' not found", flexfield_code
            )))?;

        // Check uniqueness
        if self.repository.get_context_by_code(flexfield.id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Context '{}' already exists in flexfield '{}'", code, flexfield_code
            )));
        }

        info!("Creating context {} in flexfield {}", code, flexfield_code);

        self.repository.create_context(
            org_id, flexfield.id, code, name, description, is_global, created_by,
        ).await
    }

    /// Get a context by ID
    pub async fn get_context(&self, id: Uuid) -> AtlasResult<Option<FlexfieldContext>> {
        self.repository.get_context(id).await
    }

    /// List contexts for a flexfield
    pub async fn list_contexts(&self, org_id: Uuid, flexfield_code: &str) -> AtlasResult<Vec<FlexfieldContext>> {
        let flexfield = self.repository.get_flexfield(org_id, flexfield_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Flexfield '{}' not found", flexfield_code
            )))?;

        self.repository.list_contexts(flexfield.id).await
    }

    /// Disable a context
    pub async fn disable_context(&self, id: Uuid) -> AtlasResult<FlexfieldContext> {
        info!("Disabled context {}", id);
        self.repository.update_context_enabled(id, false).await
    }

    /// Enable a context
    pub async fn enable_context(&self, id: Uuid) -> AtlasResult<FlexfieldContext> {
        info!("Enabled context {}", id);
        self.repository.update_context_enabled(id, true).await
    }

    /// Delete a context
    pub async fn delete_context(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted context {}", id);
        self.repository.delete_context(id).await
    }

    // ========================================================================
    // Segment Management
    // ========================================================================

    /// Add a segment to a context
    pub async fn create_segment(
        &self,
        org_id: Uuid,
        flexfield_code: &str,
        context_code: &str,
        segment_code: &str,
        name: &str,
        description: Option<&str>,
        display_order: i32,
        column_name: &str,
        data_type: &str,
        is_required: bool,
        is_read_only: bool,
        is_visible: bool,
        default_value: Option<&str>,
        value_set_code: Option<&str>,
        help_text: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldSegment> {
        if segment_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Segment code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Segment name is required".to_string()));
        }
        if !VALID_DATA_TYPES.contains(&data_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid data type '{}'. Must be one of: {}", data_type, VALID_DATA_TYPES.join(", ")
            )));
        }

        let flexfield = self.repository.get_flexfield(org_id, flexfield_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Flexfield '{}' not found", flexfield_code
            )))?;

        let context = self.repository.get_context_by_code(flexfield.id, context_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Context '{}' not found in flexfield '{}'", context_code, flexfield_code
            )))?;

        if !context.is_enabled {
            return Err(AtlasError::WorkflowError(format!(
                "Context '{}' is disabled", context_code
            )));
        }

        // Check segment limit
        let existing = self.repository.list_segments_by_context(context.id).await?;
        if existing.len() >= MAX_SEGMENTS_PER_CONTEXT {
            return Err(AtlasError::ValidationFailed(format!(
                "Context '{}' has reached the maximum of {} segments",
                context_code, MAX_SEGMENTS_PER_CONTEXT
            )));
        }

        // Check uniqueness within context
        if self.repository.get_segment_by_code(context.id, segment_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Segment '{}' already exists in context '{}'", segment_code, context_code
            )));
        }

        // Resolve value set if specified
        let (value_set_id, vs_code) = if let Some(vs_code) = value_set_code {
            let vs = self.repository.get_value_set(org_id, vs_code).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Value set '{}' not found", vs_code
                )))?;
            (Some(vs.id), Some(vs.code.clone()))
        } else {
            (None, None)
        };

        info!("Creating segment {} in context {} / flexfield {}", segment_code, context_code, flexfield_code);

        self.repository.create_segment(
            org_id, flexfield.id, context.id,
            segment_code, name, description,
            display_order, column_name, data_type,
            is_required, is_read_only, is_visible,
            default_value,
            value_set_id, vs_code.as_deref(),
            help_text, created_by,
        ).await
    }

    /// Get a segment by ID
    pub async fn get_segment(&self, id: Uuid) -> AtlasResult<Option<FlexfieldSegment>> {
        self.repository.get_segment(id).await
    }

    /// List segments for a context
    pub async fn list_segments_by_context(
        &self,
        org_id: Uuid,
        flexfield_code: &str,
        context_code: &str,
    ) -> AtlasResult<Vec<FlexfieldSegment>> {
        let flexfield = self.repository.get_flexfield(org_id, flexfield_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Flexfield '{}' not found", flexfield_code
            )))?;

        let context = self.repository.get_context_by_code(flexfield.id, context_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Context '{}' not found", context_code
            )))?;

        self.repository.list_segments_by_context(context.id).await
    }

    /// List all segments for a flexfield (across all contexts)
    pub async fn list_segments_by_flexfield(
        &self,
        org_id: Uuid,
        flexfield_code: &str,
    ) -> AtlasResult<Vec<FlexfieldSegment>> {
        let flexfield = self.repository.get_flexfield(org_id, flexfield_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Flexfield '{}' not found", flexfield_code
            )))?;

        self.repository.list_segments_by_flexfield(flexfield.id).await
    }

    /// Update a segment
    #[allow(clippy::too_many_arguments)]
    pub async fn update_segment(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        display_order: Option<i32>,
        is_required: Option<bool>,
        is_read_only: Option<bool>,
        is_visible: Option<bool>,
        default_value: Option<&str>,
        value_set_code: Option<&str>,
    ) -> AtlasResult<FlexfieldSegment> {
        let segment = self.repository.get_segment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Segment {} not found", id)))?;

        let (vs_id, vs_code) = if let Some(vs_code) = value_set_code {
            let vs = self.repository.get_value_set(segment.organization_id, vs_code).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Value set '{}' not found", vs_code
                )))?;
            (Some(Some(vs.id)), Some(Some(vs.code.clone())))
        } else {
            (None, None)
        };

        info!("Updated segment {} ({})", segment.segment_code, segment.name);

        self.repository.update_segment(
            id, name, description, display_order,
            is_required, is_read_only, is_visible,
            default_value,
            vs_id.flatten(), vs_code.flatten().as_deref(),
        ).await
    }

    /// Delete a segment
    pub async fn delete_segment(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted segment {}", id);
        self.repository.delete_segment(id).await
    }

    // ========================================================================
    // Value Set Management
    // ========================================================================

    /// Create a new value set
    pub async fn create_value_set(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        validation_type: &str,
        data_type: &str,
        max_length: i32,
        min_length: i32,
        format_mask: Option<&str>,
        table_validation: Option<serde_json::Value>,
        independent_values: Option<serde_json::Value>,
        parent_value_set_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldValueSet> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Value set code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Value set name is required".to_string()));
        }
        if !VALID_VALIDATION_TYPES.contains(&validation_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid validation type '{}'. Must be one of: {}",
                validation_type, VALID_VALIDATION_TYPES.join(", ")
            )));
        }
        if !VALID_DATA_TYPES.contains(&data_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid data type '{}'. Must be one of: {}", data_type, VALID_DATA_TYPES.join(", ")
            )));
        }
        if max_length < 0 {
            return Err(AtlasError::ValidationFailed("Max length must be >= 0".to_string()));
        }
        if min_length < 0 {
            return Err(AtlasError::ValidationFailed("Min length must be >= 0".to_string()));
        }
        if min_length > max_length {
            return Err(AtlasError::ValidationFailed("Min length must be <= max length".to_string()));
        }

        // Check uniqueness
        if self.repository.get_value_set(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Value set '{}' already exists", code
            )));
        }

        // Validate parent value set if specified
        if let Some(parent_code) = parent_value_set_code {
            let parent = self.repository.get_value_set(org_id, parent_code).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Parent value set '{}' not found", parent_code
                )))?;
            if parent.validation_type != "independent" {
                return Err(AtlasError::ValidationFailed(
                    "Parent value set must have 'independent' validation type".to_string(),
                ));
            }
        }

        info!("Creating value set {} ({})", code, name);

        self.repository.create_value_set(
            org_id, code, name, description, validation_type, data_type,
            max_length, min_length, format_mask, table_validation,
            independent_values, parent_value_set_code, created_by,
        ).await
    }

    /// Get a value set by code
    pub async fn get_value_set(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FlexfieldValueSet>> {
        self.repository.get_value_set(org_id, code).await
    }

    /// List all value sets
    pub async fn list_value_sets(&self, org_id: Uuid) -> AtlasResult<Vec<FlexfieldValueSet>> {
        self.repository.list_value_sets(org_id).await
    }

    /// Delete a value set
    pub async fn delete_value_set(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleted value set {}", code);
        self.repository.delete_value_set(org_id, code).await
    }

    // ========================================================================
    // Value Set Entries
    // ========================================================================

    /// Add an entry to a value set
    pub async fn create_value_set_entry(
        &self,
        org_id: Uuid,
        value_set_code: &str,
        value: &str,
        meaning: Option<&str>,
        description: Option<&str>,
        parent_value: Option<&str>,
        is_enabled: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        sort_order: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldValueSetEntry> {
        if value.is_empty() {
            return Err(AtlasError::ValidationFailed("Value is required".to_string()));
        }

        let vs = self.repository.get_value_set(org_id, value_set_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Value set '{}' not found", value_set_code
            )))?;

        // For dependent value sets, parent_value is required
        if vs.validation_type == "dependent" && parent_value.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Parent value is required for dependent value sets".to_string(),
            ));
        }

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        info!("Adding entry '{}' to value set {}", value, value_set_code);

        self.repository.create_value_set_entry(
            org_id, vs.id, value, meaning, description,
            parent_value, is_enabled, effective_from, effective_to,
            sort_order, created_by,
        ).await
    }

    /// List entries for a value set
    pub async fn list_value_set_entries(
        &self,
        org_id: Uuid,
        value_set_code: &str,
        parent_value: Option<&str>,
    ) -> AtlasResult<Vec<FlexfieldValueSetEntry>> {
        let vs = self.repository.get_value_set(org_id, value_set_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Value set '{}' not found", value_set_code
            )))?;

        self.repository.list_value_set_entries(vs.id, parent_value).await
    }

    /// Delete a value set entry
    pub async fn delete_value_set_entry(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted value set entry {}", id);
        self.repository.delete_value_set_entry(id).await
    }

    // ========================================================================
    // Flexfield Data (CRUD)
    // ========================================================================

    /// Set flexfield data for an entity record.
    /// Validates all segment values against configured value sets.
    pub async fn set_data(
        &self,
        org_id: Uuid,
        entity_name: &str,
        entity_id: Uuid,
        context_code: &str,
        segment_values: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldData> {
        let flexfield = self.repository.get_flexfield_by_entity(org_id, entity_name).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "No active flexfield for entity '{}'", entity_name
            )))?;

        if !flexfield.is_active {
            return Err(AtlasError::WorkflowError(format!(
                "Flexfield '{}' is not active", flexfield.code
            )));
        }

        let context = self.repository.get_context_by_code(flexfield.id, context_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Context '{}' not found", context_code
            )))?;

        if !context.is_enabled {
            return Err(AtlasError::WorkflowError(format!(
                "Context '{}' is disabled", context_code
            )));
        }

        // Validate segment values
        let segments = self.repository.list_segments_by_context(context.id).await?;
        let values_map = segment_values.as_object()
            .ok_or_else(|| AtlasError::ValidationFailed("segment_values must be a JSON object".to_string()))?;

        for segment in &segments {
            if !segment.is_visible {
                continue;
            }

            let value = values_map.get(&segment.segment_code)
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Check required
            if segment.is_required && value.is_empty() {
                return Err(AtlasError::ValidationFailed(format!(
                    "Segment '{}' is required", segment.name
                )));
            }

            // Skip further validation if empty and not required
            if value.is_empty() {
                continue;
            }

            // Validate against value set
            if let Some(vs_id) = segment.value_set_id {
                let valid = self.repository.validate_value(vs_id, value, None).await?;
                if !valid {
                    return Err(AtlasError::ValidationFailed(format!(
                        "Value '{}' is not valid for segment '{}' (value set validation failed)",
                        value, segment.name
                    )));
                }
            }

            // Validate data type
            Self::validate_data_type(value, &segment.data_type, &segment.name)?;
        }

        info!("Setting flexfield data for {} {} (context: {})", entity_name, entity_id, context_code);

        self.repository.set_flexfield_data(
            org_id, flexfield.id, entity_name, entity_id,
            context_code, segment_values, created_by,
        ).await
    }

    /// Get flexfield data for an entity record
    pub async fn get_data(
        &self,
        entity_name: &str,
        entity_id: Uuid,
        context_code: Option<&str>,
    ) -> AtlasResult<Vec<FlexfieldData>> {
        self.repository.get_flexfield_data(entity_name, entity_id, context_code).await
    }

    /// Delete all flexfield data for an entity record
    pub async fn delete_data(
        &self,
        entity_name: &str,
        entity_id: Uuid,
    ) -> AtlasResult<()> {
        self.repository.delete_flexfield_data(entity_name, entity_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get flexfield dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<atlas_shared::FlexfieldDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Validation Helpers
    // ========================================================================

    /// Validate a value against a data type
    pub fn validate_data_type(value: &str, data_type: &str, segment_name: &str) -> AtlasResult<()> {
        match data_type {
            "number"
                if value.parse::<f64>().is_err() => {
                    return Err(AtlasError::ValidationFailed(format!(
                        "Segment '{}' requires a numeric value, got '{}'", segment_name, value
                    )));
                }
            "date"
                if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() => {
                    return Err(AtlasError::ValidationFailed(format!(
                        "Segment '{}' requires a date value (YYYY-MM-DD), got '{}'", segment_name, value
                    )));
                }
            "datetime"
                // Accept ISO 8601 formats
                if value.parse::<chrono::DateTime<chrono::Utc>>().is_err()
                    && value.parse::<chrono::NaiveDateTime>().is_err()
                => {
                    return Err(AtlasError::ValidationFailed(format!(
                        "Segment '{}' requires a datetime value, got '{}'", segment_name, value
                    )));
                }
            _ => { /* any value is valid */ }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_validation_types() {
        assert!(VALID_VALIDATION_TYPES.contains(&"none"));
        assert!(VALID_VALIDATION_TYPES.contains(&"independent"));
        assert!(VALID_VALIDATION_TYPES.contains(&"dependent"));
        assert!(VALID_VALIDATION_TYPES.contains(&"table"));
        assert!(VALID_VALIDATION_TYPES.contains(&"format_only"));
    }

    #[test]
    fn test_valid_data_types() {
        assert!(VALID_DATA_TYPES.contains(&"string"));
        assert!(VALID_DATA_TYPES.contains(&"number"));
        assert!(VALID_DATA_TYPES.contains(&"date"));
        assert!(VALID_DATA_TYPES.contains(&"datetime"));
    }

    #[test]
    fn test_validate_data_type_string() {
        assert!(DescriptiveFlexfieldEngine::validate_data_type("hello", "string", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("", "string", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("123", "string", "test").is_ok());
    }

    #[test]
    fn test_validate_data_type_number() {
        assert!(DescriptiveFlexfieldEngine::validate_data_type("123", "number", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("45.67", "number", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("-10", "number", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("0", "number", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("abc", "number", "test").is_err());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("12.34.56", "number", "test").is_err());
    }

    #[test]
    fn test_validate_data_type_date() {
        assert!(DescriptiveFlexfieldEngine::validate_data_type("2024-06-15", "date", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("2024-01-01", "date", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("not-a-date", "date", "test").is_err());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("06/15/2024", "date", "test").is_err());
    }

    #[test]
    fn test_validate_data_type_datetime() {
        assert!(DescriptiveFlexfieldEngine::validate_data_type("2024-06-15T10:30:00Z", "datetime", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("2024-06-15T10:30:00", "datetime", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("not-datetime", "datetime", "test").is_err());
    }

    #[test]
    fn test_max_segments_per_context() {
        assert_eq!(MAX_SEGMENTS_PER_CONTEXT, 30);
    }

    #[test]
    fn test_validate_data_type_number_edge_cases() {
        // Scientific notation
        assert!(DescriptiveFlexfieldEngine::validate_data_type("1e5", "number", "test").is_ok());
        // Negative float
        assert!(DescriptiveFlexfieldEngine::validate_data_type("-99.99", "number", "test").is_ok());
        // Just a sign
        assert!(DescriptiveFlexfieldEngine::validate_data_type("-", "number", "test").is_err());
    }

    #[test]
    fn test_validate_data_type_empty_number() {
        // Empty string should pass (not required check is done separately)
        // Actually empty string fails to parse as f64
        assert!(DescriptiveFlexfieldEngine::validate_data_type("", "number", "test").is_err());
    }

    #[test]
    fn test_validate_data_type_empty_date() {
        assert!(DescriptiveFlexfieldEngine::validate_data_type("", "date", "test").is_err());
    }

    #[test]
    fn test_validate_data_type_empty_datetime() {
        assert!(DescriptiveFlexfieldEngine::validate_data_type("", "datetime", "test").is_err());
    }

    #[test]
    fn test_validate_data_type_empty_string() {
        // Empty string is valid for string type
        assert!(DescriptiveFlexfieldEngine::validate_data_type("", "string", "test").is_ok());
    }

    #[test]
    fn test_validate_data_type_date_edge_cases() {
        assert!(DescriptiveFlexfieldEngine::validate_data_type("2024-02-29", "date", "test").is_ok()); // leap year
        assert!(DescriptiveFlexfieldEngine::validate_data_type("2024-12-31", "date", "test").is_ok());
        assert!(DescriptiveFlexfieldEngine::validate_data_type("2024-13-01", "date", "test").is_err()); // invalid month
    }
}
