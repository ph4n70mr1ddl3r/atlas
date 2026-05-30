use std::sync::Arc;
use atlas_core::hcm::*;

#[derive(Clone)]
pub struct HcmState {
    pub absence_engine: Arc<AbsenceEngine>,
    pub time_and_labor_engine: Arc<TimeAndLaborEngine>,
    pub payroll_engine: Arc<PayrollEngine>,
    pub compensation_engine: Arc<CompensationEngine>,
    pub benefits_engine: Arc<BenefitsEngine>,
    pub performance_engine: Arc<PerformanceEngine>,
    pub recruiting_engine: Arc<RecruitingEngine>,
    pub learning_management_engine: Arc<LearningManagementEngine>,
    pub succession_planning_engine: Arc<SuccessionPlanningEngine>,
    pub goal_management_engine: Arc<GoalManagementEngine>,
    pub approval_authority_engine: Arc<ApprovalAuthorityEngine>,
    pub approval_delegation_engine: Arc<ApprovalDelegationEngine>,
    pub data_archiving_engine: Arc<DataArchivingEngine>,
}
