use std::sync::Arc;
use atlas_core::projects::*;

#[derive(Clone)]
pub struct ProjectsState {
    pub project_costing_engine: Arc<ProjectCostingEngine>,
    pub project_resource_engine: Arc<ProjectResourceManagementEngine>,
    pub joint_venture_engine: Arc<JointVentureEngine>,
    pub project_billing_engine: Arc<ProjectBillingEngine>,
}
