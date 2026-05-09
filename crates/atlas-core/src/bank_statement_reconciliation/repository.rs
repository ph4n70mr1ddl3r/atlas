//! Bank Statement Reconciliation Repository Trait
//!
//! Defines the persistence contract for bank statement auto-reconciliation data.

use async_trait::async_trait;
use atlas_shared::AtlasResult;

/// Repository trait for bank statement reconciliation persistence.
/// In a full implementation, each method would store/retrieve from PostgreSQL.
#[async_trait]
pub trait BankStatementReconciliationRepository: Send + Sync {}

/// PostgreSQL-backed implementation (stub — real SQL in production).
pub struct PostgresBankStatementReconciliationRepository;

impl PostgresBankStatementReconciliationRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BankStatementReconciliationRepository for PostgresBankStatementReconciliationRepository {}
