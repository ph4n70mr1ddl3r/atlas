//! Financials Entity Definitions

use atlas_core::schema::SchemaBuilder;
use atlas_core::schema::WorkflowBuilder;
use atlas_shared::EntityDefinition;

/// Chart of Accounts entity
pub fn chart_of_accounts_definition() -> EntityDefinition {
    SchemaBuilder::new("chart_of_accounts", "Chart of Account")
        .plural_label("Chart of Accounts")
        .table_name("fin_chart_of_accounts")
        .description("Chart of accounts for the general ledger")
        .icon("book")
        .required_string("account_number", "Account Number")
        .required_string("name", "Account Name")
        .enumeration("account_type", "Account Type", vec![
            "asset", "liability", "equity", "revenue", "expense"
        ])
        .enumeration("subtype", "Subtype", vec![
            "current_asset", "fixed_asset", "current_liability",
            "long_term_liability", "operating_revenue", "other_revenue",
            "cost_of_goods", "operating_expense", "other_expense"
        ])
        .reference("parent_account_id", "Parent Account", "chart_of_accounts")
        .boolean("is_active", "Active")
        .string("description", "Description")
        .build()
}

/// Journal Entry entity with workflow
pub fn journal_entry_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("journal_entry_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted for Review")
        .final_state("posted", "Posted")
        .final_state("rejected", "Rejected")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "posted", "post")
        .transition("submitted", "rejected", "reject")
        .build();

    SchemaBuilder::new("journal_entries", "Journal Entry")
        .plural_label("Journal Entries")
        .table_name("fin_journal_entries")
        .description("General ledger journal entries")
        .icon("file-text")
        .required_string("entry_number", "Entry Number")
        .date("entry_date", "Entry Date")
        .string("description", "Description")
        .enumeration("entry_type", "Type", vec![
            "standard", "adjusting", "closing", "reversing"
        ])
        .decimal("total_debit", "Total Debit", 18, 2)
        .decimal("total_credit", "Total Credit", 18, 2)
        .reference("created_by_id", "Created By", "employees")
        .boolean("is_posted", "Is Posted")
        .workflow(workflow)
        .build()
}

/// Invoice entity with workflow
pub fn invoice_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("invoice_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("sent", "Sent")
        .working_state("partial", "Partially Paid")
        .final_state("paid", "Paid")
        .final_state("voided", "Voided")
        .transition("draft", "sent", "send")
        .transition("sent", "partial", "partial_payment")
        .transition("sent", "paid", "mark_paid")
        .transition("partial", "paid", "mark_paid")
        .transition("draft", "voided", "void")
        .transition("sent", "voided", "void")
        .build();

    SchemaBuilder::new("invoices", "Invoice")
        .plural_label("Invoices")
        .table_name("fin_invoices")
        .description("Customer invoices")
        .icon("receipt")
        .required_string("invoice_number", "Invoice Number")
        .reference("customer_id", "Customer", "customers")
        .date("invoice_date", "Invoice Date")
        .date("due_date", "Due Date")
        .enumeration("status", "Status", vec![
            "draft", "sent", "partial", "paid", "overdue", "voided"
        ])
        .currency("subtotal", "Subtotal", "USD")
        .currency("tax_amount", "Tax", "USD")
        .currency("total_amount", "Total", "USD")
        .currency("amount_paid", "Amount Paid", "USD")
        .currency("balance_due", "Balance Due", "USD")
        .enumeration("payment_terms", "Payment Terms", vec![
            "net_15", "net_30", "net_45", "net_60", "due_on_receipt"
        ])
        .rich_text("notes", "Notes")
        .workflow(workflow)
        .build()
}

/// Budget entity
pub fn budget_definition() -> EntityDefinition {
    SchemaBuilder::new("budgets", "Budget")
        .plural_label("Budgets")
        .table_name("fin_budgets")
        .description("Departmental budgets")
        .icon("bar-chart")
        .required_string("name", "Budget Name")
        .reference("department_id", "Department", "departments")
        .enumeration("period", "Period", vec![
            "monthly", "quarterly", "yearly"
        ])
        .date("start_date", "Start Date")
        .date("end_date", "End Date")
        .currency("total_budget", "Total Budget", "USD")
        .currency("allocated", "Allocated", "USD")
        .currency("spent", "Spent", "USD")
        .reference("owner_id", "Owner", "employees")
        .build()
}
