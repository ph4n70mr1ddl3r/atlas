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
