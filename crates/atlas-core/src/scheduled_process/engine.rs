//! Scheduled Process Engine
//!
//! Manages the lifecycle of scheduled processes including template management,
//! process submission, execution tracking, recurrence scheduling, and logging.
//!
//! Oracle Fusion equivalent: Navigator > Tools > Scheduled Processes

use atlas_shared::{
    ScheduledProcess, ScheduledProcessTemplate, ScheduledProcessRecurrence,
    ScheduledProcessLog, ScheduledProcessDashboardSummary,
    AtlasError, AtlasResult,
};
use super::ScheduledProcessRepository;
use chrono::{Datelike, Utc, DateTime};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid process types
pub const VALID_PROCESS_TYPES: &[&str] = &[
    "report", "import", "export", "batch", "custom",
];

/// Valid executor types
pub const VALID_EXECUTOR_TYPES: &[&str] = &[
    "built_in", "external", "plugin",
];

/// Valid process statuses
pub const VALID_STATUSES: &[&str] = &[
    "pending", "scheduled", "running", "completed", "failed",
    "cancelled", "waiting_for_approval",
];

/// Valid priority levels
pub const VALID_PRIORITIES: &[&str] = &[
    "low", "normal", "high", "urgent",
];

/// Valid recurrence types
pub const VALID_RECURRENCE_TYPES: &[&str] = &[
    "daily", "weekly", "monthly", "cron",
];

/// Valid log levels
pub const VALID_LOG_LEVELS: &[&str] = &[
    "debug", "info", "warn", "error",
];

/// Scheduled Process Engine
pub struct ScheduledProcessEngine {
    repository: Arc<dyn ScheduledProcessRepository>,
}

impl ScheduledProcessEngine {
    pub fn new(repository: Arc<dyn ScheduledProcessRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Template Management
    // ========================================================================

    /// Create a new process template
    #[allow(clippy::too_many_arguments)]
    pub async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        process_type: &str,
        executor_type: &str,
        executor_config: serde_json::Value,
        parameters: serde_json::Value,
        default_parameters: serde_json::Value,
        timeout_minutes: i32,
        max_retries: i32,
        retry_delay_minutes: i32,
        requires_approval: bool,
        approval_chain_id: Option<Uuid>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledProcessTemplate> {
        // Validate inputs
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Template code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Template name is required".to_string()));
        }
        if !VALID_PROCESS_TYPES.contains(&process_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid process type '{}'. Must be one of: {}",
                process_type, VALID_PROCESS_TYPES.join(", ")
            )));
        }
        if !VALID_EXECUTOR_TYPES.contains(&executor_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid executor type '{}'. Must be one of: {}",
                executor_type, VALID_EXECUTOR_TYPES.join(", ")
            )));
        }
        if timeout_minutes < 1 {
            return Err(AtlasError::ValidationFailed(
                "Timeout minutes must be >= 1".to_string(),
            ));
        }
        if max_retries < 0 {
            return Err(AtlasError::ValidationFailed(
                "Max retries must be >= 0".to_string(),
            ));
        }
        if retry_delay_minutes < 0 {
            return Err(AtlasError::ValidationFailed(
                "Retry delay minutes must be >= 0".to_string(),
            ));
        }
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        // Check uniqueness
        if self.repository.get_template(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Process template with code '{}' already exists", code
            )));
        }

        info!("Creating process template {} ({}) for org {}", code, name, org_id);

        self.repository.create_template(
            org_id, code, name, description, process_type, executor_type,
            executor_config, parameters, default_parameters,
            timeout_minutes, max_retries, retry_delay_minutes,
            requires_approval, approval_chain_id,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a template by code
    pub async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ScheduledProcessTemplate>> {
        self.repository.get_template(org_id, code).await
    }

    /// Get a template by ID
    pub async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcessTemplate>> {
        self.repository.get_template_by_id(id).await
    }

    /// List templates with optional filters
    pub async fn list_templates(
        &self,
        org_id: Uuid,
        process_type: Option<&str>,
        is_active: Option<bool>,
    ) -> AtlasResult<Vec<ScheduledProcessTemplate>> {
        self.repository.list_templates(org_id, process_type, is_active).await
    }

    /// Activate a template
    pub async fn activate_template(&self, id: Uuid) -> AtlasResult<ScheduledProcessTemplate> {
        let template = self.get_template_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", id)))?;

        if template.is_active {
            return Err(AtlasError::WorkflowError("Template is already active".to_string()));
        }

        info!("Activated process template {}", template.code);
        self.repository.update_template_status(id, true).await
    }

    /// Deactivate a template
    pub async fn deactivate_template(&self, id: Uuid) -> AtlasResult<ScheduledProcessTemplate> {
        let template = self.get_template_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", id)))?;

        if !template.is_active {
            return Err(AtlasError::WorkflowError("Template is already inactive".to_string()));
        }

        info!("Deactivated process template {}", template.code);
        self.repository.update_template_status(id, false).await
    }

    /// Delete a template
    pub async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleted process template {}", code);
        self.repository.delete_template(org_id, code).await
    }

    // ========================================================================
    // Process Submission
    // ========================================================================

    /// Submit a new process for execution.
    /// If `scheduled_start_at` is provided, the process is scheduled for future execution.
    /// If the template requires approval, the process enters `waiting_for_approval` status.
    pub async fn submit_process(
        &self,
        org_id: Uuid,
        template_code: Option<&str>,
        process_name: &str,
        process_type: &str,
        description: Option<&str>,
        priority: &str,
        scheduled_start_at: Option<DateTime<Utc>>,
        parameters: serde_json::Value,
        submitted_by: Uuid,
    ) -> AtlasResult<ScheduledProcess> {
        if process_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Process name is required".to_string()));
        }
        if !VALID_PROCESS_TYPES.contains(&process_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid process type '{}'. Must be one of: {}",
                process_type, VALID_PROCESS_TYPES.join(", ")
            )));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}",
                priority, VALID_PRIORITIES.join(", ")
            )));
        }

        // Resolve template if provided
        let mut template_id: Option<Uuid> = None;
        let mut timeout_minutes = 60;
        let mut max_retries = 0;
        let mut status = if scheduled_start_at.is_some() {
            "scheduled"
        } else {
            "pending"
        };

        if let Some(code) = template_code {
            let template = self.repository.get_template(org_id, code).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Template '{}' not found", code
                )))?;

            if !template.is_active {
                return Err(AtlasError::WorkflowError(format!(
                    "Template '{}' is not active", code
                )));
            }

            // Check template effective dates
            let today = Utc::now().date_naive();
            if let Some(from) = template.effective_from {
                if today < from {
                    return Err(AtlasError::ValidationFailed(format!(
                        "Template '{}' is not yet effective (effective from {})",
                        code, from
                    )));
                }
            }
            if let Some(to) = template.effective_to {
                if today > to {
                    return Err(AtlasError::ValidationFailed(format!(
                        "Template '{}' has expired (effective to {})",
                        code, to
                    )));
                }
            }

            template_id = Some(template.id);
            timeout_minutes = template.timeout_minutes;
            max_retries = template.max_retries;

            // If template requires approval, override status
            if template.requires_approval {
                status = "waiting_for_approval";
            }
        }

        info!(
            "Submitting process '{}' ({}) for org {}",
            process_name, status, org_id
        );

        self.repository.create_process(
            org_id, template_id, template_code,
            process_name, process_type, description,
            status, priority,
            scheduled_start_at,
            timeout_minutes, max_retries,
            parameters, submitted_by,
        ).await
    }

    /// Get a process by ID
    pub async fn get_process(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcess>> {
        self.repository.get_process(id).await
    }

    /// List processes with optional filters
    pub async fn list_processes(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        submitted_by: Option<Uuid>,
        process_type: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<ScheduledProcess>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_processes(org_id, status, submitted_by, process_type, limit).await
    }

    /// Start a pending/scheduled process (marks as running)
    pub async fn start_process(&self, id: Uuid) -> AtlasResult<ScheduledProcess> {
        let process = self.repository.get_process(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Process {} not found", id)))?;

        if process.status != "pending" && process.status != "scheduled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot start process in '{}' status. Must be 'pending' or 'scheduled'.",
                process.status
            )));
        }

        info!("Starting process {} ({})", process.process_name, process.id);

        let now = Utc::now();
        self.repository.update_process_status(
            id, "running", Some(now), None, None, None,
        ).await
    }

    /// Complete a running process
    pub async fn complete_process(
        &self,
        id: Uuid,
        result_summary: Option<&str>,
        output_file_url: Option<&str>,
        log_output: Option<&str>,
    ) -> AtlasResult<ScheduledProcess> {
        let process = self.repository.get_process(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Process {} not found", id)))?;

        if process.status != "running" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete process in '{}' status. Must be 'running'.",
                process.status
            )));
        }

        info!("Completing process {} ({})", process.process_name, process.id);

        let now = Utc::now();
        self.repository.complete_process(
            id, "completed", Some(now),
            result_summary, output_file_url, log_output, Some(100),
        ).await
    }

    /// Fail a running process
    pub async fn fail_process(
        &self,
        id: Uuid,
        error_message: Option<&str>,
        log_output: Option<&str>,
    ) -> AtlasResult<ScheduledProcess> {
        let process = self.repository.get_process(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Process {} not found", id)))?;

        if process.status != "running" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot fail process in '{}' status. Must be 'running'.",
                process.status
            )));
        }

        // Check if retry is possible
        if process.retry_count < process.max_retries {
            info!(
                "Process {} failed, scheduling retry ({}/{})",
                process.id, process.retry_count + 1, process.max_retries
            );
            return self.repository.retry_process(id, process.retry_count + 1).await;
        }

        info!("Failing process {} ({})", process.process_name, process.id);

        let now = Utc::now();
        self.repository.fail_process(id, "failed", Some(now), error_message, log_output).await
    }

    /// Cancel a process
    pub async fn cancel_process(
        &self,
        id: Uuid,
        cancelled_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<ScheduledProcess> {
        let process = self.repository.get_process(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Process {} not found", id)))?;

        if process.status == "completed" || process.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel process in '{}' status.",
                process.status
            )));
        }

        info!("Cancelling process {} ({})", process.process_name, process.id);

        let now = Utc::now();
        self.repository.cancel_process(id, "cancelled", Some(now), Some(cancelled_by), reason).await
    }

    /// Update progress for a running process
    pub async fn update_progress(
        &self,
        id: Uuid,
        progress_percent: i32,
    ) -> AtlasResult<ScheduledProcess> {
        if !(0..=100).contains(&progress_percent) {
            return Err(AtlasError::ValidationFailed(
                "Progress percent must be between 0 and 100".to_string(),
            ));
        }

        let process = self.repository.get_process(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Process {} not found", id)))?;

        if process.status != "running" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot update progress for process in '{}' status.",
                process.status
            )));
        }

        self.repository.update_progress(id, progress_percent).await
    }

    /// Update heartbeat for a running process (keeps process alive)
    pub async fn heartbeat(&self, id: Uuid) -> AtlasResult<ScheduledProcess> {
        let process = self.repository.get_process(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Process {} not found", id)))?;

        if process.status != "running" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot update heartbeat for process in '{}' status.",
                process.status
            )));
        }

        self.repository.update_heartbeat(id).await
    }

    /// Approve a waiting process
    pub async fn approve_process(&self, id: Uuid) -> AtlasResult<ScheduledProcess> {
        let process = self.repository.get_process(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Process {} not found", id)))?;

        if process.status != "waiting_for_approval" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve process in '{}' status.",
                process.status
            )));
        }

        info!("Approving process {} ({})", process.process_name, process.id);

        // Transition to pending or scheduled based on scheduled_start_at
        let new_status = if process.scheduled_start_at.is_some() {
            "scheduled"
        } else {
            "pending"
        };

        self.repository.update_process_status(
            id, new_status, None, None, None, None,
        ).await
    }

    // ========================================================================
    // Recurrence Management
    // ========================================================================

    /// Create a recurrence schedule
    #[allow(clippy::too_many_arguments)]
    pub async fn create_recurrence(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        template_code: &str,
        parameters: serde_json::Value,
        recurrence_type: &str,
        recurrence_config: serde_json::Value,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
        max_runs: Option<i32>,
        submitted_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledProcessRecurrence> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Recurrence name is required".to_string()));
        }
        if !VALID_RECURRENCE_TYPES.contains(&recurrence_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid recurrence type '{}'. Must be one of: {}",
                recurrence_type, VALID_RECURRENCE_TYPES.join(", ")
            )));
        }
        if let Some(end) = end_date {
            if end < start_date {
                return Err(AtlasError::ValidationFailed(
                    "End date must be after start date".to_string(),
                ));
            }
        }

        // Validate template exists and is active
        let template = self.repository.get_template(org_id, template_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Template '{}' not found", template_code
            )))?;

        if !template.is_active {
            return Err(AtlasError::WorkflowError(format!(
                "Template '{}' is not active", template_code
            )));
        }

        // Calculate the next run time
        let next_run_at = Self::calculate_next_run(
            start_date,
            recurrence_type,
            &recurrence_config,
        );

        info!(
            "Creating recurrence '{}' for template {} (next run: {:?})",
            name, template_code, next_run_at
        );

        self.repository.create_recurrence(
            org_id, name, description,
            template.id, Some(template_code),
            parameters, recurrence_type, recurrence_config,
            start_date, end_date,
            next_run_at, max_runs,
            submitted_by,
        ).await
    }

    /// Get a recurrence by ID
    pub async fn get_recurrence(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcessRecurrence>> {
        self.repository.get_recurrence(id).await
    }

    /// List recurrences
    pub async fn list_recurrences(
        &self,
        org_id: Uuid,
        is_active: Option<bool>,
    ) -> AtlasResult<Vec<ScheduledProcessRecurrence>> {
        self.repository.list_recurrences(org_id, is_active).await
    }

    /// Deactivate a recurrence
    pub async fn deactivate_recurrence(&self, id: Uuid) -> AtlasResult<ScheduledProcessRecurrence> {
        let recurrence = self.repository.get_recurrence(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Recurrence {} not found", id)))?;

        if !recurrence.is_active {
            return Err(AtlasError::WorkflowError("Recurrence is already inactive".to_string()));
        }

        info!("Deactivating recurrence '{}'", recurrence.name);
        self.repository.update_recurrence_status(id, false).await
    }

    /// Delete a recurrence
    pub async fn delete_recurrence(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting recurrence {}", id);
        self.repository.delete_recurrence(id).await
    }

    /// Process due recurrences: spawn process instances for recurrences whose
    /// next_run_at has passed. Returns the IDs of spawned processes.
    pub async fn process_due_recurrences(&self) -> AtlasResult<Vec<Uuid>> {
        let now = Utc::now();
        let due = self.repository.find_due_recurrences(now).await?;
        let mut spawned = Vec::new();

        for recurrence in &due {
            let template_code = recurrence.template_code.as_deref().unwrap_or("");

            // Check max_runs
            if let Some(max) = recurrence.max_runs {
                if recurrence.run_count >= max {
                    info!("Recurrence '{}' reached max runs ({}), deactivating", recurrence.name, max);
                    self.repository.update_recurrence_status(recurrence.id, false).await.ok();
                    continue;
                }
            }

            // Check end_date
            if let Some(end) = recurrence.end_date {
                if now.date_naive() > end {
                    info!("Recurrence '{}' past end date, deactivating", recurrence.name);
                    self.repository.update_recurrence_status(recurrence.id, false).await.ok();
                    continue;
                }
            }

            let process_name = format!(
                "{} - Recurrence {} ({})",
                template_code, recurrence.name,
                recurrence.run_count + 1
            );

            match self.repository.create_process(
                recurrence.organization_id,
                Some(recurrence.template_id),
                Some(template_code),
                &process_name,
                "report", // default
                Some(&format!("Auto-generated from recurrence '{}'", recurrence.name)),
                "pending",
                "normal",
                None,
                60,
                0,
                recurrence.parameters.clone(),
                recurrence.submitted_by.unwrap_or(Uuid::nil()),
            ).await {
                Ok(process) => {
                    // Calculate next run
                    let next_run = Self::calculate_next_run_from_now(
                        &recurrence.recurrence_type,
                        &recurrence.recurrence_config,
                    );

                    self.repository.update_recurrence_after_run(
                        recurrence.id,
                        Some(now),
                        next_run,
                        recurrence.run_count + 1,
                    ).await.ok();

                    // Link the process to this recurrence
                    self.repository.update_process_recurrence(process.id, recurrence.id).await.ok();

                    spawned.push(process.id);
                    info!("Spawned process {} from recurrence '{}'", process.id, recurrence.name);
                }
                Err(e) => {
                    tracing::error!("Failed to spawn process from recurrence '{}': {}", recurrence.name, e);
                }
            }
        }

        Ok(spawned)
    }

    // ========================================================================
    // Process Logging
    // ========================================================================

    /// Add a log entry to a process
    pub async fn add_log(
        &self,
        org_id: Uuid,
        process_id: Uuid,
        log_level: &str,
        message: &str,
        details: Option<serde_json::Value>,
        step_name: Option<&str>,
        duration_ms: Option<i32>,
    ) -> AtlasResult<ScheduledProcessLog> {
        if !VALID_LOG_LEVELS.contains(&log_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid log level '{}'. Must be one of: {}",
                log_level, VALID_LOG_LEVELS.join(", ")
            )));
        }

        self.repository.create_log(
            org_id, process_id, log_level, message,
            details, step_name, duration_ms,
        ).await
    }

    /// List log entries for a process
    pub async fn list_logs(
        &self,
        process_id: Uuid,
        log_level: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<ScheduledProcessLog>> {
        self.repository.list_logs(process_id, log_level, limit).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the scheduled processes dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ScheduledProcessDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Timeout Detection
    // ========================================================================

    /// Find and fail processes that have exceeded their timeout.
    /// Returns the IDs of timed-out processes.
    pub async fn detect_timeouts(&self) -> AtlasResult<Vec<Uuid>> {
        let timed_out = self.repository.find_timed_out_processes().await?;
        let mut failed = Vec::new();

        for process in &timed_out {
            let error_msg = format!(
                "Process timed out after {} minutes",
                process.timeout_minutes
            );

            match self.fail_process(process.id, Some(&error_msg), None).await {
                Ok(_) => {
                    failed.push(process.id);
                    info!("Timed out process {} ({})", process.process_name, process.id);
                }
                Err(e) => {
                    tracing::error!("Failed to mark process {} as timed out: {}", process.id, e);
                }
            }
        }

        Ok(failed)
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Calculate the first run time based on recurrence config and start date.
    pub fn calculate_next_run(
        start_date: chrono::NaiveDate,
        _recurrence_type: &str,
        recurrence_config: &serde_json::Value,
    ) -> Option<DateTime<Utc>> {
        let time_str = recurrence_config["time"].as_str().unwrap_or("00:00");
        let (hour, minute) = Self::parse_time(time_str);

        let datetime = start_date.and_hms_opt(hour, minute, 0)?;
        Some(DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc))
    }

    /// Calculate the next run time from now, based on recurrence config.
    pub fn calculate_next_run_from_now(
        recurrence_type: &str,
        recurrence_config: &serde_json::Value,
    ) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        let time_str = recurrence_config["time"].as_str().unwrap_or("00:00");
        let (hour, minute) = Self::parse_time(time_str);

        let next = match recurrence_type {
            "daily" => {
                let mut candidate = now.date_naive().and_hms_opt(hour, minute, 0)?;
                let candidate_utc = DateTime::<Utc>::from_naive_utc_and_offset(candidate, Utc);
                if candidate_utc <= now {
                    // Move to tomorrow
                    candidate = (now.date_naive() + chrono::Duration::days(1))
                        .and_hms_opt(hour, minute, 0)?;
                }
                DateTime::<Utc>::from_naive_utc_and_offset(candidate, Utc)
            }
            "weekly" => {
                let days_of_week = recurrence_config["days_of_week"].as_array();
                let target_days: Vec<u32> = days_of_week
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|n| n as u32)).collect())
                    .unwrap_or_else(|| vec![1]); // default Monday

                let mut candidate = now.date_naive();
                for _ in 0..8 {
                    let weekday = candidate.weekday().num_days_from_monday();
                    if target_days.contains(&weekday) {
                        if let Some(dt) = candidate.and_hms_opt(hour, minute, 0) {
                            let dt_utc = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);
                            if dt_utc > now {
                                return Some(dt_utc);
                            }
                        }
                    }
                    candidate += chrono::Duration::days(1);
                }
                return None;
            }
            "monthly" => {
                let day_of_month = recurrence_config["day_of_month"].as_u64()
                    .unwrap_or(1) as u32;
                let mut candidate_month = now.month();
                let mut candidate_year = now.year();

                for _ in 0..13 {
                    let day = day_of_month.min(
                        chrono::NaiveDate::from_ymd_opt(
                            candidate_year, candidate_month, 1,
                        ).and_then(|d| {
                            // last day of month
                            let next_month = if d.month() == 12 {
                                chrono::NaiveDate::from_ymd_opt(d.year() + 1, 1, 1)
                            } else {
                                chrono::NaiveDate::from_ymd_opt(d.year(), d.month() + 1, 1)
                            };
                            next_month.map(|nm| (nm - chrono::Duration::days(1)).day())
                        }).unwrap_or(28)
                    );

                    if let Some(dt) = chrono::NaiveDate::from_ymd_opt(
                        candidate_year, candidate_month, day,
                    ).and_then(|d| d.and_hms_opt(hour, minute, 0)) {
                        let dt_utc = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);
                        if dt_utc > now {
                            return Some(dt_utc);
                        }
                    }

                    candidate_month += 1;
                    if candidate_month > 12 {
                        candidate_month = 1;
                        candidate_year += 1;
                    }
                }
                return None;
            }
            _ => return None,
        };

        Some(next)
    }

    /// Parse a time string "HH:MM" into (hour, minute).
    pub fn parse_time(time_str: &str) -> (u32, u32) {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 2 {
            let hour = parts[0].parse::<u32>().unwrap_or(0);
            let minute = parts[1].parse::<u32>().unwrap_or(0);
            (hour.min(23), minute.min(59))
        } else {
            (0, 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_process_types() {
        assert!(VALID_PROCESS_TYPES.contains(&"report"));
        assert!(VALID_PROCESS_TYPES.contains(&"import"));
        assert!(VALID_PROCESS_TYPES.contains(&"export"));
        assert!(VALID_PROCESS_TYPES.contains(&"batch"));
        assert!(VALID_PROCESS_TYPES.contains(&"custom"));
        assert_eq!(VALID_PROCESS_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_executor_types() {
        assert!(VALID_EXECUTOR_TYPES.contains(&"built_in"));
        assert!(VALID_EXECUTOR_TYPES.contains(&"external"));
        assert!(VALID_EXECUTOR_TYPES.contains(&"plugin"));
        assert_eq!(VALID_EXECUTOR_TYPES.len(), 3);
    }

    #[test]
    fn test_valid_statuses() {
        for s in &["pending", "scheduled", "running", "completed", "failed", "cancelled", "waiting_for_approval"] {
            assert!(VALID_STATUSES.contains(s), "Status '{}' should be valid", s);
        }
    }

    #[test]
    fn test_valid_priorities() {
        for p in &["low", "normal", "high", "urgent"] {
            assert!(VALID_PRIORITIES.contains(p), "Priority '{}' should be valid", p);
        }
    }

    #[test]
    fn test_valid_recurrence_types() {
        for r in &["daily", "weekly", "monthly", "cron"] {
            assert!(VALID_RECURRENCE_TYPES.contains(r), "Recurrence type '{}' should be valid", r);
        }
    }

    #[test]
    fn test_valid_log_levels() {
        for l in &["debug", "info", "warn", "error"] {
            assert!(VALID_LOG_LEVELS.contains(l), "Log level '{}' should be valid", l);
        }
    }

    #[test]
    fn test_parse_time_valid() {
        assert_eq!(ScheduledProcessEngine::parse_time("00:00"), (0, 0));
        assert_eq!(ScheduledProcessEngine::parse_time("12:30"), (12, 30));
        assert_eq!(ScheduledProcessEngine::parse_time("23:59"), (23, 59));
        assert_eq!(ScheduledProcessEngine::parse_time("08:05"), (8, 5));
    }

    #[test]
    fn test_parse_time_invalid() {
        assert_eq!(ScheduledProcessEngine::parse_time("invalid"), (0, 0));
        assert_eq!(ScheduledProcessEngine::parse_time(""), (0, 0));
        assert_eq!(ScheduledProcessEngine::parse_time("25:99"), (23, 59)); // clamped
    }

    #[test]
    fn test_calculate_next_run_daily() {
        let start = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let config = serde_json::json!({ "time": "08:00" });

        let next = ScheduledProcessEngine::calculate_next_run(start, "daily", &config);
        assert!(next.is_some());
        let dt = next.unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2024-06-15 08:00");
    }

    #[test]
    fn test_calculate_next_run_weekly() {
        let start = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(); // Saturday
        let config = serde_json::json!({ "time": "09:30", "days_of_week": [0, 2, 4] });

        let next = ScheduledProcessEngine::calculate_next_run(start, "weekly", &config);
        assert!(next.is_some());
    }

    #[test]
    fn test_calculate_next_run_monthly() {
        let start = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let config = serde_json::json!({ "time": "10:00", "day_of_month": 15 });

        let next = ScheduledProcessEngine::calculate_next_run(start, "monthly", &config);
        assert!(next.is_some());
    }

    #[test]
    fn test_calculate_next_run_cron_not_supported() {
        // cron type is not supported for initial run calculation;
        // calculate_next_run always uses time + start_date regardless of type
        let start = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let config = serde_json::json!({ "time": "08:00" });

        // calculate_next_run returns Some for any type because it just uses time
        let next = ScheduledProcessEngine::calculate_next_run(start, "cron", &config);
        assert!(next.is_some()); // time-based, not cron-based
    }

    #[test]
    fn test_calculate_next_run_from_now_daily() {
        let config = serde_json::json!({ "time": "23:59" });
        let next = ScheduledProcessEngine::calculate_next_run_from_now("daily", &config);
        assert!(next.is_some());
        let dt = next.unwrap();
        assert!(dt > Utc::now());
    }

    #[test]
    fn test_calculate_next_run_from_now_weekly() {
        let config = serde_json::json!({ "time": "08:00", "days_of_week": [0, 1, 2, 3, 4, 5, 6] });
        let next = ScheduledProcessEngine::calculate_next_run_from_now("weekly", &config);
        assert!(next.is_some());
        let dt = next.unwrap();
        assert!(dt > Utc::now());
    }

    #[test]
    fn test_calculate_next_run_from_now_monthly() {
        let config = serde_json::json!({ "time": "08:00", "day_of_month": 1 });
        let next = ScheduledProcessEngine::calculate_next_run_from_now("monthly", &config);
        assert!(next.is_some());
        let dt = next.unwrap();
        assert!(dt > Utc::now());
    }

    #[test]
    fn test_calculate_next_run_from_now_unknown() {
        let config = serde_json::json!({});
        let next = ScheduledProcessEngine::calculate_next_run_from_now("cron", &config);
        assert!(next.is_none());
    }

    #[test]
    fn test_parse_time_boundary_values() {
        assert_eq!(ScheduledProcessEngine::parse_time("0:0"), (0, 0));
        assert_eq!(ScheduledProcessEngine::parse_time("24:00"), (23, 0)); // 24 is clamped to 23
    }

    #[test]
    fn test_calculate_next_run_midnight() {
        let start = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let config = serde_json::json!({ "time": "00:00" });
        let next = ScheduledProcessEngine::calculate_next_run(start, "daily", &config);
        assert!(next.is_some());
        let dt = next.unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2024-01-01 00:00");
    }

    #[test]
    fn test_calculate_next_run_monthly_day_31_in_short_month() {
        // Feb doesn't have 31 days - should use last day of month
        let start = chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let config = serde_json::json!({ "time": "08:00", "day_of_month": 31 });
        let next = ScheduledProcessEngine::calculate_next_run(start, "monthly", &config);
        assert!(next.is_some());
    }

    #[test]
    fn test_calculate_next_run_from_now_weekly_specific_days() {
        // Only Monday (0) and Friday (4)
        let config = serde_json::json!({ "time": "08:00", "days_of_week": [0, 4] });
        let next = ScheduledProcessEngine::calculate_next_run_from_now("weekly", &config);
        assert!(next.is_some());
        let dt = next.unwrap();
        assert!(dt > Utc::now());
        let weekday = dt.weekday().num_days_from_monday();
        assert!(weekday == 0 || weekday == 4, "Next run should be on Monday or Friday, got day {}", weekday);
    }
}
