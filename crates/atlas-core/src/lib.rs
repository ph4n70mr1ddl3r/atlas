//! Atlas Core Engine
//! 
//! The declarative foundation of Atlas ERP. This module contains:
//! - Schema engine for dynamic entity definitions
//! - Workflow engine for state machine execution
//! - Validation engine for declarative rules
//! - Formula engine for computed fields
//! - Security engine for access control
//! - Audit engine for change tracking
//! - Configuration engine for hot-reload
//! - Event bus for inter-service communication
//! - Notification engine (Oracle Fusion bell-icon notifications)
//! - Approval engine (Oracle Fusion multi-level approvals)

pub mod schema;
pub mod workflow;
pub mod validation;
pub mod formula;
pub mod security;
pub mod audit;
pub mod config;
pub mod eventbus;
pub mod notification;
pub mod approval;
pub mod period_close;
pub mod currency;
pub mod tax;
pub mod intercompany;
pub mod reconciliation;
pub mod expense;

pub use schema::*;
pub use workflow::{
    WorkflowEngine, StateMachine, GuardEvaluator, GuardResult,
    ActionExecutor, ActionResult,
    WorkflowState, StateHistoryEntry, TransitionResult,
    AvailableTransitions, TransitionInfo,
    repository::{WorkflowStateRepository, PostgresWorkflowStateRepository, InMemoryWorkflowStateRepository},
};

// Re-export the workflow engine's User type under a distinct path
// so downstream crates can import it without colliding with
// atlas_shared::User.
pub use workflow::engine::User as WorkflowUser;

pub use validation::*;
pub use formula::*;
pub use security::*;
pub use audit::*;
pub use config::*;
pub use eventbus::*;
pub use notification::{NotificationEngine, PostgresNotificationRepository as PostgresNotificationRepo};
pub use approval::{ApprovalEngine, PostgresApprovalRepository as PostgresApprovalRepo};
pub use period_close::{PeriodCloseEngine, PostgresPeriodCloseRepository as PostgresPeriodCloseRepo};
pub use currency::{CurrencyEngine, PostgresCurrencyRepository as PostgresCurrencyRepo};
pub use tax::{TaxEngine, PostgresTaxRepository as PostgresTaxRepo};
pub use intercompany::{IntercompanyEngine, PostgresIntercompanyRepository as PostgresIntercompanyRepo};
pub use reconciliation::{ReconciliationEngine, PostgresReconciliationRepository as PostgresReconciliationRepo};
pub use expense::{ExpenseEngine, PostgresExpenseRepository as PostgresExpenseRepo};

mod mock_repos;
pub use mock_repos::*;
