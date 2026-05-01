//! Journal Import Engine
//!
//! Core journal import operations:
//! - Import format definition management (create/query formats)
//! - Column mapping management
//! - Import batch creation and data loading
//! - Row-level validation (account codes, amounts, required fields)
//! - Balancing validation (total debits == total credits)
//! - Import workflow (upload → validate → import → post)
//! - Error handling and correction
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Import Journals

use atlas_shared::{
    JournalImportFormat, JournalImportColumnMapping,
    JournalImportBatch, JournalImportRow,
    JournalImportError, JournalImportDashboardSummary,
    AtlasError, AtlasResult,
};
use super::JournalImportRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid source types for import formats
#[allow(dead_code)]
const VALID_SOURCE_TYPES: &[&str] = &[
    "file", "api", "subledger",
];

/// Valid file formats
#[allow(dead_code)]
const VALID_FILE_FORMATS: &[&str] = &[
    "csv", "json", "fixed_width",
];

/// Valid format statuses
#[allow(dead_code)]
const VALID_FORMAT_STATUSES: &[&str] = &[
    "active", "inactive",
];

/// Valid batch statuses
#[allow(dead_code)]
const VALID_BATCH_STATUSES: &[&str] = &[
    "uploaded", "validating", "validated", "importing",
    "completed", "completed_with_errors", "failed",
];

/// Valid row statuses
#[allow(dead_code)]
const VALID_ROW_STATUSES: &[&str] = &[
    "pending", "valid", "error", "imported", "skipped",
];

/// Valid target fields for column mappings
#[allow(dead_code)]
const VALID_TARGET_FIELDS: &[&str] = &[
    "account_code", "account_name", "description",
    "entered_dr", "entered_cr", "currency_code",
    "exchange_rate", "gl_date", "reference",
    "line_type", "cost_center", "department", "project_code",
];

/// Valid data types for column mappings
#[allow(dead_code)]
const VALID_DATA_TYPES: &[&str] = &[
    "string", "number", "date",
];

/// Valid error severities
#[allow(dead_code)]
const VALID_ERROR_SEVERITIES: &[&str] = &[
    "error", "warning",
];

/// Journal Import Engine
pub struct JournalImportEngine {
    repository: Arc<dyn JournalImportRepository>,
}

impl JournalImportEngine {
    pub fn new(repository: Arc<dyn JournalImportRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Import Format Management
    // ========================================================================

    /// Create a new import format definition
    pub async fn create_format(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        source_type: &str,
        file_format: &str,
        delimiter: Option<&str>,
        header_row: bool,
        ledger_id: Option<Uuid>,
        currency_code: &str,
        default_date: Option<chrono::NaiveDate>,
        default_journal_type: Option<&str>,
        balancing_segment: Option<&str>,
        validation_enabled: bool,
        auto_post: bool,
        max_errors_allowed: i32,
        column_mappings: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportFormat> {
        // Validate inputs
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Format code and name are required".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        if !VALID_SOURCE_TYPES.contains(&source_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid source_type '{}'. Must be one of: {}",
                source_type, VALID_SOURCE_TYPES.join(", ")
            )));
        }
        if !VALID_FILE_FORMATS.contains(&file_format) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid file_format '{}'. Must be one of: {}",
                file_format, VALID_FILE_FORMATS.join(", ")
            )));
        }
        if max_errors_allowed < 0 {
            return Err(AtlasError::ValidationFailed(
                "max_errors_allowed must be non-negative".to_string(),
            ));
        }

        // Check uniqueness
        if self.repository.get_format_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(
                format!("Import format code '{}' already exists", code)
            ));
        }

        info!("Creating journal import format '{}' for org {}", code, org_id);

        self.repository.create_format(
            org_id, code, name, description,
            source_type, file_format, delimiter, header_row,
            ledger_id, currency_code, default_date,
            default_journal_type, balancing_segment,
            validation_enabled, auto_post,
            max_errors_allowed, column_mappings, created_by,
        ).await
    }

    /// Get an import format by ID
    pub async fn get_format(&self, id: Uuid) -> AtlasResult<Option<JournalImportFormat>> {
        self.repository.get_format(id).await
    }

    /// Get an import format by code
    pub async fn get_format_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<JournalImportFormat>> {
        self.repository.get_format_by_code(org_id, code).await
    }

    /// List import formats for an organization
    pub async fn list_formats(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalImportFormat>> {
        if let Some(s) = status {
            if !VALID_FORMAT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_FORMAT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_formats(org_id, status).await
    }

    /// Delete (deactivate) an import format
    pub async fn delete_format(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating journal import format '{}' for org {}", code, org_id);
        self.repository.delete_format(org_id, code).await
    }

    // ========================================================================
    // Column Mapping Management
    // ========================================================================

    /// Add a column mapping to an import format
    pub async fn add_column_mapping(
        &self,
        org_id: Uuid,
        format_id: Uuid,
        column_position: i32,
        source_column: &str,
        target_field: &str,
        data_type: &str,
        is_required: bool,
        default_value: Option<&str>,
        transformation: Option<&str>,
        validation_rule: Option<&str>,
    ) -> AtlasResult<JournalImportColumnMapping> {
        if source_column.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Source column name is required".to_string(),
            ));
        }
        if !VALID_TARGET_FIELDS.contains(&target_field) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid target_field '{}'. Must be one of: {}",
                target_field, VALID_TARGET_FIELDS.join(", ")
            )));
        }
        if !VALID_DATA_TYPES.contains(&data_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid data_type '{}'. Must be one of: {}",
                data_type, VALID_DATA_TYPES.join(", ")
            )));
        }

        // Validate format exists and belongs to org
        let format = self.repository.get_format(format_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Import format {} not found", format_id)
            ))?;

        if format.organization_id != org_id {
            return Err(AtlasError::Forbidden(
                "Format does not belong to this organization".to_string(),
            ));
        }

        info!("Adding column mapping '{}' → '{}' to format {}",
            source_column, target_field, format.code);

        self.repository.create_column_mapping(
            org_id, format_id, column_position, source_column,
            target_field, data_type, is_required,
            default_value, transformation, validation_rule,
        ).await
    }

    /// List column mappings for a format
    pub async fn list_column_mappings(&self, format_id: Uuid) -> AtlasResult<Vec<JournalImportColumnMapping>> {
        self.repository.list_column_mappings(format_id).await
    }

    // ========================================================================
    // Import Batch Management
    // ========================================================================

    /// Create a new import batch
    pub async fn create_batch(
        &self,
        org_id: Uuid,
        format_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        source: &str,
        source_file_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportBatch> {
        // Validate format exists and is active
        let format = self.repository.get_format(format_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Import format {} not found", format_id)
            ))?;

        if format.organization_id != org_id {
            return Err(AtlasError::Forbidden(
                "Format does not belong to this organization".to_string(),
            ));
        }

        if format.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot create batch for inactive format '{}'", format.code)
            ));
        }

        let batch_number = format!("JI-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating journal import batch {} using format '{}'", batch_number, format.code);

        self.repository.create_batch(
            org_id, format_id, &batch_number, name, description,
            source, source_file_name,
            format.ledger_id, &format.currency_code,
            created_by,
        ).await
    }

    /// Get an import batch by ID
    pub async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<JournalImportBatch>> {
        self.repository.get_batch(id).await
    }

    /// Get an import batch by number
    pub async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<JournalImportBatch>> {
        self.repository.get_batch_by_number(org_id, batch_number).await
    }

    /// List import batches
    pub async fn list_batches(
        &self,
        org_id: Uuid,
        format_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<JournalImportBatch>> {
        if let Some(s) = status {
            if !VALID_BATCH_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_BATCH_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_batches(org_id, format_id, status).await
    }

    // ========================================================================
    // Import Data (Row) Management
    // ========================================================================

    /// Add a row to an import batch
    pub async fn add_row(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        raw_data: serde_json::Value,
        account_code: Option<&str>,
        account_name: Option<&str>,
        description: Option<&str>,
        entered_dr: &str,
        entered_cr: &str,
        currency_code: Option<&str>,
        exchange_rate: Option<&str>,
        gl_date: Option<chrono::NaiveDate>,
        reference: Option<&str>,
        line_type: Option<&str>,
        cost_center: Option<&str>,
        department: Option<&str>,
        project_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportRow> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Import batch {} not found", batch_id)
            ))?;

        if batch.organization_id != org_id {
            return Err(AtlasError::Forbidden(
                "Batch does not belong to this organization".to_string(),
            ));
        }

        if batch.status != "uploaded" && batch.status != "validating" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add rows to batch in '{}' status. Must be 'uploaded'.", batch.status)
            ));
        }

        // Validate amounts
        let dr: f64 = entered_dr.parse().map_err(|_| AtlasError::ValidationFailed(
            "entered_dr must be a valid number".to_string(),
        ))?;
        let cr: f64 = entered_cr.parse().map_err(|_| AtlasError::ValidationFailed(
            "entered_cr must be a valid number".to_string(),
        ))?;

        if dr < 0.0 || cr < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Debit and credit amounts must be non-negative".to_string(),
            ));
        }

        if dr > 0.0 && cr > 0.0 {
            return Err(AtlasError::ValidationFailed(
                "A row cannot have both debit and credit amounts".to_string(),
            ));
        }

        let rows = self.repository.list_batch_rows(batch_id).await?;
        let row_number = (rows.len() as i32) + 1;

        info!("Adding row {} to import batch {}", row_number, batch.batch_number);

        self.repository.create_row(
            org_id, batch_id, row_number, raw_data,
            account_code, account_name, description,
            &format!("{:.2}", dr), &format!("{:.2}", cr),
            currency_code, exchange_rate, gl_date,
            reference, line_type, cost_center,
            department, project_code, "pending",
            None, None,
            created_by,
        ).await
    }

    /// List rows in a batch
    pub async fn list_batch_rows(&self, batch_id: Uuid) -> AtlasResult<Vec<JournalImportRow>> {
        self.repository.list_batch_rows(batch_id).await
    }

    /// Get a specific row
    pub async fn get_row(&self, id: Uuid) -> AtlasResult<Option<JournalImportRow>> {
        self.repository.get_row(id).await
    }

    /// Update a row (for error correction)
    pub async fn update_row(
        &self,
        row_id: Uuid,
        account_code: Option<&str>,
        description: Option<&str>,
        entered_dr: Option<&str>,
        entered_cr: Option<&str>,
    ) -> AtlasResult<JournalImportRow> {
        let row = self.repository.get_row(row_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Import row {} not found", row_id)
            ))?;

        if row.status != "error" && row.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot update row in '{}' status. Must be 'error' or 'pending'.", row.status)
            ));
        }

        let new_dr = entered_dr.unwrap_or(&row.entered_dr);
        let new_cr = entered_cr.unwrap_or(&row.entered_cr);

        let dr: f64 = new_dr.parse().map_err(|_| AtlasError::ValidationFailed(
            "entered_dr must be a valid number".to_string(),
        ))?;
        let cr: f64 = new_cr.parse().map_err(|_| AtlasError::ValidationFailed(
            "entered_cr must be a valid number".to_string(),
        ))?;

        if dr < 0.0 || cr < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Debit and credit amounts must be non-negative".to_string(),
            ));
        }

        info!("Updating import row {} for correction", row_id);

        self.repository.update_row(
            row_id,
            account_code,
            description,
            Some(&format!("{:.2}", dr)),
            Some(&format!("{:.2}", cr)),
            "pending",
            None,
            None,
        ).await
    }

    // ========================================================================
    // Import Validation
    // ========================================================================

    /// Validate all rows in a batch
    pub async fn validate_batch(&self, batch_id: Uuid) -> AtlasResult<JournalImportBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Import batch {} not found", batch_id)
            ))?;

        if batch.status != "uploaded" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot validate batch in '{}' status. Must be 'uploaded'.", batch.status)
            ));
        }

        info!("Validating import batch {}", batch.batch_number);

        // Update status to validating
        self.repository.update_batch_status(batch_id, "validating", None, None).await?;

        let rows = self.repository.list_batch_rows(batch_id).await?;
        let mut valid_count = 0i32;
        let mut error_count = 0i32;
        let mut total_debit = 0.0_f64;
        let mut total_credit = 0.0_f64;
        let mut errors: Vec<JournalImportError> = Vec::new();

        for row in &rows {
            let mut row_errors: Vec<JournalImportError> = Vec::new();
            let row_debit: f64;
            let row_credit: f64;

            // Validate account code
            if row.account_code.is_none() || row.account_code.as_ref().map_or(true, |s| s.is_empty()) {
                row_errors.push(JournalImportError {
                    row_number: row.row_number,
                    field: "account_code".to_string(),
                    error: "Account code is required".to_string(),
                    severity: "error".to_string(),
                    raw_value: row.account_code.clone(),
                });
            }

            // Validate amounts
            let dr: f64 = row.entered_dr.parse().unwrap_or(0.0);
            let cr: f64 = row.entered_cr.parse().unwrap_or(0.0);

            if dr < 0.0 {
                row_errors.push(JournalImportError {
                    row_number: row.row_number,
                    field: "entered_dr".to_string(),
                    error: "Debit amount cannot be negative".to_string(),
                    severity: "error".to_string(),
                    raw_value: Some(row.entered_dr.clone()),
                });
            }

            if cr < 0.0 {
                row_errors.push(JournalImportError {
                    row_number: row.row_number,
                    field: "entered_cr".to_string(),
                    error: "Credit amount cannot be negative".to_string(),
                    severity: "error".to_string(),
                    raw_value: Some(row.entered_cr.clone()),
                });
            }

            // Validate that row has either debit or credit
            if dr.abs() < 0.01 && cr.abs() < 0.01 {
                row_errors.push(JournalImportError {
                    row_number: row.row_number,
                    field: "entered_dr".to_string(),
                    error: "Row must have either a debit or credit amount".to_string(),
                    severity: "error".to_string(),
                    raw_value: None,
                });
            }

            // Validate not both debit and credit
            if dr > 0.0 && cr > 0.0 {
                row_errors.push(JournalImportError {
                    row_number: row.row_number,
                    field: "entered_cr".to_string(),
                    error: "Row cannot have both debit and credit amounts".to_string(),
                    severity: "error".to_string(),
                    raw_value: Some(row.entered_cr.clone()),
                });
            }

            if row_errors.is_empty() {
                row_debit = dr;
                row_credit = cr;
                total_debit += row_debit;
                total_credit += row_credit;
                valid_count += 1;

                self.repository.update_row(
                    row.id, None, None, None, None,
                    "valid", None, None,
                ).await?;
            } else {
                error_count += 1;
                errors.extend(row_errors);

                let first_error = errors.iter()
                    .filter(|e| e.row_number == row.row_number)
                    .next();

                self.repository.update_row(
                    row.id, None, None, None, None,
                    "error",
                    first_error.map(|e| e.error.as_str()),
                    first_error.map(|e| e.field.as_str()),
                ).await?;
            }
        }

        // Check balancing
        let is_balanced = (total_debit - total_credit).abs() < 0.01;

        // Determine final status
        let format = self.repository.get_format(batch.format_id).await?;
        let max_errors = format.map_or(100, |f| f.max_errors_allowed);

        let new_status = if error_count == 0 {
            "validated"
        } else if error_count <= max_errors {
            "validated" // Still valid if under max errors
        } else {
            "failed"
        };

        let error_count_i32 = error_count;
        let imported = 0i32;

        self.repository.update_batch_totals(
            batch_id,
            rows.len() as i32,
            valid_count,
            error_count_i32,
            imported,
            &format!("{:.2}", total_debit),
            &format!("{:.2}", total_credit),
            is_balanced,
            serde_json::to_value(&errors).unwrap_or(serde_json::json!([])),
        ).await?;

        info!("Validation complete for batch {}: {} valid, {} errors, balanced={}",
            batch.batch_number, valid_count, error_count, is_balanced);

        self.repository.update_batch_status(batch_id, new_status, None, None).await
    }

    // ========================================================================
    // Import Processing
    // ========================================================================

    /// Import validated rows into journal entries
    pub async fn import_batch(&self, batch_id: Uuid) -> AtlasResult<JournalImportBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Import batch {} not found", batch_id)
            ))?;

        if batch.status != "validated" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot import batch in '{}' status. Must be 'validated'.", batch.status)
            ));
        }

        info!("Importing journal entries from batch {}", batch.batch_number);

        self.repository.update_batch_status(batch_id, "importing", None, None).await?;

        let rows = self.repository.list_batch_rows(batch_id).await?;
        let valid_rows: Vec<&JournalImportRow> = rows.iter()
            .filter(|r| r.status == "valid")
            .collect();

        let mut imported_count = 0i32;

        for row in &valid_rows {
            // In a full implementation, this would create GL journal entries
            // via the GeneralLedgerEngine. For now, mark rows as imported.
            self.repository.update_row(
                row.id, None, None, None, None,
                "imported", None, None,
            ).await?;

            imported_count += 1;
        }

        let error_rows = rows.iter().filter(|r| r.status == "error").count() as i32;

        let new_status = if error_rows > 0 {
            "completed_with_errors"
        } else {
            "completed"
        };

        self.repository.update_batch_totals(
            batch_id,
            rows.len() as i32,
            valid_rows.len() as i32,
            error_rows,
            imported_count,
            &batch.total_debit,
            &batch.total_credit,
            batch.is_balanced,
            batch.errors.clone(),
        ).await?;

        info!("Import complete for batch {}: {} rows imported, {} errors",
            batch.batch_number, imported_count, error_rows);

        self.repository.update_batch_status(batch_id, new_status, None, Some(chrono::Utc::now())).await
    }

    /// Delete a batch (only if not completed)
    pub async fn delete_batch(&self, batch_id: Uuid) -> AtlasResult<()> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Import batch {} not found", batch_id)
            ))?;

        if batch.status == "completed" || batch.status == "completed_with_errors" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot delete batch in '{}' status.", batch.status)
            ));
        }

        info!("Deleting import batch {}", batch.batch_number);

        // Delete all rows first
        self.repository.delete_batch_rows(batch_id).await?;
        self.repository.delete_batch(batch_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get journal import dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<JournalImportDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Pure Validation Helpers (for unit testing)
    // ========================================================================

    /// Validate a single row's data without database
    pub fn validate_row_data(
        account_code: Option<&str>,
        entered_dr: &str,
        entered_cr: &str,
    ) -> Vec<JournalImportError> {
        let mut errors = Vec::new();

        if account_code.is_none() || account_code.map_or(true, |s| s.is_empty()) {
            errors.push(JournalImportError {
                row_number: 0,
                field: "account_code".to_string(),
                error: "Account code is required".to_string(),
                severity: "error".to_string(),
                raw_value: account_code.map(|s| s.to_string()),
            });
        }

        let dr: f64 = entered_dr.parse().unwrap_or(-1.0);
        let cr: f64 = entered_cr.parse().unwrap_or(-1.0);

        if dr < 0.0 {
            errors.push(JournalImportError {
                row_number: 0,
                field: "entered_dr".to_string(),
                error: "Debit amount must be non-negative".to_string(),
                severity: "error".to_string(),
                raw_value: Some(entered_dr.to_string()),
            });
        }

        if cr < 0.0 {
            errors.push(JournalImportError {
                row_number: 0,
                field: "entered_cr".to_string(),
                error: "Credit amount must be non-negative".to_string(),
                severity: "error".to_string(),
                raw_value: Some(entered_cr.to_string()),
            });
        }

        if dr.abs() < 0.01 && cr.abs() < 0.01 {
            errors.push(JournalImportError {
                row_number: 0,
                field: "entered_dr".to_string(),
                error: "Row must have either a debit or credit amount".to_string(),
                severity: "error".to_string(),
                raw_value: None,
            });
        }

        if dr > 0.0 && cr > 0.0 {
            errors.push(JournalImportError {
                row_number: 0,
                field: "entered_cr".to_string(),
                error: "Row cannot have both debit and credit amounts".to_string(),
                severity: "error".to_string(),
                raw_value: Some(entered_cr.to_string()),
            });
        }

        errors
    }

    /// Check if a batch is balanced given its total debit and credit
    pub fn check_balancing(total_debit: f64, total_credit: f64) -> bool {
        (total_debit - total_credit).abs() < 0.01
    }

    /// Parse an amount string, returning 0.0 on failure
    pub fn parse_amount(amount: &str) -> f64 {
        amount.parse().unwrap_or(0.0)
    }

    /// Calculate totals from a list of (debit, credit) tuples
    pub fn calculate_totals(rows: &[(f64, f64)]) -> (f64, f64) {
        let total_debit: f64 = rows.iter().map(|(dr, _)| *dr).sum();
        let total_credit: f64 = rows.iter().map(|(_, cr)| *cr).sum();
        (total_debit, total_credit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_source_types() {
        assert!(VALID_SOURCE_TYPES.contains(&"file"));
        assert!(VALID_SOURCE_TYPES.contains(&"api"));
        assert!(VALID_SOURCE_TYPES.contains(&"subledger"));
        assert_eq!(VALID_SOURCE_TYPES.len(), 3);
    }

    #[test]
    fn test_valid_file_formats() {
        assert!(VALID_FILE_FORMATS.contains(&"csv"));
        assert!(VALID_FILE_FORMATS.contains(&"json"));
        assert!(VALID_FILE_FORMATS.contains(&"fixed_width"));
        assert_eq!(VALID_FILE_FORMATS.len(), 3);
    }

    #[test]
    fn test_valid_format_statuses() {
        assert!(VALID_FORMAT_STATUSES.contains(&"active"));
        assert!(VALID_FORMAT_STATUSES.contains(&"inactive"));
        assert_eq!(VALID_FORMAT_STATUSES.len(), 2);
    }

    #[test]
    fn test_valid_batch_statuses() {
        assert!(VALID_BATCH_STATUSES.contains(&"uploaded"));
        assert!(VALID_BATCH_STATUSES.contains(&"validating"));
        assert!(VALID_BATCH_STATUSES.contains(&"validated"));
        assert!(VALID_BATCH_STATUSES.contains(&"importing"));
        assert!(VALID_BATCH_STATUSES.contains(&"completed"));
        assert!(VALID_BATCH_STATUSES.contains(&"completed_with_errors"));
        assert!(VALID_BATCH_STATUSES.contains(&"failed"));
        assert_eq!(VALID_BATCH_STATUSES.len(), 7);
    }

    #[test]
    fn test_valid_row_statuses() {
        assert!(VALID_ROW_STATUSES.contains(&"pending"));
        assert!(VALID_ROW_STATUSES.contains(&"valid"));
        assert!(VALID_ROW_STATUSES.contains(&"error"));
        assert!(VALID_ROW_STATUSES.contains(&"imported"));
        assert!(VALID_ROW_STATUSES.contains(&"skipped"));
        assert_eq!(VALID_ROW_STATUSES.len(), 5);
    }

    #[test]
    fn test_valid_target_fields() {
        assert!(VALID_TARGET_FIELDS.contains(&"account_code"));
        assert!(VALID_TARGET_FIELDS.contains(&"entered_dr"));
        assert!(VALID_TARGET_FIELDS.contains(&"entered_cr"));
        assert!(VALID_TARGET_FIELDS.contains(&"description"));
        assert!(VALID_TARGET_FIELDS.contains(&"currency_code"));
        assert!(VALID_TARGET_FIELDS.contains(&"gl_date"));
        assert!(VALID_TARGET_FIELDS.contains(&"reference"));
        assert!(VALID_TARGET_FIELDS.contains(&"line_type"));
        assert!(VALID_TARGET_FIELDS.contains(&"cost_center"));
        assert!(VALID_TARGET_FIELDS.contains(&"department"));
        assert!(VALID_TARGET_FIELDS.contains(&"project_code"));
        assert_eq!(VALID_TARGET_FIELDS.len(), 13);
    }

    #[test]
    fn test_valid_data_types() {
        assert!(VALID_DATA_TYPES.contains(&"string"));
        assert!(VALID_DATA_TYPES.contains(&"number"));
        assert!(VALID_DATA_TYPES.contains(&"date"));
        assert_eq!(VALID_DATA_TYPES.len(), 3);
    }

    #[test]
    fn test_valid_error_severities() {
        assert!(VALID_ERROR_SEVERITIES.contains(&"error"));
        assert!(VALID_ERROR_SEVERITIES.contains(&"warning"));
        assert_eq!(VALID_ERROR_SEVERITIES.len(), 2);
    }

    // ========================================================================
    // Row Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_row_data_valid_debit() {
        let errors = JournalImportEngine::validate_row_data(
            Some("1000"), "100.00", "0.00",
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_row_data_valid_credit() {
        let errors = JournalImportEngine::validate_row_data(
            Some("2000"), "0.00", "250.00",
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_row_data_missing_account() {
        let errors = JournalImportEngine::validate_row_data(
            None, "100.00", "0.00",
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "account_code");
    }

    #[test]
    fn test_validate_row_data_empty_account() {
        let errors = JournalImportEngine::validate_row_data(
            Some(""), "100.00", "0.00",
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "account_code");
    }

    #[test]
    fn test_validate_row_data_negative_debit() {
        let errors = JournalImportEngine::validate_row_data(
            Some("1000"), "-50.00", "0.00",
        );
        assert!(errors.iter().any(|e| e.field == "entered_dr"));
    }

    #[test]
    fn test_validate_row_data_negative_credit() {
        let errors = JournalImportEngine::validate_row_data(
            Some("1000"), "0.00", "-75.00",
        );
        assert!(errors.iter().any(|e| e.field == "entered_cr"));
    }

    #[test]
    fn test_validate_row_data_zero_amounts() {
        let errors = JournalImportEngine::validate_row_data(
            Some("1000"), "0.00", "0.00",
        );
        assert!(errors.iter().any(|e| e.error.contains("either a debit or credit")));
    }

    #[test]
    fn test_validate_row_data_both_dr_cr() {
        let errors = JournalImportEngine::validate_row_data(
            Some("1000"), "100.00", "50.00",
        );
        assert!(errors.iter().any(|e| e.error.contains("both debit and credit")));
    }

    #[test]
    fn test_validate_row_data_invalid_dr_number() {
        let errors = JournalImportEngine::validate_row_data(
            Some("1000"), "abc", "0.00",
        );
        assert!(errors.iter().any(|e| e.field == "entered_dr"));
    }

    #[test]
    fn test_validate_row_data_invalid_cr_number() {
        let errors = JournalImportEngine::validate_row_data(
            Some("1000"), "0.00", "xyz",
        );
        assert!(errors.iter().any(|e| e.field == "entered_cr"));
    }

    #[test]
    fn test_validate_row_data_multiple_errors() {
        let errors = JournalImportEngine::validate_row_data(
            None, "-10.00", "-20.00",
        );
        // Missing account + negative debit + negative credit
        assert!(errors.len() >= 3);
    }

    // ========================================================================
    // Balancing Tests
    // ========================================================================

    #[test]
    fn test_check_balancing_balanced() {
        assert!(JournalImportEngine::check_balancing(1000.0, 1000.0));
    }

    #[test]
    fn test_check_balancing_unbalanced() {
        assert!(!JournalImportEngine::check_balancing(1000.0, 999.0));
    }

    #[test]
    fn test_check_balancing_near_zero() {
        assert!(JournalImportEngine::check_balancing(1000.0, 1000.005));
    }

    #[test]
    fn test_check_balancing_both_zero() {
        assert!(JournalImportEngine::check_balancing(0.0, 0.0));
    }

    // ========================================================================
    // Amount Parsing Tests
    // ========================================================================

    #[test]
    fn test_parse_amount_valid() {
        assert!((JournalImportEngine::parse_amount("123.45") - 123.45).abs() < 0.001);
        assert!((JournalImportEngine::parse_amount("0.00") - 0.0).abs() < 0.001);
        assert!((JournalImportEngine::parse_amount("1000000.99") - 1000000.99).abs() < 0.001);
    }

    #[test]
    fn test_parse_amount_invalid() {
        assert_eq!(JournalImportEngine::parse_amount("abc"), 0.0);
        assert_eq!(JournalImportEngine::parse_amount(""), 0.0);
    }

    // ========================================================================
    // Totals Calculation Tests
    // ========================================================================

    #[test]
    fn test_calculate_totals_balanced() {
        let rows = vec![
            (1000.0, 0.0),
            (0.0, 500.0),
            (0.0, 500.0),
        ];
        let (dr, cr) = JournalImportEngine::calculate_totals(&rows);
        assert!((dr - 1000.0).abs() < 0.01);
        assert!((cr - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_totals_unbalanced() {
        let rows = vec![
            (1000.0, 0.0),
            (0.0, 300.0),
        ];
        let (dr, cr) = JournalImportEngine::calculate_totals(&rows);
        assert!((dr - 1000.0).abs() < 0.01);
        assert!((cr - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_totals_empty() {
        let rows: Vec<(f64, f64)> = vec![];
        let (dr, cr) = JournalImportEngine::calculate_totals(&rows);
        assert!((dr - 0.0).abs() < 0.01);
        assert!((cr - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_totals_large_batch() {
        let rows: Vec<(f64, f64)> = (0..1000)
            .map(|i| if i % 2 == 0 { (100.0, 0.0) } else { (0.0, 100.0) })
            .collect();
        let (dr, cr) = JournalImportEngine::calculate_totals(&rows);
        assert!((dr - 50000.0).abs() < 0.01);
        assert!((cr - 50000.0).abs() < 0.01);
    }
}
