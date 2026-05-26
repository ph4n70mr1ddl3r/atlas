use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Service Request Management (Oracle Fusion CX Service)
// ============================================================================

/// Service request category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub default_priority: Option<String>,
    pub default_sla_hours: Option<i32>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Service request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub priority: String,
    pub status: String,
    pub request_type: String,
    pub channel: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub assigned_group: Option<String>,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
    pub resolution: Option<String>,
    pub resolution_code: Option<String>,
    pub sla_due_date: Option<chrono::NaiveDate>,
    pub sla_breached: bool,
    pub parent_request_id: Option<Uuid>,
    pub related_object_type: Option<String>,
    pub related_object_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Service request communication/update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequestUpdate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_id: Uuid,
    pub update_type: String,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub is_internal: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Service request assignment history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequestAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_id: Uuid,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub assigned_group: Option<String>,
    pub assigned_by: Option<Uuid>,
    pub assigned_by_name: Option<String>,
    pub assignment_type: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Service request dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequestDashboard {
    pub total_open: i32,
    pub total_resolved: i32,
    pub total_closed: i32,
    pub total_unassigned: i32,
    pub sla_breached_count: i32,
    pub by_priority: serde_json::Value,
    pub by_status: serde_json::Value,
    pub by_category: serde_json::Value,
    pub by_channel: serde_json::Value,
    pub average_resolution_hours: String,
}

// ============================================================================
// Lead and Opportunity Management (Oracle Fusion CX Sales)
// ============================================================================

/// Lead source (e.g. website, referral, trade show)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadSource {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lead rating / scoring model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadRatingModel {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub scoring_criteria: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales lead
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesLead {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lead_number: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub lead_source_id: Option<Uuid>,
    pub lead_source_name: Option<String>,
    pub lead_rating_model_id: Option<Uuid>,
    pub lead_score: String,
    pub lead_rating: String,
    pub estimated_value: String,
    pub currency_code: String,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub converted_opportunity_id: Option<Uuid>,
    pub converted_customer_id: Option<Uuid>,
    pub converted_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub address: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Opportunity pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityStage {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub probability: String,
    pub display_order: i32,
    pub is_won: bool,
    pub is_lost: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesOpportunity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub opportunity_number: String,
    pub name: String,
    pub description: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub lead_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub stage_name: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub probability: String,
    pub weighted_amount: String,
    pub expected_close_date: Option<chrono::NaiveDate>,
    pub actual_close_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub competitor: Option<String>,
    pub lost_reason: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Opportunity line item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub opportunity_id: Uuid,
    pub line_number: i32,
    pub product_name: String,
    pub product_code: Option<String>,
    pub description: Option<String>,
    pub quantity: String,
    pub unit_price: String,
    pub line_amount: String,
    pub discount_percent: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales activity (call, meeting, task)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subject: String,
    pub description: Option<String>,
    pub activity_type: String,
    pub status: String,
    pub priority: String,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Opportunity stage history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityStageHistory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub opportunity_id: Uuid,
    pub from_stage: Option<String>,
    pub to_stage: String,
    pub changed_by: Option<Uuid>,
    pub changed_by_name: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Lead and Opportunity dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesPipelineDashboard {
    pub total_leads: i32,
    pub new_leads: i32,
    pub qualified_leads: i32,
    pub converted_leads: i32,
    pub total_opportunities: i32,
    pub open_opportunities: i32,
    pub won_opportunities: i32,
    pub lost_opportunities: i32,
    pub total_pipeline_value: String,
    pub weighted_pipeline_value: String,
    pub total_won_value: String,
    pub average_deal_size: String,
    pub win_rate: String,
    pub by_stage: serde_json::Value,
    pub by_owner: serde_json::Value,
}

// ============================================================================
// Marketing Campaign Management (Oracle Fusion CX Marketing)
// ============================================================================

/// Campaign Type
/// Oracle Fusion: CX Marketing > Campaign Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub channel: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Marketing Campaign
/// Oracle Fusion: CX Marketing > Campaigns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketingCampaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_number: String,
    pub name: String,
    pub description: Option<String>,
    pub campaign_type_id: Option<Uuid>,
    pub campaign_type_name: Option<String>,
    pub status: String,
    pub channel: String,
    pub budget: String,
    pub actual_cost: String,
    pub currency_code: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub expected_responses: i32,
    pub expected_revenue: String,
    pub actual_responses: i32,
    pub actual_revenue: String,
    pub converted_leads: i32,
    pub converted_opportunities: i32,
    pub converted_won: i32,
    pub parent_campaign_id: Option<Uuid>,
    pub parent_campaign_name: Option<String>,
    pub tags: serde_json::Value,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub activated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Campaign Member
/// Oracle Fusion: CX Marketing > Campaign Members
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub lead_id: Option<Uuid>,
    pub lead_number: Option<String>,
    pub status: String,
    pub response: Option<String>,
    pub responded_at: Option<DateTime<Utc>>,
    pub converted_contact_id: Option<Uuid>,
    pub converted_lead_id: Option<Uuid>,
    pub converted_opportunity_id: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Campaign Response
/// Oracle Fusion: CX Marketing > Campaign Responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_id: Uuid,
    pub member_id: Option<Uuid>,
    pub response_type: String,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub lead_id: Option<Uuid>,
    pub description: Option<String>,
    pub value: String,
    pub currency_code: String,
    pub source_url: Option<String>,
    pub responded_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Marketing Dashboard
/// Oracle Fusion: CX Marketing > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketingDashboard {
    pub total_campaigns: i32,
    pub active_campaigns: i32,
    pub completed_campaigns: i32,
    pub total_budget: String,
    pub total_actual_cost: String,
    pub total_expected_revenue: String,
    pub total_actual_revenue: String,
    pub total_responses: i32,
    pub total_converted_leads: i32,
    pub overall_roi: String,
    pub campaigns_by_status: serde_json::Value,
    pub campaigns_by_channel: serde_json::Value,
    pub top_campaigns: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Receiving Management (Oracle Fusion SCM > Receiving)
// ═══════════════════════════════════════════════════════════════════════════════

/// Receiving Location
/// Oracle Fusion: SCM > Receiving > Locations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivingLocation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub location_type: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receipt Header
/// Oracle Fusion: SCM > Receiving > Receipts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptHeader {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_number: String,
    pub receipt_type: String,
    pub receipt_source: String,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub supplier_number: Option<String>,
    pub purchase_order_id: Option<Uuid>,
    pub purchase_order_number: Option<String>,
    pub receiving_location_id: Option<Uuid>,
    pub receiving_location_code: Option<String>,
    pub receiving_date: Option<chrono::NaiveDate>,
    pub packing_slip_number: Option<String>,
    pub bill_of_lading: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub waybill_number: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub total_received_qty: String,
    pub total_inspected_qty: String,
    pub total_accepted_qty: String,
    pub total_rejected_qty: String,
    pub total_delivered_qty: String,
    pub received_by: Option<Uuid>,
    pub received_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receipt Line
/// Oracle Fusion: SCM > Receiving > Receipt Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub line_number: i32,
    pub purchase_order_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub ordered_qty: String,
    pub ordered_uom: Option<String>,
    pub received_qty: String,
    pub received_uom: Option<String>,
    pub accepted_qty: String,
    pub rejected_qty: String,
    pub inspection_status: String,
    pub delivery_status: String,
    pub lot_number: Option<String>,
    pub serial_numbers: serde_json::Value,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub manufacture_date: Option<chrono::NaiveDate>,
    pub unit_price: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receipt Inspection
/// Oracle Fusion: SCM > Receiving > Inspections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptInspection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub receipt_line_id: Uuid,
    pub inspection_number: String,
    pub inspection_template: Option<String>,
    pub inspector_id: Option<Uuid>,
    pub inspector_name: Option<String>,
    pub inspection_date: Option<chrono::NaiveDate>,
    pub sample_size: Option<String>,
    pub quantity_inspected: String,
    pub quantity_accepted: String,
    pub quantity_rejected: String,
    pub disposition: String,
    pub rejection_reason: Option<String>,
    pub quality_score: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inspection Detail
/// Oracle Fusion: SCM > Receiving > Inspection Details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionDetail {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inspection_id: Uuid,
    pub check_number: i32,
    pub check_name: String,
    pub check_type: String,
    pub specification: Option<String>,
    pub result: String,
    pub measured_value: Option<String>,
    pub expected_value: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Receipt Delivery
/// Oracle Fusion: SCM > Receiving > Deliveries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptDelivery {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub receipt_line_id: Uuid,
    pub delivery_number: String,
    pub subinventory: Option<String>,
    pub locator: Option<String>,
    pub quantity_delivered: String,
    pub uom: Option<String>,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub delivered_by: Option<Uuid>,
    pub delivered_by_name: Option<String>,
    pub delivery_date: Option<DateTime<Utc>>,
    pub destination_type: String,
    pub account_code: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receipt Return (Return to Supplier)
/// Oracle Fusion: SCM > Receiving > Returns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptReturn {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub return_number: String,
    pub receipt_id: Option<Uuid>,
    pub receipt_line_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub return_type: String,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_returned: String,
    pub uom: Option<String>,
    pub unit_price: Option<String>,
    pub currency: Option<String>,
    pub return_reason: Option<String>,
    pub return_date: Option<chrono::NaiveDate>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub credit_expected: bool,
    pub credit_memo_number: Option<String>,
    pub status: String,
    pub shipped_at: Option<DateTime<Utc>>,
    pub credited_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receiving Dashboard
/// Oracle Fusion: SCM > Receiving > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivingDashboard {
    pub total_receipts: i32,
    pub pending_receipts: i32,
    pub received_today: i32,
    pub pending_inspections: i32,
    pub pending_deliveries: i32,
    pub total_returns: i32,
    pub receipts_by_status: serde_json::Value,
    pub top_suppliers: serde_json::Value,
    pub recent_receipts: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Supplier Scorecard Management (Oracle Fusion Supplier Portal > Supplier Performance)
// ═══════════════════════════════════════════════════════════════════════════════

/// Scorecard Template
/// Oracle Fusion: Supplier Portal > Performance > Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorecardTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub evaluation_period: String,
    pub is_active: bool,
    pub total_weight: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scorecard Category (KPI category within a template)
/// Oracle Fusion: Supplier Portal > Performance > KPI Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorecardCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub weight: String,
    pub sort_order: i32,
    pub scoring_model: String,
    pub target_score: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Scorecard
/// Oracle Fusion: Supplier Portal > Performance > Scorecards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierScorecard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub scorecard_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub supplier_number: Option<String>,
    pub evaluation_period_start: chrono::NaiveDate,
    pub evaluation_period_end: chrono::NaiveDate,
    pub status: String,
    pub overall_score: String,
    pub overall_grade: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub review_date: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scorecard Line (individual KPI score)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorecardLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scorecard_id: Uuid,
    pub category_id: Uuid,
    pub line_number: i32,
    pub kpi_name: String,
    pub kpi_description: Option<String>,
    pub weight: String,
    pub target_value: Option<String>,
    pub actual_value: Option<String>,
    pub score: String,
    pub weighted_score: String,
    pub evidence: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Performance Review
/// Oracle Fusion: Supplier Portal > Performance > Reviews
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierPerformanceReview {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub review_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub scorecard_id: Option<Uuid>,
    pub review_type: String,
    pub review_period: Option<String>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub previous_score: Option<String>,
    pub current_score: Option<String>,
    pub score_change: Option<String>,
    pub rating: Option<String>,
    pub strengths: Option<String>,
    pub improvement_areas: Option<String>,
    pub action_items: Option<String>,
    pub follow_up_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Review Action Item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewActionItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub review_id: Uuid,
    pub action_number: i32,
    pub description: String,
    pub assignee_id: Option<Uuid>,
    pub assignee_name: Option<String>,
    pub priority: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Scorecard Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierScorecardDashboard {
    pub total_templates: i32,
    pub total_scorecards: i32,
    pub pending_reviews: i32,
    pub average_score: String,
    pub scorecards_by_status: serde_json::Value,
    pub scorecards_by_grade: serde_json::Value,
    pub top_performers: serde_json::Value,
    pub bottom_performers: serde_json::Value,
    pub recent_reviews: serde_json::Value,
}

// ============================================================================
// Channel Revenue Management (Trade Promotion Management)
// Oracle Fusion Cloud CX > Channel Revenue Management
// ============================================================================

/// Trade Promotion header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePromotion {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub promotion_number: String,
    pub name: String,
    pub description: Option<String>,
    pub promotion_type: String,
    pub status: String,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub partner_id: Option<Uuid>,
    pub partner_number: Option<String>,
    pub partner_name: Option<String>,
    pub fund_id: Option<Uuid>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub sell_in_start_date: Option<chrono::NaiveDate>,
    pub sell_in_end_date: Option<chrono::NaiveDate>,
    pub sell_out_start_date: Option<chrono::NaiveDate>,
    pub sell_out_end_date: Option<chrono::NaiveDate>,
    pub product_category: Option<String>,
    pub product_id: Option<Uuid>,
    pub product_number: Option<String>,
    pub product_name: Option<String>,
    pub customer_segment: Option<String>,
    pub territory: Option<String>,
    pub expected_revenue: f64,
    pub planned_budget: f64,
    pub actual_spend: f64,
    pub accrued_amount: f64,
    pub claimed_amount: f64,
    pub settled_amount: f64,
    pub currency_code: String,
    pub discount_pct: Option<f64>,
    pub discount_amount: Option<f64>,
    pub volume_threshold: Option<f64>,
    pub volume_uom: Option<String>,
    pub tier_config: serde_json::Value,
    pub objectives: Option<String>,
    pub terms_and_conditions: Option<String>,
    pub approval_status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Trade Promotion Line (product-level detail)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePromotionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub promotion_id: Uuid,
    pub line_number: i32,
    pub product_id: Option<Uuid>,
    pub product_number: Option<String>,
    pub product_name: Option<String>,
    pub product_category: Option<String>,
    pub discount_type: String,
    pub discount_value: f64,
    pub unit_of_measure: Option<String>,
    pub quantity_from: Option<f64>,
    pub quantity_to: Option<f64>,
    pub planned_quantity: f64,
    pub actual_quantity: f64,
    pub planned_amount: f64,
    pub actual_amount: f64,
    pub accrual_amount: f64,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Promotion Fund (budget allocation for channel activities)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionFund {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub fund_number: String,
    pub name: String,
    pub description: Option<String>,
    pub fund_type: String,
    pub status: String,
    pub partner_id: Option<Uuid>,
    pub partner_number: Option<String>,
    pub partner_name: Option<String>,
    pub total_budget: f64,
    pub allocated_amount: f64,
    pub committed_amount: f64,
    pub utilized_amount: f64,
    pub available_amount: f64,
    pub currency_code: String,
    pub fund_year: Option<i32>,
    pub fund_quarter: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub approval_status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Trade Claim (channel partner claim against a promotion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeClaim {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub claim_number: String,
    pub promotion_id: Option<Uuid>,
    pub promotion_number: Option<String>,
    pub fund_id: Option<Uuid>,
    pub fund_number: Option<String>,
    pub claim_type: String,
    pub status: String,
    pub priority: Option<String>,
    pub partner_id: Option<Uuid>,
    pub partner_number: Option<String>,
    pub partner_name: Option<String>,
    pub claim_date: chrono::NaiveDate,
    pub sell_in_from: Option<chrono::NaiveDate>,
    pub sell_in_to: Option<chrono::NaiveDate>,
    pub product_id: Option<Uuid>,
    pub product_number: Option<String>,
    pub product_name: Option<String>,
    pub quantity: f64,
    pub unit_of_measure: Option<String>,
    pub unit_price: Option<f64>,
    pub claimed_amount: f64,
    pub approved_amount: f64,
    pub paid_amount: f64,
    pub currency_code: String,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub reference_document: Option<String>,
    pub proof_of_performance: serde_json::Value,
    pub rejection_reason: Option<String>,
    pub resolution_notes: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Trade Settlement (payment to channel partner)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSettlement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub settlement_number: String,
    pub claim_id: Option<Uuid>,
    pub claim_number: Option<String>,
    pub promotion_id: Option<Uuid>,
    pub promotion_number: Option<String>,
    pub partner_id: Option<Uuid>,
    pub partner_number: Option<String>,
    pub partner_name: Option<String>,
    pub settlement_type: String,
    pub status: String,
    pub settlement_date: chrono::NaiveDate,
    pub settlement_amount: f64,
    pub currency_code: String,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub bank_account: Option<String>,
    pub gl_account: Option<String>,
    pub cost_center: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Channel Revenue Management Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRevenueDashboard {
    pub total_promotions: i32,
    pub active_promotions: i32,
    pub total_planned_budget: f64,
    pub total_actual_spend: f64,
    pub budget_utilization_pct: f64,
    pub total_expected_revenue: f64,
    pub roi_pct: f64,
    pub total_claims: i32,
    pub pending_claims: i32,
    pub approved_claims: i32,
    pub rejected_claims: i32,
    pub total_claimed_amount: f64,
    pub total_approved_amount: f64,
    pub total_paid_amount: f64,
    pub total_funds: i32,
    pub active_funds: i32,
    pub total_fund_budget: f64,
    pub total_fund_utilized: f64,
    pub fund_utilization_pct: f64,
    pub total_settlements: i32,
    pub pending_settlements: i32,
    pub completed_settlements: i32,
    pub total_settlement_amount: f64,
    pub promotions_by_status: serde_json::Value,
    pub promotions_by_type: serde_json::Value,
    pub claims_by_status: serde_json::Value,
    pub spend_trend: serde_json::Value,
    pub top_partners: serde_json::Value,
}

// ============================================================================
// Territory Management (Oracle Fusion CX Sales > Territory Management)
// ============================================================================

/// Sales territory definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Territory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub territory_type: String, // geography, product, industry, customer, hybrid
    pub parent_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Territory member assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerritoryMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub territory_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub role: String, // owner, member, backup
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Territory routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerritoryRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub territory_id: Uuid,
    pub entity_type: String, // lead, opportunity, account, contact
    pub field_name: String,
    pub match_operator: String, // equals, contains, starts_with, ends_with, in, not_null
    pub match_value: String,
    pub priority: i32,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Territory revenue quota
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerritoryQuota {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub territory_id: Uuid,
    pub period_name: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub revenue_quota: String,
    pub actual_revenue: String,
    pub attainment_percent: String,
    pub currency_code: String,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Territory Management Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerritoryDashboard {
    pub total_territories: i32,
    pub active_territories: i32,
    pub top_level_territories: i32,
    pub total_members: i32,
    pub total_quota: String,
    pub total_actual: String,
    pub attainment_percent: String,
    pub quota_count: i32,
    pub by_type: serde_json::Value,
}

// ============================================================================
// Promotions Management (Oracle Fusion Trade Management > Trade Promotion)
// ============================================================================

/// Trade promotion definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoMgmtPromotion {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub promotion_type: String, // trade, consumer, channel, co_op
    pub status: String,        // draft, active, on_hold, completed, cancelled
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub territory_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub budget_amount: String,
    pub spent_amount: String,
    pub currency_code: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Promotional offer within a promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoMgmtOffer {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub promotion_id: Uuid,
    pub offer_type: String,       // discount, buy_get, bundle, free_item, rebate
    pub description: Option<String>,
    pub discount_type: String,    // percentage, fixed_amount, fixed_price
    pub discount_value: String,
    pub buy_quantity: Option<i32>,
    pub get_quantity: Option<i32>,
    pub minimum_purchase: Option<String>,
    pub maximum_discount: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Promotion fund allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoMgmtFund {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub promotion_id: Uuid,
    pub fund_type: String,        // marketing_development, cooperative, trade_spend, display
    pub allocated_amount: String,
    pub committed_amount: String,
    pub spent_amount: String,
    pub currency_code: String,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Promotion claim (accrual or settlement)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoMgmtClaim {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub promotion_id: Uuid,
    pub claim_number: String,
    pub claim_type: String,       // accrual, settlement, deduction, lump_sum
    pub status: String,           // submitted, under_review, approved, rejected, paid
    pub amount: String,
    pub approved_amount: Option<String>,
    pub paid_amount: Option<String>,
    pub currency_code: String,
    pub claim_date: chrono::NaiveDate,
    pub settlement_date: Option<chrono::NaiveDate>,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub description: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Promotions Management Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoMgmtDashboard {
    pub total_promotions: i32,
    pub active_promotions: i32,
    pub total_budget: String,
    pub total_spent: String,
    pub utilization_percent: String,
    pub total_claims: i32,
    pub pending_claims: i32,
    pub by_status: serde_json::Value,
    pub by_type: serde_json::Value,
}

// ============================================================================
// Loyalty Management Types (Oracle Fusion CX > Loyalty Management)
// ============================================================================

/// Loyalty Program definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoyaltyProgram {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_number: String,
    pub name: String,
    pub description: String,
    pub program_type: String,
    pub status: String,
    pub currency_code: String,
    pub points_name: String,
    pub enrollment_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub accrual_rate: f64,
    pub accrual_basis: String,
    pub minimum_accrual_amount: f64,
    pub rounding_method: String,
    pub points_expiry_days: Option<i32>,
    pub tier_qualification_period: String,
    pub auto_upgrade: bool,
    pub auto_downgrade: bool,
    pub max_points_per_member: Option<f64>,
    pub allow_point_transfer: bool,
    pub allow_redemption: bool,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Loyalty Tier definition within a program
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoyaltyTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub tier_code: String,
    pub tier_name: String,
    pub tier_level: i32,
    pub minimum_points: f64,
    pub maximum_points: Option<f64>,
    pub accrual_bonus_percentage: f64,
    pub benefits: String,
    pub color: String,
    pub icon: String,
    pub is_default: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Loyalty Member enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoyaltyMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub member_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: String,
    pub tier_id: Option<Uuid>,
    pub tier_code: String,
    pub current_points: f64,
    pub lifetime_points: f64,
    pub redeemed_points: f64,
    pub expired_points: f64,
    pub enrollment_date: chrono::NaiveDate,
    pub status: String,
    pub last_activity_date: Option<chrono::NaiveDate>,
    pub next_tier_points_remaining: Option<f64>,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Loyalty Point Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoyaltyPointTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub member_id: Uuid,
    pub transaction_number: String,
    pub transaction_type: String,
    pub points: f64,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: String,
    pub description: String,
    pub reference_amount: Option<f64>,
    pub reference_currency: String,
    pub tier_bonus_applied: f64,
    pub promo_bonus_applied: f64,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub reversal_reason: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Loyalty Reward in the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoyaltyReward {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub reward_code: String,
    pub name: String,
    pub description: String,
    pub reward_type: String,
    pub points_required: f64,
    pub cash_value: f64,
    pub currency_code: String,
    pub tier_restriction: String,
    pub quantity_available: Option<i32>,
    pub quantity_claimed: i32,
    pub max_per_member: Option<i32>,
    pub image_url: String,
    pub is_active: bool,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Loyalty Redemption (reward claim)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoyaltyRedemption {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub member_id: Uuid,
    pub reward_id: Uuid,
    pub redemption_number: String,
    pub points_spent: f64,
    pub quantity: i32,
    pub status: String,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub cancelled_reason: String,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Loyalty Management Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoyaltyDashboard {
    pub organization_id: Uuid,
    pub total_programs: i64,
    pub active_programs: i64,
    pub total_members: i64,
    pub active_members: i64,
    pub total_points_issued: f64,
    pub total_points_redeemed: f64,
    pub total_points_expired: f64,
    pub total_redemptions: i64,
    pub pending_redemptions: i64,
    pub members_by_tier: serde_json::Value,
    pub top_members: serde_json::Value,
    pub recent_transactions: serde_json::Value,
}

