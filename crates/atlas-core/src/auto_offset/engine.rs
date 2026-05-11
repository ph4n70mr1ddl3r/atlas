//! Automatic Offset Engine
//!
//! Manages the automatic generation of intercompany offset entries:
//! - Create offset templates with due-to / due-from account mappings
//! - Add template lines per balancing segment value (entity)
//! - Generate offset entries from journal entry lines
//! - Post offset entries to the GL
//! - Full lifecycle: generated → posted → reversed → cancelled
//! - Dashboard and reporting
//!
//! Oracle Fusion Cloud ERP: Financials > General Ledger > Automatic Offsets

use super::{AutoOffsetRepository, AtlasResult, AutoOffsetTemplate, AutoOffsetTemplateLine, AutoOffsetGeneration, AutoOffsetLine, AutoOffsetActivity, AutoOffsetDashboard};
use atlas_shared::AtlasError;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_BALANCING_SEGMENTS: &[&str] = &[
    "entity", "department", "cost_center", "location", "intercompany",
];
const VALID_GENERATION_METHODS: &[&str] = &[
    "single_entry", "multi_entry", "net_zero",
];
const VALID_SOURCE_TYPES: &[&str] = &[
    "journal_entry", "invoice", "payment", "receipt", "manual",
];
#[allow(dead_code)]
const VALID_OFFSET_TYPES: &[&str] = &["due_to", "due_from", "clearing"];
#[allow(dead_code)]
const VALID_GENERATION_STATUSES: &[&str] = &[
    "generated", "posted", "reversed", "cancelled",
];

/// Automatic Offset Engine
pub struct AutoOffsetEngine {
    repository: Arc<dyn AutoOffsetRepository>,
}

impl AutoOffsetEngine {
    pub fn new(repository: Arc<dyn AutoOffsetRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Template CRUD
    // ========================================================================

    /// Create a new offset template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        template_code: &str,
        template_name: &str,
        description: Option<&str>,
        balancing_segment: &str,
        intercompany_segment: Option<&str>,
        generation_method: &str,
        default_offset_account: &str,
        default_offset_account_description: Option<&str>,
        enable_intra_entity: bool,
        intra_entity_account: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoOffsetTemplate> {
        if template_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Template code is required".into()));
        }
        if template_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Template name is required".into()));
        }
        if !VALID_BALANCING_SEGMENTS.contains(&balancing_segment) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid balancing_segment '{}'. Must be one of: {}",
                balancing_segment, VALID_BALANCING_SEGMENTS.join(", ")
            )));
        }
        if !VALID_GENERATION_METHODS.contains(&generation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid generation_method '{}'. Must be one of: {}",
                generation_method, VALID_GENERATION_METHODS.join(", ")
            )));
        }
        if default_offset_account.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Default offset account is required".into(),
            ));
        }
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".into(),
                ));
            }
        }
        // Check duplicate
        if let Some(_existing) = self.repository.get_template_by_code(org_id, template_code).await? {
            return Err(AtlasError::Conflict(format!(
                "Offset template code '{template_code}' already exists"
            )));
        }

        info!("Auto Offset Engine: Creating template '{}' ({})", template_code, template_name);

        self.repository.create_template(
            org_id, template_code, template_name, description,
            balancing_segment, intercompany_segment, generation_method,
            default_offset_account, default_offset_account_description,
            enable_intra_entity, intra_entity_account,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a template by ID
    pub async fn get_template(&self, id: Uuid) -> AtlasResult<Option<AutoOffsetTemplate>> {
        self.repository.get_template(id).await
    }

    /// Get a template by code
    pub async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AutoOffsetTemplate>> {
        self.repository.get_template_by_code(org_id, code).await
    }

    /// List templates
    pub async fn list_templates(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<AutoOffsetTemplate>> {
        self.repository.list_templates(org_id, is_active).await
    }

    /// Activate a template
    pub async fn activate_template(&self, id: Uuid) -> AtlasResult<AutoOffsetTemplate> {
        info!("Auto Offset Engine: Activating template {}", id);
        self.repository.update_template_status(id, true).await
    }

    /// Deactivate a template
    pub async fn deactivate_template(&self, id: Uuid) -> AtlasResult<AutoOffsetTemplate> {
        info!("Auto Offset Engine: Deactivating template {}", id);
        self.repository.update_template_status(id, false).await
    }

    /// Delete a template
    pub async fn delete_template(&self, id: Uuid) -> AtlasResult<()> {
        info!("Auto Offset Engine: Deleting template {}", id);
        self.repository.delete_template(id).await
    }

    // ========================================================================
    // Template Lines
    // ========================================================================

    /// Add a line to an offset template
    pub async fn add_template_line(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        line_number: i32,
        balancing_segment_value: &str,
        due_to_account: &str,
        due_to_account_description: Option<&str>,
        due_from_account: &str,
        due_from_account_description: Option<&str>,
        clearing_account: Option<&str>,
        priority: i32,
    ) -> AtlasResult<AutoOffsetTemplateLine> {
        if balancing_segment_value.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Balancing segment value is required".into(),
            ));
        }
        if due_to_account.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Due-to account is required".into(),
            ));
        }
        if due_from_account.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Due-from account is required".into(),
            ));
        }
        if priority < 0 {
            return Err(AtlasError::ValidationFailed(
                "Priority must be non-negative".into(),
            ));
        }

        // Verify template exists
        let _template = self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Template {template_id} not found")
            ))?;

        info!("Auto Offset Engine: Adding line {} for segment '{}' to template {}",
              line_number, balancing_segment_value, template_id);

        self.repository.add_template_line(
            org_id, template_id, line_number, balancing_segment_value,
            due_to_account, due_to_account_description,
            due_from_account, due_from_account_description,
            clearing_account, priority,
        ).await
    }

    /// List template lines
    pub async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<AutoOffsetTemplateLine>> {
        self.repository.list_template_lines(template_id).await
    }

    /// Delete a template line
    pub async fn delete_template_line(&self, line_id: Uuid) -> AtlasResult<()> {
        self.repository.delete_template_line(line_id).await
    }

    // ========================================================================
    // Offset Generation
    // ========================================================================

    /// Generate offset entries from a set of journal lines
    /// This is the core business logic: analyze lines by balancing segment,
    /// calculate imbalances, and create due-to / due-from entries.
    pub async fn generate_offsets(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        fiscal_year: i32,
        period_name: &str,
        generation_date: chrono::NaiveDate,
        currency_code: &str,
        journal_lines: &[JournalLine],
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoOffsetGeneration> {
        // Validate source type
        if !VALID_SOURCE_TYPES.contains(&source_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid source_type '{}'. Must be one of: {}",
                source_type, VALID_SOURCE_TYPES.join(", ")
            )));
        }

        // Validate template exists and is active
        let template = self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Template {template_id} not found")
            ))?;

        if !template.is_active {
            return Err(AtlasError::ValidationFailed(
                "Template is not active".into(),
            ));
        }

        if journal_lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No journal lines provided for offset generation".into(),
            ));
        }

        // Calculate net amounts per balancing segment
        let mut segment_balances: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for line in journal_lines {
            let net = line.debit - line.credit;
            *segment_balances.entry(line.balancing_segment_value.clone()).or_insert(0.0) += net;
        }

        // Check if any individual segment is imbalanced
        let has_imbalance = segment_balances.values().any(|v| v.abs() > 0.001);
        if !has_imbalance {
            // Everything balances; no offsets needed
            return Err(AtlasError::ValidationFailed(
                "Journal lines are already balanced across segments. No offsets needed.".into(),
            ));
        }

        let segments_affected = segment_balances.keys().len() as i32;

        // Generate offset lines for each imbalanced segment
        let mut offset_line_data: Vec<(String, String, String, String, f64)> = Vec::new(); // (from, to, type, account, amount)

        let template_lines = self.repository.list_template_lines(template_id).await?;
        let segments: Vec<&String> = segment_balances.keys().collect();

        // For each pair of segments, calculate the offset
        for (i, seg_a) in segments.iter().enumerate() {
            for seg_b in segments.iter().skip(i + 1) {
                let bal_a = segment_balances.get(*seg_a).unwrap_or(&0.0);
                let bal_b = segment_balances.get(*seg_b).unwrap_or(&0.0);

                // If segment A has surplus and B has deficit (or vice versa)
                if (*bal_a > 0.0 && *bal_b < 0.0) || (*bal_a < 0.0 && *bal_b > 0.0) {
                    let offset_amount = if bal_a.abs() < bal_b.abs() { bal_a.abs() } else { bal_b.abs() };

                    // Find matching template line for segment A
                    let line_a = template_lines.iter()
                        .find(|l| l.balancing_segment_value == **seg_a);

                    // Find matching template line for segment B
                    let line_b = template_lines.iter()
                        .find(|l| l.balancing_segment_value == **seg_b);

                    let (due_to_acct, due_from_acct) = if let (Some(la), Some(_lb)) = (line_a, line_b) {
                        (la.due_to_account.clone(), la.due_from_account.clone())
                    } else {
                        (template.default_offset_account.clone(), template.default_offset_account.clone())
                    };

                    // Generate due-to for segment that needs debit offset
                    if *bal_a > 0.0 {
                        // A has surplus → A gives to B → A credits due-to, B debits due-from
                        offset_line_data.push(((*seg_a).clone(), (*seg_b).clone(), "due_to".into(), due_to_acct.clone(), -offset_amount));
                        offset_line_data.push(((*seg_b).clone(), (*seg_a).clone(), "due_from".into(), due_from_acct.clone(), offset_amount));
                    } else {
                        // B has surplus → B gives to A
                        offset_line_data.push(((*seg_b).clone(), (*seg_a).clone(), "due_to".into(), due_to_acct.clone(), -offset_amount));
                        offset_line_data.push(((*seg_a).clone(), (*seg_b).clone(), "due_from".into(), due_from_acct.clone(), offset_amount));
                    }
                }
            }
        }

        let total_debit: f64 = offset_line_data.iter()
            .filter(|(_, _, _, _, amt)| *amt > 0.0)
            .map(|(_, _, _, _, amt)| *amt)
            .sum();
        let total_credit: f64 = offset_line_data.iter()
            .filter(|(_, _, _, _, amt)| *amt < 0.0)
            .map(|(_, _, _, _, amt)| amt.abs())
            .sum();

        // Generate a unique generation number
        let generation_number = format!("AO-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Auto Offset Engine: Generating {} offset lines for {} ({})",
              offset_line_data.len(), generation_number, source_type);

        // Create the generation record
        let generation = self.repository.create_generation(
            org_id, &generation_number, template_id,
            source_type, source_id, source_number,
            fiscal_year, period_name, generation_date, currency_code,
            journal_lines.len() as i32, segments_affected,
            offset_line_data.len() as i32, total_debit, total_credit,
            created_by,
        ).await?;

        // Create offset lines
        for (idx, (from_seg, to_seg, offset_type, account, amount)) in offset_line_data.iter().enumerate() {
            self.repository.add_offset_line(
                org_id, generation.id, (idx + 1) as i32,
                from_seg, to_seg, offset_type, account,
                None, // account description
                amount.abs(), currency_code,
            ).await.ok(); // Best effort
        }

        // Log activity
        self.repository.log_activity(
            org_id, generation.id, None,
            "generated", Some(&format!("Generated {} offset lines", offset_line_data.len())),
            None, Some("generated"), created_by, None,
            serde_json::json!({ "total_debit": total_debit, "total_credit": total_credit }),
        ).await.ok();

        Ok(generation)
    }

    /// Get a generation by ID
    pub async fn get_generation(&self, id: Uuid) -> AtlasResult<Option<AutoOffsetGeneration>> {
        self.repository.get_generation(id).await
    }

    /// Get a generation by number
    pub async fn get_generation_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AutoOffsetGeneration>> {
        self.repository.get_generation_by_number(org_id, number).await
    }

    /// List generations
    pub async fn list_generations(&self, org_id: Uuid, status: Option<&str>, source_type: Option<&str>) -> AtlasResult<Vec<AutoOffsetGeneration>> {
        self.repository.list_generations(org_id, status, source_type).await
    }

    /// Post an offset generation to the GL
    pub async fn post_generation(&self, generation_id: Uuid, posted_by: Uuid) -> AtlasResult<AutoOffsetGeneration> {
        info!("Auto Offset Engine: Posting generation {}", generation_id);

        let gen = self.repository.get_generation(generation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Generation {generation_id} not found")
            ))?;

        if gen.status != "generated" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot post generation in '{}' status. Must be 'generated'.", gen.status
            )));
        }

        let updated = self.repository.update_generation_status(generation_id, "posted", Some(posted_by)).await?;

        self.repository.log_activity(
            gen.organization_id, generation_id, None,
            "posted", Some("Offset generation posted to GL"),
            Some("generated"), Some("posted"), Some(posted_by), None,
            serde_json::json!({}),
        ).await.ok();

        Ok(updated)
    }

    /// Reverse a posted offset generation
    pub async fn reverse_generation(&self, generation_id: Uuid, reversed_by: Uuid) -> AtlasResult<AutoOffsetGeneration> {
        info!("Auto Offset Engine: Reversing generation {}", generation_id);

        let gen = self.repository.get_generation(generation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Generation {generation_id} not found")
            ))?;

        if gen.status != "posted" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot reverse generation in '{}' status. Must be 'posted'.", gen.status
            )));
        }

        let updated = self.repository.update_generation_status(generation_id, "reversed", Some(reversed_by)).await?;

        self.repository.log_activity(
            gen.organization_id, generation_id, None,
            "reversed", Some("Offset generation reversed"),
            Some("posted"), Some("reversed"), Some(reversed_by), None,
            serde_json::json!({}),
        ).await.ok();

        Ok(updated)
    }

    /// Cancel a generated (unposted) offset generation
    pub async fn cancel_generation(&self, generation_id: Uuid, cancelled_by: Uuid) -> AtlasResult<AutoOffsetGeneration> {
        info!("Auto Offset Engine: Cancelling generation {}", generation_id);

        let gen = self.repository.get_generation(generation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Generation {generation_id} not found")
            ))?;

        if gen.status != "generated" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot cancel generation in '{}' status. Must be 'generated'.", gen.status
            )));
        }

        let updated = self.repository.update_generation_status(generation_id, "cancelled", Some(cancelled_by)).await?;

        self.repository.log_activity(
            gen.organization_id, generation_id, None,
            "cancelled", Some("Offset generation cancelled"),
            Some("generated"), Some("cancelled"), Some(cancelled_by), None,
            serde_json::json!({}),
        ).await.ok();

        Ok(updated)
    }

    // ========================================================================
    // Offset Lines & Activities
    // ========================================================================

    /// List offset lines for a generation
    pub async fn list_offset_lines(&self, generation_id: Uuid) -> AtlasResult<Vec<AutoOffsetLine>> {
        self.repository.list_offset_lines(generation_id).await
    }

    /// List activities for a generation
    pub async fn list_activities(&self, generation_id: Uuid) -> AtlasResult<Vec<AutoOffsetActivity>> {
        self.repository.list_activities(generation_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<AutoOffsetDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

/// Represents a source journal line for offset generation
#[derive(Debug, Clone)]
pub struct JournalLine {
    pub balancing_segment_value: String,
    pub account_code: String,
    pub debit: f64,
    pub credit: f64,
}
