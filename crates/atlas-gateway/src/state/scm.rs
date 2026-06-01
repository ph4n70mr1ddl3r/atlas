use atlas_core::scm::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct ScmState {
    pub sourcing_engine: Arc<SourcingEngine>,
    pub procurement_contract_engine: Arc<ProcurementContractEngine>,
    pub inventory_engine: Arc<InventoryEngine>,
    pub customer_returns_engine: Arc<CustomerReturnsEngine>,
    pub pricing_engine: Arc<PricingEngine>,
    pub purchase_requisition_engine: Arc<PurchaseRequisitionEngine>,
    pub product_information_engine: Arc<ProductInformationEngine>,
    pub quality_engine: Arc<QualityManagementEngine>,
    pub order_management_engine: Arc<OrderManagementEngine>,
    pub manufacturing_engine: Arc<ManufacturingEngine>,
    pub warehouse_management_engine: Arc<WarehouseManagementEngine>,
    pub shipping_engine: Arc<ShippingEngine>,
    pub receiving_engine: Arc<ReceivingEngine>,
    pub supplier_qualification_engine: Arc<SupplierQualificationEngine>,
    pub scorecard_engine: Arc<SupplierScorecardEngine>,
    pub landed_cost_engine: Arc<LandedCostEngine>,
    pub clm_engine: Arc<ContractLifecycleEngine>,
    pub demand_planning_engine: Arc<DemandPlanningEngine>,
    pub planning_engine: Arc<SupplyChainPlanningEngine>,
    pub configurator_engine: Arc<ProductConfiguratorEngine>,
    pub transportation_engine: Arc<TransportationManagementEngine>,
    pub channel_revenue_engine: Arc<ChannelRevenueEngine>,
    pub rebate_management_engine: Arc<RebateManagementEngine>,
}
