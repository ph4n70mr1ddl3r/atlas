use atlas_core::crm::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct CrmState {
    pub sales_commission_engine: Arc<SalesCommissionEngine>,
    pub subscription_engine: Arc<SubscriptionEngine>,
    pub lead_opportunity_engine: Arc<LeadOpportunityEngine>,
    pub marketing_engine: Arc<MarketingEngine>,
    pub service_request_engine: Arc<ServiceRequestEngine>,
    pub loyalty_engine: Arc<LoyaltyManagementEngine>,
    pub promotions_engine: Arc<PromotionsManagementEngine>,
}
