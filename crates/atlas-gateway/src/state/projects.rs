use atlas_core::projects::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectsState {
    pub project_costing_engine: Arc<ProjectCostingEngine>,
    pub project_resource_engine: Arc<ProjectResourceManagementEngine>,
    pub joint_venture_engine: Arc<JointVentureEngine>,
    pub project_billing_engine: Arc<ProjectBillingEngine>,
}
