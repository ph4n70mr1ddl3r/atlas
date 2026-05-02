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
pub mod order_management;
pub mod approval_delegation;
pub mod compensation;
pub mod manufacturing;
pub mod warehouse_management;
pub mod absence;
pub mod time_and_labor;
pub mod approval_authority;
pub mod data_archiving;
pub mod service_request;
pub mod lead_opportunity;
pub mod demand_planning;
mod autoinvoice;
mod shipping;
mod recruiting;
pub mod revenue;
pub mod marketing;
pub mod receiving;
pub mod supplier_scorecard;
pub mod kpi;
pub mod account_monitor;
pub mod goal_management;
pub mod contract_lifecycle;
pub mod enterprise_asset_management;
pub mod risk_management;
pub mod product_configurator;
pub mod territory_management;
pub mod transportation_management;
pub mod sustainability;
pub mod promotions_management;
pub mod project_billing;
pub mod quality_management;
pub mod cost_accounting;
pub mod accounts_payable;
pub mod supply_chain_planning;
pub mod health_safety;
pub mod funds_reservation;
pub mod rebate_management;
pub mod project_resource_management;
pub mod loyalty_management;
pub mod general_ledger;
pub mod accounts_receivable;
pub mod payment_management;
pub mod netting;
pub mod financial_statements;
pub mod journal_import;
pub mod inflation_adjustment;
pub mod impairment_management;
pub mod bank_account_transfer;
pub mod tax_reporting;
pub mod subscription;
pub mod financial_consolidation;
pub mod joint_venture;
pub mod deferred_revenue;
pub mod revenue_management;
pub mod cash_flow_forecast;
pub mod regulatory_reporting;
pub mod advance_payment;
pub mod customer_deposit;
pub mod cash_position;
pub mod accounting_hub;
pub mod financial_controls;
pub mod payment_terms;
pub mod lockbox;
pub mod ar_aging;

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
use crate::middleware::{auth_middleware, admin_auth_middleware};
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

        // ========================================================================
        // Order Management (Oracle Fusion SCM > Order Management)
        // ========================================================================
        .route("/orders", post(order_management::create_order))
        .route("/orders", get(order_management::list_orders))
        .route("/orders/:id", get(order_management::get_order_by_id))
        .route("/orders/by-number/:order_number", get(order_management::get_order))
        .route("/orders/:id/submit", post(order_management::submit_order))
        .route("/orders/:id/confirm", post(order_management::confirm_order))
        .route("/orders/:id/close", post(order_management::close_order))
        .route("/orders/:id/cancel", post(order_management::cancel_order))

        // Order Lines
        .route("/orders/lines", post(order_management::add_order_line))
        .route("/orders/lines/:id", get(order_management::get_order_line))
        .route("/orders/lines/:id/ship", post(order_management::ship_order_line))
        .route("/orders/lines/:id/cancel", post(order_management::cancel_order_line))
        .route("/orders/:order_id/lines", get(order_management::list_order_lines))

        // Order Holds
        .route("/orders/holds", post(order_management::apply_hold))
        .route("/orders/holds/:id/release", post(order_management::release_hold))
        .route("/orders/:order_id/holds", get(order_management::list_holds))

        // Shipments
        .route("/orders/shipments", post(order_management::create_shipment))
        .route("/orders/shipments", get(order_management::list_shipments))
        .route("/orders/shipments/:id", get(order_management::get_shipment))
        .route("/orders/shipments/:id/confirm", post(order_management::confirm_shipment))
        .route("/orders/shipments/:id/tracking", post(order_management::update_tracking))
        .route("/orders/shipments/:id/deliver", post(order_management::confirm_delivery))

        // Order Management Dashboard
        .route("/orders/dashboard", get(order_management::get_order_management_dashboard))

        // ═══════════════════════════════════════════════════════
        // Approval Delegation Rules (Oracle Fusion BPM Worklist > Delegation)
        // ═══════════════════════════════════════════════════════

        // Delegation Rules
        .route("/approval-delegation/rules", post(approval_delegation::create_delegation_rule))
        .route("/approval-delegation/rules", get(approval_delegation::list_delegation_rules))
        .route("/approval-delegation/rules/my", get(approval_delegation::list_my_delegation_rules))
        .route("/approval-delegation/rules/:id", get(approval_delegation::get_delegation_rule))
        .route("/approval-delegation/rules/:id", delete(approval_delegation::delete_delegation_rule))
        .route("/approval-delegation/rules/:id/activate", post(approval_delegation::activate_delegation_rule))
        .route("/approval-delegation/rules/:id/cancel", post(approval_delegation::cancel_delegation_rule))

        // Process Scheduled Rules (Admin/Cron)
        .route("/approval-delegation/process-scheduled", post(approval_delegation::process_scheduled_delegations))

        // Delegation History
        .route("/approval-delegation/history", get(approval_delegation::list_delegation_history))

        // Delegation Dashboard
        .route("/approval-delegation/dashboard", get(approval_delegation::get_delegation_dashboard))

        // ═══════════════════════════════════════════════════════
        // Approval Authority Limits (Oracle Fusion BPM > Document Approval Limits)
        // ═══════════════════════════════════════════════════════

        // Limit CRUD
        .route("/approval-authority/limits", post(approval_authority::create_authority_limit))
        .route("/approval-authority/limits", get(approval_authority::list_authority_limits))
        .route("/approval-authority/limits/:id", get(approval_authority::get_authority_limit))
        .route("/approval-authority/limits/:id", delete(approval_authority::delete_authority_limit))
        .route("/approval-authority/limits/:id/activate", post(approval_authority::activate_authority_limit))
        .route("/approval-authority/limits/:id/deactivate", post(approval_authority::deactivate_authority_limit))

        // Authority Check
        .route("/approval-authority/check", post(approval_authority::check_authority))

        // Check Audit Trail
        .route("/approval-authority/audits", get(approval_authority::list_check_audits))

        // Authority Dashboard
        .route("/approval-authority/dashboard", get(approval_authority::get_authority_dashboard))

        // ═══════════════════════════════════════════════════════
        // Data Archiving & Retention Management (Oracle Fusion ILM)
        // ═══════════════════════════════════════════════════════

        // Retention Policies
        .route("/data-archiving/policies", post(data_archiving::create_retention_policy))
        .route("/data-archiving/policies", get(data_archiving::list_retention_policies))
        .route("/data-archiving/policies/:id", get(data_archiving::get_retention_policy))
        .route("/data-archiving/policies/:id", delete(data_archiving::delete_retention_policy))
        .route("/data-archiving/policies/:id/activate", post(data_archiving::activate_retention_policy))
        .route("/data-archiving/policies/:id/deactivate", post(data_archiving::deactivate_retention_policy))

        // Legal Holds
        .route("/data-archiving/legal-holds", post(data_archiving::create_legal_hold))
        .route("/data-archiving/legal-holds", get(data_archiving::list_legal_holds))
        .route("/data-archiving/legal-holds/:id", get(data_archiving::get_legal_hold))
        .route("/data-archiving/legal-holds/:id", delete(data_archiving::delete_legal_hold))
        .route("/data-archiving/legal-holds/:id/release", post(data_archiving::release_legal_hold))

        // Legal Hold Items
        .route("/data-archiving/legal-holds/:id/items", post(data_archiving::add_legal_hold_items))
        .route("/data-archiving/legal-holds/:id/items", get(data_archiving::list_legal_hold_items))
        .route("/data-archiving/legal-holds/items/:id", delete(data_archiving::remove_legal_hold_item))
        .route("/data-archiving/holds/check", get(data_archiving::check_legal_hold))

        // Archive Operations
        .route("/data-archiving/archive", post(data_archiving::execute_archive))
        .route("/data-archiving/archived", get(data_archiving::list_archived_records))
        .route("/data-archiving/archived/:id", get(data_archiving::get_archived_record))
        .route("/data-archiving/archived/:id/restore", post(data_archiving::restore_archived_record))
        .route("/data-archiving/archived/:id/purge", post(data_archiving::purge_archived_record))

        // Archive Batches
        .route("/data-archiving/batches", get(data_archiving::list_archive_batches))
        .route("/data-archiving/batches/:id", get(data_archiving::get_archive_batch))

        // Archive Audit
        .route("/data-archiving/audit", get(data_archiving::list_archive_audit))

        // Data Archiving Dashboard
        .route("/data-archiving/dashboard", get(data_archiving::get_data_archiving_dashboard))

        // ═══════════════════════════════════════════════════════
        // Manufacturing Execution (Oracle Fusion SCM > Manufacturing)
        // ═══════════════════════════════════════════════════════

        // Work Definitions
        .route("/manufacturing/definitions", post(manufacturing::create_work_definition))
        .route("/manufacturing/definitions", get(manufacturing::list_work_definitions))
        .route("/manufacturing/definitions/:definition_number", get(manufacturing::get_work_definition))
        .route("/manufacturing/definitions/:id/activate", post(manufacturing::activate_work_definition))
        .route("/manufacturing/definitions/:id/deactivate", post(manufacturing::deactivate_work_definition))
        .route("/manufacturing/definitions/delete/:id", delete(manufacturing::delete_work_definition))

        // Work Definition BOM Components
        .route("/manufacturing/definitions/:id/components", post(manufacturing::add_work_definition_component))
        .route("/manufacturing/definitions/:id/components", get(manufacturing::list_work_definition_components))
        .route("/manufacturing/definitions/components/:id", delete(manufacturing::delete_work_definition_component))

        // Work Definition Routing Operations
        .route("/manufacturing/definitions/:id/operations", post(manufacturing::add_work_definition_operation))
        .route("/manufacturing/definitions/:id/operations", get(manufacturing::list_work_definition_operations))
        .route("/manufacturing/definitions/operations/:id", delete(manufacturing::delete_work_definition_operation))

        // Work Orders
        .route("/manufacturing/work-orders", post(manufacturing::create_work_order))
        .route("/manufacturing/work-orders", get(manufacturing::list_work_orders))
        .route("/manufacturing/work-orders/:work_order_number", get(manufacturing::get_work_order))
        .route("/manufacturing/work-orders/:id/release", post(manufacturing::release_work_order))
        .route("/manufacturing/work-orders/:id/start", post(manufacturing::start_work_order))
        .route("/manufacturing/work-orders/:id/complete", post(manufacturing::complete_work_order))
        .route("/manufacturing/work-orders/:id/close", post(manufacturing::close_work_order))
        .route("/manufacturing/work-orders/:id/cancel", post(manufacturing::cancel_work_order))

        // Production Reporting
        .route("/manufacturing/work-orders/:id/report-completion", post(manufacturing::report_completion))
        .route("/manufacturing/work-orders/:id/issue-materials", post(manufacturing::issue_materials))
        .route("/manufacturing/materials/:id/return", post(manufacturing::return_material))

        // Work Order Operations & Materials
        .route("/manufacturing/work-orders/:id/operations", get(manufacturing::list_work_order_operations))
        .route("/manufacturing/work-orders/operations/:id/status", post(manufacturing::update_operation_status))
        .route("/manufacturing/work-orders/:id/materials", get(manufacturing::list_work_order_materials))

        // Manufacturing Dashboard
        .route("/manufacturing/dashboard", get(manufacturing::get_manufacturing_dashboard))

        // ═══════════════════════════════════════════════════════
        // Warehouse Management (Oracle Fusion Cloud Warehouse Management)
        // ═══════════════════════════════════════════════════════

        // Warehouses
        .route("/warehouse/warehouses", post(warehouse_management::create_warehouse))
        .route("/warehouse/warehouses", get(warehouse_management::list_warehouses))
        .route("/warehouse/warehouses/:id", get(warehouse_management::get_warehouse))
        .route("/warehouse/warehouses/:id/delete", delete(warehouse_management::delete_warehouse))

        // Warehouse Zones
        .route("/warehouse/warehouses/:id/zones", post(warehouse_management::create_zone))
        .route("/warehouse/warehouses/:id/zones", get(warehouse_management::list_zones))
        .route("/warehouse/zones/:id", delete(warehouse_management::delete_zone))

        // Put-Away Rules
        .route("/warehouse/warehouses/:id/put-away-rules", post(warehouse_management::create_put_away_rule))
        .route("/warehouse/warehouses/:id/put-away-rules", get(warehouse_management::list_put_away_rules))
        .route("/warehouse/put-away-rules/:id", delete(warehouse_management::delete_put_away_rule))

        // Warehouse Tasks
        .route("/warehouse/tasks", post(warehouse_management::create_task))
        .route("/warehouse/warehouses/:id/tasks", post(warehouse_management::create_task_for_warehouse))
        .route("/warehouse/tasks", get(warehouse_management::list_tasks))
        .route("/warehouse/tasks/:id", get(warehouse_management::get_task))
        .route("/warehouse/tasks/:id/start", post(warehouse_management::start_task))
        .route("/warehouse/tasks/:id/complete", post(warehouse_management::complete_task))
        .route("/warehouse/tasks/:id/cancel", post(warehouse_management::cancel_task))
        .route("/warehouse/tasks/:id", delete(warehouse_management::delete_task))

        // Pick Waves
        .route("/warehouse/warehouses/:id/waves", post(warehouse_management::create_wave))
        .route("/warehouse/waves", get(warehouse_management::list_waves))
        .route("/warehouse/waves/:id", get(warehouse_management::get_wave))
        .route("/warehouse/waves/:id/release", post(warehouse_management::release_wave))
        .route("/warehouse/waves/:id/complete", post(warehouse_management::complete_wave))
        .route("/warehouse/waves/:id/cancel", post(warehouse_management::cancel_wave))
        .route("/warehouse/waves/:id", delete(warehouse_management::delete_wave))

        // Warehouse Dashboard
        .route("/warehouse/dashboard", get(warehouse_management::get_warehouse_dashboard))

        // ═══════════════════════════════════════════════════════════════
        // Absence Management (Oracle Fusion Cloud HCM)
        // ═══════════════════════════════════════════════════════════════

        // Absence Types
        .route("/absence/types", post(absence::create_absence_type))
        .route("/absence/types", get(absence::list_absence_types))
        .route("/absence/types/:code", get(absence::get_absence_type))
        .route("/absence/types/:code", delete(absence::delete_absence_type))

        // Absence Plans
        .route("/absence/plans", post(absence::create_absence_plan))
        .route("/absence/plans", get(absence::list_absence_plans))
        .route("/absence/plans/:code", get(absence::get_absence_plan))
        .route("/absence/plans/:code", delete(absence::delete_absence_plan))

        // Absence Entries
        .route("/absence/entries", post(absence::create_entry))
        .route("/absence/entries", get(absence::list_entries))
        .route("/absence/entries/:id", get(absence::get_entry))
        .route("/absence/entries/:id/submit", post(absence::submit_entry))
        .route("/absence/entries/:id/approve", post(absence::approve_entry))
        .route("/absence/entries/:id/reject", post(absence::reject_entry))
        .route("/absence/entries/:id/cancel", post(absence::cancel_entry))

        // Absence Entry History
        .route("/absence/entries/:id/history", get(absence::get_entry_history))

        // Absence Balances
        .route("/absence/balances", get(absence::get_balance))
        .route("/absence/balances/list", get(absence::list_balances))

        // Absence Dashboard
        .route("/absence/dashboard", get(absence::get_absence_dashboard))

        // ═══════════════════════════════════════════════════════════════
        // Time and Labor Management (Oracle Fusion Cloud HCM Time and Labor)
        // ═══════════════════════════════════════════════════════════════

        // Work Schedules
        .route("/time-and-labor/schedules", post(time_and_labor::create_work_schedule))
        .route("/time-and-labor/schedules", get(time_and_labor::list_work_schedules))
        .route("/time-and-labor/schedules/:code", get(time_and_labor::get_work_schedule))
        .route("/time-and-labor/schedules/:code", delete(time_and_labor::delete_work_schedule))

        // Overtime Rules
        .route("/time-and-labor/overtime-rules", post(time_and_labor::create_overtime_rule))
        .route("/time-and-labor/overtime-rules", get(time_and_labor::list_overtime_rules))
        .route("/time-and-labor/overtime-rules/:code", get(time_and_labor::get_overtime_rule))
        .route("/time-and-labor/overtime-rules/:code", delete(time_and_labor::delete_overtime_rule))

        // Time Cards
        .route("/time-and-labor/time-cards", post(time_and_labor::create_time_card))
        .route("/time-and-labor/time-cards", get(time_and_labor::list_time_cards))
        .route("/time-and-labor/time-cards/:id", get(time_and_labor::get_time_card))
        .route("/time-and-labor/time-cards/:id/submit", post(time_and_labor::submit_time_card))
        .route("/time-and-labor/time-cards/:id/approve", post(time_and_labor::approve_time_card))
        .route("/time-and-labor/time-cards/:id/reject", post(time_and_labor::reject_time_card))
        .route("/time-and-labor/time-cards/:id/cancel", post(time_and_labor::cancel_time_card))

        // Time Entries
        .route("/time-and-labor/entries", post(time_and_labor::create_time_entry))
        .route("/time-and-labor/entries/time-card/:time_card_id", get(time_and_labor::list_time_entries))
        .route("/time-and-labor/entries/:id", delete(time_and_labor::delete_time_entry))

        // Time Card History
        .route("/time-and-labor/time-cards/:time_card_id/history", get(time_and_labor::get_time_card_history))

        // Labor Distributions
        .route("/time-and-labor/distributions", post(time_and_labor::create_labor_distribution))
        .route("/time-and-labor/distributions/entry/:time_entry_id", get(time_and_labor::list_labor_distributions))
        .route("/time-and-labor/distributions/:id", delete(time_and_labor::delete_labor_distribution))

        // Time and Labor Dashboard
        .route("/time-and-labor/dashboard", get(time_and_labor::get_time_and_labor_dashboard))

        // ═══════════════════════════════════════════════════════════════
        // Compensation Management (Oracle Fusion Cloud HCM Compensation Workbench)
        // ═══════════════════════════════════════════════════════════════

        // Compensation Plans
        .route("/compensation/plans", post(compensation::create_plan))
        .route("/compensation/plans", get(compensation::list_plans))
        .route("/compensation/plans/:code", get(compensation::get_plan))
        .route("/compensation/plans/:code", delete(compensation::delete_plan))

        // Plan Components
        .route("/compensation/plans/:plan_code/components", post(compensation::create_component))
        .route("/compensation/plans/:plan_code/components", get(compensation::list_components))

        // Compensation Cycles
        .route("/compensation/cycles", post(compensation::create_cycle))
        .route("/compensation/cycles", get(compensation::list_cycles))
        .route("/compensation/cycles/:id", get(compensation::get_cycle))
        .route("/compensation/cycles/:id/transition", post(compensation::transition_cycle))
        .route("/compensation/cycles/:id", delete(compensation::delete_cycle))

        // Budget Pools
        .route("/compensation/cycles/:cycle_id/pools", post(compensation::create_budget_pool))
        .route("/compensation/cycles/:cycle_id/pools", get(compensation::list_budget_pools))

        // Worksheets
        .route("/compensation/cycles/:cycle_id/worksheets", post(compensation::create_worksheet))
        .route("/compensation/cycles/:cycle_id/worksheets", get(compensation::list_worksheets))
        .route("/compensation/worksheets/:id", get(compensation::get_worksheet))
        .route("/compensation/worksheets/:id/submit", post(compensation::submit_worksheet))
        .route("/compensation/worksheets/:id/approve", post(compensation::approve_worksheet))
        .route("/compensation/worksheets/:id/reject", post(compensation::reject_worksheet))

        // Worksheet Lines
        .route("/compensation/worksheets/:worksheet_id/lines", post(compensation::add_worksheet_line))
        .route("/compensation/worksheets/:worksheet_id/lines", get(compensation::list_worksheet_lines))
        .route("/compensation/lines/:line_id", put(compensation::update_worksheet_line))
        .route("/compensation/lines/:line_id", delete(compensation::delete_worksheet_line))

        // Compensation Statements
        .route("/compensation/cycles/:cycle_id/statements", post(compensation::generate_statement))
        .route("/compensation/cycles/:cycle_id/statements", get(compensation::list_statements))
        .route("/compensation/statements/:id", get(compensation::get_statement))
        .route("/compensation/statements/:id/publish", post(compensation::publish_statement))

        // Compensation Dashboard
        .route("/compensation/dashboard", get(compensation::get_dashboard))

        // ═══════════════════════════════════════════════════════
        // Service Request Management (Oracle Fusion CX Service)
        // ═══════════════════════════════════════════════════════

        // Service Categories
        .route("/service/categories", post(service_request::create_category))
        .route("/service/categories", get(service_request::list_categories))
        .route("/service/categories/:code", get(service_request::get_category))
        .route("/service/categories/:code", delete(service_request::delete_category))

        // Service Requests
        .route("/service/requests", post(service_request::create_request))
        .route("/service/requests", get(service_request::list_requests))
        .route("/service/requests/:id", get(service_request::get_request))
        .route("/service/requests/number/:number", get(service_request::get_request_by_number))

        // Service Request Lifecycle
        .route("/service/requests/:id/status", post(service_request::update_request_status))
        .route("/service/requests/:id/resolve", post(service_request::resolve_request))

        // Service Request Assignments
        .route("/service/requests/:id/assign", post(service_request::assign_request))
        .route("/service/requests/:id/assignments", get(service_request::list_assignments))

        // Service Request Updates / Communications
        .route("/service/requests/:id/updates", post(service_request::add_update))
        .route("/service/requests/:id/updates", get(service_request::list_updates))

        // Service Request Dashboard
        .route("/service/dashboard", get(service_request::get_service_request_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Lead and Opportunity Management (Oracle Fusion CX Sales)
        // ═══════════════════════════════════════════════════════════

        // Lead Sources
        .route("/sales/lead-sources", post(lead_opportunity::create_lead_source))
        .route("/sales/lead-sources", get(lead_opportunity::list_lead_sources))
        .route("/sales/lead-sources/:code", delete(lead_opportunity::delete_lead_source))

        // Sales Leads
        .route("/sales/leads", post(lead_opportunity::create_lead))
        .route("/sales/leads", get(lead_opportunity::list_leads))
        .route("/sales/leads/:id", get(lead_opportunity::get_lead))
        .route("/sales/leads/:id/status", post(lead_opportunity::update_lead_status))
        .route("/sales/leads/:id/score", post(lead_opportunity::update_lead_score))
        .route("/sales/leads/:id/convert", post(lead_opportunity::convert_lead))
        .route("/sales/leads/:id", delete(lead_opportunity::delete_lead))

        // Opportunity Stages
        .route("/sales/opportunity-stages", post(lead_opportunity::create_opportunity_stage))
        .route("/sales/opportunity-stages", get(lead_opportunity::list_opportunity_stages))
        .route("/sales/opportunity-stages/:code", delete(lead_opportunity::delete_opportunity_stage))

        // Sales Opportunities
        .route("/sales/opportunities", post(lead_opportunity::create_opportunity))
        .route("/sales/opportunities", get(lead_opportunity::list_opportunities))
        .route("/sales/opportunities/:id", get(lead_opportunity::get_opportunity))
        .route("/sales/opportunities/:id/stage", post(lead_opportunity::update_opportunity_stage))
        .route("/sales/opportunities/:id/win", post(lead_opportunity::close_opportunity_won))
        .route("/sales/opportunities/:id/lose", post(lead_opportunity::close_opportunity_lost))
        .route("/sales/opportunities/:id/history", get(lead_opportunity::list_stage_history))
        .route("/sales/opportunities/:id", delete(lead_opportunity::delete_opportunity))

        // Opportunity Lines
        .route("/sales/opportunities/:opportunity_id/lines", post(lead_opportunity::add_opportunity_line))
        .route("/sales/opportunities/:opportunity_id/lines", get(lead_opportunity::list_opportunity_lines))
        .route("/sales/opportunity-lines/:id", delete(lead_opportunity::delete_opportunity_line))

        // Sales Activities
        .route("/sales/activities", post(lead_opportunity::create_activity))
        .route("/sales/activities", get(lead_opportunity::list_activities))
        .route("/sales/activities/:id/complete", post(lead_opportunity::complete_activity))
        .route("/sales/activities/:id/cancel", post(lead_opportunity::cancel_activity))
        .route("/sales/activities/:id", delete(lead_opportunity::delete_activity))

        // Sales Pipeline Dashboard
        .route("/sales/dashboard", get(lead_opportunity::get_sales_pipeline_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Marketing Campaign Management (Oracle Fusion CX Marketing)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Campaign Types
        .route("/marketing/campaign-types", post(marketing::create_campaign_type))
        .route("/marketing/campaign-types", get(marketing::list_campaign_types))
        .route("/marketing/campaign-types/:code", delete(marketing::delete_campaign_type))

        // Marketing Campaigns
        .route("/marketing/campaigns", post(marketing::create_campaign))
        .route("/marketing/campaigns", get(marketing::list_campaigns))
        .route("/marketing/campaigns/:id", get(marketing::get_campaign))
        .route("/marketing/campaigns/:id/activate", post(marketing::activate_campaign))
        .route("/marketing/campaigns/:id/pause", post(marketing::pause_campaign))
        .route("/marketing/campaigns/:id/complete", post(marketing::complete_campaign))
        .route("/marketing/campaigns/:id/cancel", post(marketing::cancel_campaign))
        .route("/marketing/campaigns/:id", delete(marketing::delete_campaign))

        // Campaign Members
        .route("/marketing/campaigns/:campaign_id/members", post(marketing::add_campaign_member))
        .route("/marketing/campaigns/:campaign_id/members", get(marketing::list_campaign_members))
        .route("/marketing/members/:id/status", post(marketing::update_member_status))
        .route("/marketing/members/:id", delete(marketing::delete_campaign_member))

        // Campaign Responses
        .route("/marketing/campaigns/:campaign_id/responses", post(marketing::create_campaign_response))
        .route("/marketing/campaigns/:campaign_id/responses", get(marketing::list_campaign_responses))
        .route("/marketing/responses/:id", delete(marketing::delete_campaign_response))

        // Marketing Dashboard
        .route("/marketing/dashboard", get(marketing::get_marketing_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Demand Planning (Oracle Fusion SCM > Demand Management)
        // ═══════════════════════════════════════════════════════════

        // Forecast Methods
        .route("/demand/methods", post(demand_planning::create_method))
        .route("/demand/methods", get(demand_planning::list_methods))
        .route("/demand/methods/:id", get(demand_planning::get_method))
        .route("/demand/methods-by-code/:code", delete(demand_planning::delete_method))

        // Demand Schedules
        .route("/demand/schedules", post(demand_planning::create_schedule))
        .route("/demand/schedules", get(demand_planning::list_schedules))
        .route("/demand/schedules/:id", get(demand_planning::get_schedule))
        .route("/demand/schedules/:id/submit", post(demand_planning::submit_schedule))
        .route("/demand/schedules/:id/approve", post(demand_planning::approve_schedule))
        .route("/demand/schedules/:id/activate", post(demand_planning::activate_schedule))
        .route("/demand/schedules/:id/close", post(demand_planning::close_schedule))
        .route("/demand/schedules/:id/cancel", post(demand_planning::cancel_schedule))
        .route("/demand/schedules-by-number/:schedule_number", delete(demand_planning::delete_schedule))

        // Schedule Lines
        .route("/demand/schedules/:schedule_id/lines", post(demand_planning::add_schedule_line))
        .route("/demand/schedules/:schedule_id/lines", get(demand_planning::list_schedule_lines))
        .route("/demand/schedule-lines/:id", delete(demand_planning::delete_schedule_line))

        // Demand History
        .route("/demand/history", post(demand_planning::create_history))
        .route("/demand/history", get(demand_planning::list_history))
        .route("/demand/history/:id", delete(demand_planning::delete_history))

        // Forecast Consumption
        .route("/demand/consumption", post(demand_planning::consume_forecast))
        .route("/demand/consumption/:schedule_line_id", get(demand_planning::list_consumption))
        .route("/demand/consumption-entries/:id", delete(demand_planning::delete_consumption))

        // Accuracy Measurement
        .route("/demand/accuracy", post(demand_planning::measure_accuracy))
        .route("/demand/schedules/:schedule_id/accuracy", get(demand_planning::list_accuracy))

        // Demand Planning Dashboard
        .route("/demand/dashboard", get(demand_planning::get_demand_planning_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // AutoInvoice (Oracle Fusion Receivables AutoInvoice)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Grouping Rules
        .route("/autoinvoice/grouping-rules", post(autoinvoice::create_grouping_rule))
        .route("/autoinvoice/grouping-rules", get(autoinvoice::list_grouping_rules))
        .route("/autoinvoice/grouping-rules/:id", get(autoinvoice::get_grouping_rule))
        .route("/autoinvoice/grouping-rules/:id", delete(autoinvoice::delete_grouping_rule))

        // Validation Rules
        .route("/autoinvoice/validation-rules", post(autoinvoice::create_validation_rule))
        .route("/autoinvoice/validation-rules", get(autoinvoice::list_validation_rules))
        .route("/autoinvoice/validation-rules/:id", delete(autoinvoice::delete_validation_rule))

        // Batch Import & Processing
        .route("/autoinvoice/batches", post(autoinvoice::import_batch))
        .route("/autoinvoice/batches", get(autoinvoice::list_batches))
        .route("/autoinvoice/batches/:id", get(autoinvoice::get_batch))
        .route("/autoinvoice/batches/:id/validate", post(autoinvoice::validate_batch))
        .route("/autoinvoice/batches/:id/process", post(autoinvoice::process_batch))
        .route("/autoinvoice/import-and-process", post(autoinvoice::import_and_process))

        // Batch Lines & Results
        .route("/autoinvoice/batches/:id/lines", get(autoinvoice::get_batch_lines))
        .route("/autoinvoice/batches/:id/results", get(autoinvoice::get_batch_results))

        // Invoice Management
        .route("/autoinvoice/invoices/:id", get(autoinvoice::get_invoice))
        .route("/autoinvoice/invoices/:id/status", put(autoinvoice::update_invoice_status))

        // AutoInvoice Dashboard
        .route("/autoinvoice/dashboard", get(autoinvoice::get_autoinvoice_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Shipping Execution (Oracle Fusion SCM > Shipping Execution)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Carriers
        .route("/shipping/carriers", post(shipping::create_carrier))
        .route("/shipping/carriers", get(shipping::list_carriers))
        .route("/shipping/carriers/:id", get(shipping::get_carrier))
        .route("/shipping/carriers-by-code/:code", delete(shipping::delete_carrier))

        // Shipping Methods
        .route("/shipping/methods", post(shipping::create_shipping_method))
        .route("/shipping/methods", get(shipping::list_shipping_methods))
        .route("/shipping/methods-by-code/:code", delete(shipping::delete_shipping_method))

        // Shipments
        .route("/shipping/shipments", post(shipping::create_shipment))
        .route("/shipping/shipments", get(shipping::list_shipments))
        .route("/shipping/shipments/:id", get(shipping::get_shipment))
        .route("/shipping/shipments/:id/confirm", post(shipping::confirm_shipment))
        .route("/shipping/shipments/:id/ship", post(shipping::ship_confirm))
        .route("/shipping/shipments/:id/deliver", post(shipping::deliver_shipment))
        .route("/shipping/shipments/:id/cancel", post(shipping::cancel_shipment))
        .route("/shipping/shipments-by-number/:shipment_number", delete(shipping::delete_shipment))

        // Shipment Lines
        .route("/shipping/shipments/:shipment_id/lines", post(shipping::add_shipment_line))
        .route("/shipping/shipments/:shipment_id/lines", get(shipping::list_shipment_lines))
        .route("/shipping/shipment-lines/:id", delete(shipping::delete_shipment_line))
        .route("/shipping/shipment-lines/:id/shipped-qty", put(shipping::update_shipped_quantity))

        // Packing Slips
        .route("/shipping/shipments/:shipment_id/packing-slips", post(shipping::create_packing_slip))
        .route("/shipping/shipments/:shipment_id/packing-slips", get(shipping::list_packing_slips))
        .route("/shipping/packing-slips/:id", delete(shipping::delete_packing_slip))

        // Packing Slip Lines
        .route("/shipping/packing-slips/:packing_slip_id/lines", post(shipping::add_packing_slip_line))
        .route("/shipping/packing-slips/:packing_slip_id/lines", get(shipping::list_packing_slip_lines))
        .route("/shipping/packing-slip-lines/:id", delete(shipping::delete_packing_slip_line))

        // Shipping Dashboard
        .route("/shipping/dashboard", get(shipping::get_shipping_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════════
        // Recruiting Management (Oracle Fusion HCM > Recruiting)
        // ═════════════════════════════════════════════════════════════════════════════════════

        // Job Requisitions
        .route("/recruiting/requisitions", post(recruiting::create_requisition))
        .route("/recruiting/requisitions", get(recruiting::list_requisitions))
        .route("/recruiting/requisitions/:id", get(recruiting::get_requisition))
        .route("/recruiting/requisitions/:id/open", post(recruiting::open_requisition))
        .route("/recruiting/requisitions/:id/hold", post(recruiting::hold_requisition))
        .route("/recruiting/requisitions/:id/close", post(recruiting::close_requisition))
        .route("/recruiting/requisitions/:id/cancel", post(recruiting::cancel_requisition))
        .route("/recruiting/requisitions-by-number/:number", delete(recruiting::delete_requisition))

        // Candidates
        .route("/recruiting/candidates", post(recruiting::create_candidate))
        .route("/recruiting/candidates", get(recruiting::list_candidates))
        .route("/recruiting/candidates/:id", get(recruiting::get_candidate))
        .route("/recruiting/candidates/:id/status", post(recruiting::update_candidate_status))
        .route("/recruiting/candidates/:id", delete(recruiting::delete_candidate))

        // Job Applications
        .route("/recruiting/applications", post(recruiting::create_application))
        .route("/recruiting/applications", get(recruiting::list_applications))
        .route("/recruiting/applications/:id", get(recruiting::get_application))
        .route("/recruiting/applications/:id/status", post(recruiting::update_application_status))
        .route("/recruiting/applications/:id/withdraw", post(recruiting::withdraw_application))

        // Interviews
        .route("/recruiting/applications/:application_id/interviews", post(recruiting::create_interview))
        .route("/recruiting/applications/:application_id/interviews", get(recruiting::list_interviews))
        .route("/recruiting/interviews/:id/complete", post(recruiting::complete_interview))
        .route("/recruiting/interviews/:id/cancel", post(recruiting::cancel_interview))
        .route("/recruiting/interviews/:id", delete(recruiting::delete_interview))

        // Job Offers
        .route("/recruiting/applications/:application_id/offers", post(recruiting::create_offer))
        .route("/recruiting/offers", get(recruiting::list_offers))
        .route("/recruiting/offers/:id", get(recruiting::get_offer))
        .route("/recruiting/offers/:id/approve", post(recruiting::approve_offer))
        .route("/recruiting/offers/:id/extend", post(recruiting::extend_offer))
        .route("/recruiting/offers/:id/accept", post(recruiting::accept_offer))
        .route("/recruiting/offers/:id/decline", post(recruiting::decline_offer))
        .route("/recruiting/offers/:id/withdraw", post(recruiting::withdraw_offer))
        .route("/recruiting/offers/:id", delete(recruiting::delete_offer))

        // Recruiting Dashboard
        .route("/recruiting/dashboard", get(recruiting::get_recruiting_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════════
        // Revenue Recognition (Oracle Fusion Financials > Revenue Management / ASC 606)
        // ═════════════════════════════════════════════════════════════════════════════════════

        // Revenue Policies
        .route("/revenue/policies", post(revenue::create_policy))
        .route("/revenue/policies", get(revenue::list_policies))
        .route("/revenue/policies/:code", get(revenue::get_policy))
        .route("/revenue/policies/:code", delete(revenue::delete_policy))

        // Revenue Contracts
        .route("/revenue/contracts", post(revenue::create_contract))
        .route("/revenue/contracts", get(revenue::list_contracts))
        .route("/revenue/contracts/:id", get(revenue::get_contract))
        .route("/revenue/contracts/:id/activate", post(revenue::activate_contract))
        .route("/revenue/contracts/:id/cancel", post(revenue::cancel_contract))

        // Performance Obligations
        .route("/revenue/contracts/:contract_id/obligations", post(revenue::create_obligation))
        .route("/revenue/contracts/:contract_id/obligations", get(revenue::list_obligations))
        .route("/revenue/obligations/:id", get(revenue::get_obligation))

        // Transaction Price Allocation (ASC 606 Step 4)
        .route("/revenue/contracts/:contract_id/allocate", post(revenue::allocate_transaction_price))

        // Revenue Scheduling (ASC 606 Step 5)
        .route("/revenue/obligations/:obligation_id/schedule/straight-line", post(revenue::generate_straight_line_schedule))
        .route("/revenue/obligations/:obligation_id/schedule/point-in-time", post(revenue::schedule_point_in_time))

        // Revenue Recognition Execution
        .route("/revenue/schedule-lines/:line_id/recognize", post(revenue::recognize_revenue))
        .route("/revenue/schedule-lines/:line_id/reverse", post(revenue::reverse_recognition))
        .route("/revenue/obligations/:obligation_id/schedule-lines", get(revenue::list_schedule_lines))
        .route("/revenue/contracts/:contract_id/schedule-lines", get(revenue::list_contract_schedule_lines))

        // Contract Modifications
        .route("/revenue/contracts/:contract_id/modifications", post(revenue::create_modification))
        .route("/revenue/contracts/:contract_id/modifications", get(revenue::list_modifications))

        // ═══════════════════════════════════════════════════════════════════════════════════════
        // Receiving Management (Oracle Fusion SCM > Receiving)
        // ═══════════════════════════════════════════════════════════════════════════════════════

        // Receiving Locations
        .route("/receiving/locations", post(receiving::create_location))
        .route("/receiving/locations", get(receiving::list_locations))
        .route("/receiving/locations/:code", delete(receiving::delete_location))

        // Receipts
        .route("/receiving/receipts", post(receiving::create_receipt))
        .route("/receiving/receipts", get(receiving::list_receipts))
        .route("/receiving/receipts/:id", get(receiving::get_receipt))
        .route("/receiving/receipts/:id/confirm", post(receiving::confirm_receipt))
        .route("/receiving/receipts/:id/close", post(receiving::close_receipt))
        .route("/receiving/receipts/:id/cancel", post(receiving::cancel_receipt))

        // Receipt Lines
        .route("/receiving/receipts/:receipt_id/lines", post(receiving::add_receipt_line))
        .route("/receiving/receipts/:receipt_id/lines", get(receiving::list_receipt_lines))

        // Inspections
        .route("/receiving/receipts/:receipt_id/inspections", post(receiving::create_inspection))
        .route("/receiving/receipts/:receipt_id/inspections", get(receiving::list_inspections))
        .route("/receiving/inspections/:id/complete", post(receiving::complete_inspection))

        // Inspection Details
        .route("/receiving/inspections/:inspection_id/details", post(receiving::add_inspection_detail))
        .route("/receiving/inspections/:inspection_id/details", get(receiving::list_inspection_details))

        // Deliveries
        .route("/receiving/receipts/:receipt_id/deliveries", post(receiving::create_delivery))
        .route("/receiving/receipts/:receipt_id/deliveries", get(receiving::list_deliveries))

        // Returns to Supplier
        .route("/receiving/returns", post(receiving::create_return))
        .route("/receiving/returns", get(receiving::list_returns))
        .route("/receiving/returns/:id/submit", post(receiving::submit_return))
        .route("/receiving/returns/:id/ship", post(receiving::ship_return))
        .route("/receiving/returns/:id/credit", post(receiving::credit_return))
        .route("/receiving/returns/:id/cancel", post(receiving::cancel_return))

        // Receiving Dashboard
        .route("/receiving/dashboard", get(receiving::get_receiving_dashboard))

        // ═══════════════════════════════════════════════════════════════════════════════════════
        // Supplier Scorecard Management (Oracle Fusion Supplier Portal > Performance)
        // ═══════════════════════════════════════════════════════════════════════════════════════

        // Templates
        .route("/supplier-scorecard/templates", post(supplier_scorecard::create_template))
        .route("/supplier-scorecard/templates", get(supplier_scorecard::list_templates))
        .route("/supplier-scorecard/templates/:id", get(supplier_scorecard::get_template))
        .route("/supplier-scorecard/templates-by-code/:code", delete(supplier_scorecard::delete_template))

        // Categories
        .route("/supplier-scorecard/categories", post(supplier_scorecard::create_category))
        .route("/supplier-scorecard/templates/:template_id/categories", get(supplier_scorecard::list_categories))
        .route("/supplier-scorecard/categories/:id", delete(supplier_scorecard::delete_category))

        // Scorecards
        .route("/supplier-scorecard/scorecards", post(supplier_scorecard::create_scorecard))
        .route("/supplier-scorecard/scorecards", get(supplier_scorecard::list_scorecards))
        .route("/supplier-scorecard/scorecards/:id", get(supplier_scorecard::get_scorecard))
        .route("/supplier-scorecard/scorecards/:id/submit", post(supplier_scorecard::submit_scorecard))
        .route("/supplier-scorecard/scorecards/:id/approve", post(supplier_scorecard::approve_scorecard))
        .route("/supplier-scorecard/scorecards/:id/reject", post(supplier_scorecard::reject_scorecard))
        .route("/supplier-scorecard/scorecards-by-number/:scorecard_number", delete(supplier_scorecard::delete_scorecard))

        // Scorecard Lines
        .route("/supplier-scorecard/scorecards/:scorecard_id/lines", post(supplier_scorecard::add_scorecard_line))
        .route("/supplier-scorecard/scorecards/:scorecard_id/lines", get(supplier_scorecard::list_scorecard_lines))
        .route("/supplier-scorecard/lines/:id", delete(supplier_scorecard::delete_scorecard_line))

        // Performance Reviews
        .route("/supplier-scorecard/reviews", post(supplier_scorecard::create_review))
        .route("/supplier-scorecard/reviews", get(supplier_scorecard::list_reviews))
        .route("/supplier-scorecard/reviews/:id", get(supplier_scorecard::get_review))
        .route("/supplier-scorecard/reviews/:id/complete", post(supplier_scorecard::complete_review))
        .route("/supplier-scorecard/reviews-by-number/:review_number", delete(supplier_scorecard::delete_review))

        // Review Action Items
        .route("/supplier-scorecard/reviews/:review_id/action-items", post(supplier_scorecard::create_action_item))
        .route("/supplier-scorecard/reviews/:review_id/action-items", get(supplier_scorecard::list_action_items))
        .route("/supplier-scorecard/action-items/:id/complete", post(supplier_scorecard::complete_action_item))
        .route("/supplier-scorecard/action-items/:id", delete(supplier_scorecard::delete_action_item))

        // Supplier Scorecard Dashboard
        .route("/supplier-scorecard/dashboard", get(supplier_scorecard::get_scorecard_dashboard))

        // ═══════════════════════════════════════════════════════
        // KPI & Embedded Analytics (Oracle Fusion OTBI-inspired)
        // ═══════════════════════════════════════════════════════

        // KPI Definitions
        .route("/kpi/definitions", post(kpi::create_kpi))
        .route("/kpi/definitions", get(kpi::list_kpis))
        .route("/kpi/definitions/id/:id", get(kpi::get_kpi))
        .route("/kpi/definitions/code/:code", delete(kpi::delete_kpi))

        // KPI Data Points
        .route("/kpi/definitions/:kpi_id/data-points", post(kpi::record_data_point))
        .route("/kpi/definitions/:kpi_id/data-points/latest", get(kpi::get_latest_data_point))
        .route("/kpi/definitions/:kpi_id/data-points", get(kpi::list_data_points))
        .route("/kpi/data-points/:id", delete(kpi::delete_data_point))

        // Dashboards
        .route("/kpi/dashboards", post(kpi::create_dashboard))
        .route("/kpi/dashboards", get(kpi::list_dashboards))
        .route("/kpi/dashboards/id/:id", get(kpi::get_dashboard))
        .route("/kpi/dashboards/code/:code", delete(kpi::delete_dashboard))

        // Dashboard Widgets
        .route("/kpi/dashboards/:dashboard_id/widgets", post(kpi::add_widget))
        .route("/kpi/dashboards/:dashboard_id/widgets", get(kpi::list_widgets))
        .route("/kpi/widgets/:id", delete(kpi::delete_widget))

        // KPI Analytics Dashboard
        .route("/kpi/dashboard", get(kpi::get_kpi_dashboard))

        // ========================================================================
        // Account Monitor & Balance Inquiry (Oracle Fusion GL Account Monitor)
        // ========================================================================

        // Account Groups
        .route("/account-monitor/groups", post(account_monitor::create_account_group))
        .route("/account-monitor/groups", get(account_monitor::list_account_groups))
        .route("/account-monitor/groups/id/:id", get(account_monitor::get_account_group))
        .route("/account-monitor/groups/code/:code", delete(account_monitor::delete_account_group))

        // Group Members
        .route("/account-monitor/groups/:group_id/members", post(account_monitor::add_group_member))
        .route("/account-monitor/groups/:group_id/members", get(account_monitor::list_group_members))
        .route("/account-monitor/members/:id", delete(account_monitor::remove_group_member))

        // Balance Snapshots
        .route("/account-monitor/groups/:group_id/snapshots", post(account_monitor::capture_snapshot))
        .route("/account-monitor/groups/:group_id/snapshots", get(account_monitor::list_snapshots))
        .route("/account-monitor/snapshots/alerts", get(account_monitor::get_alert_snapshots))
        .route("/account-monitor/snapshots/:id", delete(account_monitor::delete_snapshot))

        // Saved Balance Inquiries
        .route("/account-monitor/inquiries", post(account_monitor::create_saved_inquiry))
        .route("/account-monitor/inquiries", get(account_monitor::list_saved_inquiries))
        .route("/account-monitor/inquiries/id/:id", get(account_monitor::get_saved_inquiry))
        .route("/account-monitor/inquiries/id/:id", delete(account_monitor::delete_saved_inquiry))

        // Account Monitor Dashboard
        .route("/account-monitor/dashboard", get(account_monitor::get_account_monitor_summary))

        // Goal Management - Library Categories
        .route("/goal-management/library/categories", post(goal_management::create_library_category))
        .route("/goal-management/library/categories", get(goal_management::list_library_categories))
        .route("/goal-management/library/categories/:code", delete(goal_management::delete_library_category))

        // Goal Management - Library Templates
        .route("/goal-management/library/templates", post(goal_management::create_library_template))
        .route("/goal-management/library/templates", get(goal_management::list_library_templates))
        .route("/goal-management/library/templates/:code", delete(goal_management::delete_library_template))

        // Goal Management - Plans
        .route("/goal-management/plans", post(goal_management::create_goal_plan))
        .route("/goal-management/plans", get(goal_management::list_goal_plans))
        .route("/goal-management/plans/id/:id", get(goal_management::get_goal_plan))
        .route("/goal-management/plans/id/:id/status", post(goal_management::update_goal_plan_status))
        .route("/goal-management/plans/code/:code", delete(goal_management::delete_goal_plan))

        // Goal Management - Goals
        .route("/goal-management/goals", post(goal_management::create_goal))
        .route("/goal-management/goals", get(goal_management::list_goals))
        .route("/goal-management/goals/id/:id", get(goal_management::get_goal))
        .route("/goal-management/goals/id/:id/progress", post(goal_management::update_goal_progress))
        .route("/goal-management/goals/id/:id", delete(goal_management::delete_goal))

        // Goal Management - Alignments
        .route("/goal-management/alignments", post(goal_management::create_goal_alignment))
        .route("/goal-management/alignments/goal/:goal_id", get(goal_management::list_goal_alignments))
        .route("/goal-management/alignments/id/:id", delete(goal_management::delete_goal_alignment))

        // Goal Management - Notes
        .route("/goal-management/goals/:goal_id/notes", post(goal_management::create_goal_note))
        .route("/goal-management/goals/:goal_id/notes", get(goal_management::list_goal_notes))
        .route("/goal-management/notes/id/:id", delete(goal_management::delete_goal_note))

        // Goal Management - Dashboard
        .route("/goal-management/dashboard", get(goal_management::get_goal_management_summary))

        // ═══════════════════════════════════════════════════════════
        // Contract Lifecycle Management (Oracle Fusion Enterprise Contracts)
        // ═══════════════════════════════════════════════════════════

        // Contract Types
        .route("/clm/contract-types", post(contract_lifecycle::create_contract_type))
        .route("/clm/contract-types", get(contract_lifecycle::list_contract_types))
        .route("/clm/contract-types/id/:id", get(contract_lifecycle::get_contract_type))
        .route("/clm/contract-types/code/:code", delete(contract_lifecycle::delete_contract_type))

        // Clause Library
        .route("/clm/clauses", post(contract_lifecycle::create_clause))
        .route("/clm/clauses", get(contract_lifecycle::list_clauses))
        .route("/clm/clauses/code/:code", delete(contract_lifecycle::delete_clause))

        // Contract Templates
        .route("/clm/templates", post(contract_lifecycle::create_template))
        .route("/clm/templates", get(contract_lifecycle::list_templates))
        .route("/clm/templates/code/:code", delete(contract_lifecycle::delete_template))

        // Contracts
        .route("/clm/contracts", post(contract_lifecycle::create_contract))
        .route("/clm/contracts", get(contract_lifecycle::list_contracts))
        .route("/clm/contracts/id/:id", get(contract_lifecycle::get_contract))
        .route("/clm/contracts/id/:id/status", post(contract_lifecycle::transition_contract))
        .route("/clm/contracts/number/:number", delete(contract_lifecycle::delete_contract))

        // Contract Parties
        .route("/clm/contracts/:contract_id/parties", post(contract_lifecycle::add_contract_party))
        .route("/clm/contracts/:contract_id/parties", get(contract_lifecycle::list_contract_parties))
        .route("/clm/parties/:id", delete(contract_lifecycle::remove_contract_party))

        // Contract Milestones
        .route("/clm/contracts/:contract_id/milestones", post(contract_lifecycle::create_milestone))
        .route("/clm/contracts/:contract_id/milestones", get(contract_lifecycle::list_milestones))
        .route("/clm/milestones/:id/complete", post(contract_lifecycle::complete_milestone))
        .route("/clm/milestones/:id", delete(contract_lifecycle::delete_milestone))

        // Contract Deliverables
        .route("/clm/contracts/:contract_id/deliverables", post(contract_lifecycle::create_deliverable))
        .route("/clm/contracts/:contract_id/deliverables", get(contract_lifecycle::list_deliverables))
        .route("/clm/deliverables/:id/accept", post(contract_lifecycle::accept_deliverable))
        .route("/clm/deliverables/:id/reject", post(contract_lifecycle::reject_deliverable))
        .route("/clm/deliverables/:id", delete(contract_lifecycle::delete_deliverable))

        // Contract Amendments
        .route("/clm/contracts/:contract_id/amendments", post(contract_lifecycle::create_amendment))
        .route("/clm/contracts/:contract_id/amendments", get(contract_lifecycle::list_amendments))
        .route("/clm/amendments/:id/approve", post(contract_lifecycle::approve_amendment))
        .route("/clm/amendments/:id/reject", post(contract_lifecycle::reject_amendment))

        // Contract Risk Assessments
        .route("/clm/contracts/:contract_id/risks", post(contract_lifecycle::create_risk))
        .route("/clm/contracts/:contract_id/risks", get(contract_lifecycle::list_risks))
        .route("/clm/risks/:id", delete(contract_lifecycle::delete_risk))

        // CLM Dashboard
        .route("/clm/dashboard", get(contract_lifecycle::get_clm_dashboard))

        // ═══════════════════════════════════════════════════════════
        // Risk Management & Internal Controls (Oracle Fusion GRC)
        // ═══════════════════════════════════════════════════════════

        // Risk Categories
        .route("/risk/categories", post(risk_management::create_category))
        .route("/risk/categories", get(risk_management::list_categories))
        .route("/risk/categories/id/:id", get(risk_management::get_category))
        .route("/risk/categories/code/:code", delete(risk_management::delete_category))

        // Risk Register
        .route("/risk/risks", post(risk_management::create_risk))
        .route("/risk/risks", get(risk_management::list_risks))
        .route("/risk/risks/id/:id", get(risk_management::get_risk))
        .route("/risk/risks/id/:id/status", post(risk_management::update_risk_status))
        .route("/risk/risks/id/:id/assess", post(risk_management::assess_risk))
        .route("/risk/risks/number/:risk_number", delete(risk_management::delete_risk))

        // Control Registry
        .route("/risk/controls", post(risk_management::create_control))
        .route("/risk/controls", get(risk_management::list_controls))
        .route("/risk/controls/id/:id", get(risk_management::get_control))
        .route("/risk/controls/id/:id/status", post(risk_management::update_control_status))
        .route("/risk/controls/id/:id/effectiveness", post(risk_management::update_control_effectiveness))
        .route("/risk/controls/number/:control_number", delete(risk_management::delete_control))

        // Risk-Control Mappings
        .route("/risk/mappings", post(risk_management::create_mapping))
        .route("/risk/risks/id/:risk_id/mappings", get(risk_management::list_risk_mappings))
        .route("/risk/controls/id/:control_id/mappings", get(risk_management::list_control_mappings))
        .route("/risk/mappings/:id", delete(risk_management::delete_mapping))

        // Control Tests
        .route("/risk/tests", post(risk_management::create_control_test))
        .route("/risk/tests/id/:id", get(risk_management::get_control_test))
        .route("/risk/controls/id/:control_id/tests", get(risk_management::list_control_tests))
        .route("/risk/tests/id/:id/start", post(risk_management::start_control_test))
        .route("/risk/tests/id/:id/complete", post(risk_management::complete_control_test))
        .route("/risk/tests/number/:test_number", delete(risk_management::delete_control_test))

        // Issues & Remediations
        .route("/risk/issues", post(risk_management::create_issue))
        .route("/risk/issues", get(risk_management::list_issues))
        .route("/risk/issues/id/:id", get(risk_management::get_issue))
        .route("/risk/issues/id/:id/status", post(risk_management::update_issue_status))
        .route("/risk/issues/id/:id/resolve", post(risk_management::resolve_issue))
        .route("/risk/issues/number/:issue_number", delete(risk_management::delete_issue))

        // Risk Dashboard
        .route("/risk/dashboard", get(risk_management::get_risk_dashboard))

        // ========================================================================
        // Enterprise Asset Management (eAM) Routes
        // ========================================================================
        // Asset Locations
        .route("/eam/locations", post(enterprise_asset_management::create_location))
        .route("/eam/locations", get(enterprise_asset_management::list_locations))
        .route("/eam/locations/id/:id", get(enterprise_asset_management::get_location))
        .route("/eam/locations/code/:code", delete(enterprise_asset_management::delete_location))

        // Asset Definitions
        .route("/eam/assets", post(enterprise_asset_management::create_asset))
        .route("/eam/assets", get(enterprise_asset_management::list_assets))
        .route("/eam/assets/id/:id", get(enterprise_asset_management::get_asset))
        .route("/eam/assets/id/:id/status", post(enterprise_asset_management::update_asset_status))
        .route("/eam/assets/id/:id/meter", post(enterprise_asset_management::update_asset_meter))
        .route("/eam/assets/number/:asset_number", delete(enterprise_asset_management::delete_asset))

        // Work Orders
        .route("/eam/work-orders", post(enterprise_asset_management::create_work_order))
        .route("/eam/work-orders", get(enterprise_asset_management::list_work_orders))
        .route("/eam/work-orders/id/:id", get(enterprise_asset_management::get_work_order))
        .route("/eam/work-orders/id/:id/status", post(enterprise_asset_management::update_work_order_status))
        .route("/eam/work-orders/id/:id/complete", post(enterprise_asset_management::complete_work_order))
        .route("/eam/work-orders/number/:wo_number", delete(enterprise_asset_management::delete_work_order))

        // Preventive Maintenance Schedules
        .route("/eam/pm-schedules", post(enterprise_asset_management::create_pm_schedule))
        .route("/eam/pm-schedules", get(enterprise_asset_management::list_pm_schedules))
        .route("/eam/pm-schedules/id/:id", get(enterprise_asset_management::get_pm_schedule))
        .route("/eam/pm-schedules/id/:id/status", post(enterprise_asset_management::update_pm_schedule_status))
        .route("/eam/pm-schedules/number/:schedule_number", delete(enterprise_asset_management::delete_pm_schedule))

        // Maintenance Dashboard
        .route("/eam/dashboard", get(enterprise_asset_management::get_maintenance_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Product Configurator (Oracle Fusion Cloud SCM > Product Management > Configurator)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Configuration Models
        .route("/configurator/models", post(product_configurator::create_model))
        .route("/configurator/models", get(product_configurator::list_models))
        .route("/configurator/models/:id", get(product_configurator::get_model))
        .route("/configurator/models/:id/activate", post(product_configurator::activate_model))
        .route("/configurator/models/:id/deactivate", post(product_configurator::deactivate_model))
        .route("/configurator/models/number/:model_number", delete(product_configurator::delete_model))

        // Configuration Features
        .route("/configurator/models/:model_id/features", post(product_configurator::create_feature))
        .route("/configurator/models/:model_id/features", get(product_configurator::list_features))
        .route("/configurator/features/:id", delete(product_configurator::delete_feature))

        // Configuration Options
        .route("/configurator/features/:feature_id/options", post(product_configurator::create_option))
        .route("/configurator/features/:feature_id/options", get(product_configurator::list_options))
        .route("/configurator/options/:id", delete(product_configurator::delete_option))

        // Configuration Rules
        .route("/configurator/models/:model_id/rules", post(product_configurator::create_rule))
        .route("/configurator/models/:model_id/rules", get(product_configurator::list_rules))
        .route("/configurator/rules/:id", delete(product_configurator::delete_rule))

        // Configuration Instances
        .route("/configurator/instances", post(product_configurator::create_instance))
        .route("/configurator/instances", get(product_configurator::list_instances))
        .route("/configurator/instances/:id", get(product_configurator::get_instance))
        .route("/configurator/instances/:id/submit", post(product_configurator::submit_instance))
        .route("/configurator/instances/:id/approve", post(product_configurator::approve_instance))
        .route("/configurator/instances/:id/cancel", post(product_configurator::cancel_instance))
        .route("/configurator/instances/number/:instance_number", delete(product_configurator::delete_instance))

        // Configurator Dashboard
        .route("/configurator/dashboard", get(product_configurator::get_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Transportation Management (Oracle Fusion SCM > Transportation Management)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Carriers
        .route("/transport/carriers", post(transportation_management::create_carrier))
        .route("/transport/carriers", get(transportation_management::list_carriers))
        .route("/transport/carriers/id/:id", get(transportation_management::get_carrier))
        .route("/transport/carriers/id/:id/suspend", post(transportation_management::suspend_carrier))
        .route("/transport/carriers/id/:id/reactivate", post(transportation_management::reactivate_carrier))
        .route("/transport/carriers/id/:id/blacklist", post(transportation_management::blacklist_carrier))
        .route("/transport/carriers/id/:id/performance", post(transportation_management::update_carrier_performance))
        .route("/transport/carriers/code/:code", delete(transportation_management::delete_carrier))

        // Carrier Services
        .route("/transport/carriers/:carrier_id/services", post(transportation_management::create_carrier_service))
        .route("/transport/carriers/:carrier_id/services", get(transportation_management::list_carrier_services))
        .route("/transport/services/:id/toggle", post(transportation_management::toggle_carrier_service))
        .route("/transport/services/:id", delete(transportation_management::delete_carrier_service))

        // Transport Lanes
        .route("/transport/lanes", post(transportation_management::create_lane))
        .route("/transport/lanes", get(transportation_management::list_lanes))
        .route("/transport/lanes/id/:id", get(transportation_management::get_lane))
        .route("/transport/lanes/id/:id/deactivate", post(transportation_management::deactivate_lane))
        .route("/transport/lanes/code/:code", delete(transportation_management::delete_lane))

        // Shipments
        .route("/transport/shipments", post(transportation_management::create_shipment))
        .route("/transport/shipments", get(transportation_management::list_shipments))
        .route("/transport/shipments/id/:id", get(transportation_management::get_shipment))
        .route("/transport/shipments/id/:id/book", post(transportation_management::book_shipment))
        .route("/transport/shipments/id/:id/pickup", post(transportation_management::confirm_pickup))
        .route("/transport/shipments/id/:id/transit", post(transportation_management::start_transit))
        .route("/transport/shipments/id/:id/arrive", post(transportation_management::arrive_at_destination))
        .route("/transport/shipments/id/:id/deliver", post(transportation_management::confirm_delivery))
        .route("/transport/shipments/id/:id/cancel", post(transportation_management::cancel_shipment))
        .route("/transport/shipments/id/:id/exception", post(transportation_management::mark_exception))
        .route("/transport/shipments/id/:id/assign-carrier", post(transportation_management::assign_carrier))
        .route("/transport/shipments/id/:id/tracking", post(transportation_management::update_tracking))
        .route("/transport/shipments/number/:number", delete(transportation_management::delete_shipment))

        // Shipment Stops
        .route("/transport/shipments/:shipment_id/stops", post(transportation_management::add_stop))
        .route("/transport/shipments/:shipment_id/stops", get(transportation_management::list_stops))
        .route("/transport/stops/:id/status", post(transportation_management::update_stop_status))

        // Shipment Lines
        .route("/transport/shipments/:shipment_id/lines", post(transportation_management::add_shipment_line))
        .route("/transport/shipments/:shipment_id/lines", get(transportation_management::list_shipment_lines))
        .route("/transport/shipments/id/:id/recalculate", post(transportation_management::recalculate_shipment_totals))

        // Tracking Events
        .route("/transport/shipments/:shipment_id/tracking-events", post(transportation_management::add_tracking_event))
        .route("/transport/shipments/:shipment_id/tracking-events", get(transportation_management::list_tracking_events))

        // Freight Rates
        .route("/transport/freight-rates", post(transportation_management::create_freight_rate))
        .route("/transport/freight-rates", get(transportation_management::list_freight_rates))
        .route("/transport/freight-rates/id/:id/expire", post(transportation_management::expire_freight_rate))
        .route("/transport/freight-rates/code/:code", delete(transportation_management::delete_freight_rate))

        // Transportation Dashboard
        .route("/transport/dashboard", get(transportation_management::get_transportation_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Territory Management (Oracle Fusion CX Sales > Territory Management)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Territory CRUD
        .route("/territories", post(territory_management::create_territory))
        .route("/territories", get(territory_management::list_territories))
        .route("/territories/:id", get(territory_management::get_territory))
        .route("/territories/:id", put(territory_management::update_territory))
        .route("/territories/:id", delete(territory_management::delete_territory))
        .route("/territories/:id/activate", post(territory_management::activate_territory))
        .route("/territories/:id/deactivate", post(territory_management::deactivate_territory))

        // Territory Members
        .route("/territories/:territory_id/members", post(territory_management::add_member))
        .route("/territories/:territory_id/members", get(territory_management::list_members))
        .route("/territories/members/:member_id", delete(territory_management::remove_member))

        // Territory Routing Rules
        .route("/territories/:territory_id/rules", post(territory_management::add_rule))
        .route("/territories/:territory_id/rules", get(territory_management::list_rules))
        .route("/territories/rules/:rule_id", delete(territory_management::remove_rule))

        // Entity Routing
        .route("/territories/route", post(territory_management::route_entity))

        // Territory Quotas
        .route("/territories/:territory_id/quotas", post(territory_management::set_quota))
        .route("/territories/:territory_id/quotas", get(territory_management::list_quotas))
        .route("/territories/quotas/:quota_id/attainment", put(territory_management::update_attainment))
        .route("/territories/quotas/:quota_id", delete(territory_management::delete_quota))

        // Territory Dashboard
        .route("/territories/dashboard", get(territory_management::get_territory_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Promotions Management (Oracle Fusion Trade Management > Trade Promotion)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Promotion CRUD
        .route("/promotions", post(promotions_management::create_promotion))
        .route("/promotions", get(promotions_management::list_promotions))
        .route("/promotions/:id", get(promotions_management::get_promotion))
        .route("/promotions/:id", put(promotions_management::update_promotion))
        .route("/promotions/:id", delete(promotions_management::delete_promotion))

        // Promotion Lifecycle
        .route("/promotions/:id/activate", post(promotions_management::activate_promotion))
        .route("/promotions/:id/hold", post(promotions_management::hold_promotion))
        .route("/promotions/:id/complete", post(promotions_management::complete_promotion))
        .route("/promotions/:id/cancel", post(promotions_management::cancel_promotion))

        // Promotional Offers
        .route("/promotions/:promotion_id/offers", post(promotions_management::create_offer))
        .route("/promotions/:promotion_id/offers", get(promotions_management::list_offers))
        .route("/promotions/offers/:offer_id", delete(promotions_management::delete_offer))

        // Fund Allocation
        .route("/promotions/:promotion_id/funds", post(promotions_management::create_fund))
        .route("/promotions/:promotion_id/funds", get(promotions_management::list_funds))
        .route("/promotions/funds/:fund_id/committed", put(promotions_management::update_fund_committed))
        .route("/promotions/funds/:fund_id/spent", put(promotions_management::update_fund_spent))
        .route("/promotions/funds/:fund_id", delete(promotions_management::delete_fund))

        // Claims Processing
        .route("/promotions/:promotion_id/claims", post(promotions_management::create_claim))
        .route("/promotions/:promotion_id/claims", get(promotions_management::list_claims))
        .route("/promotions/claims/:claim_id", get(promotions_management::get_claim))
        .route("/promotions/claims/:claim_id/review", post(promotions_management::review_claim))
        .route("/promotions/claims/:claim_id/approve", post(promotions_management::approve_claim))
        .route("/promotions/claims/:claim_id/reject", post(promotions_management::reject_claim))
        .route("/promotions/claims/:claim_id/settle", post(promotions_management::settle_claim))
        .route("/promotions/claims/:claim_id", delete(promotions_management::delete_claim))

        // Promotions Dashboard
        .route("/promotions/dashboard", get(promotions_management::get_promotions_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Sustainability & ESG Management (Oracle Fusion Sustainability)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Facilities
        .route("/sustainability/facilities", post(sustainability::create_facility))
        .route("/sustainability/facilities", get(sustainability::list_facilities))
        .route("/sustainability/facilities/id/:id", get(sustainability::get_facility))
        .route("/sustainability/facilities/id/:id/status", post(sustainability::update_facility_status))
        .route("/sustainability/facilities/code/:facility_code", delete(sustainability::delete_facility))

        // Emission Factors
        .route("/sustainability/emission-factors", post(sustainability::create_emission_factor))
        .route("/sustainability/emission-factors", get(sustainability::list_emission_factors))
        .route("/sustainability/emission-factors/id/:id", get(sustainability::get_emission_factor))
        .route("/sustainability/emission-factors/code/:factor_code", delete(sustainability::delete_emission_factor))

        // Environmental Activities
        .route("/sustainability/activities", post(sustainability::create_activity))
        .route("/sustainability/activities", get(sustainability::list_activities))
        .route("/sustainability/activities/id/:id", get(sustainability::get_activity))
        .route("/sustainability/activities/id/:id/status", post(sustainability::update_activity_status))
        .route("/sustainability/activities/number/:activity_number", delete(sustainability::delete_activity))

        // ESG Metrics
        .route("/sustainability/metrics", post(sustainability::create_metric))
        .route("/sustainability/metrics", get(sustainability::list_metrics))
        .route("/sustainability/metrics/id/:id", get(sustainability::get_metric))
        .route("/sustainability/metrics/code/:metric_code", delete(sustainability::delete_metric))

        // ESG Metric Readings
        .route("/sustainability/metric-readings", post(sustainability::create_metric_reading))
        .route("/sustainability/metrics/:metric_id/readings", get(sustainability::list_metric_readings))
        .route("/sustainability/metric-readings/id/:id", delete(sustainability::delete_metric_reading))

        // Sustainability Goals
        .route("/sustainability/goals", post(sustainability::create_goal))
        .route("/sustainability/goals", get(sustainability::list_goals))
        .route("/sustainability/goals/id/:id", get(sustainability::get_goal))
        .route("/sustainability/goals/id/:id/progress", post(sustainability::update_goal_progress))
        .route("/sustainability/goals/id/:id/status", post(sustainability::update_goal_status))
        .route("/sustainability/goals/code/:goal_code", delete(sustainability::delete_goal))

        // Carbon Offsets
        .route("/sustainability/carbon-offsets", post(sustainability::create_carbon_offset))
        .route("/sustainability/carbon-offsets", get(sustainability::list_carbon_offsets))
        .route("/sustainability/carbon-offsets/id/:id", get(sustainability::get_carbon_offset))
        .route("/sustainability/carbon-offsets/id/:id/retire", post(sustainability::retire_carbon_offset))
        .route("/sustainability/carbon-offsets/number/:offset_number", delete(sustainability::delete_carbon_offset))

        // Sustainability Dashboard
        .route("/sustainability/dashboard", get(sustainability::get_sustainability_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Project Billing (Oracle Fusion Project Management > Project Billing)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Bill Rate Schedules
        .route("/project-billing/schedules", get(project_billing::list_schedules))
        .route("/project-billing/schedules", post(project_billing::create_schedule))
        .route("/project-billing/schedules/:id", get(project_billing::get_schedule))
        .route("/project-billing/schedules/:id/activate", post(project_billing::activate_schedule))
        .route("/project-billing/schedules/:id/deactivate", post(project_billing::deactivate_schedule))
        .route("/project-billing/schedules/number/:schedule_number", delete(project_billing::delete_schedule))

        // Bill Rate Lines
        .route("/project-billing/schedules/:schedule_id/rate-lines", get(project_billing::list_rate_lines))
        .route("/project-billing/schedules/:schedule_id/rate-lines", post(project_billing::add_rate_line))
        .route("/project-billing/schedules/:schedule_id/rate-lines/:id", delete(project_billing::delete_rate_line))
        .route("/project-billing/schedules/:schedule_id/find-rate/:role_name", get(project_billing::find_rate_for_role))

        // Project Billing Configs
        .route("/project-billing/configs", get(project_billing::list_billing_configs))
        .route("/project-billing/configs", post(project_billing::create_billing_config))
        .route("/project-billing/configs/:id", get(project_billing::get_billing_config))
        .route("/project-billing/configs/project/:project_id", get(project_billing::get_billing_config_by_project))
        .route("/project-billing/configs/:id/activate", post(project_billing::activate_billing_config))
        .route("/project-billing/configs/:id/cancel", post(project_billing::cancel_billing_config))

        // Billing Events
        .route("/project-billing/events", get(project_billing::list_billing_events))
        .route("/project-billing/events", post(project_billing::create_billing_event))
        .route("/project-billing/events/:id", get(project_billing::get_billing_event))
        .route("/project-billing/events/:id/complete", post(project_billing::complete_billing_event))
        .route("/project-billing/events/:id/cancel", post(project_billing::cancel_billing_event))
        .route("/project-billing/events/number/:event_number", delete(project_billing::delete_billing_event))

        // Project Invoices
        .route("/project-billing/invoices", get(project_billing::list_invoices))
        .route("/project-billing/invoices", post(project_billing::create_invoice))
        .route("/project-billing/invoices/:id", get(project_billing::get_invoice))
        .route("/project-billing/invoices/:invoice_id/lines", get(project_billing::get_invoice_lines))
        .route("/project-billing/invoices/:id/submit", post(project_billing::submit_invoice))
        .route("/project-billing/invoices/:id/approve", post(project_billing::approve_invoice))
        .route("/project-billing/invoices/:id/reject", post(project_billing::reject_invoice))
        .route("/project-billing/invoices/:id/post", post(project_billing::post_invoice))
        .route("/project-billing/invoices/:id/cancel", post(project_billing::cancel_invoice))

        // Project Billing Dashboard
        .route("/project-billing/dashboard", get(project_billing::get_project_billing_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Quality Management (Oracle Fusion Quality Management)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Inspection Plans
        .route("/quality/plans", get(quality_management::list_plans))
        .route("/quality/plans", post(quality_management::create_plan))
        .route("/quality/plans/:code", get(quality_management::get_plan))
        .route("/quality/plans/:code", delete(quality_management::delete_plan))

        // Plan Criteria
        .route("/quality/plans/:plan_id/criteria", post(quality_management::create_criterion))
        .route("/quality/plans/:plan_id/criteria", get(quality_management::list_criteria))
        .route("/quality/criteria/:id", delete(quality_management::delete_criterion))

        // Inspections
        .route("/quality/inspections", get(quality_management::list_inspections))
        .route("/quality/inspections", post(quality_management::create_inspection))
        .route("/quality/inspections/:id", get(quality_management::get_inspection))
        .route("/quality/inspections/:id/start", post(quality_management::start_inspection))
        .route("/quality/inspections/:id/complete", post(quality_management::complete_inspection))
        .route("/quality/inspections/:id/cancel", post(quality_management::cancel_inspection))

        // Inspection Results
        .route("/quality/inspections/:inspection_id/results", post(quality_management::create_result))
        .route("/quality/inspections/:inspection_id/results", get(quality_management::list_results))

        // Non-Conformance Reports
        .route("/quality/ncrs", get(quality_management::list_ncrs))
        .route("/quality/ncrs", post(quality_management::create_ncr))
        .route("/quality/ncrs/:id", get(quality_management::get_ncr))
        .route("/quality/ncrs/:id/investigate", post(quality_management::investigate_ncr))
        .route("/quality/ncrs/:id/corrective-action", post(quality_management::start_ncr_corrective_action))
        .route("/quality/ncrs/:id/resolve", post(quality_management::resolve_ncr))
        .route("/quality/ncrs/:id/close", post(quality_management::close_ncr))

        // Corrective & Preventive Actions
        .route("/quality/ncrs/:ncr_id/actions", post(quality_management::create_corrective_action))
        .route("/quality/ncrs/:ncr_id/actions", get(quality_management::list_corrective_actions))
        .route("/quality/actions/:id", get(quality_management::get_corrective_action))
        .route("/quality/actions/:id/start", post(quality_management::start_corrective_action))
        .route("/quality/actions/:id/complete", post(quality_management::complete_corrective_action))
        .route("/quality/actions/:id/verify", post(quality_management::verify_corrective_action))

        // Quality Holds
        .route("/quality/holds", get(quality_management::list_holds))
        .route("/quality/holds", post(quality_management::create_hold))
        .route("/quality/holds/:id", get(quality_management::get_hold))
        .route("/quality/holds/:id/release", post(quality_management::release_hold))

        // Quality Dashboard
        .route("/quality/dashboard", get(quality_management::get_quality_dashboard))

        // ================================================================
        // Cost Accounting (Oracle Fusion Cost Management)
        // ================================================================

        // Cost Books
        .route("/cost-accounting/books", post(cost_accounting::create_cost_book))
        .route("/cost-accounting/books", get(cost_accounting::list_cost_books))
        .route("/cost-accounting/books/:id", get(cost_accounting::get_cost_book))
        .route("/cost-accounting/books/:id", put(cost_accounting::update_cost_book))
        .route("/cost-accounting/books/:id", delete(cost_accounting::delete_cost_book))
        .route("/cost-accounting/books/:id/deactivate", post(cost_accounting::deactivate_cost_book))
        .route("/cost-accounting/books/:id/activate", post(cost_accounting::activate_cost_book))

        // Cost Elements
        .route("/cost-accounting/elements", post(cost_accounting::create_cost_element))
        .route("/cost-accounting/elements", get(cost_accounting::list_cost_elements))
        .route("/cost-accounting/elements/:id", get(cost_accounting::get_cost_element))
        .route("/cost-accounting/elements/:id", put(cost_accounting::update_cost_element))
        .route("/cost-accounting/elements/:id", delete(cost_accounting::delete_cost_element))

        // Cost Profiles
        .route("/cost-accounting/profiles", post(cost_accounting::create_cost_profile))
        .route("/cost-accounting/profiles", get(cost_accounting::list_cost_profiles))
        .route("/cost-accounting/profiles/:id", get(cost_accounting::get_cost_profile))
        .route("/cost-accounting/profiles/:id", delete(cost_accounting::delete_cost_profile))

        // Standard Costs
        .route("/cost-accounting/standard-costs", post(cost_accounting::create_standard_cost))
        .route("/cost-accounting/standard-costs", get(cost_accounting::list_standard_costs))
        .route("/cost-accounting/standard-costs/:id", get(cost_accounting::get_standard_cost))
        .route("/cost-accounting/standard-costs/:id", put(cost_accounting::update_standard_cost))
        .route("/cost-accounting/standard-costs/:id/supersede", post(cost_accounting::supersede_standard_cost))
        .route("/cost-accounting/standard-costs/:id", delete(cost_accounting::delete_standard_cost))

        // Cost Adjustments
        .route("/cost-accounting/adjustments", post(cost_accounting::create_cost_adjustment))
        .route("/cost-accounting/adjustments", get(cost_accounting::list_cost_adjustments))
        .route("/cost-accounting/adjustments/:id", get(cost_accounting::get_cost_adjustment))
        .route("/cost-accounting/adjustments/:id/submit", post(cost_accounting::submit_adjustment))
        .route("/cost-accounting/adjustments/:id/approve", post(cost_accounting::approve_adjustment))
        .route("/cost-accounting/adjustments/:id/reject", post(cost_accounting::reject_adjustment))
        .route("/cost-accounting/adjustments/:id/post", post(cost_accounting::post_adjustment))
        .route("/cost-accounting/adjustments/:id", delete(cost_accounting::delete_cost_adjustment))

        // Cost Adjustment Lines
        .route("/cost-accounting/adjustments/:adjustment_id/lines", post(cost_accounting::add_adjustment_line))
        .route("/cost-accounting/adjustments/:adjustment_id/lines", get(cost_accounting::list_adjustment_lines))
        .route("/cost-accounting/adjustment-lines/:id", delete(cost_accounting::delete_adjustment_line))

        // Cost Variances
        .route("/cost-accounting/variances", post(cost_accounting::create_cost_variance))
        .route("/cost-accounting/variances", get(cost_accounting::list_cost_variances))
        .route("/cost-accounting/variances/:id", get(cost_accounting::get_cost_variance))
        .route("/cost-accounting/variances/:id/analyze", post(cost_accounting::analyze_variance))

        // Cost Accounting Dashboard
        .route("/cost-accounting/dashboard", get(cost_accounting::get_cost_accounting_dashboard))

        // ================================================================
        // Accounts Payable (Oracle Fusion Payables)
        // ================================================================
        .route("/ap/invoices", post(accounts_payable::create_ap_invoice))
        .route("/ap/invoices", get(accounts_payable::list_ap_invoices))
        .route("/ap/invoices/:id", get(accounts_payable::get_ap_invoice))
        .route("/ap/invoices/:id/submit", post(accounts_payable::submit_ap_invoice))
        .route("/ap/invoices/:id/approve", post(accounts_payable::approve_ap_invoice))
        .route("/ap/invoices/:id/cancel", post(accounts_payable::cancel_ap_invoice))
        // Invoice Lines
        .route("/ap/invoices/:invoice_id/lines", post(accounts_payable::add_ap_invoice_line))
        .route("/ap/invoices/:invoice_id/lines", get(accounts_payable::list_ap_invoice_lines))
        .route("/ap/invoices/:invoice_id/lines/:line_id", delete(accounts_payable::delete_ap_invoice_line))
        // Invoice Distributions
        .route("/ap/invoices/:invoice_id/distributions", post(accounts_payable::add_ap_distribution))
        .route("/ap/invoices/:invoice_id/distributions", get(accounts_payable::list_ap_distributions))
        // Invoice Holds
        .route("/ap/invoices/:invoice_id/holds", post(accounts_payable::apply_ap_hold))
        .route("/ap/invoices/:invoice_id/holds", get(accounts_payable::list_ap_holds))
        .route("/ap/holds/:hold_id/release", post(accounts_payable::release_ap_hold))
        // Payments
        .route("/ap/payments", post(accounts_payable::create_ap_payment))
        .route("/ap/payments", get(accounts_payable::list_ap_payments))
        .route("/ap/payments/:id", get(accounts_payable::get_ap_payment))
        .route("/ap/payments/:id/confirm", post(accounts_payable::confirm_ap_payment))
        // AP Aging
        .route("/ap/aging", get(accounts_payable::get_ap_aging))

        // ========================================================================
        // Supply Chain Planning (MRP)
        // ========================================================================
        // Planning Scenarios
        .route("/scp/scenarios", post(supply_chain_planning::create_scenario))
        .route("/scp/scenarios", get(supply_chain_planning::list_scenarios))
        .route("/scp/scenarios/:id", get(supply_chain_planning::get_scenario))
        .route("/scp/scenarios/:id/run", post(supply_chain_planning::run_mrp))
        .route("/scp/scenarios/:id/cancel", post(supply_chain_planning::cancel_scenario))
        // Planning Parameters
        .route("/scp/parameters", post(supply_chain_planning::upsert_parameter))
        .route("/scp/parameters", get(supply_chain_planning::list_parameters))
        .route("/scp/parameters/:item_id", delete(supply_chain_planning::delete_parameter))
        // Supply/Demand Entries
        .route("/scp/supply-demand", post(supply_chain_planning::create_supply_demand))
        .route("/scp/scenarios/:scenario_id/supply-demand", get(supply_chain_planning::list_supply_demand))
        // Planned Orders
        .route("/scp/scenarios/:scenario_id/orders", get(supply_chain_planning::list_planned_orders))
        .route("/scp/orders/:id", get(supply_chain_planning::get_planned_order))
        .route("/scp/orders/:id/firm", post(supply_chain_planning::firm_planned_order))
        .route("/scp/orders/:id/cancel", post(supply_chain_planning::cancel_planned_order))
        // Planning Exceptions
        .route("/scp/scenarios/:scenario_id/exceptions", get(supply_chain_planning::list_exceptions))
        .route("/scp/exceptions/:id/resolve", post(supply_chain_planning::resolve_exception))
        .route("/scp/exceptions/:id/dismiss", post(supply_chain_planning::dismiss_exception))
        // Planning Dashboard
        .route("/scp/dashboard", get(supply_chain_planning::get_dashboard))

        // ========================================================================
        // Workplace Health & Safety (EHS)
        // ========================================================================
        .route("/health-safety/incidents", post(health_safety::create_incident))
        .route("/health-safety/incidents", get(health_safety::list_incidents))
        .route("/health-safety/incidents/id/:id", get(health_safety::get_incident))
        .route("/health-safety/incidents/id/:id/status", post(health_safety::update_incident_status))
        .route("/health-safety/incidents/id/:id/investigation", post(health_safety::update_incident_investigation))
        .route("/health-safety/incidents/id/:id/close", post(health_safety::close_incident))
        .route("/health-safety/incidents/number/:incident_number", delete(health_safety::delete_incident))

        .route("/health-safety/hazards", post(health_safety::create_hazard))
        .route("/health-safety/hazards", get(health_safety::list_hazards))
        .route("/health-safety/hazards/id/:id", get(health_safety::get_hazard))
        .route("/health-safety/hazards/id/:id/status", post(health_safety::update_hazard_status))
        .route("/health-safety/hazards/id/:id/residual-risk", post(health_safety::assess_hazard_residual_risk))
        .route("/health-safety/hazards/code/:hazard_code", delete(health_safety::delete_hazard))

        .route("/health-safety/inspections", post(health_safety::create_inspection))
        .route("/health-safety/inspections", get(health_safety::list_inspections))
        .route("/health-safety/inspections/id/:id", get(health_safety::get_inspection))
        .route("/health-safety/inspections/id/:id/complete", post(health_safety::complete_inspection))
        .route("/health-safety/inspections/id/:id/status", post(health_safety::update_inspection_status))
        .route("/health-safety/inspections/number/:inspection_number", delete(health_safety::delete_inspection))

        .route("/health-safety/corrective-actions", post(health_safety::create_corrective_action))
        .route("/health-safety/corrective-actions", get(health_safety::list_corrective_actions))
        .route("/health-safety/corrective-actions/id/:id", get(health_safety::get_corrective_action))
        .route("/health-safety/corrective-actions/id/:id/status", post(health_safety::update_corrective_action_status))
        .route("/health-safety/corrective-actions/id/:id/complete", post(health_safety::complete_corrective_action))
        .route("/health-safety/corrective-actions/number/:action_number", delete(health_safety::delete_corrective_action))

        .route("/health-safety/dashboard", get(health_safety::get_health_safety_dashboard))

        // ══════════════════════════════════════════════════════════════════════
        // Funds Reservation & Budgetary Control (Oracle Fusion: Budgetary Control)
        // ══════════════════════════════════════════════════════════════════════
        .route("/funds-reservation/reservations", post(funds_reservation::create_reservation))
        .route("/funds-reservation/reservations", get(funds_reservation::list_reservations))
        .route("/funds-reservation/reservations/id/:id", get(funds_reservation::get_reservation))
        .route("/funds-reservation/reservations/number/:number", get(funds_reservation::get_reservation_by_number))
        .route("/funds-reservation/reservations/id/:id/consume", post(funds_reservation::consume_reservation))
        .route("/funds-reservation/reservations/id/:id/release", post(funds_reservation::release_reservation))
        .route("/funds-reservation/reservations/id/:id/cancel", post(funds_reservation::cancel_reservation))
        .route("/funds-reservation/reservations/number/:number", delete(funds_reservation::delete_reservation))
        .route("/funds-reservation/reservations/id/:reservation_id/lines", post(funds_reservation::create_reservation_line))
        .route("/funds-reservation/reservations/id/:reservation_id/lines", get(funds_reservation::list_reservation_lines))
        .route("/funds-reservation/fund-availability", get(funds_reservation::check_fund_availability))
        .route("/funds-reservation/dashboard", get(funds_reservation::get_dashboard))

        // ══════════════════════════════════════════════════════════════════════
        // Rebate Management (Oracle Fusion: Trade Management > Rebates)
        // ══════════════════════════════════════════════════════════════════════
        // Agreements
        .route("/rebate/agreements", post(rebate_management::create_agreement))
        .route("/rebate/agreements", get(rebate_management::list_agreements))
        .route("/rebate/agreements/:id", get(rebate_management::get_agreement))
        .route("/rebate/agreements/:id/activate", post(rebate_management::activate_agreement))
        .route("/rebate/agreements/:id/hold", post(rebate_management::hold_agreement))
        .route("/rebate/agreements/:id/terminate", post(rebate_management::terminate_agreement))
        .route("/rebate/agreements/number/:number", delete(rebate_management::delete_agreement))
        // Tiers
        .route("/rebate/agreements/:agreement_id/tiers", post(rebate_management::create_tier))
        .route("/rebate/agreements/:agreement_id/tiers", get(rebate_management::list_tiers))
        .route("/rebate/tiers/:id", delete(rebate_management::delete_tier))
        // Transactions
        .route("/rebate/agreements/:agreement_id/transactions", post(rebate_management::create_transaction))
        .route("/rebate/transactions/:id", get(rebate_management::get_transaction))
        .route("/rebate/agreements/:agreement_id/transactions", get(rebate_management::list_transactions))
        .route("/rebate/transactions/:id/status", post(rebate_management::update_transaction_status))
        .route("/rebate/transactions/number/:number", delete(rebate_management::delete_transaction))
        // Accruals
        .route("/rebate/agreements/:agreement_id/accruals", post(rebate_management::create_accrual))
        .route("/rebate/accruals/:id", get(rebate_management::get_accrual))
        .route("/rebate/agreements/:agreement_id/accruals", get(rebate_management::list_accruals))
        .route("/rebate/accruals/:id/post", post(rebate_management::post_accrual))
        .route("/rebate/accruals/:id/reverse", post(rebate_management::reverse_accrual))
        .route("/rebate/accruals/number/:number", delete(rebate_management::delete_accrual))
        // Settlements
        .route("/rebate/agreements/:agreement_id/settlements", post(rebate_management::create_settlement))
        .route("/rebate/settlements/:id", get(rebate_management::get_settlement))
        .route("/rebate/agreements/:agreement_id/settlements", get(rebate_management::list_settlements))
        .route("/rebate/settlements/:id/approve", post(rebate_management::approve_settlement))
        .route("/rebate/settlements/:id/pay", post(rebate_management::pay_settlement))
        .route("/rebate/settlements/:id/cancel", post(rebate_management::cancel_settlement))
        .route("/rebate/settlements/number/:number", delete(rebate_management::delete_settlement))
        .route("/rebate/settlements/:settlement_id/lines", get(rebate_management::list_settlement_lines))
        // Dashboard
        .route("/rebate/dashboard", get(rebate_management::get_rebate_dashboard))

        // ══════════════════════════════════════════════════════════════════════
        // Project Resource Management (Oracle Fusion: Project Management)
        // ══════════════════════════════════════════════════════════════════════
        // Profiles
        .route("/resource/profiles", post(project_resource_management::create_profile))
        .route("/resource/profiles", get(project_resource_management::list_profiles))
        .route("/resource/profiles/id/:id", get(project_resource_management::get_profile))
        .route("/resource/profiles/id/:id/availability", post(project_resource_management::update_availability))
        .route("/resource/profiles/number/:number", delete(project_resource_management::delete_profile))
        // Requests
        .route("/resource/requests", post(project_resource_management::create_request))
        .route("/resource/requests", get(project_resource_management::list_requests))
        .route("/resource/requests/id/:id", get(project_resource_management::get_request))
        .route("/resource/requests/id/:id/submit", post(project_resource_management::submit_request))
        .route("/resource/requests/id/:id/fulfill", post(project_resource_management::fulfill_request))
        .route("/resource/requests/id/:id/cancel", post(project_resource_management::cancel_request))
        .route("/resource/requests/number/:number", delete(project_resource_management::delete_request))
        // Assignments
        .route("/resource/assignments", post(project_resource_management::create_assignment))
        .route("/resource/assignments", get(project_resource_management::list_assignments))
        .route("/resource/assignments/id/:id", get(project_resource_management::get_assignment))
        .route("/resource/assignments/id/:id/activate", post(project_resource_management::activate_assignment))
        .route("/resource/assignments/id/:id/complete", post(project_resource_management::complete_assignment))
        .route("/resource/assignments/id/:id/cancel", post(project_resource_management::cancel_assignment))
        .route("/resource/assignments/number/:number", delete(project_resource_management::delete_assignment))
        // Utilization
        .route("/resource/utilization", post(project_resource_management::create_utilization_entry))
        .route("/resource/utilization", get(project_resource_management::list_utilization_entries))
        .route("/resource/utilization/id/:id/approve", post(project_resource_management::approve_utilization_entry))
        .route("/resource/utilization/id/:id/reject", post(project_resource_management::reject_utilization_entry))
        .route("/resource/utilization/id/:id", delete(project_resource_management::delete_utilization_entry))
        // Dashboard
        .route("/resource/dashboard", get(project_resource_management::get_resource_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Loyalty Management (Oracle Fusion CX > Loyalty Management)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Programs
        .route("/loyalty/programs", post(loyalty_management::create_program))
        .route("/loyalty/programs", get(loyalty_management::list_programs))
        .route("/loyalty/programs/:id", get(loyalty_management::get_program))
        .route("/loyalty/programs/:id/activate", post(loyalty_management::activate_program))
        .route("/loyalty/programs/:id/suspend", post(loyalty_management::suspend_program))
        .route("/loyalty/programs/:id/close", post(loyalty_management::close_program))
        .route("/loyalty/programs/number/:number", delete(loyalty_management::delete_program))

        // Tiers
        .route("/loyalty/programs/:program_id/tiers", post(loyalty_management::create_tier))
        .route("/loyalty/programs/:program_id/tiers", get(loyalty_management::list_tiers))
        .route("/loyalty/tiers/:tier_id", delete(loyalty_management::delete_tier))

        // Members
        .route("/loyalty/programs/:program_id/members", post(loyalty_management::enroll_member))
        .route("/loyalty/members/:id", get(loyalty_management::get_member))
        .route("/loyalty/programs/:program_id/members", get(loyalty_management::list_members))
        .route("/loyalty/members/:id/suspend", post(loyalty_management::suspend_member))
        .route("/loyalty/members/:id/reactivate", post(loyalty_management::reactivate_member))
        .route("/loyalty/members/number/:number", delete(loyalty_management::delete_member))

        // Point Transactions
        .route("/loyalty/programs/:program_id/accrue", post(loyalty_management::accrue_points))
        .route("/loyalty/programs/:program_id/adjust", post(loyalty_management::adjust_points))
        .route("/loyalty/transactions/:id/reverse", post(loyalty_management::reverse_transaction))
        .route("/loyalty/members/:member_id/transactions", get(loyalty_management::list_transactions))
        .route("/loyalty/transactions/number/:number", delete(loyalty_management::delete_transaction))

        // Rewards
        .route("/loyalty/programs/:program_id/rewards", post(loyalty_management::create_reward))
        .route("/loyalty/programs/:program_id/rewards", get(loyalty_management::list_rewards))
        .route("/loyalty/rewards/:id/deactivate", post(loyalty_management::deactivate_reward))
        .route("/loyalty/rewards/code/:code", delete(loyalty_management::delete_reward))

        // Redemptions
        .route("/loyalty/programs/:program_id/redeem", post(loyalty_management::redeem_reward))
        .route("/loyalty/redemptions/:id/fulfill", post(loyalty_management::fulfill_redemption))
        .route("/loyalty/redemptions/:id/cancel", post(loyalty_management::cancel_redemption))
        .route("/loyalty/members/:member_id/redemptions", get(loyalty_management::list_redemptions))

        // Dashboard
        .route("/loyalty/dashboard", get(loyalty_management::get_loyalty_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // General Ledger (Oracle Fusion GL > Chart of Accounts, Journal Entries)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Chart of Accounts
        .route("/gl/accounts", get(general_ledger::list_gl_accounts))
        .route("/gl/accounts", post(general_ledger::create_gl_account))
        .route("/gl/accounts/:id", get(general_ledger::get_gl_account))

        // Journal Entries
        .route("/gl/journal-entries", get(general_ledger::list_journal_entries))
        .route("/gl/journal-entries", post(general_ledger::create_journal_entry))
        .route("/gl/journal-entries/:id", get(general_ledger::get_journal_entry))

        // Journal Lines
        .route("/gl/journal-entries/:entry_id/lines", get(general_ledger::list_journal_lines))
        .route("/gl/journal-entries/:entry_id/lines", post(general_ledger::add_journal_line))

        // Journal Entry Workflow
        .route("/gl/journal-entries/:id/post", post(general_ledger::post_journal_entry))
        .route("/gl/journal-entries/:id/reverse", post(general_ledger::reverse_journal_entry))

        // Trial Balance
        .route("/gl/trial-balance", get(general_ledger::generate_trial_balance))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Accounts Receivable (Oracle Fusion Receivables)
        // ═════════════════════════════════════════════════════════════════════════════════

        // AR Transactions
        .route("/ar/transactions", get(accounts_receivable::list_ar_transactions))
        .route("/ar/transactions", post(accounts_receivable::create_ar_transaction))
        .route("/ar/transactions/:id", get(accounts_receivable::get_ar_transaction))
        .route("/ar/transactions/:id/complete", post(accounts_receivable::complete_ar_transaction))
        .route("/ar/transactions/:id/post", post(accounts_receivable::post_ar_transaction))
        .route("/ar/transactions/:id/cancel", post(accounts_receivable::cancel_ar_transaction))

        // AR Transaction Lines
        .route("/ar/transactions/:transaction_id/lines", get(accounts_receivable::list_transaction_lines))
        .route("/ar/transactions/:transaction_id/lines", post(accounts_receivable::add_transaction_line))

        // AR Receipts
        .route("/ar/receipts", get(accounts_receivable::list_receipts))
        .route("/ar/receipts", post(accounts_receivable::create_receipt))
        .route("/ar/receipts/:id/confirm", post(accounts_receivable::confirm_receipt))
        .route("/ar/receipts/:receipt_id/apply/:transaction_id", post(accounts_receivable::apply_receipt))
        .route("/ar/receipts/:id/reverse", post(accounts_receivable::reverse_receipt))

        // AR Credit Memos
        .route("/ar/credit-memos", post(accounts_receivable::create_credit_memo))
        .route("/ar/credit-memos/:id/approve", post(accounts_receivable::approve_credit_memo))
        .route("/ar/credit-memos/:memo_id/apply/:transaction_id", post(accounts_receivable::apply_credit_memo))

        // AR Aging
        .route("/ar/aging", get(accounts_receivable::get_ar_aging))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Payment Management (Oracle Fusion Payments)
        // ═════════════════════════════════════════════════════════════════════════════════

        .route("/payments", get(payment_management::list_payments))
        .route("/payments", post(payment_management::create_payment))
        .route("/payments/:id", get(payment_management::get_payment))
        .route("/payments/:id/issue", post(payment_management::issue_payment))
        .route("/payments/:id/clear", post(payment_management::clear_payment))
        .route("/payments/:id/void", post(payment_management::void_payment))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Netting (Oracle Fusion Netting)
        // ═════════════════════════════════════════════════════════════════════════════════

        .route("/netting/agreements", get(netting::list_netting_agreements))
        .route("/netting/agreements", post(netting::create_netting_agreement))
        .route("/netting/agreements/:id", get(netting::get_netting_agreement))
        .route("/netting/agreements/:id/activate", post(netting::activate_netting_agreement))
        .route("/netting/batches", post(netting::create_netting_batch))
        .route("/netting/batches/:id/submit", post(netting::submit_netting_batch))
        .route("/netting/batches/:id/approve", post(netting::approve_netting_batch))
        .route("/netting/batches/:id/settle", post(netting::settle_netting_batch))
        .route("/netting/dashboard", get(netting::get_netting_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Financial Statements (Oracle Fusion GL > Financial Statements)
        // ═════════════════════════════════════════════════════════════════════════════════

        .route("/financial-statements", get(financial_statements::list_financial_statements))
        .route("/financial-statements/generate", post(financial_statements::generate_financial_statement))
        .route("/financial-statements/:id", get(financial_statements::get_financial_statement))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Journal Import (Oracle Fusion GL > Import Journals)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Import Formats
        .route("/journal-import/formats", get(journal_import::list_import_formats))
        .route("/journal-import/formats", post(journal_import::create_import_format))
        .route("/journal-import/formats/:id", get(journal_import::get_import_format))
        .route("/journal-import/formats/:id", delete(journal_import::delete_import_format))

        // Column Mappings
        .route("/journal-import/formats/:format_id/mappings", post(journal_import::add_column_mapping))
        .route("/journal-import/formats/:format_id/mappings", get(journal_import::list_column_mappings))

        // Import Batches
        .route("/journal-import/batches", get(journal_import::list_import_batches))
        .route("/journal-import/batches", post(journal_import::create_import_batch))
        .route("/journal-import/batches/:id", get(journal_import::get_import_batch))
        .route("/journal-import/batches/:id", delete(journal_import::delete_import_batch))

        // Import Data Rows
        .route("/journal-import/batches/:batch_id/rows", post(journal_import::add_import_row))
        .route("/journal-import/batches/:batch_id/rows", get(journal_import::list_import_rows))

        // Import Processing
        .route("/journal-import/batches/:batch_id/validate", post(journal_import::validate_import_batch))
        .route("/journal-import/batches/:batch_id/import", post(journal_import::import_batch))

        // Journal Import Dashboard
        .route("/journal-import/dashboard", get(journal_import::get_journal_import_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Inflation Adjustment (IAS 29) (Oracle Fusion GL > Inflation Adjustment)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Inflation Indices
        .route("/inflation/indices", post(inflation_adjustment::create_inflation_index))
        .route("/inflation/indices", get(inflation_adjustment::list_inflation_indices))
        .route("/inflation/indices/:id", get(inflation_adjustment::get_inflation_index))

        // Index Rates
        .route("/inflation/rates", post(inflation_adjustment::add_index_rate))

        // Adjustment Runs
        .route("/inflation/runs", post(inflation_adjustment::create_adjustment_run))
        .route("/inflation/runs/:id/submit", post(inflation_adjustment::submit_adjustment_run))
        .route("/inflation/runs/:id/approve", post(inflation_adjustment::approve_adjustment_run))

        // Dashboard
        .route("/inflation/dashboard", get(inflation_adjustment::get_inflation_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Impairment Management (IAS 36/ASC 360) (Oracle Fusion Fixed Assets > Impairment)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Impairment Indicators
        .route("/impairment/indicators", post(impairment_management::create_impairment_indicator))
        .route("/impairment/indicators", get(impairment_management::list_impairment_indicators))

        // Impairment Tests
        .route("/impairment/tests", post(impairment_management::create_impairment_test))
        .route("/impairment/tests", get(impairment_management::list_impairment_tests))
        .route("/impairment/tests/:id/submit", post(impairment_management::submit_impairment_test))
        .route("/impairment/tests/:id/approve", post(impairment_management::approve_impairment_test))

        // Dashboard
        .route("/impairment/dashboard", get(impairment_management::get_impairment_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Bank Account Transfers (Oracle Fusion Cash Management > Bank Transfers)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Transfer Types
        .route("/bank-transfers/types", post(bank_account_transfer::create_bank_transfer_type))

        // Transfers
        .route("/bank-transfers", post(bank_account_transfer::create_bank_transfer))
        .route("/bank-transfers", get(bank_account_transfer::list_bank_transfers))
        .route("/bank-transfers/:id", get(bank_account_transfer::get_bank_transfer))
        .route("/bank-transfers/:id/submit", post(bank_account_transfer::submit_bank_transfer))
        .route("/bank-transfers/:id/approve", post(bank_account_transfer::approve_bank_transfer))
        .route("/bank-transfers/:id/complete", post(bank_account_transfer::complete_bank_transfer))

        // Dashboard
        .route("/bank-transfers/dashboard", get(bank_account_transfer::get_bank_transfer_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Tax Reporting & Filing (Oracle Fusion Tax > Tax Reporting)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Tax Return Templates
        .route("/tax-reporting/templates", post(tax_reporting::create_tax_template))
        .route("/tax-reporting/templates", get(tax_reporting::list_tax_templates))

        // Tax Returns
        .route("/tax-reporting/returns", post(tax_reporting::create_tax_return))
        .route("/tax-reporting/returns", get(tax_reporting::list_tax_returns))
        .route("/tax-reporting/returns/:id", get(tax_reporting::get_tax_return))
        .route("/tax-reporting/returns/:id/file", post(tax_reporting::file_tax_return))
        .route("/tax-reporting/returns/:id/pay", post(tax_reporting::pay_tax_return))

        // Dashboard
        .route("/tax-reporting/dashboard", get(tax_reporting::get_tax_reporting_dashboard))

        // ═════════════════════════════════════════════════════════════════════════════════
        // Subscription Management (Oracle Fusion Subscription Management)
        // ═════════════════════════════════════════════════════════════════════════════════

        // Product Catalog
        .route("/subscription/products", post(subscription::create_product))
        .route("/subscription/products", get(subscription::list_products))
        .route("/subscription/products/:code", get(subscription::get_product))
        .route("/subscription/products/:code", delete(subscription::delete_product))

        // Price Tiers
        .route("/subscription/products/:product_id/price-tiers", post(subscription::create_price_tier))

        // Subscriptions
        .route("/subscription/subscriptions", post(subscription::create_subscription))
        .route("/subscription/subscriptions", get(subscription::list_subscriptions))
        .route("/subscription/subscriptions/:id", get(subscription::get_subscription))
        .route("/subscription/subscriptions/:id/activate", post(subscription::activate_subscription))
        .route("/subscription/subscriptions/:id/suspend", post(subscription::suspend_subscription))
        .route("/subscription/subscriptions/:id/reactivate", post(subscription::reactivate_subscription))
        .route("/subscription/subscriptions/:id/cancel", post(subscription::cancel_subscription))
        .route("/subscription/subscriptions/:id/renew", post(subscription::renew_subscription))

        // Amendments
        .route("/subscription/subscriptions/:id/amendments", post(subscription::create_amendment))
        .route("/subscription/subscriptions/:id/amendments", get(subscription::list_amendments))
        .route("/subscription/amendments/:id/apply", post(subscription::apply_amendment))
        .route("/subscription/amendments/:id/cancel", post(subscription::cancel_amendment))

        // Billing & Revenue Schedules
        .route("/subscription/subscriptions/:id/billing-schedule", get(subscription::list_billing_schedule))
        .route("/subscription/subscriptions/:id/revenue-schedule", get(subscription::list_revenue_schedule))
        .route("/subscription/revenue-lines/:line_id/recognize", post(subscription::recognize_revenue))

        // Dashboard
        .route("/subscription/dashboard", get(subscription::get_subscription_dashboard))

        // Financial Consolidation (Oracle Fusion: General Ledger > Financial Consolidation)
        .route("/financial-consolidation/ledgers", post(financial_consolidation::create_ledger))
        .route("/financial-consolidation/ledgers", get(financial_consolidation::list_ledgers))
        .route("/financial-consolidation/ledgers/:code", get(financial_consolidation::get_ledger))
        .route("/financial-consolidation/ledgers/:ledger_id/entities", post(financial_consolidation::add_entity))
        .route("/financial-consolidation/ledgers/:ledger_id/entities", get(financial_consolidation::list_entities))
        .route("/financial-consolidation/scenarios", post(financial_consolidation::create_scenario))
        .route("/financial-consolidation/scenarios", get(financial_consolidation::list_scenarios))
        .route("/financial-consolidation/scenarios/:scenario_id/execute", post(financial_consolidation::execute_consolidation))
        .route("/financial-consolidation/scenarios/:scenario_id/approve", post(financial_consolidation::approve_scenario))
        .route("/financial-consolidation/scenarios/:scenario_id/post", post(financial_consolidation::post_scenario))
        .route("/financial-consolidation/scenarios/:scenario_id/reverse", post(financial_consolidation::reverse_scenario))
        .route("/financial-consolidation/elimination-rules", post(financial_consolidation::create_elimination_rule))
        .route("/financial-consolidation/elimination-rules", get(financial_consolidation::list_elimination_rules))
        .route("/financial-consolidation/dashboard", get(financial_consolidation::get_consolidation_dashboard))

        // Joint Venture Management (Oracle Fusion: Financials > Joint Venture Management)
        .route("/joint-venture/ventures", post(joint_venture::create_venture))
        .route("/joint-venture/ventures", get(joint_venture::list_ventures))
        .route("/joint-venture/ventures/:id", get(joint_venture::get_venture))
        .route("/joint-venture/ventures/:id/activate", post(joint_venture::activate_venture))
        .route("/joint-venture/ventures/:id/close", post(joint_venture::close_venture))
        .route("/joint-venture/ventures/:venture_id/partners", post(joint_venture::add_partner))
        .route("/joint-venture/ventures/:venture_id/partners", get(joint_venture::list_partners))
        .route("/joint-venture/ventures/:venture_id/afes", post(joint_venture::create_afe))
        .route("/joint-venture/ventures/:venture_id/afes", get(joint_venture::list_afes))
        .route("/joint-venture/afes/:id/submit", post(joint_venture::submit_afe))
        .route("/joint-venture/afes/:id/approve", post(joint_venture::approve_afe))
        .route("/joint-venture/ventures/:venture_id/cost-distributions", post(joint_venture::create_cost_distribution))
        .route("/joint-venture/ventures/:venture_id/cost-distributions", get(joint_venture::list_cost_distributions))
        .route("/joint-venture/cost-distributions/:id/post", post(joint_venture::post_cost_distribution))
        .route("/joint-venture/dashboard", get(joint_venture::get_joint_venture_dashboard))

        // Deferred Revenue/Cost Management (Oracle Fusion: Revenue Management > Deferral Schedules)
        .route("/deferred-revenue/templates", post(deferred_revenue::create_template))
        .route("/deferred-revenue/templates", get(deferred_revenue::list_templates))
        .route("/deferred-revenue/templates/:code", get(deferred_revenue::get_template))
        .route("/deferred-revenue/templates/:code", delete(deferred_revenue::delete_template))
        .route("/deferred-revenue/schedules", post(deferred_revenue::create_schedule))
        .route("/deferred-revenue/schedules", get(deferred_revenue::list_schedules))
        .route("/deferred-revenue/schedules/:id", get(deferred_revenue::get_schedule))
        .route("/deferred-revenue/schedules/:schedule_id/lines", get(deferred_revenue::list_schedule_lines))
        .route("/deferred-revenue/schedules/recognize", post(deferred_revenue::recognize_pending))
        .route("/deferred-revenue/schedules/:id/hold", post(deferred_revenue::hold_schedule))
        .route("/deferred-revenue/schedules/:id/resume", post(deferred_revenue::resume_schedule))
        .route("/deferred-revenue/schedules/:id/cancel", post(deferred_revenue::cancel_schedule))
        .route("/deferred-revenue/dashboard", get(deferred_revenue::get_deferred_revenue_dashboard))

        // ═══════════════════════════════════════════════════════
        // Revenue Management (ASC 606 / IFRS 15)
        // ═══════════════════════════════════════════════════════
        .route("/revenue-management/contracts", post(revenue_management::create_contract))
        .route("/revenue-management/contracts", get(revenue_management::list_contracts))
        .route("/revenue-management/contracts/:number", get(revenue_management::get_contract))
        .route("/revenue-management/contracts/:id/activate", post(revenue_management::activate_contract))
        .route("/revenue-management/contracts/:id/cancel", post(revenue_management::cancel_contract))
        .route("/revenue-management/obligations", post(revenue_management::create_obligation))
        .route("/revenue-management/contracts/:contract_id/obligations", get(revenue_management::list_obligations))
        .route("/revenue-management/contracts/:contract_id/allocate", post(revenue_management::allocate_transaction_price))
        .route("/revenue-management/ssp", post(revenue_management::create_ssp))
        .route("/revenue-management/ssp", get(revenue_management::list_ssps))
        .route("/revenue-management/obligations/:obligation_id/satisfy", post(revenue_management::satisfy_obligation))
        .route("/revenue-management/contracts/:contract_id/events", get(revenue_management::list_recognition_events))
        .route("/revenue-management/dashboard", get(revenue_management::get_revenue_management_dashboard))

        // ═══════════════════════════════════════════════════════
        // Cash Flow Forecasting
        // ═══════════════════════════════════════════════════════
        .route("/cash-flow-forecasts", post(cash_flow_forecast::create_forecast))
        .route("/cash-flow-forecasts", get(cash_flow_forecast::list_forecasts))
        .route("/cash-flow-forecasts/:id", get(cash_flow_forecast::get_forecast))
        .route("/cash-flow-forecasts/:id/activate", post(cash_flow_forecast::activate_forecast))
        .route("/cash-flow-forecasts/:id/approve", post(cash_flow_forecast::approve_forecast))
        .route("/cash-flow-forecasts/scenarios", post(cash_flow_forecast::create_scenario))
        .route("/cash-flow-forecasts/:forecast_id/scenarios", get(cash_flow_forecast::list_scenarios))
        .route("/cash-flow-forecasts/entries", post(cash_flow_forecast::create_entry))
        .route("/cash-flow-forecasts/:forecast_id/entries", get(cash_flow_forecast::list_entries))
        .route("/cash-flow-forecasts/dashboard", get(cash_flow_forecast::get_cash_forecast_dashboard))

        // ═══════════════════════════════════════════════════════
        // Regulatory Reporting
        // ═══════════════════════════════════════════════════════
        .route("/regulatory-templates", post(regulatory_reporting::create_reg_template))
        .route("/regulatory-templates", get(regulatory_reporting::list_reg_templates))
        .route("/regulatory-templates/:code", delete(regulatory_reporting::delete_reg_template))
        .route("/regulatory-reports", post(regulatory_reporting::create_reg_report))
        .route("/regulatory-reports", get(regulatory_reporting::list_reg_reports))
        .route("/regulatory-reports/:id/review", post(regulatory_reporting::submit_for_review))
        .route("/regulatory-reports/:id/approve", post(regulatory_reporting::approve_reg_report))
        .route("/regulatory-reports/:id/reject", post(regulatory_reporting::reject_reg_report))
        .route("/regulatory-filings", post(regulatory_reporting::create_filing))
        .route("/regulatory-filings", get(regulatory_reporting::list_filings))
        .route("/regulatory-reporting/dashboard", get(regulatory_reporting::get_regulatory_dashboard))

        // ═══════════════════════════════════════════════════════
        // Advance Payments (Supplier Prepayments)
        // ═══════════════════════════════════════════════════════
        .route("/advance-payments", post(advance_payment::create_advance))
        .route("/advance-payments", get(advance_payment::list_advances))
        .route("/advance-payments/:id", get(advance_payment::get_advance))
        .route("/advance-payments/:id/approve", post(advance_payment::approve_advance))
        .route("/advance-payments/:id/pay", post(advance_payment::pay_advance))
        .route("/advance-payments/:id/cancel", post(advance_payment::cancel_advance))
        .route("/advance-payments/apply", post(advance_payment::apply_to_invoice))
        .route("/advance-payments/dashboard", get(advance_payment::get_advance_dashboard))

        // ═══════════════════════════════════════════════════════
        // Customer Deposits
        // ═══════════════════════════════════════════════════════
        .route("/customer-deposits", post(customer_deposit::create_deposit))
        .route("/customer-deposits", get(customer_deposit::list_deposits))
        .route("/customer-deposits/:id", get(customer_deposit::get_deposit))
        .route("/customer-deposits/:id/receive", post(customer_deposit::receive_deposit))
        .route("/customer-deposits/:id/refund", post(customer_deposit::refund_deposit))
        .route("/customer-deposits/:id/cancel", post(customer_deposit::cancel_deposit))
        .route("/customer-deposits/apply", post(customer_deposit::apply_deposit_to_invoice))
        .route("/customer-deposits/dashboard", get(customer_deposit::get_deposit_dashboard))

        // ═══════════════════════════════════════════════════════
        // Cash Position
        // ═══════════════════════════════════════════════════════
        .route("/cash-positions", post(cash_position::record_position))
        .route("/cash-positions", get(cash_position::list_positions))
        .route("/cash-positions/dashboard", get(cash_position::get_cash_position_dashboard))

        // ═══════════════════════════════════════════════════════
        // Accounting Hub
        // ═══════════════════════════════════════════════════════
        .route("/accounting-hub/rules", post(accounting_hub::create_mapping_rule))
        .route("/accounting-hub/rules", get(accounting_hub::list_mapping_rules))
        .route("/accounting-hub/rules/:code", delete(accounting_hub::delete_mapping_rule))
        .route("/accounting-hub/dashboard", get(accounting_hub::get_accounting_hub_dashboard))

        // ═══════════════════════════════════════════════════════
        // Financial Controls
        // ═══════════════════════════════════════════════════════
        .route("/financial-controls/rules", post(financial_controls::create_control_rule))
        .route("/financial-controls/rules", get(financial_controls::list_control_rules))
        .route("/financial-controls/rules/:code", delete(financial_controls::delete_control_rule))
        .route("/financial-controls/dashboard", get(financial_controls::get_financial_controls_dashboard))

        // ════════════════════════════════════════════════════════════════════════════════
        // Payment Terms Management (Oracle Fusion: Financials > Payment Terms)
        // ════════════════════════════════════════════════════════════════════════════════
        .route("/payment-terms", post(payment_terms::create_term))
        .route("/payment-terms", get(payment_terms::list_terms))
        .route("/payment-terms/:id", get(payment_terms::get_term))
        .route("/payment-terms/:id/activate", post(payment_terms::activate_term))
        .route("/payment-terms/:id/deactivate", post(payment_terms::deactivate_term))
        .route("/payment-terms/:id", delete(payment_terms::delete_term))
        .route("/payment-terms/:term_id/discount-schedules", post(payment_terms::create_discount_schedule))
        .route("/payment-terms/:term_id/discount-schedules", get(payment_terms::list_discount_schedules))
        .route("/payment-terms/:term_id/discount-schedules/:schedule_id", delete(payment_terms::delete_discount_schedule))
        .route("/payment-terms/:term_id/installments", post(payment_terms::create_installment))
        .route("/payment-terms/:term_id/installments", get(payment_terms::list_installments))
        .route("/payment-terms/:term_id/installments/:installment_id", delete(payment_terms::delete_installment))
        .route("/payment-terms/dashboard", get(payment_terms::get_payment_terms_dashboard))

        // ════════════════════════════════════════════════════════════════════════════════
        // Lockbox Processing (Oracle Fusion: AR > Lockbox)
        // ════════════════════════════════════════════════════════════════════════════════
        .route("/lockbox/batches", post(lockbox::create_batch))
        .route("/lockbox/batches", get(lockbox::list_batches))
        .route("/lockbox/batches/:id", get(lockbox::get_batch))
        .route("/lockbox/batches/:id/validate", post(lockbox::validate_batch))
        .route("/lockbox/batches/:id/apply", post(lockbox::apply_batch))
        .route("/lockbox/batches/:batch_id/receipts", post(lockbox::create_receipt))
        .route("/lockbox/batches/:batch_id/receipts", get(lockbox::list_receipts))
        .route("/lockbox/receipts/:receipt_id/apply", post(lockbox::manual_apply_receipt))
        .route("/lockbox/receipts/:receipt_id/applications", get(lockbox::list_applications))
        .route("/lockbox/formats", post(lockbox::create_format))
        .route("/lockbox/formats", get(lockbox::list_formats))
        .route("/lockbox/dashboard", get(lockbox::get_lockbox_dashboard))

        // ════════════════════════════════════════════════════════════════════════════════
        // AR Aging Analysis (Oracle Fusion: AR > Aging Reports)
        // ════════════════════════════════════════════════════════════════════════════════
        .route("/ar-aging/definitions", post(ar_aging::create_definition))
        .route("/ar-aging/definitions", get(ar_aging::list_definitions))
        .route("/ar-aging/definitions/:id", get(ar_aging::get_definition))
        .route("/ar-aging/definitions/:id", delete(ar_aging::delete_definition))
        .route("/ar-aging/definitions/:def_id/buckets", post(ar_aging::create_bucket))
        .route("/ar-aging/definitions/:def_id/buckets", get(ar_aging::list_buckets))
        .route("/ar-aging/snapshots", post(ar_aging::create_snapshot))
        .route("/ar-aging/snapshots", get(ar_aging::list_snapshots))
        .route("/ar-aging/snapshots/:id", get(ar_aging::get_snapshot))
        .route("/ar-aging/snapshots/:snapshot_id/lines", get(ar_aging::list_snapshot_lines))
        .route("/ar-aging/snapshots/:snapshot_id/summary", get(ar_aging::get_aging_summary))
        .route("/ar-aging/dashboard", get(ar_aging::get_ar_aging_dashboard))

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
        .layer(middleware::from_fn(admin_auth_middleware))
        .layer(middleware::from_fn(auth_middleware))
}
