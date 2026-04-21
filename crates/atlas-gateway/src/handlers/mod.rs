//! API Handlers
//! 
//! Request handlers for all API endpoints.

mod schema;
mod records;
pub mod auth;
mod admin;
mod reports;
pub mod fusion;
pub mod advanced;
pub mod period_close;
pub mod currency;
pub mod tax;
pub mod intercompany;
pub mod reconciliation;
pub mod expense;
pub mod budget;
pub mod fixed_assets;
pub mod subledger_accounting;
mod encumbrance;
pub mod cash_management;
pub mod sourcing;
pub mod lease;
pub mod project_costing;
pub mod cost_allocation;
pub mod financial_reporting;
pub mod multi_book;
pub mod procurement_contracts;
pub mod inventory;
pub mod customer_returns;
pub mod pricing;
pub mod sales_commission;
pub mod treasury;
pub mod grant_management;
pub mod supplier_qualification;
pub mod recurring_journal;
pub mod manual_journal;
pub mod descriptive_flexfield;
pub mod cross_validation;
pub mod scheduled_process;
mod segregation_of_duties;
mod allocation;
mod currency_revaluation;
mod purchase_requisition;
mod corporate_card;
pub mod performance;
pub mod benefits;
pub mod credit_management;
pub mod product_information;
pub mod transfer_pricing;

pub use schema::*;
pub use records::*;
pub use auth::*;
pub use admin::*;
pub use reports::*;
pub use fusion::*;
pub use advanced::*;
pub use period_close::*;
pub use currency::*;
pub use tax::*;
pub use intercompany::*;
pub use reconciliation::*;
pub use expense::*;
pub use budget::*;
pub use fixed_assets::*;
pub use subledger_accounting::*;
pub use encumbrance::*;
pub use cash_management::*;
pub use sourcing::*;
pub use lease::*;
pub use project_costing::*;
pub use cost_allocation::*;
pub use financial_reporting::*;
pub use multi_book::*;
pub use procurement_contracts::*;
pub use inventory::*;
pub use customer_returns::*;
pub use pricing::*;
pub use sales_commission::*;
pub use treasury::*;
pub use grant_management::{
    create_sponsor, list_sponsors, get_sponsor, delete_sponsor,
    create_indirect_cost_rate, list_indirect_cost_rates,
    create_award as create_grant_award, list_awards as list_grant_awards,
    get_award as get_grant_award, activate_award, suspend_award,
    complete_award, terminate_award,
    create_budget_line as create_grant_budget_line,
    list_budget_lines as list_grant_budget_lines,
    create_expenditure as create_grant_expenditure,
    list_expenditures as list_grant_expenditures,
    approve_expenditure, reverse_expenditure,
    create_billing as create_grant_billing,
    list_billings as list_grant_billings,
    submit_billing as submit_grant_billing,
    approve_billing as approve_grant_billing,
    mark_billing_paid,
    create_compliance_report, list_compliance_reports,
    submit_compliance_report, approve_compliance_report,
    get_grant_dashboard,
};
pub use supplier_qualification::{
    create_qualification_area, get_qualification_area,
    list_qualification_areas, delete_qualification_area,
    create_qualification_question, list_qualification_questions,
    delete_qualification_question,
    create_initiative, get_initiative, list_initiatives,
    activate_initiative, complete_initiative, cancel_initiative,
    invite_supplier, list_invitations,
    submit_invitation_response, start_evaluation,
    qualify_supplier, disqualify_supplier,
    create_response, list_responses, score_response,
    create_certification, list_certifications,
    revoke_certification, renew_certification,
    get_qualification_dashboard,
};
pub use recurring_journal::{
    create_schedule as create_recurring_schedule,
    get_schedule as get_recurring_schedule,
    list_schedules as list_recurring_schedules,
    activate_schedule as activate_recurring_schedule,
    deactivate_schedule as deactivate_recurring_schedule,
    delete_schedule as delete_recurring_schedule,
    add_schedule_line as add_recurring_schedule_line,
    list_schedule_lines as list_recurring_schedule_lines,
    delete_schedule_line as delete_recurring_schedule_line,
    generate_journal,
    get_generation,
    list_generations as list_recurring_generations,
    post_generation,
    reverse_generation,
    cancel_generation,
    list_generation_lines as list_recurring_generation_lines,
    get_dashboard as get_recurring_journal_dashboard,
};
pub use descriptive_flexfield::{
    create_value_set, list_value_sets, get_value_set, delete_value_set,
    create_value_set_entry, list_value_set_entries, delete_value_set_entry,
    create_flexfield, list_flexfields, get_flexfield,
    activate_flexfield, deactivate_flexfield, delete_flexfield,
    create_context, list_contexts, disable_context, enable_context, delete_context,
    create_segment, list_segments_by_context, list_segments_by_flexfield, delete_segment,
    set_flexfield_data, get_flexfield_data, delete_flexfield_data,
    get_flexfield_dashboard,
};
pub use cross_validation::{
    create_rule as create_cvr_rule,
    list_rules as list_cvr_rules,
    get_rule as get_cvr_rule,
    enable_rule as enable_cvr_rule,
    disable_rule as disable_cvr_rule,
    delete_rule as delete_cvr_rule,
    create_rule_line as create_cvr_rule_line,
    list_rule_lines as list_cvr_rule_lines,
    delete_rule_line as delete_cvr_rule_line,
    validate_combination,
    get_cvr_dashboard,
};
pub use scheduled_process::{
    create_template as create_process_template,
    get_template as get_process_template,
    list_templates as list_process_templates,
    activate_template as activate_process_template,
    deactivate_template as deactivate_process_template,
    delete_template as delete_process_template,
    submit_process,
    get_process,
    list_processes,
    start_process,
    complete_process,
    cancel_process,
    update_progress,
    approve_process,
    create_recurrence as create_process_recurrence,
    get_recurrence as get_process_recurrence,
    list_recurrences as list_process_recurrences,
    deactivate_recurrence as deactivate_process_recurrence,
    delete_recurrence as delete_process_recurrence,
    process_due_recurrences,
    list_process_logs,
    add_process_log,
    get_scheduled_process_dashboard,
};
pub use segregation_of_duties::{
    create_sod_rule,
    get_sod_rule,
    list_sod_rules,
    activate_sod_rule,
    deactivate_sod_rule,
    delete_sod_rule,
    assign_sod_role,
    list_sod_assignments,
    remove_sod_assignment,
    check_sod_conflict,
    run_sod_detection,
    list_sod_violations,
    get_sod_violation,
    resolve_sod_violation,
    accept_sod_exception,
    create_sod_mitigation,
    list_sod_mitigations,
    approve_sod_mitigation,
    revoke_sod_mitigation,
    get_sod_dashboard,
};
pub use allocation::{
    create_allocation_pool,
    get_allocation_pool,
    list_allocation_pools,
    activate_allocation_pool,
    deactivate_allocation_pool,
    delete_allocation_pool,
    create_allocation_basis,
    get_allocation_basis,
    list_allocation_bases,
    activate_allocation_basis,
    deactivate_allocation_basis,
    delete_allocation_basis,
    add_allocation_basis_detail,
    list_allocation_basis_details,
    recalculate_basis_percentages,
    create_allocation_rule,
    get_allocation_rule,
    list_allocation_rules,
    activate_allocation_rule,
    deactivate_allocation_rule,
    delete_allocation_rule,
    execute_allocation,
    get_allocation_run,
    list_allocation_runs,
    post_allocation_run,
    reverse_allocation_run,
    cancel_allocation_run,
    get_allocation_dashboard,
};
pub use currency_revaluation::{
    create_revaluation_definition,
    get_revaluation_definition,
    list_revaluation_definitions,
    activate_revaluation_definition,
    deactivate_revaluation_definition,
    delete_revaluation_definition,
    add_revaluation_account,
    list_revaluation_accounts,
    remove_revaluation_account,
    execute_revaluation,
    get_revaluation_run,
    list_revaluation_runs,
    post_revaluation_run,
    reverse_revaluation_run,
    cancel_revaluation_run,
    get_revaluation_dashboard,
};
pub use purchase_requisition::{
    create_requisition,
    get_requisition,
    list_requisitions,
    update_requisition,
    delete_requisition,
    add_requisition_line,
    list_requisition_lines,
    remove_requisition_line,
    add_requisition_distribution,
    list_requisition_distributions,
    submit_requisition,
    approve_requisition,
    reject_requisition,
    cancel_requisition,
    close_requisition,
    return_requisition,
    list_requisition_approvals,
    autocreate,
    list_autocreate_links,
    cancel_autocreate_link,
    get_requisition_dashboard,
};
pub use corporate_card::{
    create_program as create_corporate_card_program,
    get_program as get_corporate_card_program,
    list_programs as list_corporate_card_programs,
    issue_card,
    get_card as get_corporate_card,
    list_cards as list_corporate_cards,
    suspend_card,
    reactivate_card,
    cancel_card,
    report_lost,
    report_stolen,
    import_transaction,
    get_transaction as get_corporate_card_transaction,
    list_transactions as list_corporate_card_transactions,
    match_transaction as match_corporate_card_transaction,
    unmatch_transaction as unmatch_corporate_card_transaction,
    dispute_transaction as dispute_corporate_card_transaction,
    resolve_dispute as resolve_corporate_card_dispute,
    import_statement as import_corporate_card_statement,
    get_statement as get_corporate_card_statement,
    list_statements as list_corporate_card_statements,
    reconcile_statement as reconcile_corporate_card_statement,
    pay_statement as pay_corporate_card_statement,
    request_limit_override,
    approve_limit_override,
    reject_limit_override,
    list_limit_overrides,
    get_corporate_card_dashboard,
};
pub use benefits::{
    create_benefits_plan,
    get_benefits_plan,
    list_benefits_plans,
    delete_benefits_plan,
    create_enrollment as create_benefits_enrollment,
    get_enrollment as get_benefits_enrollment,
    list_enrollments as list_benefits_enrollments,
    activate_enrollment as activate_benefits_enrollment,
    waive_enrollment as waive_benefits_enrollment,
    cancel_enrollment as cancel_benefits_enrollment,
    suspend_enrollment as suspend_benefits_enrollment,
    reactivate_enrollment as reactivate_benefits_enrollment,
    generate_deductions,
    list_deductions as list_benefits_deductions,
    get_benefits_dashboard,
};
pub use performance::{
    create_rating_model,
    get_rating_model,
    list_rating_models,
    delete_rating_model,
    create_review_cycle,
    get_review_cycle,
    list_review_cycles,
    transition_cycle,
    create_competency,
    get_competency,
    list_competencies,
    delete_competency,
    create_document as create_performance_document,
    get_document as get_performance_document,
    list_documents as list_performance_documents,
    transition_document,
    submit_self_evaluation,
    submit_manager_evaluation,
    finalize_document as finalize_performance_document,
    create_goal,
    list_goals as list_performance_goals,
    complete_goal,
    rate_goal,
    delete_goal as delete_performance_goal,
    upsert_competency_assessment,
    list_competency_assessments as list_competency_assessments_for_document,
    create_feedback as create_performance_feedback,
    list_feedback as list_performance_feedback,
    submit_feedback as submit_performance_feedback,
    get_performance_dashboard,
};
pub use credit_management::{
    create_scoring_model,
    get_scoring_model,
    list_scoring_models,
    delete_scoring_model,
    create_profile,
    get_profile,
    list_profiles,
    update_profile_status,
    update_profile_score,
    delete_profile,
    create_credit_limit,
    list_credit_limits,
    update_credit_limit,
    set_temp_limit,
    delete_credit_limit,
    create_check_rule,
    list_check_rules,
    delete_check_rule,
    calculate_exposure,
    get_latest_exposure,
    perform_credit_check,
    create_hold,
    list_holds,
    release_hold,
    override_hold,
    create_review,
    list_reviews,
    start_review,
    complete_review,
    approve_review,
    reject_review,
    cancel_review,
    get_credit_dashboard,
};
pub use manual_journal::{
    create_batch as create_journal_batch,
    get_batch as get_journal_batch,
    list_batches as list_journal_batches,
    delete_batch as delete_journal_batch,
    submit_batch,
    approve_batch as approve_journal_batch,
    reject_batch as reject_journal_batch,
    post_batch as post_journal_batch,
    reverse_batch as reverse_journal_batch,
    create_entry as create_journal_entry,
    get_entry as get_journal_entry,
    list_entries_by_batch as list_journal_entries_by_batch,
    list_entries as list_journal_entries,
    delete_entry as delete_journal_entry,
    add_line as add_journal_line,
    list_lines as list_journal_lines,
    get_dashboard as get_manual_journal_dashboard,
};

use axum::{
    Router,
    routing::{get, post, put, delete},
    Json,
    middleware,
};
use serde_json::Value;
use crate::AppState;
use crate::middleware::auth_middleware;
use std::sync::Arc;

/// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

/// Metrics endpoint (placeholder)
pub async fn metrics() -> Json<Value> {
    Json(serde_json::json!({
        "uptime": "N/A",
        "requests_total": 0,
        "active_connections": 0,
    }))
}

/// API routes (v1) - requires authentication
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Schema introspection (requires auth)
        .route("/schema/:entity", get(get_entity_schema))
        .route("/schema/:entity/form", get(get_entity_form))
        .route("/schema/:entity/list", get(get_entity_list_view))
        
        // CRUD operations (requires auth)
        .route("/:entity", post(create_record))
        .route("/:entity", get(list_records))
        .route("/:entity/:id", get(get_record))
        .route("/:entity/:id", put(update_record))
        .route("/:entity/:id", delete(delete_record))
        
        // Workflow operations (requires auth)
        .route("/:entity/:id/transitions", get(get_transitions))
        .route("/:entity/:id/:action", post(execute_action))
        
        // Audit (requires auth)
        .route("/:entity/:id/history", get(get_record_history))
        
        // Reports (requires auth)
        .route("/reports/dashboard", get(dashboard_report))
        .route("/reports/:entity", get(generate_entity_report))
        
        // Import/Export (requires auth)
        .route("/import", post(import_data))
        .route("/export/:entity", get(export_data))
        
        // ═══════════════════════════════════════════════════════
        // Oracle Fusion-inspired features
        // ═══════════════════════════════════════════════════════
        
        // Notifications (bell icon)
        .route("/notifications", get(list_notifications))
        .route("/notifications/unread-count", get(get_unread_count))
        .route("/notifications/:id/read", put(mark_notification_read))
        .route("/notifications/read-all", put(mark_all_notifications_read))
        .route("/notifications/:id/dismiss", put(dismiss_notification))
        
        // Saved Searches (personalized views)
        .route("/saved-searches", get(list_saved_searches))
        .route("/saved-searches", post(create_saved_search))
        .route("/saved-searches/:id", delete(delete_saved_search))
        
        // Approval Chains
        .route("/approval-chains", get(list_approval_chains))
        .route("/approval-chains", post(create_approval_chain))
        .route("/approvals/pending", get(get_pending_approvals))
        .route("/approvals/:step_id/approve", post(approve_approval_step))
        .route("/approvals/:step_id/reject", post(reject_approval_step))
        .route("/approvals/:step_id/delegate", post(delegate_approval_step))
        
        // Duplicate Detection
        .route("/duplicates/check", post(check_duplicates))
        
        // ═══════════════════════════════════════════════════════
        // Advanced Oracle Fusion features (Phase 2)
        // ═══════════════════════════════════════════════════════
        
        // Structured Filtering (advanced list endpoint)
        .route("/:entity/filtered", get(list_records_advanced))
        
        // Bulk Operations
        .route("/bulk", post(execute_bulk_operation))
        
        // Comments / Notes on Records
        .route("/:entity/:id/comments", get(list_comments))
        .route("/:entity/:id/comments", post(create_comment))
        .route("/:entity/:id/comments/:comment_id", delete(delete_comment))
        .route("/:entity/:id/comments/:comment_id/pin", put(toggle_pin_comment))
        
        // Favorites / Bookmarks
        .route("/favorites", get(list_favorites))
        .route("/:entity/:id/favorite", post(add_favorite))
        .route("/:entity/:id/favorite", delete(remove_favorite))
        .route("/:entity/:id/favorite", get(check_favorite))
        
        // CSV Export
        .route("/export/:entity/csv", get(export_csv))
        
        // CSV Import
        .route("/import/csv", post(import_csv))
        
        // Related Records
        .route("/:entity/:id/related/:related_entity", get(get_related_records))
        
        // Effective Dating
        .route("/:entity/:id/effective", get(get_effective_record))
        .route("/:entity/:id/effective", post(create_effective_version))
        
        // ═══════════════════════════════════════════════════════
        // Period Close Management (Oracle Fusion GL Period Close)
        // ═══════════════════════════════════════════════════════
        
        // Accounting Calendars
        .route("/period-close/calendars", get(list_calendars))
        .route("/period-close/calendars", post(create_calendar))
        .route("/period-close/calendars/:calendar_id", get(get_calendar))
        .route("/period-close/calendars/:calendar_id", delete(delete_calendar))
        
        // Period Generation & Listing
        .route("/period-close/calendars/:calendar_id/periods/generate", post(generate_periods))
        .route("/period-close/calendars/:calendar_id/periods", get(list_periods))
        .route("/period-close/periods/:period_id", get(get_period))
        
        // Period Status Changes
        .route("/period-close/periods/:period_id/open", post(open_period))
        .route("/period-close/periods/:period_id/pending-close", post(pending_close_period))
        .route("/period-close/periods/:period_id/close", post(close_period))
        .route("/period-close/periods/:period_id/permanently-close", post(permanently_close_period))
        .route("/period-close/periods/:period_id/reopen", post(reopen_period))
        
        // Subledger Status
        .route("/period-close/periods/:period_id/subledger", post(update_subledger_status))
        
        // Period Close Checklist
        .route("/period-close/periods/:period_id/checklist", get(list_checklist_items))
        .route("/period-close/periods/:period_id/checklist", post(create_checklist_item))
        .route("/period-close/periods/:period_id/checklist/:item_id", put(update_checklist_item))
        .route("/period-close/periods/:period_id/checklist/:item_id", delete(delete_checklist_item))
        
        // Period Exceptions
        .route("/period-close/periods/:period_id/exceptions", post(grant_period_exception))
        .route("/period-close/periods/:period_id/exceptions/:user_id", delete(revoke_period_exception))
        
        // Period Close Dashboard
        .route("/period-close/calendars/:calendar_id/summary", get(get_close_summary))
        
        // Posting Validation
        .route("/period-close/calendars/:calendar_id/check-posting", get(check_posting_allowed))
        
        // ═══════════════════════════════════════════════════════
        // Multi-Currency Management (Oracle Fusion GL Currency)
        // ═══════════════════════════════════════════════════════
        
        // Currency Definitions
        .route("/currencies", get(list_currencies))
        .route("/currencies", post(create_currency))
        .route("/currencies/base", get(get_base_currency))
        .route("/currencies/:code", delete(delete_currency))
        
        // Exchange Rates
        .route("/exchange-rates", post(set_exchange_rate))
        .route("/exchange-rates", get(list_exchange_rates))
        .route("/exchange-rates/:from/:to", get(get_exchange_rate))
        .route("/exchange-rates/:id", delete(delete_exchange_rate))
        
        // Currency Conversion
        .route("/currency/convert", post(convert_currency))
        
        // Unrealized Gain/Loss
        .route("/currency/gain-loss", post(calculate_gain_loss))
        
        // Bulk Rate Import
        .route("/exchange-rates/import", post(import_rates))
        
        // ═══════════════════════════════════════════════════════
        // Tax Management (Oracle Fusion Tax)
        // ═══════════════════════════════════════════════════════
        
        // Tax Regimes
        .route("/tax/regimes", get(list_tax_regimes))
        .route("/tax/regimes", post(create_tax_regime))
        .route("/tax/regimes/:code", get(get_tax_regime))
        .route("/tax/regimes/:code", delete(delete_tax_regime))
        
        // Tax Jurisdictions
        .route("/tax/jurisdictions", get(list_tax_jurisdictions))
        .route("/tax/jurisdictions", post(create_tax_jurisdiction))
        .route("/tax/jurisdictions/:regime_code/:code", delete(delete_tax_jurisdiction))
        
        // Tax Rates
        .route("/tax/rates", post(create_tax_rate))
        .route("/tax/rates/:regime_code", get(list_tax_rates))
        .route("/tax/rates/:regime_code/:code", delete(delete_tax_rate))
        
        // Tax Determination Rules
        .route("/tax/rules", post(create_determination_rule))
        .route("/tax/rules/:regime_code", get(list_determination_rules))
        
        // Tax Calculation
        .route("/tax/calculate", post(calculate_tax))
        
        // Tax Lines (per transaction)
        .route("/tax/lines/:entity_type/:entity_id", get(get_tax_lines))
        
        // Tax Reporting
        .route("/tax/reports", post(generate_tax_report))
        .route("/tax/reports", get(list_tax_reports))
        
        // ═══════════════════════════════════════════════════════
        // Intercompany Transactions (Oracle Fusion Intercompany)
        // ═══════════════════════════════════════════════════════
        
        // Intercompany Batches
        .route("/intercompany/batches", get(list_intercompany_batches))
        .route("/intercompany/batches", post(create_intercompany_batch))
        .route("/intercompany/batches/:batch_number", get(get_intercompany_batch))
        .route("/intercompany/batches/:batch_id/submit", post(submit_intercompany_batch))
        .route("/intercompany/batches/:batch_id/approve", post(approve_intercompany_batch))
        .route("/intercompany/batches/:batch_id/post", post(post_intercompany_batch))
        .route("/intercompany/batches/:batch_id/reject", post(reject_intercompany_batch))
        
        // Intercompany Transactions
        .route("/intercompany/transactions", post(create_intercompany_transaction))
        .route("/intercompany/transactions/batch/:batch_id", get(list_intercompany_transactions))
        .route("/intercompany/transactions/entity/:entity_id", get(list_entity_transactions))
        
        // Intercompany Settlements
        .route("/intercompany/settlements", post(create_intercompany_settlement))
        .route("/intercompany/settlements", get(list_intercompany_settlements))
        
        // Intercompany Balances
        .route("/intercompany/balances/summary", get(get_intercompany_balance_summary))
        .route("/intercompany/balances/:from_entity_id/:to_entity_id", get(get_intercompany_balance))
        
        // ═══════════════════════════════════════════════════════════
        // Bank Reconciliation (Oracle Fusion Cash Management)
        // ═══════════════════════════════════════════════════════════
        
        // Bank Accounts
        .route("/reconciliation/bank-accounts", get(list_bank_accounts))
        .route("/reconciliation/bank-accounts", post(create_bank_account))
        .route("/reconciliation/bank-accounts/:id", get(get_bank_account))
        .route("/reconciliation/bank-accounts/:id", delete(delete_bank_account))
        
        // Bank Statements
        .route("/reconciliation/statements", post(create_bank_statement))
        .route("/reconciliation/statements/bank-account/:bank_account_id", get(list_bank_statements))
        .route("/reconciliation/statements/:statement_id", get(get_bank_statement))
        .route("/reconciliation/statements/:statement_id/lines", get(list_statement_lines))
        
        // System Transactions
        .route("/reconciliation/system-transactions", post(create_system_transaction))
        .route("/reconciliation/system-transactions/unreconciled/:bank_account_id", get(list_unreconciled_transactions))
        
        // Auto-Matching
        .route("/reconciliation/statements/:statement_id/auto-match", post(auto_match_statement))
        
        // Manual Matching
        .route("/reconciliation/statements/:statement_id/manual-match", post(manual_match))
        .route("/reconciliation/matches/:match_id/unmatch", post(unmatch))
        .route("/reconciliation/statements/:statement_id/matches", get(list_matches))
        
        // Reconciliation Summary
        .route("/reconciliation/summary", get(get_reconciliation_summary))
        .route("/reconciliation/summaries", get(list_reconciliation_summaries))
        
        // Matching Rules
        .route("/reconciliation/rules", post(create_matching_rule))
        .route("/reconciliation/rules", get(list_matching_rules))
        .route("/reconciliation/rules/:id", delete(delete_matching_rule))
        
        // ═════════════════════════════════════════════════════════════
        // Expense Management (Oracle Fusion Expenses)
        // ═════════════════════════════════════════════════════════════
        
        // Expense Categories
        .route("/expense/categories", get(list_expense_categories))
        .route("/expense/categories", post(create_expense_category))
        .route("/expense/categories/:code", get(get_expense_category))
        .route("/expense/categories/:code", delete(delete_expense_category))
        
        // Expense Policies
        .route("/expense/policies", get(list_expense_policies))
        .route("/expense/policies", post(create_expense_policy))
        .route("/expense/policies/:id", delete(delete_expense_policy))
        
        // Expense Reports
        .route("/expense/reports", get(list_expense_reports))
        .route("/expense/reports", post(create_expense_report))
        .route("/expense/reports/:id", get(get_expense_report))
        .route("/expense/reports/:id/submit", post(submit_expense_report))
        .route("/expense/reports/:id/approve", post(approve_expense_report))
        .route("/expense/reports/:id/reject", post(reject_expense_report))
        .route("/expense/reports/:id/reimburse", post(reimburse_expense_report))
        
        // Expense Lines
        .route("/expense/reports/:report_id/lines", get(list_expense_lines))
        .route("/expense/reports/:report_id/lines", post(add_expense_line))
        .route("/expense/reports/:report_id/lines/:line_id", delete(delete_expense_line))

        // ══════════════════════════════════════════════════════════════════
        // Budget Management (Oracle Fusion General Ledger > Budgets)
        // ══════════════════════════════════════════════════════════════════

        // Budget Definitions
        .route("/budget/definitions", get(list_budget_definitions))
        .route("/budget/definitions", post(create_budget_definition))
        .route("/budget/definitions/:code", get(get_budget_definition))
        .route("/budget/definitions/:code", delete(delete_budget_definition))

        // Budget Versions
        .route("/budget/definitions/:budget_code/versions", post(create_budget_version))
        .route("/budget/definitions/:budget_code/versions", get(list_budget_versions))
        .route("/budget/versions/:version_id", get(get_budget_version))

        // Budget Version Workflow
        .route("/budget/versions/:version_id/submit", post(submit_budget_version))
        .route("/budget/versions/:version_id/approve", post(approve_budget_version))
        .route("/budget/versions/:version_id/activate", post(activate_budget_version))
        .route("/budget/versions/:version_id/reject", post(reject_budget_version))
        .route("/budget/versions/:version_id/close", post(close_budget_version))

        // Budget Lines
        .route("/budget/versions/:version_id/lines", get(list_budget_lines))
        .route("/budget/versions/:version_id/lines", post(add_budget_line))
        .route("/budget/versions/:version_id/lines/:line_id", delete(delete_budget_line))

        // Budget Transfers
        .route("/budget/versions/:version_id/transfers", post(create_budget_transfer))
        .route("/budget/versions/:version_id/transfers", get(list_budget_transfers))
        .route("/budget/transfers/:transfer_id/approve", post(approve_budget_transfer))
        .route("/budget/transfers/:transfer_id/reject", post(reject_budget_transfer))

        // Budget Variance Report
        .route("/budget/versions/:version_id/variance", get(get_budget_variance))

        // Budget Control Check
        .route("/budget/definitions/:budget_code/check", post(check_budget_control))

        // ════════════════════════════════════════════════════════════════════
        // Fixed Assets Management (Oracle Fusion Fixed Assets)
        // ════════════════════════════════════════════════════════════════════

        // Asset Categories
        .route("/fixed-assets/categories", get(list_asset_categories))
        .route("/fixed-assets/categories", post(create_asset_category))
        .route("/fixed-assets/categories/:code", get(get_asset_category))
        .route("/fixed-assets/categories/:code", delete(delete_asset_category))

        // Asset Books
        .route("/fixed-assets/books", get(list_asset_books))
        .route("/fixed-assets/books", post(create_asset_book))

        // Fixed Assets
        .route("/fixed-assets/assets", get(list_fixed_assets))
        .route("/fixed-assets/assets", post(create_fixed_asset))
        .route("/fixed-assets/assets/:id", get(get_fixed_asset))

        // Asset Lifecycle
        .route("/fixed-assets/assets/:id/acquire", post(acquire_fixed_asset))
        .route("/fixed-assets/assets/:id/place-in-service", post(place_asset_in_service))

        // Depreciation
        .route("/fixed-assets/assets/:id/depreciate", post(calculate_depreciation))
        .route("/fixed-assets/assets/:id/depreciation-history", get(list_depreciation_history))

        // Asset Transfers
        .route("/fixed-assets/transfers", get(list_asset_transfers))
        .route("/fixed-assets/transfers", post(create_asset_transfer))
        .route("/fixed-assets/transfers/:id/approve", post(approve_asset_transfer))
        .route("/fixed-assets/transfers/:id/reject", post(reject_asset_transfer))

        // Asset Retirements
        .route("/fixed-assets/retirements", get(list_asset_retirements))
        .route("/fixed-assets/retirements", post(create_asset_retirement))
        .route("/fixed-assets/retirements/:id/approve", post(approve_asset_retirement))

        // ════════════════════════════════════════════════════════════════════════════
        // Subledger Accounting (Oracle Fusion GL > Subledger Accounting)
        // ════════════════════════════════════════════════════════════════════════════

        // Accounting Methods
        .route("/sla/methods", get(list_accounting_methods))
        .route("/sla/methods", post(create_accounting_method))
        .route("/sla/methods/:code", get(get_accounting_method))
        .route("/sla/methods/:code", delete(delete_accounting_method))

        // Derivation Rules
        .route("/sla/methods/:method_id/rules", get(list_derivation_rules))
        .route("/sla/methods/:method_id/rules", post(create_derivation_rule))
        .route("/sla/methods/:method_id/rules/:code", delete(delete_derivation_rule))

        // Resolve Account Code
        .route("/sla/resolve-account", post(resolve_account_code))

        // Journal Entries
        .route("/sla/entries", get(list_journal_entries))
        .route("/sla/entries", post(create_journal_entry))
        .route("/sla/entries/:id", get(get_journal_entry))

        // Journal Lines
        .route("/sla/entries/:entry_id/lines", get(list_journal_lines))
        .route("/sla/entries/:entry_id/lines", post(add_journal_line))

        // Entry Lifecycle
        .route("/sla/entries/:id/account", post(account_journal_entry))
        .route("/sla/entries/:id/post", post(post_journal_entry))
        .route("/sla/entries/:id/reverse", post(reverse_journal_entry))

        // Auto-Accounting
        .route("/sla/entries/:entry_id/generate-lines", post(generate_journal_lines))

        // Transfer to GL
        .route("/sla/transfer-to-gl", post(transfer_to_gl))
        .route("/sla/transfers/:id", get(get_transfer_log))
        .route("/sla/transfers", get(list_transfer_logs))

        // SLA Events
        .route("/sla/events", get(list_sla_events))

        // SLA Dashboard
        .route("/sla/dashboard", get(get_sla_dashboard))

        // ════════════════════════════════════════════════════════════════════════════════
        // Encumbrance Management (Oracle Fusion GL > Encumbrance Management)
        // ════════════════════════════════════════════════════════════════════════════════

        // Encumbrance Types
        .route("/encumbrance/types", get(list_encumbrance_types))
        .route("/encumbrance/types", post(create_encumbrance_type))
        .route("/encumbrance/types/:code", get(get_encumbrance_type))
        .route("/encumbrance/types/:code", delete(delete_encumbrance_type))

        // Encumbrance Entries
        .route("/encumbrance/entries", post(create_encumbrance_entry))
        .route("/encumbrance/entries", get(list_encumbrance_entries))
        .route("/encumbrance/entries/:id", get(get_encumbrance_entry))
        .route("/encumbrance/entries/:id/activate", post(activate_encumbrance_entry))
        .route("/encumbrance/entries/:id/cancel", post(cancel_encumbrance_entry))

        // Encumbrance Lines
        .route("/encumbrance/entries/:entry_id/lines", post(add_encumbrance_line))
        .route("/encumbrance/entries/:entry_id/lines", get(list_encumbrance_lines))
        .route("/encumbrance/lines/:line_id", delete(delete_encumbrance_line))

        // Liquidations
        .route("/encumbrance/liquidations", post(create_liquidation))
        .route("/encumbrance/liquidations", get(list_liquidations))
        .route("/encumbrance/liquidations/:id/reverse", post(reverse_liquidation))

        // Year-End Carry-Forward
        .route("/encumbrance/carry-forward", post(process_carry_forward))
        .route("/encumbrance/carry-forward", get(list_carry_forwards))

        // Encumbrance Summary Dashboard
        .route("/encumbrance/summary", get(get_encumbrance_summary))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Cash Position & Cash Forecasting (Oracle Fusion Treasury Management)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Cash Positions
        .route("/cash-management/positions", post(upsert_cash_position))
        .route("/cash-management/positions", get(list_cash_positions))
        .route("/cash-management/positions/:bank_account_id", get(get_cash_position))
        .route("/cash-management/positions/summary", get(get_cash_position_summary))

        // Forecast Templates
        .route("/cash-management/templates", post(create_forecast_template))
        .route("/cash-management/templates", get(list_forecast_templates))
        .route("/cash-management/templates/:code", get(get_forecast_template))
        .route("/cash-management/templates/:code", delete(delete_forecast_template))

        // Forecast Sources
        .route("/cash-management/sources", post(create_forecast_source))
        .route("/cash-management/sources/:template_code", get(list_forecast_sources))
        .route("/cash-management/sources/:template_code/:code", delete(delete_forecast_source))

        // Cash Forecasts
        .route("/cash-management/forecasts", post(generate_forecast))
        .route("/cash-management/forecasts", get(list_cash_forecasts))
        .route("/cash-management/forecasts/:id", get(get_cash_forecast))
        .route("/cash-management/forecasts/:id/approve", post(approve_cash_forecast))
        .route("/cash-management/forecasts/:forecast_id/lines", get(list_forecast_lines))

        // Forecast Summary (Dashboard)
        .route("/cash-management/forecast-summary", get(get_forecast_summary))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Procurement Sourcing (Oracle Fusion SCM > Procurement > Sourcing)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Sourcing Events
        .route("/sourcing/events", post(create_sourcing_event))
        .route("/sourcing/events", get(list_sourcing_events))
        .route("/sourcing/events/:id", get(get_sourcing_event))
        .route("/sourcing/events/:id/publish", post(publish_sourcing_event))
        .route("/sourcing/events/:id/close", post(close_sourcing_event))
        .route("/sourcing/events/:id/cancel", post(cancel_sourcing_event))

        // Event Lines
        .route("/sourcing/events/:event_id/lines", post(add_event_line))
        .route("/sourcing/events/:event_id/lines", get(list_event_lines))

        // Supplier Invitations
        .route("/sourcing/events/:event_id/invites", post(invite_supplier))
        .route("/sourcing/events/:event_id/invites", get(list_invites))

        // Supplier Responses
        .route("/sourcing/events/:event_id/responses", post(submit_response))
        .route("/sourcing/events/:event_id/responses", get(list_responses))
        .route("/sourcing/responses/:id", get(get_response))
        .route("/sourcing/responses/:response_id/lines", post(add_response_line))
        .route("/sourcing/responses/:response_id/lines", get(list_response_lines))

        // Scoring & Evaluation
        .route("/sourcing/events/:event_id/criteria", post(add_scoring_criterion))
        .route("/sourcing/events/:event_id/criteria", get(list_scoring_criteria))
        .route("/sourcing/responses/:response_id/score", post(score_response))
        .route("/sourcing/events/:event_id/evaluate", post(evaluate_responses))

        // Award Management
        .route("/sourcing/events/:event_id/awards", post(create_award))
        .route("/sourcing/events/:event_id/awards", get(list_awards))
        .route("/sourcing/awards/:id", get(get_award))
        .route("/sourcing/awards/:id/approve", post(approve_award))
        .route("/sourcing/awards/:id/reject", post(reject_award))
        .route("/sourcing/awards/:award_id/lines", get(list_award_lines))

        // Sourcing Templates
        .route("/sourcing/templates", post(create_sourcing_template))
        .route("/sourcing/templates", get(list_sourcing_templates))
        .route("/sourcing/templates/:code", get(get_sourcing_template))
        .route("/sourcing/templates/:code", delete(delete_sourcing_template))

        // Sourcing Dashboard
        .route("/sourcing/summary", get(get_sourcing_summary))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Lease Accounting (ASC 842 / IFRS 16) (Oracle Fusion Lease Management)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Lease Contracts
        .route("/lease/contracts", get(list_leases))
        .route("/lease/contracts", post(create_lease))
        .route("/lease/contracts/:id", get(get_lease))
        .route("/lease/contracts/:id/activate", post(activate_lease))

        // Lease Payments
        .route("/lease/contracts/:id/payments", get(list_lease_payments))
        .route("/lease/contracts/:id/payments", post(process_lease_payment))

        // Lease Modifications
        .route("/lease/contracts/:id/modifications", post(create_lease_modification))
        .route("/lease/contracts/:id/modifications", get(list_lease_modifications))

        // Lease Impairment
        .route("/lease/contracts/:id/impairment", post(record_lease_impairment))

        // Lease Termination
        .route("/lease/contracts/:id/terminate", post(terminate_lease))
        .route("/lease/contracts/:id/terminations", get(list_lease_terminations))

        // Lease Dashboard
        .route("/lease/dashboard", get(get_lease_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Project Costing (Oracle Fusion Project Management > Project Costing)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Cost Transactions
        .route("/project-costing/transactions", get(list_cost_transactions))
        .route("/project-costing/transactions", post(create_cost_transaction))
        .route("/project-costing/transactions/:id", get(get_cost_transaction))
        .route("/project-costing/transactions/:id/approve", post(approve_cost_transaction))
        .route("/project-costing/transactions/:id/reverse", post(reverse_cost_transaction))

        // Burden Schedules
        .route("/project-costing/burden-schedules", get(list_burden_schedules))
        .route("/project-costing/burden-schedules", post(create_burden_schedule))
        .route("/project-costing/burden-schedules/:code", get(get_burden_schedule))
        .route("/project-costing/burden-schedules/:id/activate", post(activate_burden_schedule))
        .route("/project-costing/burden-schedules/:schedule_id/lines", get(list_burden_schedule_lines))
        .route("/project-costing/burden-schedules/:schedule_id/lines", post(add_burden_schedule_line))

        // Cost Adjustments
        .route("/project-costing/adjustments", get(list_cost_adjustments))
        .route("/project-costing/adjustments", post(create_cost_adjustment))
        .route("/project-costing/adjustments/:id/approve", post(approve_cost_adjustment))

        // Cost Distributions
        .route("/project-costing/transactions/:id/distribute", post(distribute_cost_transaction))
        .route("/project-costing/transactions/:transaction_id/distributions", get(list_cost_distributions))
        .route("/project-costing/distributions/post", post(post_distributions))

        // Project Costing Dashboard
        .route("/project-costing/dashboard", get(get_costing_summary))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Cost Allocation (Oracle Fusion GL > Allocations / Mass Allocations)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Allocation Pools
        .route("/cost-allocation/pools", get(list_allocation_pools))
        .route("/cost-allocation/pools", post(create_allocation_pool))
        .route("/cost-allocation/pools/:code", get(get_allocation_pool))
        .route("/cost-allocation/pools/:code", delete(delete_allocation_pool))

        // Allocation Bases
        .route("/cost-allocation/bases", get(list_allocation_bases))
        .route("/cost-allocation/bases", post(create_allocation_base))
        .route("/cost-allocation/bases/:code", get(get_allocation_base))
        .route("/cost-allocation/bases/:code", delete(delete_allocation_base))

        // Base Values
        .route("/cost-allocation/base-values", post(set_base_value))
        .route("/cost-allocation/base-values", get(list_base_values))

        // Allocation Rules
        .route("/cost-allocation/rules", get(list_allocation_rules))
        .route("/cost-allocation/rules", post(create_allocation_rule))
        .route("/cost-allocation/rules/:id", get(get_allocation_rule))
        .route("/cost-allocation/rules/:id/activate", post(activate_allocation_rule))
        .route("/cost-allocation/rules/:id/deactivate", post(deactivate_allocation_rule))

        // Rule Targets
        .route("/cost-allocation/rules/:rule_id/targets", post(add_rule_target))
        .route("/cost-allocation/rules/:rule_id/targets", get(list_rule_targets))

        // Allocation Runs
        .route("/cost-allocation/rules/:rule_id/execute", post(execute_allocation_rule))
        .route("/cost-allocation/runs", get(list_allocation_runs))
        .route("/cost-allocation/runs/:id", get(get_allocation_run))
        .route("/cost-allocation/runs/:id/post", post(post_allocation_run))
        .route("/cost-allocation/runs/:id/reverse", post(reverse_allocation_run))
        .route("/cost-allocation/runs/:run_id/lines", get(list_allocation_run_lines))

        // Cost Allocation Dashboard
        .route("/cost-allocation/summary", get(get_allocation_summary))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Financial Reporting (Oracle Fusion GL > Financial Reporting Center)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Report Templates
        .route("/financial-reporting/templates", get(list_financial_templates))
        .route("/financial-reporting/templates", post(create_financial_template))
        .route("/financial-reporting/templates/:code", get(get_financial_template))
        .route("/financial-reporting/templates/:code", delete(delete_financial_template))

        // Report Rows
        .route("/financial-reporting/templates/:template_id/rows", get(list_financial_rows))
        .route("/financial-reporting/templates/:template_id/rows", post(create_financial_row))
        .route("/financial-reporting/rows/:id", delete(delete_financial_row))

        // Report Columns
        .route("/financial-reporting/templates/:template_id/columns", get(list_financial_columns))
        .route("/financial-reporting/templates/:template_id/columns", post(create_financial_column))
        .route("/financial-reporting/columns/:id", delete(delete_financial_column))

        // Report Generation
        .route("/financial-reporting/templates/:template_code/generate", post(generate_financial_report))
        .route("/financial-reporting/runs", get(list_financial_runs))
        .route("/financial-reporting/runs/:id", get(get_financial_run))
        .route("/financial-reporting/runs/:run_id/results", get(get_financial_run_results))

        // Report Lifecycle
        .route("/financial-reporting/runs/:id/approve", post(approve_financial_report))
        .route("/financial-reporting/runs/:id/publish", post(publish_financial_report))
        .route("/financial-reporting/runs/:id/archive", post(archive_financial_report))

        // Quick Templates
        .route("/financial-reporting/quick/trial-balance", post(create_financial_trial_balance))
        .route("/financial-reporting/quick/income-statement", post(create_financial_income_statement))
        .route("/financial-reporting/quick/balance-sheet", post(create_financial_balance_sheet))

        // Favourites
        .route("/financial-reporting/favourites", get(list_financial_favourites))
        .route("/financial-reporting/favourites/:template_id", post(add_financial_favourite))
        .route("/financial-reporting/favourites/:template_id", delete(remove_financial_favourite))

        // Dashboard
        .route("/financial-reporting/dashboard", get(get_financial_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Multi-Book Accounting (Oracle Fusion GL > Multi-Book Accounting)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Accounting Books
        .route("/multi-book/books", get(list_accounting_books))
        .route("/multi-book/books", post(create_accounting_book))
        .route("/multi-book/books/:code", get(get_accounting_book))
        .route("/multi-book/books/:code/status", put(update_accounting_book_status))
        .route("/multi-book/books/:code", delete(delete_accounting_book))

        // Account Mappings
        .route("/multi-book/mappings", post(create_account_mapping))
        .route("/multi-book/mappings", get(list_account_mappings))
        .route("/multi-book/mappings/:id", delete(delete_account_mapping))

        // Journal Entries
        .route("/multi-book/entries", post(create_book_journal_entry))
        .route("/multi-book/entries/:id", get(get_book_journal_entry))
        .route("/multi-book/books/:book_id/entries", get(list_book_journal_entries))
        .route("/multi-book/entries/:entry_id/lines", get(get_book_journal_lines))
        .route("/multi-book/entries/:id/post", post(post_book_journal_entry))
        .route("/multi-book/entries/:id/reverse", post(reverse_book_journal_entry))

        // Propagation
        .route("/multi-book/entries/:id/propagate", post(propagate_entry))
        .route("/multi-book/propagation-logs", get(list_propagation_logs))

        // Multi-Book Dashboard
        .route("/multi-book/dashboard", get(get_multi_book_summary))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Procurement Contracts (Oracle Fusion SCM > Procurement > Contracts)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Contract Types
        .route("/procurement-contracts/types", get(list_contract_types))
        .route("/procurement-contracts/types", post(create_contract_type))
        .route("/procurement-contracts/types/:code", get(get_contract_type))
        .route("/procurement-contracts/types/:code", delete(delete_contract_type))

        // Contracts
        .route("/procurement-contracts", post(create_contract))
        .route("/procurement-contracts", get(list_contracts))
        .route("/procurement-contracts/:id", get(get_contract))
        .route("/procurement-contracts/:id/submit", post(submit_contract))
        .route("/procurement-contracts/:id/approve", post(approve_contract))
        .route("/procurement-contracts/:id/reject", post(reject_contract))
        .route("/procurement-contracts/:id/terminate", post(terminate_contract))
        .route("/procurement-contracts/:id/close", post(close_contract))

        // Contract Lines
        .route("/procurement-contracts/:contract_id/lines", post(add_contract_line))
        .route("/procurement-contracts/:contract_id/lines", get(list_contract_lines))
        .route("/procurement-contracts/lines/:line_id", delete(delete_contract_line))

        // Milestones
        .route("/procurement-contracts/:contract_id/milestones", post(add_milestone))
        .route("/procurement-contracts/:contract_id/milestones", get(list_milestones))
        .route("/procurement-contracts/milestones/:milestone_id", put(update_milestone))

        // Renewals
        .route("/procurement-contracts/:contract_id/renewals", post(renew_contract))
        .route("/procurement-contracts/:contract_id/renewals", get(list_renewals))

        // Spend Tracking
        .route("/procurement-contracts/:contract_id/spend", post(record_spend))
        .route("/procurement-contracts/:contract_id/spend", get(list_spend_entries))

        // Dashboard
        .route("/procurement-contracts/dashboard", get(get_dashboard_summary))

        // ═══════════════════════════════════════════════════════
        // Inventory Management (Oracle Fusion SCM > Inventory)
        // ═══════════════════════════════════════════════════════
        .route("/inventory/organizations", post(create_inventory_org))
        .route("/inventory/organizations", get(list_inventory_orgs))
        .route("/inventory/organizations/:code", get(get_inventory_org))
        .route("/inventory/organizations/:code", delete(delete_inventory_org))
        .route("/inventory/items", post(create_item))
        .route("/inventory/items", get(list_items))
        .route("/inventory/items/:id", get(get_item))
        .route("/inventory/on-hand", get(list_on_hand_balances))
        .route("/inventory/transactions/receive", post(receive_item))
        .route("/inventory/transactions/issue", post(issue_item))
        .route("/inventory/transactions/transfer", post(transfer_item))
        .route("/inventory/transactions/adjust", post(adjust_item))
        .route("/inventory/transactions", get(list_transactions))
        .route("/inventory/subinventories", post(create_subinventory))
        .route("/inventory/subinventories/:inventory_org_id", get(list_subinventories))
        .route("/inventory/dashboard", get(get_inventory_dashboard))

        // ═══════════════════════════════════════════════════════
        // Customer Returns / RMA (Oracle Fusion Order Management > Returns)
        // ═══════════════════════════════════════════════════════

        // Return Reasons
        .route("/returns/reasons", post(create_return_reason))
        .route("/returns/reasons", get(list_return_reasons))
        .route("/returns/reasons/:code", get(get_return_reason))
        .route("/returns/reasons/:code", delete(delete_return_reason))

        // Return Authorizations (RMAs)
        .route("/returns/rmas", post(create_rma))
        .route("/returns/rmas", get(list_rmas))
        .route("/returns/rmas/:id", get(get_rma))
        .route("/returns/rmas/:id/submit", post(submit_rma))
        .route("/returns/rmas/:id/approve", post(approve_rma))
        .route("/returns/rmas/:id/reject", post(reject_rma))
        .route("/returns/rmas/:id/cancel", post(cancel_rma))

        // Return Lines
        .route("/returns/rmas/:rma_id/lines", post(add_return_line))
        .route("/returns/rmas/:rma_id/lines", get(list_return_lines))
        .route("/returns/lines/:line_id/receive", post(receive_return_line))
        .route("/returns/lines/:line_id/inspect", post(inspect_return_line))

        // Credit Memos
        .route("/returns/rmas/:rma_id/credit-memo", post(generate_credit_memo))
        .route("/returns/credit-memos", get(list_credit_memos))
        .route("/returns/credit-memos/:id", get(get_credit_memo))
        .route("/returns/credit-memos/:id/issue", post(issue_credit_memo))
        .route("/returns/credit-memos/:id/cancel", post(cancel_credit_memo))

        // Dashboard
        .route("/returns/dashboard", get(get_returns_dashboard))

        // ═══════════════════════════════════════════════════════
        // Advanced Pricing (Oracle Fusion Order Management > Pricing)
        // ═══════════════════════════════════════════════════════

        // Price Lists
        .route("/pricing/price-lists", post(create_price_list))
        .route("/pricing/price-lists", get(list_price_lists))
        .route("/pricing/price-lists/:code", get(get_price_list))
        .route("/pricing/price-lists/:code", delete(delete_price_list))
        .route("/pricing/price-lists/:id/activate", post(activate_price_list))
        .route("/pricing/price-lists/:id/deactivate", post(deactivate_price_list))

        // Price List Lines
        .route("/pricing/price-lists/:price_list_id/lines", post(add_price_list_line))
        .route("/pricing/price-lists/:price_list_id/lines", get(list_price_list_lines))
        .route("/pricing/lines/:id", delete(delete_price_list_line))

        // Price Tiers
        .route("/pricing/lines/:price_list_line_id/tiers", post(add_price_tier))
        .route("/pricing/lines/:price_list_line_id/tiers", get(list_price_tiers))

        // Discount Rules
        .route("/pricing/discount-rules", post(create_discount_rule))
        .route("/pricing/discount-rules", get(list_discount_rules))
        .route("/pricing/discount-rules/:code", get(get_discount_rule))
        .route("/pricing/discount-rules/:code", delete(delete_discount_rule))

        // Charge Definitions
        .route("/pricing/charges", post(create_charge_definition))
        .route("/pricing/charges", get(list_charge_definitions))
        .route("/pricing/charges/:code", get(get_charge_definition))
        .route("/pricing/charges/:code", delete(delete_charge_definition))

        // Pricing Strategies
        .route("/pricing/strategies", post(create_pricing_strategy))
        .route("/pricing/strategies", get(list_pricing_strategies))

        // Price Calculation
        .route("/pricing/calculate", post(calculate_price))

        // Calculation Logs
        .route("/pricing/calculation-logs", get(list_calculation_logs))

        // Pricing Dashboard
        .route("/pricing/dashboard", get(get_pricing_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Sales Commission (Oracle Fusion Incentive Compensation)
        // ═══════════════════════════════════════════════════════════

        // Sales Representatives
        .route("/commission/reps", post(create_rep))
        .route("/commission/reps", get(list_reps))
        .route("/commission/reps/:code", get(get_rep))
        .route("/commission/reps/:code", delete(delete_rep))

        // Commission Plans
        .route("/commission/plans", post(create_commission_plan))
        .route("/commission/plans", get(list_commission_plans))
        .route("/commission/plans/:code", get(get_commission_plan))
        .route("/commission/plans/:code", delete(delete_commission_plan))
        .route("/commission/plans/:id/activate", post(activate_commission_plan))
        .route("/commission/plans/:id/deactivate", post(deactivate_commission_plan))

        // Commission Rate Tiers
        .route("/commission/plans/:plan_id/tiers", post(add_rate_tier))
        .route("/commission/plans/:plan_id/tiers", get(list_rate_tiers))

        // Plan Assignments
        .route("/commission/assignments", post(assign_plan))
        .route("/commission/assignments", get(list_assignments))

        // Sales Quotas
        .route("/commission/quotas", post(create_quota))
        .route("/commission/quotas", get(list_quotas))
        .route("/commission/quotas/:id", get(get_quota))

        // Commission Transactions
        .route("/commission/transactions", post(credit_transaction))
        .route("/commission/transactions", get(list_commission_transactions))
        .route("/commission/transactions/:id", get(get_commission_transaction))

        // Payouts
        .route("/commission/payouts", post(process_payout))
        .route("/commission/payouts", get(list_payouts))
        .route("/commission/payouts/:id", get(get_payout))
        .route("/commission/payouts/:id/lines", get(get_payout_lines))
        .route("/commission/payouts/:id/approve", post(approve_payout))
        .route("/commission/payouts/:id/reject", post(reject_payout))

        // Commission Dashboard
        .route("/commission/dashboard", get(get_commission_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Treasury Management (Oracle Fusion Treasury)
        // ═══════════════════════════════════════════════════════════

        // Counterparties
        .route("/treasury/counterparties", post(create_counterparty))
        .route("/treasury/counterparties", get(list_counterparties))
        .route("/treasury/counterparties/:code", get(get_counterparty))
        .route("/treasury/counterparties/:code", delete(delete_counterparty))

        // Treasury Deals
        .route("/treasury/deals", post(create_deal))
        .route("/treasury/deals", get(list_deals))
        .route("/treasury/deals/:id", get(get_deal))

        // Deal Lifecycle
        .route("/treasury/deals/:id/authorize", post(authorize_deal))
        .route("/treasury/deals/:id/settle", post(settle_deal))
        .route("/treasury/deals/:id/mature", post(mature_deal))
        .route("/treasury/deals/:id/cancel", post(cancel_deal))

        // Deal Settlements
        .route("/treasury/deals/:id/settlements", get(list_deal_settlements))

        // Treasury Dashboard
        .route("/treasury/dashboard", get(get_treasury_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Grant Management (Oracle Fusion Grants Management)
        // ═══════════════════════════════════════════════════════════

        // Sponsors
        .route("/grants/sponsors", post(create_sponsor))
        .route("/grants/sponsors", get(list_sponsors))
        .route("/grants/sponsors/:code", get(get_sponsor))
        .route("/grants/sponsors/:code", delete(delete_sponsor))

        // Indirect Cost Rates
        .route("/grants/indirect-cost-rates", post(create_indirect_cost_rate))
        .route("/grants/indirect-cost-rates", get(list_indirect_cost_rates))

        // Awards
        .route("/grants/awards", post(create_grant_award))
        .route("/grants/awards", get(list_grant_awards))
        .route("/grants/awards/:id", get(get_grant_award))
        .route("/grants/awards/:id/activate", post(activate_award))
        .route("/grants/awards/:id/suspend", post(suspend_award))
        .route("/grants/awards/:id/complete", post(complete_award))
        .route("/grants/awards/:id/terminate", post(terminate_award))

        // Budget Lines
        .route("/grants/awards/:award_id/budget-lines", post(create_grant_budget_line))
        .route("/grants/awards/:award_id/budget-lines", get(list_grant_budget_lines))

        // Expenditures
        .route("/grants/awards/:award_id/expenditures", post(create_grant_expenditure))
        .route("/grants/awards/:award_id/expenditures", get(list_grant_expenditures))
        .route("/grants/expenditures/:id/approve", post(approve_expenditure))
        .route("/grants/expenditures/:id/reverse", post(reverse_expenditure))

        // Billing
        .route("/grants/awards/:award_id/billings", post(create_grant_billing))
        .route("/grants/awards/:award_id/billings", get(list_grant_billings))
        .route("/grants/billings/:id/submit", post(submit_grant_billing))
        .route("/grants/billings/:id/approve", post(approve_grant_billing))
        .route("/grants/billings/:id/pay", post(mark_billing_paid))

        // Compliance Reports
        .route("/grants/awards/:award_id/reports", post(create_compliance_report))
        .route("/grants/awards/:award_id/reports", get(list_compliance_reports))
        .route("/grants/reports/:id/submit", post(submit_compliance_report))
        .route("/grants/reports/:id/approve", post(approve_compliance_report))

        // Grant Dashboard
        .route("/grants/dashboard", get(get_grant_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Supplier Qualification (Oracle Fusion Procurement > Supplier Qualification)
        // ═══════════════════════════════════════════════════════════

        // Qualification Areas
        .route("/supplier-qualification/areas", post(create_qualification_area))
        .route("/supplier-qualification/areas", get(list_qualification_areas))
        .route("/supplier-qualification/areas/:code", get(get_qualification_area))
        .route("/supplier-qualification/areas/:code", delete(delete_qualification_area))

        // Qualification Questions
        .route("/supplier-qualification/areas/:area_id/questions", post(create_qualification_question))
        .route("/supplier-qualification/areas/:area_id/questions", get(list_qualification_questions))
        .route("/supplier-qualification/questions/:id", delete(delete_qualification_question))

        // Initiatives
        .route("/supplier-qualification/initiatives", post(create_initiative))
        .route("/supplier-qualification/initiatives", get(list_initiatives))
        .route("/supplier-qualification/initiatives/:id", get(get_initiative))
        .route("/supplier-qualification/initiatives/:id/activate", post(activate_initiative))
        .route("/supplier-qualification/initiatives/:id/complete", post(complete_initiative))
        .route("/supplier-qualification/initiatives/:id/cancel", post(cancel_initiative))

        // Supplier Invitations
        .route("/supplier-qualification/initiatives/:initiative_id/invitations", post(invite_supplier))
        .route("/supplier-qualification/initiatives/:initiative_id/invitations", get(list_invitations))

        // Invitation Lifecycle
        .route("/supplier-qualification/invitations/:invitation_id/submit", post(submit_invitation_response))
        .route("/supplier-qualification/invitations/:invitation_id/evaluate", post(start_evaluation))
        .route("/supplier-qualification/invitations/:invitation_id/qualify", post(qualify_supplier))
        .route("/supplier-qualification/invitations/:invitation_id/disqualify", post(disqualify_supplier))

        // Responses
        .route("/supplier-qualification/invitations/:invitation_id/responses", post(create_response))
        .route("/supplier-qualification/invitations/:invitation_id/responses", get(list_responses))
        .route("/supplier-qualification/responses/:response_id/score", post(score_response))

        // Certifications
        .route("/supplier-qualification/certifications", post(create_certification))
        .route("/supplier-qualification/certifications", get(list_certifications))
        .route("/supplier-qualification/certifications/:id/revoke", post(revoke_certification))
        .route("/supplier-qualification/certifications/:id/renew", post(renew_certification))

        // Dashboard
        .route("/supplier-qualification/dashboard", get(get_qualification_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Manual Journal Entries (Oracle Fusion GL > Journals > New Journal)
        // ═══════════════════════════════════════════════════════════

        // Journal Batches
        .route("/journals/batches", post(create_journal_batch))
        .route("/journals/batches", get(list_journal_batches))
        .route("/journals/batches/:batch_number", get(get_journal_batch))
        .route("/journals/batches/:batch_number", delete(delete_journal_batch))
        .route("/journals/batches/:id/submit", post(submit_batch))
        .route("/journals/batches/:id/approve", post(approve_journal_batch))
        .route("/journals/batches/:id/reject", post(reject_journal_batch))
        .route("/journals/batches/:id/post", post(post_journal_batch))
        .route("/journals/batches/:id/reverse", post(reverse_journal_batch))

        // Journal Entries
        .route("/journals/batches/:batch_id/entries", post(create_journal_entry))
        .route("/journals/batches/:batch_id/entries", get(list_journal_entries_by_batch))
        .route("/journals/entries/:id", get(get_journal_entry))
        .route("/journals/entries", get(list_journal_entries))
        .route("/journals/entries/:id", delete(delete_journal_entry))

        // Journal Lines
        .route("/journals/entries/:entry_id/lines", post(add_journal_line))
        .route("/journals/entries/:entry_id/lines", get(list_journal_lines))

        // Dashboard
        .route("/journals/dashboard", get(get_manual_journal_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Recurring Journals (Oracle Fusion GL > Recurring Journals)
        // ═══════════════════════════════════════════════════════════

        // Schedules
        .route("/recurring-journals/schedules", post(create_recurring_schedule))
        .route("/recurring-journals/schedules", get(list_recurring_schedules))
        .route("/recurring-journals/schedules/:schedule_number", get(get_recurring_schedule))
        .route("/recurring-journals/schedules/:schedule_number", delete(delete_recurring_schedule))
        .route("/recurring-journals/schedules/:id/activate", post(activate_recurring_schedule))
        .route("/recurring-journals/schedules/:id/deactivate", post(deactivate_recurring_schedule))

        // Schedule Lines
        .route("/recurring-journals/schedules/:schedule_id/lines", post(add_recurring_schedule_line))
        .route("/recurring-journals/schedules/:schedule_id/lines", get(list_recurring_schedule_lines))
        .route("/recurring-journals/lines/:line_id", delete(delete_recurring_schedule_line))

        // Generation
        .route("/recurring-journals/schedules/:schedule_id/generate", post(generate_journal))
        .route("/recurring-journals/generations/:id", get(get_generation))
        .route("/recurring-journals/schedules/:schedule_id/generations", get(list_recurring_generations))
        .route("/recurring-journals/generations/:id/post", post(post_generation))
        .route("/recurring-journals/generations/:id/reverse", post(reverse_generation))
        .route("/recurring-journals/generations/:id/cancel", post(cancel_generation))
        .route("/recurring-journals/generations/:generation_id/lines", get(list_recurring_generation_lines))

        // Dashboard
        .route("/recurring-journals/dashboard", get(get_recurring_journal_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Descriptive Flexfields (Oracle Fusion DFF)
        // ═══════════════════════════════════════════════════════════

        // Value Sets
        .route("/flexfields/value-sets", post(create_value_set))
        .route("/flexfields/value-sets", get(list_value_sets))
        .route("/flexfields/value-sets/:code", get(get_value_set))
        .route("/flexfields/value-sets/:code", delete(delete_value_set))

        // Value Set Entries
        .route("/flexfields/value-sets/:code/entries", post(create_value_set_entry))
        .route("/flexfields/value-sets/:code/entries", get(list_value_set_entries))
        .route("/flexfields/value-sets/entries/:id", delete(delete_value_set_entry))

        // Flexfields
        .route("/flexfields", post(create_flexfield))
        .route("/flexfields", get(list_flexfields))
        .route("/flexfields/:code", get(get_flexfield))
        .route("/flexfields/:id/activate", post(activate_flexfield))
        .route("/flexfields/:id/deactivate", post(deactivate_flexfield))
        .route("/flexfields/:code", delete(delete_flexfield))

        // Contexts
        .route("/flexfields/:flexfield_code/contexts", post(create_context))
        .route("/flexfields/:flexfield_code/contexts", get(list_contexts))
        .route("/flexfields/contexts/:id/disable", post(disable_context))
        .route("/flexfields/contexts/:id/enable", post(enable_context))
        .route("/flexfields/contexts/:id", delete(delete_context))

        // Segments
        .route("/flexfields/:flexfield_code/contexts/:context_code/segments", post(create_segment))
        .route("/flexfields/:flexfield_code/contexts/:context_code/segments", get(list_segments_by_context))
        .route("/flexfields/:flexfield_code/segments", get(list_segments_by_flexfield))
        .route("/flexfields/segments/:id", delete(delete_segment))

        // Flexfield Data (per-record values)
        .route("/flexfields/data/:entity_name/:entity_id", post(set_flexfield_data))
        .route("/flexfields/data/:entity_name/:entity_id", get(get_flexfield_data))
        .route("/flexfields/data/:entity_name/:entity_id", delete(delete_flexfield_data))

        // Dashboard
        .route("/flexfields/dashboard", get(get_flexfield_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Cross-Validation Rules (Oracle Fusion GL > Chart of Accounts > CVR)
        // ═══════════════════════════════════════════════════════════

        // Rules
        .route("/cross-validation/rules", post(create_cvr_rule))
        .route("/cross-validation/rules", get(list_cvr_rules))
        .route("/cross-validation/rules/:code", get(get_cvr_rule))
        .route("/cross-validation/rules/:id/enable", post(enable_cvr_rule))
        .route("/cross-validation/rules/:id/disable", post(disable_cvr_rule))
        .route("/cross-validation/rules/:code", delete(delete_cvr_rule))

        // Rule Lines
        .route("/cross-validation/rules/:rule_code/lines", post(create_cvr_rule_line))
        .route("/cross-validation/rules/:rule_code/lines", get(list_cvr_rule_lines))
        .route("/cross-validation/lines/:id", delete(delete_cvr_rule_line))

        // Validation
        .route("/cross-validation/validate", post(validate_combination))

        // Dashboard
        .route("/cross-validation/dashboard", get(get_cvr_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Scheduled Processes (Oracle Fusion Enterprise Scheduler)
        // ═══════════════════════════════════════════════════════════

        // Process Templates
        .route("/scheduled-processes/templates", post(create_process_template))
        .route("/scheduled-processes/templates", get(list_process_templates))
        .route("/scheduled-processes/templates/:code", get(get_process_template))
        .route("/scheduled-processes/templates/:code", delete(delete_process_template))
        .route("/scheduled-processes/templates/:id/activate", post(activate_process_template))
        .route("/scheduled-processes/templates/:id/deactivate", post(deactivate_process_template))

        // Process Submission & Management
        .route("/scheduled-processes", post(submit_process))
        .route("/scheduled-processes", get(list_processes))
        .route("/scheduled-processes/:id", get(get_process))
        .route("/scheduled-processes/:id/start", post(start_process))
        .route("/scheduled-processes/:id/complete", post(complete_process))
        .route("/scheduled-processes/:id/cancel", post(cancel_process))
        .route("/scheduled-processes/:id/progress", post(update_progress))
        .route("/scheduled-processes/:id/approve", post(approve_process))

        // Process Logs
        .route("/scheduled-processes/:id/logs", get(list_process_logs))
        .route("/scheduled-processes/:id/logs", post(add_process_log))

        // Recurrence Schedules
        .route("/scheduled-processes/recurrences", post(create_process_recurrence))
        .route("/scheduled-processes/recurrences", get(list_process_recurrences))
        .route("/scheduled-processes/recurrences/:id", get(get_process_recurrence))
        .route("/scheduled-processes/recurrences/:id", delete(delete_process_recurrence))
        .route("/scheduled-processes/recurrences/:id/deactivate", post(deactivate_process_recurrence))

        // Cron-like trigger for due recurrences
        .route("/scheduled-processes/recurrences/process-due", post(process_due_recurrences))

        // Dashboard
        .route("/scheduled-processes/dashboard", get(get_scheduled_process_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Segregation of Duties (Oracle Fusion Advanced Access Control)
        // ═══════════════════════════════════════════════════════════

        // SoD Rules
        .route("/sod/rules", post(create_sod_rule))
        .route("/sod/rules", get(list_sod_rules))
        .route("/sod/rules/:code", get(get_sod_rule))
        .route("/sod/rules/:code", delete(delete_sod_rule))
        .route("/sod/rules/:id/activate", post(activate_sod_rule))
        .route("/sod/rules/:id/deactivate", post(deactivate_sod_rule))

        // Role Assignments
        .route("/sod/assignments", post(assign_sod_role))
        .route("/sod/assignments", get(list_sod_assignments))
        .route("/sod/assignments/:id", delete(remove_sod_assignment))

        // Conflict Detection
        .route("/sod/check", post(check_sod_conflict))
        .route("/sod/detect", post(run_sod_detection))

        // Violations
        .route("/sod/violations", get(list_sod_violations))
        .route("/sod/violations/:id", get(get_sod_violation))
        .route("/sod/violations/:id/resolve", post(resolve_sod_violation))
        .route("/sod/violations/:id/exception", post(accept_sod_exception))

        // Mitigating Controls
        .route("/sod/mitigations", post(create_sod_mitigation))
        .route("/sod/violations/:violation_id/mitigations", get(list_sod_mitigations))
        .route("/sod/mitigations/:id/approve", post(approve_sod_mitigation))
        .route("/sod/mitigations/:id/revoke", post(revoke_sod_mitigation))

        // SoD Dashboard
        .route("/sod/dashboard", get(get_sod_dashboard))

        // ═══════════════════════════════════════════════════════════
        // GL Allocations (Oracle Fusion General Ledger > Allocations)
        // ═══════════════════════════════════════════════════════════

        // Allocation Pools
        .route("/allocation/pools", post(create_allocation_pool))
        .route("/allocation/pools", get(list_allocation_pools))
        .route("/allocation/pools/:code", get(get_allocation_pool))
        .route("/allocation/pools/:code", delete(delete_allocation_pool))
        .route("/allocation/pools/:id/activate", post(activate_allocation_pool))
        .route("/allocation/pools/:id/deactivate", post(deactivate_allocation_pool))

        // Allocation Bases
        .route("/allocation/bases", post(create_allocation_basis))
        .route("/allocation/bases", get(list_allocation_bases))
        .route("/allocation/bases/:code", get(get_allocation_basis))
        .route("/allocation/bases/:code", delete(delete_allocation_basis))
        .route("/allocation/bases/:id/activate", post(activate_allocation_basis))
        .route("/allocation/bases/:id/deactivate", post(deactivate_allocation_basis))

        // Basis Details
        .route("/allocation/bases/:basis_code/details", post(add_allocation_basis_detail))
        .route("/allocation/bases/:basis_code/details", get(list_allocation_basis_details))
        .route("/allocation/bases/:basis_code/recalculate", post(recalculate_basis_percentages))

        // Allocation Rules
        .route("/allocation/rules", post(create_allocation_rule))
        .route("/allocation/rules", get(list_allocation_rules))
        .route("/allocation/rules/:code", get(get_allocation_rule))
        .route("/allocation/rules/:code", delete(delete_allocation_rule))
        .route("/allocation/rules/:id/activate", post(activate_allocation_rule))
        .route("/allocation/rules/:id/deactivate", post(deactivate_allocation_rule))

        // Allocation Runs
        .route("/allocation/runs", post(execute_allocation))
        .route("/allocation/runs", get(list_allocation_runs))
        .route("/allocation/runs/:id", get(get_allocation_run))
        .route("/allocation/runs/:id/post", post(post_allocation_run))
        .route("/allocation/runs/:id/reverse", post(reverse_allocation_run))
        .route("/allocation/runs/:id/cancel", post(cancel_allocation_run))

        // Allocation Dashboard
        .route("/allocation/dashboard", get(get_allocation_dashboard))
        
        // ═══════════════════════════════════════════════════════════════
        // Currency Revaluation (Oracle Fusion GL Currency Revaluation)
        // ═══════════════════════════════════════════════════════════════
        
        // Definitions
        .route("/currency-revaluation/definitions", get(list_revaluation_definitions))
        .route("/currency-revaluation/definitions", post(create_revaluation_definition))
        .route("/currency-revaluation/definitions/:code", get(get_revaluation_definition))
        .route("/currency-revaluation/definitions/:code", delete(delete_revaluation_definition))
        .route("/currency-revaluation/definitions/:id/activate", post(activate_revaluation_definition))
        .route("/currency-revaluation/definitions/:id/deactivate", post(deactivate_revaluation_definition))
        
        // Accounts
        .route("/currency-revaluation/definitions/:code/accounts", post(add_revaluation_account))
        .route("/currency-revaluation/definitions/:code/accounts", get(list_revaluation_accounts))
        .route("/currency-revaluation/accounts/:id", delete(remove_revaluation_account))
        
        // Runs
        .route("/currency-revaluation/runs", post(execute_revaluation))
        .route("/currency-revaluation/runs", get(list_revaluation_runs))
        .route("/currency-revaluation/runs/:id", get(get_revaluation_run))
        .route("/currency-revaluation/runs/:id/post", post(post_revaluation_run))
        .route("/currency-revaluation/runs/:id/reverse", post(reverse_revaluation_run))
        .route("/currency-revaluation/runs/:id/cancel", post(cancel_revaluation_run))
        
        // Dashboard
        .route("/currency-revaluation/dashboard", get(get_revaluation_dashboard))

        // ═══════════════════════════════════════════════════════════════
        // Purchase Requisitions (Oracle Fusion Self-Service Procurement > Requisitions)
        // ═══════════════════════════════════════════════════════════════

        // Requisitions
        .route("/requisitions", post(create_requisition))
        .route("/requisitions", get(list_requisitions))
        .route("/requisitions/:id", get(get_requisition))
        .route("/requisitions/:id", put(update_requisition))
        .route("/requisitions/:id", delete(delete_requisition))

        // Requisition Lines
        .route("/requisitions/:requisition_id/lines", post(add_requisition_line))
        .route("/requisitions/:requisition_id/lines", get(list_requisition_lines))
        .route("/requisitions/lines/:line_id", delete(remove_requisition_line))

        // Distributions
        .route("/requisitions/:requisition_id/lines/:line_id/distributions", post(add_requisition_distribution))
        .route("/requisitions/lines/:line_id/distributions", get(list_requisition_distributions))

        // Approval Workflow
        .route("/requisitions/:id/submit", post(submit_requisition))
        .route("/requisitions/:id/approve", post(approve_requisition))
        .route("/requisitions/:id/reject", post(reject_requisition))
        .route("/requisitions/:id/cancel", post(cancel_requisition))
        .route("/requisitions/:id/close", post(close_requisition))
        .route("/requisitions/:id/return", post(return_requisition))
        .route("/requisitions/:id/approvals", get(list_requisition_approvals))

        // AutoCreate (Convert to PO)
        .route("/requisitions/autocreate", post(autocreate))
        .route("/requisitions/:requisition_id/autocreate-links", get(list_autocreate_links))
        .route("/requisitions/autocreate/:link_id/cancel", post(cancel_autocreate_link))

        // Dashboard
        .route("/requisitions/dashboard", get(get_requisition_dashboard))

        // ══════════════════════════════════════════════════════════════════════
        // Corporate Card Management (Oracle Fusion Expenses > Corporate Cards)
        // ══════════════════════════════════════════════════════════════════════

        // Card Programmes
        .route("/corporate-cards/programs", get(list_corporate_card_programs))
        .route("/corporate-cards/programs", post(create_corporate_card_program))
        .route("/corporate-cards/programs/:code", get(get_corporate_card_program))

        // Cards
        .route("/corporate-cards/cards", get(list_corporate_cards))
        .route("/corporate-cards/cards", post(issue_card))
        .route("/corporate-cards/cards/:id", get(get_corporate_card))
        .route("/corporate-cards/cards/:id/suspend", post(suspend_card))
        .route("/corporate-cards/cards/:id/reactivate", post(reactivate_card))
        .route("/corporate-cards/cards/:id/cancel", post(cancel_card))
        .route("/corporate-cards/cards/:id/lost", post(report_lost))
        .route("/corporate-cards/cards/:id/stolen", post(report_stolen))

        // Transactions
        .route("/corporate-cards/transactions", get(list_corporate_card_transactions))
        .route("/corporate-cards/transactions", post(import_transaction))
        .route("/corporate-cards/transactions/:id", get(get_corporate_card_transaction))
        .route("/corporate-cards/transactions/:id/match", post(match_corporate_card_transaction))
        .route("/corporate-cards/transactions/:id/unmatch", post(unmatch_corporate_card_transaction))
        .route("/corporate-cards/transactions/:id/dispute", post(dispute_corporate_card_transaction))
        .route("/corporate-cards/transactions/:id/resolve-dispute", post(resolve_corporate_card_dispute))

        // Statements
        .route("/corporate-cards/statements", get(list_corporate_card_statements))
        .route("/corporate-cards/statements", post(import_corporate_card_statement))
        .route("/corporate-cards/statements/:id", get(get_corporate_card_statement))
        .route("/corporate-cards/statements/:id/reconcile", post(reconcile_corporate_card_statement))
        .route("/corporate-cards/statements/:id/pay", post(pay_corporate_card_statement))

        // Spending Limit Overrides
        .route("/corporate-cards/limit-overrides", get(list_limit_overrides))
        .route("/corporate-cards/limit-overrides", post(request_limit_override))
        .route("/corporate-cards/limit-overrides/:id/approve", post(approve_limit_override))
        .route("/corporate-cards/limit-overrides/:id/reject", post(reject_limit_override))

        // Dashboard
        .route("/corporate-cards/dashboard", get(get_corporate_card_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Benefits Administration (Oracle Fusion HCM > Benefits)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Benefits Plans
        .route("/benefits/plans", get(list_benefits_plans))
        .route("/benefits/plans", post(create_benefits_plan))
        .route("/benefits/plans/:code", get(get_benefits_plan))
        .route("/benefits/plans/:code", delete(delete_benefits_plan))

        // Benefits Enrollments
        .route("/benefits/enrollments", post(create_benefits_enrollment))
        .route("/benefits/enrollments", get(list_benefits_enrollments))
        .route("/benefits/enrollments/:id", get(get_benefits_enrollment))
        .route("/benefits/enrollments/:id/activate", post(activate_benefits_enrollment))
        .route("/benefits/enrollments/:id/waive", post(waive_benefits_enrollment))
        .route("/benefits/enrollments/:id/cancel", post(cancel_benefits_enrollment))
        .route("/benefits/enrollments/:id/suspend", post(suspend_benefits_enrollment))
        .route("/benefits/enrollments/:id/reactivate", post(reactivate_benefits_enrollment))

        // Benefits Deductions
        .route("/benefits/deductions/generate", post(generate_deductions))
        .route("/benefits/deductions", get(list_benefits_deductions))

        // Benefits Dashboard
        .route("/benefits/dashboard", get(get_benefits_dashboard))

        // ═══════════════════════════════════════════════════════════════════════════════════
        // Performance Management (Oracle Fusion HCM > Performance)
        // ═══════════════════════════════════════════════════════════════════════════════════

        // Rating Models
        .route("/performance/rating-models", get(list_rating_models))
        .route("/performance/rating-models", post(create_rating_model))
        .route("/performance/rating-models/:code", get(get_rating_model))
        .route("/performance/rating-models/:code", delete(delete_rating_model))

        // Review Cycles
        .route("/performance/cycles", post(create_review_cycle))
        .route("/performance/cycles", get(list_review_cycles))
        .route("/performance/cycles/:id", get(get_review_cycle))
        .route("/performance/cycles/:id/transition", post(transition_cycle))

        // Competencies
        .route("/performance/competencies", post(create_competency))
        .route("/performance/competencies", get(list_competencies))
        .route("/performance/competencies/:code", get(get_competency))
        .route("/performance/competencies/:code", delete(delete_competency))

        // Performance Documents
        .route("/performance/documents", post(create_performance_document))
        .route("/performance/documents", get(list_performance_documents))
        .route("/performance/documents/:id", get(get_performance_document))
        .route("/performance/documents/:id/transition", post(transition_document))
        .route("/performance/documents/:id/self-evaluation", post(submit_self_evaluation))
        .route("/performance/documents/:id/manager-evaluation", post(submit_manager_evaluation))
        .route("/performance/documents/:id/finalize", post(finalize_performance_document))

        // Goals
        .route("/performance/goals", post(create_goal))
        .route("/performance/documents/:document_id/goals", get(list_performance_goals))
        .route("/performance/goals/:id/complete", post(complete_goal))
        .route("/performance/goals/:id/rate", post(rate_goal))
        .route("/performance/goals/:id", delete(delete_performance_goal))

        // Competency Assessments
        .route("/performance/assessments", post(upsert_competency_assessment))
        .route("/performance/documents/:document_id/assessments", get(list_competency_assessments_for_document))

        // Feedback
        .route("/performance/feedback", post(create_performance_feedback))
        .route("/performance/feedback", get(list_performance_feedback))
        .route("/performance/feedback/:id/submit", post(submit_performance_feedback))

        // Performance Dashboard
        .route("/performance/dashboard/:review_cycle_id", get(get_performance_dashboard))

        // Credit Management - Scoring Models
        .route("/credit/scoring-models", post(create_scoring_model))
        .route("/credit/scoring-models", get(list_scoring_models))
        .route("/credit/scoring-models/:code", get(get_scoring_model))
        .route("/credit/scoring-models/:code", delete(delete_scoring_model))

        // Credit Management - Profiles
        .route("/credit/profiles", post(create_profile))
        .route("/credit/profiles", get(list_profiles))
        .route("/credit/profiles/:id", get(get_profile))
        .route("/credit/profiles/:id/status", post(update_profile_status))
        .route("/credit/profiles/:id/score", post(update_profile_score))
        .route("/credit/profiles/:id", delete(delete_profile))

        // Credit Management - Limits
        .route("/credit/limits", post(create_credit_limit))
        .route("/credit/limits/:id", put(update_credit_limit))
        .route("/credit/limits/:id/temp", post(set_temp_limit))
        .route("/credit/limits/:id", delete(delete_credit_limit))
        .route("/credit/profiles/:profile_id/limits", get(list_credit_limits))

        // Credit Management - Check Rules
        .route("/credit/check-rules", post(create_check_rule))
        .route("/credit/check-rules", get(list_check_rules))
        .route("/credit/check-rules/:id", delete(delete_check_rule))

        // Credit Management - Exposure
        .route("/credit/exposure/calculate", post(calculate_exposure))
        .route("/credit/exposure/check", post(perform_credit_check))
        .route("/credit/profiles/:profile_id/exposure", get(get_latest_exposure))

        // Credit Management - Holds
        .route("/credit/holds", post(create_hold))
        .route("/credit/holds", get(list_holds))
        .route("/credit/holds/:id/release", post(release_hold))
        .route("/credit/holds/:id/override", post(override_hold))

        // Credit Management - Reviews
        .route("/credit/reviews", post(create_review))
        .route("/credit/reviews", get(list_reviews))
        .route("/credit/reviews/:id/start", post(start_review))
        .route("/credit/reviews/:id/complete", post(complete_review))
        .route("/credit/reviews/:id/approve", post(approve_review))
        .route("/credit/reviews/:id/reject", post(reject_review))
        .route("/credit/reviews/:id/cancel", post(cancel_review))

        // Credit Management - Dashboard
        .route("/credit/dashboard", get(get_credit_dashboard))

        // ═══════════════════════════════════════════════════════
        // Product Information Management (Oracle Fusion Product Hub)
        // ═══════════════════════════════════════════════════════

        // Product Items
        .route("/pim/items", post(product_information::create_item))
        .route("/pim/items", get(product_information::list_items))
        .route("/pim/items/by-number/:item_number", get(product_information::get_item_by_number))
        .route("/pim/items/:id", get(product_information::get_item))
        .route("/pim/items/:id/status", post(product_information::update_item_status))
        .route("/pim/items/:id/lifecycle", post(product_information::update_item_lifecycle))
        .route("/pim/items/:id", delete(product_information::delete_item))

        // Item Categories
        .route("/pim/categories", post(product_information::create_category))
        .route("/pim/categories", get(product_information::list_categories))
        .route("/pim/categories/:id", get(product_information::get_category))
        .route("/pim/categories/:id", delete(product_information::delete_category))

        // Item Category Assignments
        .route("/pim/items/:item_id/categories", post(product_information::assign_item_category))
        .route("/pim/items/:item_id/categories", get(product_information::list_item_categories))
        .route("/pim/item-categories/:assignment_id", delete(product_information::remove_item_category))

        // Item Cross-References
        .route("/pim/items/:item_id/cross-references", post(product_information::create_cross_reference))
        .route("/pim/items/:item_id/cross-references", get(product_information::list_cross_references))
        .route("/pim/cross-references", get(product_information::list_all_cross_references))
        .route("/pim/cross-references/:id", delete(product_information::delete_cross_reference))

        // Item Templates
        .route("/pim/templates", post(product_information::create_template))
        .route("/pim/templates", get(product_information::list_templates))
        .route("/pim/templates/:id", delete(product_information::delete_template))

        // New Item Requests (NIR)
        .route("/pim/new-item-requests", post(product_information::create_new_item_request))
        .route("/pim/new-item-requests", get(product_information::list_new_item_requests))
        .route("/pim/new-item-requests/:id", get(product_information::get_new_item_request))
        .route("/pim/new-item-requests/:id/submit", post(product_information::submit_new_item_request))
        .route("/pim/new-item-requests/:id/approve", post(product_information::approve_new_item_request))
        .route("/pim/new-item-requests/:id/reject", post(product_information::reject_new_item_request))
        .route("/pim/new-item-requests/:id/implement", post(product_information::implement_new_item_request))
        .route("/pim/new-item-requests/:id/cancel", post(product_information::cancel_new_item_request))

        // PIM Dashboard
        .route("/pim/dashboard", get(product_information::get_pim_dashboard))

        // ═══════════════════════════════════════════════════════
        // Transfer Pricing (Oracle Fusion Financials > Transfer Pricing)
        // ═══════════════════════════════════════════════════════

        // Policies
        .route("/transfer-pricing/policies", post(transfer_pricing::create_policy))
        .route("/transfer-pricing/policies", get(transfer_pricing::list_policies))
        .route("/transfer-pricing/policies/:code", get(transfer_pricing::get_policy))
        .route("/transfer-pricing/policies/:id/activate", post(transfer_pricing::activate_policy))
        .route("/transfer-pricing/policies/:id/deactivate", post(transfer_pricing::deactivate_policy))
        .route("/transfer-pricing/policies/:code", delete(transfer_pricing::delete_policy))

        // Transactions
        .route("/transfer-pricing/transactions", post(transfer_pricing::create_transaction))
        .route("/transfer-pricing/transactions", get(transfer_pricing::list_transactions))
        .route("/transfer-pricing/transactions/:id", get(transfer_pricing::get_transaction))
        .route("/transfer-pricing/transactions/:id/submit", post(transfer_pricing::submit_transaction))
        .route("/transfer-pricing/transactions/:id/approve", post(transfer_pricing::approve_transaction))
        .route("/transfer-pricing/transactions/:id/reject", post(transfer_pricing::reject_transaction))

        // Benchmarks
        .route("/transfer-pricing/benchmarks", post(transfer_pricing::create_benchmark))
        .route("/transfer-pricing/benchmarks", get(transfer_pricing::list_benchmarks))
        .route("/transfer-pricing/benchmarks/:id", get(transfer_pricing::get_benchmark))
        .route("/transfer-pricing/benchmarks/:id/submit", post(transfer_pricing::submit_benchmark))
        .route("/transfer-pricing/benchmarks/:id/approve", post(transfer_pricing::approve_benchmark))
        .route("/transfer-pricing/benchmarks/:id/reject", post(transfer_pricing::reject_benchmark))
        .route("/transfer-pricing/benchmarks/:id", delete(transfer_pricing::delete_benchmark))

        // Comparables
        .route("/transfer-pricing/benchmarks/:benchmark_id/comparables", post(transfer_pricing::add_comparable))
        .route("/transfer-pricing/benchmarks/:benchmark_id/comparables", get(transfer_pricing::list_comparables))

        // Documentation
        .route("/transfer-pricing/documentation", post(transfer_pricing::create_documentation))
        .route("/transfer-pricing/documentation", get(transfer_pricing::list_documentation))
        .route("/transfer-pricing/documentation/:id", get(transfer_pricing::get_documentation))
        .route("/transfer-pricing/documentation/:id/submit", post(transfer_pricing::submit_documentation))
        .route("/transfer-pricing/documentation/:id/approve", post(transfer_pricing::approve_documentation))
        .route("/transfer-pricing/documentation/:id/file", post(transfer_pricing::file_documentation))

        // Dashboard
        .route("/transfer-pricing/dashboard", get(transfer_pricing::get_tp_dashboard))

        .layer(middleware::from_fn(auth_middleware))
}

/// Admin routes - requires authentication
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/schema", post(create_entity))
        .route("/schema/:entity", put(update_entity))
        .route("/schema/:entity", delete(delete_entity))
        
        .route("/workflows", post(create_workflow))
        .route("/workflows/:entity", put(update_workflow))
        
        .route("/config", get(get_config))
        .route("/config/:key", get(get_config_value))
        .route("/config/:key", put(set_config_value))
        
        // Oracle Fusion: Duplicate detection rules
        .route("/duplicate-rules", post(create_duplicate_rule))
        
        .route("/cache/clear", post(clear_cache))
        .route("/cache/invalidate/:entity", post(invalidate_entity_cache))
        .layer(middleware::from_fn(auth_middleware))
}
