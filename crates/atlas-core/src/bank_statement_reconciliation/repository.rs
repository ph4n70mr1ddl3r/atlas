//! Bank Statement Reconciliation Repository Trait
//!
//! Defines the persistence contract for bank statement auto-reconciliation data.

use async_trait::async_trait;

/// Repository trait for bank statement reconciliation persistence.
/// In a full implementation, each method would store/retrieve from `PostgreSQL`.
#[async_trait]
pub trait BankStatementReconciliationRepository: Send + Sync {}

/// PostgreSQL-backed implementation (stub — real SQL in production).
pub struct PostgresBankStatementReconciliationRepository;

impl Default for PostgresBankStatementReconciliationRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresBankStatementReconciliationRepository {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BankStatementReconciliationRepository for PostgresBankStatementReconciliationRepository {}
