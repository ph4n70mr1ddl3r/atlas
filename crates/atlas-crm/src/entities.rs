//! CRM Entity Definitions

use atlas_core::schema::SchemaBuilder;
use atlas_core::schema::WorkflowBuilder;
use atlas_shared::EntityDefinition;

/// Customer entity
pub fn customer_definition() -> EntityDefinition {
    SchemaBuilder::new("customers", "Customer")
        .plural_label("Customers")
        .table_name("crm_customers")
        .description("Customer and prospect records")
        .icon("briefcase")
        .required_string("customer_number", "Customer Number")
        .required_string("name", "Customer Name")
        .enumeration("type", "Type", vec!["individual", "company", "government", "nonprofit"])
        .enumeration("industry", "Industry", vec![
            "technology", "manufacturing", "retail", "healthcare",
            "finance", "education", "other"
        ])
        .email("email", "Email")
        .phone("phone", "Phone")
        .url("website", "Website")
        .currency("revenue", "Annual Revenue", "USD")
        .integer("employee_count", "Employee Count")
        .enumeration("status", "Status", vec!["lead", "prospect", "customer", "churned"])
        .reference("owner_id", "Account Owner", "employees")
        .address("address", "Address")
        .build()
}

/// Lead entity with qualification workflow
pub fn lead_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("lead_workflow", "new")
        .initial_state("new", "New")
        .working_state("contacted", "Contacted")
        .working_state("qualified", "Qualified")
        .working_state("proposal", "Proposal Sent")
        .final_state("won", "Won")
        .final_state("lost", "Lost")
        .transition("new", "contacted", "contact")
        .transition("contacted", "qualified", "qualify")
        .transition("qualified", "proposal", "send_proposal")
        .transition("proposal", "won", "mark_won")
        .transition("proposal", "lost", "mark_lost")
        .transition("new", "lost", "disqualify")
        .transition("contacted", "lost", "disqualify")
        .transition("qualified", "lost", "mark_lost")
        .build();

    SchemaBuilder::new("leads", "Lead")
        .plural_label("Leads")
        .table_name("crm_leads")
        .description("Sales leads and prospects")
        .icon("user-plus")
        .required_string("first_name", "First Name")
        .required_string("last_name", "Last Name")
        .email("email", "Email")
        .phone("phone", "Phone")
        .string("company", "Company")
        .string("title", "Job Title")
        .enumeration("source", "Source", vec![
            "website", "referral", "linkedin", "cold_call",
            "trade_show", "advertisement", "other"
        ])
        .enumeration("industry", "Industry", vec![
            "technology", "manufacturing", "retail", "healthcare",
            "finance", "education", "government", "other"
        ])
        .currency("estimated_value", "Estimated Value", "USD")
        .reference("assigned_to_id", "Assigned To", "employees")
        .reference("customer_id", "Converted To Customer", "customers")
        .rich_text("notes", "Notes")
        .workflow(workflow)
        .build()
}

/// Opportunity entity
pub fn opportunity_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("opportunity_workflow", "discovery")
        .initial_state("discovery", "Discovery")
        .working_state("qualification", "Qualification")
        .working_state("proposal", "Proposal")
        .working_state("negotiation", "Negotiation")
        .final_state("closed_won", "Closed Won")
        .final_state("closed_lost", "Closed Lost")
        .transition("discovery", "qualification", "qualify")
        .transition("qualification", "proposal", "propose")
        .transition("proposal", "negotiation", "negotiate")
        .transition("negotiation", "closed_won", "close_won")
        .transition("negotiation", "closed_lost", "close_lost")
        .transition("proposal", "closed_won", "close_won")
        .transition("proposal", "closed_lost", "close_lost")
        .build();

    SchemaBuilder::new("opportunities", "Opportunity")
        .plural_label("Opportunities")
        .table_name("crm_opportunities")
        .description("Sales opportunities")
        .icon("trending-up")
        .required_string("name", "Opportunity Name")
        .reference("customer_id", "Customer", "customers")
        .reference("lead_id", "Lead", "leads")
        .currency("amount", "Amount", "USD")
        .integer("probability", "Probability %")
        .date("expected_close_date", "Expected Close Date")
        .enumeration("stage", "Stage", vec![
            "discovery", "qualification", "proposal", "negotiation", "closed_won", "closed_lost"
        ])
        .reference("assigned_to_id", "Sales Rep", "employees")
        .reference("contact_id", "Primary Contact", "contacts")
        .workflow(workflow)
        .build()
}

/// Contact entity
pub fn contact_definition() -> EntityDefinition {
    SchemaBuilder::new("contacts", "Contact")
        .plural_label("Contacts")
        .table_name("crm_contacts")
        .description("Customer contacts")
        .icon("users")
        .required_string("first_name", "First Name")
        .required_string("last_name", "Last Name")
        .email("email", "Email")
        .phone("phone", "Phone")
        .string("title", "Job Title")
        .reference("customer_id", "Customer", "customers")
        .enumeration("preferred_contact", "Preferred Contact Method", vec![
            "email", "phone", "sms", "in_person"
        ])
        .boolean("is_primary", "Primary Contact")
        .boolean_default("is_active", "Active", true)
        .build()
}

/// Service Case entity with workflow
pub fn service_case_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("case_workflow", "open")
        .initial_state("open", "Open")
        .working_state("in_progress", "In Progress")
        .working_state("waiting_customer", "Waiting on Customer")
        .working_state("resolved", "Resolved")
        .final_state("closed", "Closed")
        .final_state("cancelled", "Cancelled")
        .transition("open", "in_progress", "start_work")
        .transition("in_progress", "waiting_customer", "wait_for_customer")
        .transition("waiting_customer", "in_progress", "resume")
        .transition("in_progress", "resolved", "resolve")
        .transition("resolved", "closed", "close")
        .transition("resolved", "open", "reopen")
        .transition("open", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("service_cases", "Service Case")
        .plural_label("Service Cases")
        .table_name("crm_service_cases")
        .description("Customer service cases")
        .icon("life-buoy")
        .required_string("subject", "Subject")
        .rich_text("description", "Description")
        .reference("customer_id", "Customer", "customers")
        .reference("contact_id", "Contact", "contacts")
        .enumeration("priority", "Priority", vec![
            "critical", "high", "medium", "low"
        ])
        .enumeration("category", "Category", vec![
            "billing", "technical", "general", "complaint", "feature_request"
        ])
        .reference("assigned_to_id", "Assigned To", "employees")
        .workflow(workflow)
        .build()
}
