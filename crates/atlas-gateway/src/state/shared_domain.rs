use std::sync::Arc;
use atlas_core::shared::*;

#[derive(Clone)]
pub struct SharedState {
    pub lease_accounting_engine: Arc<LeaseAccountingEngine>,
    pub cost_allocation_engine: Arc<CostAllocationEngine>,
    pub financial_reporting_engine: Arc<FinancialReportingEngine>,
    pub dff_engine: Arc<DescriptiveFlexfieldEngine>,
    pub cvr_engine: Arc<CrossValidationEngine>,
    pub scheduled_process_engine: Arc<ScheduledProcessEngine>,
    pub sod_engine: Arc<SegregationOfDutiesEngine>,
    pub kpi_engine: Arc<KpiEngine>,
    pub eam_engine: Arc<EnterpriseAssetManagementEngine>,
    pub risk_management_engine: Arc<RiskManagementEngine>,
    pub sustainability_engine: Arc<SustainabilityEngine>,
    pub ecm_engine: Arc<EngineeringChangeEngine>,
    pub health_safety_engine: Arc<HealthSafetyEngine>,
    pub transfer_pricing_engine: Arc<TransferPricingEngine>,
    pub cost_accounting_engine: Arc<CostAccountingEngine>,
    pub funds_reservation_engine: Arc<FundsReservationEngine>,
    pub territory_engine: Arc<TerritoryManagementEngine>,
}
