//! Financials Entity Definitions
//!
//! Oracle Fusion Cloud ERP-inspired entity definitions for all financial modules:
//! - Chart of Accounts
//! - General Ledger (Journal Entries)
//! - Invoices
//! - Budgets
//! - Expense Reports
//! - Accounts Payable (AP Invoices, AP Payments, AP Holds)
//! - Accounts Receivable (AR Transactions, AR Receipts, AR Credit Memos, AR Adjustments)
//! - Fixed Assets (Asset Categories, Asset Books, Fixed Assets, Depreciation)
//! - Cost Management (Cost Books, Cost Elements, Cost Profiles, Standard Costs, Cost Adjustments, Variances)
//! - Revenue Recognition (Policies, Contracts, Performance Obligations, Schedules, Modifications)
//! - Subledger Accounting (Methods, Derivation Rules, Journal Entries)
//! - Cash Management (Cash Positions, Forecast Templates, Cash Forecasts)
//! - Tax Management (Regimes, Jurisdictions, Rates, Determination Rules)
//! - Intercompany (Batches, Transactions, Settlements)
//! - Period Close (Calendars, Periods, Checklist)
//! - Lease Accounting (Lease Contracts, Payments, Modifications, Terminations)
//! - Bank Reconciliation (Bank Accounts, Statements, Matching)
//! - Encumbrance Management (Types, Entries, Liquidations, Carry-Forwards)
//! - Currency Management (Currencies, Exchange Rates)
//! - Multi-Book Accounting (Books, Mappings, Journal Entries)
//! - Financial Consolidation (Ledgers, Entities, Scenarios, Adjustments)

use atlas_core::schema::SchemaBuilder;
use atlas_core::schema::WorkflowBuilder;
use atlas_shared::EntityDefinition;

// ============================================================================
// General Ledger
// ============================================================================

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

/// Expense report entity with workflow
/// Oracle Fusion: Expenses > Expense Reports
pub fn expense_report_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("expense_report_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted for Approval")
        .final_state("approved", "Approved")
        .final_state("rejected", "Rejected")
        .final_state("reimbursed", "Reimbursed")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "rejected", "reject")
        .transition("approved", "reimbursed", "reimburse")
        .build();

    SchemaBuilder::new("expense_reports", "Expense Report")
        .plural_label("Expense Reports")
        .table_name("fin_expense_reports")
        .description("Employee expense reports for reimbursement")
        .icon("receipt")
        .required_string("report_number", "Report Number")
        .required_string("title", "Title")
        .string("description", "Description")
        .reference("employee_id", "Employee", "employees")
        .reference("department_id", "Department", "departments")
        .string("purpose", "Purpose")
        .reference("project_id", "Project", "projects")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "approved", "rejected", "reimbursed", "cancelled"
        ])
        .currency("total_amount", "Total Amount", "USD")
        .currency("reimbursable_amount", "Reimbursable Amount", "USD")
        .date("trip_start_date", "Trip Start Date")
        .date("trip_end_date", "Trip End Date")
        .string("cost_center", "Cost Center")
        .workflow(workflow)
        .build()
}

// ============================================================================
// Accounts Payable (Oracle Fusion: Financials > Payables)
// ============================================================================

/// AP Invoice entity with full workflow
/// Oracle Fusion: Payables > Invoices
pub fn ap_invoice_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("ap_invoice_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted for Review")
        .working_state("on_hold", "On Hold")
        .working_state("approved", "Approved")
        .final_state("paid", "Paid")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "on_hold", "apply_hold")
        .transition("submitted", "cancelled", "cancel")
        .transition("on_hold", "submitted", "release_hold")
        .transition("approved", "paid", "mark_paid")
        .transition("approved", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("ap_invoices", "AP Invoice")
        .plural_label("AP Invoices")
        .table_name("fin_ap_invoices")
        .description("Supplier invoices in accounts payable")
        .icon("file-invoice-dollar")
        .required_string("invoice_number", "Invoice Number")
        .date("invoice_date", "Invoice Date")
        .enumeration("invoice_type", "Invoice Type", vec![
            "standard", "credit_memo", "debit_memo", "prepayment", "expense_report", "po_default"
        ])
        .reference("supplier_id", "Supplier", "suppliers")
        .string("supplier_number", "Supplier Number")
        .string("supplier_name", "Supplier Name")
        .string("supplier_site", "Supplier Site")
        .string("invoice_currency_code", "Invoice Currency")
        .string("payment_currency_code", "Payment Currency")
        .string("exchange_rate", "Exchange Rate")
        .enumeration("exchange_rate_type", "Exchange Rate Type", vec![
            "daily", "spot", "corporate", "user"
        ])
        .currency("invoice_amount", "Invoice Amount", "USD")
        .currency("tax_amount", "Tax Amount", "USD")
        .currency("total_amount", "Total Amount", "USD")
        .currency("amount_paid", "Amount Paid", "USD")
        .currency("amount_remaining", "Amount Remaining", "USD")
        .enumeration("payment_terms", "Payment Terms", vec![
            "immediate", "net_10", "net_15", "net_30", "net_45", "net_60", "net_90"
        ])
        .enumeration("payment_method", "Payment Method", vec![
            "check", "electronic", "wire", "ach", "swift"
        ])
        .date("payment_due_date", "Payment Due Date")
        .date("discount_date", "Discount Date")
        .date("gl_date", "GL Date")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "on_hold", "approved", "paid", "cancelled"
        ])
        .string("po_number", "PO Number")
        .string("receipt_number", "Receipt Number")
        .string("source", "Source")
        .workflow(workflow)
        .build()
}

/// AP Invoice Line entity
/// Oracle Fusion: Payables > Invoice Lines
pub fn ap_invoice_line_definition() -> EntityDefinition {
    SchemaBuilder::new("ap_invoice_lines", "AP Invoice Line")
        .plural_label("AP Invoice Lines")
        .table_name("fin_ap_invoice_lines")
        .description("Invoice line items")
        .icon("list")
        .reference("invoice_id", "Invoice", "ap_invoices")
        .integer("line_number", "Line Number")
        .enumeration("line_type", "Line Type", vec![
            "item", "freight", "tax", "miscellaneous", "withholding"
        ])
        .string("description", "Description")
        .currency("amount", "Amount", "USD")
        .string("unit_price", "Unit Price")
        .string("quantity_invoiced", "Quantity Invoiced")
        .string("unit_of_measure", "UOM")
        .string("po_line_number", "PO Line Number")
        .string("product_code", "Product Code")
        .string("tax_code", "Tax Code")
        .currency("tax_amount", "Tax Amount", "USD")
        .build()
}

/// AP Invoice Distribution entity
/// Oracle Fusion: Payables > Invoice Distributions
pub fn ap_invoice_distribution_definition() -> EntityDefinition {
    SchemaBuilder::new("ap_invoice_distributions", "AP Invoice Distribution")
        .plural_label("AP Invoice Distributions")
        .table_name("fin_ap_invoice_distributions")
        .description("GL account distributions for invoice charges")
        .icon("sitemap")
        .reference("invoice_id", "Invoice", "ap_invoices")
        .reference("invoice_line_id", "Invoice Line", "ap_invoice_lines")
        .integer("distribution_line_number", "Distribution Line Number")
        .enumeration("distribution_type", "Distribution Type", vec![
            "charge", "tax", "withholding", "variance"
        ])
        .string("account_combination", "Account Combination")
        .string("description", "Description")
        .currency("amount", "Amount", "USD")
        .string("currency_code", "Currency Code")
        .string("gl_account", "GL Account")
        .string("cost_center", "Cost Center")
        .string("department", "Department")
        .reference("project_id", "Project", "projects")
        .string("expenditure_type", "Expenditure Type")
        .string("tax_code", "Tax Code")
        .date("accounting_date", "Accounting Date")
        .build()
}

/// AP Invoice Hold entity
/// Oracle Fusion: Payables > Invoice Holds
pub fn ap_invoice_hold_definition() -> EntityDefinition {
    SchemaBuilder::new("ap_invoice_holds", "AP Invoice Hold")
        .plural_label("AP Invoice Holds")
        .table_name("fin_ap_invoice_holds")
        .description("Holds placed on AP invoices")
        .icon("hand-paper")
        .reference("invoice_id", "Invoice", "ap_invoices")
        .enumeration("hold_type", "Hold Type", vec![
            "system", "manual", "matching", "approval", "variance", "budget"
        ])
        .string("hold_reason", "Hold Reason")
        .enumeration("hold_status", "Hold Status", vec![
            "active", "released"
        ])
        .string("release_reason", "Release Reason")
        .build()
}

/// AP Payment entity with workflow
/// Oracle Fusion: Payables > Payments
pub fn ap_payment_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("ap_payment_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted")
        .working_state("confirmed", "Confirmed")
        .final_state("cancelled", "Cancelled")
        .final_state("reversed", "Reversed")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "confirmed", "confirm")
        .transition("draft", "cancelled", "cancel")
        .transition("confirmed", "reversed", "reverse")
        .build();

    SchemaBuilder::new("ap_payments", "AP Payment")
        .plural_label("AP Payments")
        .table_name("fin_ap_payments")
        .description("Payments to suppliers")
        .icon("credit-card")
        .required_string("payment_number", "Payment Number")
        .date("payment_date", "Payment Date")
        .enumeration("payment_method", "Payment Method", vec![
            "check", "electronic", "wire", "ach", "swift"
        ])
        .string("payment_currency_code", "Payment Currency")
        .currency("payment_amount", "Payment Amount", "USD")
        .reference("supplier_id", "Supplier", "suppliers")
        .string("supplier_number", "Supplier Number")
        .string("supplier_name", "Supplier Name")
        .string("bank_account_name", "Bank Account")
        .string("payment_document", "Payment Document")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "confirmed", "cancelled", "reversed"
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Accounts Receivable (Oracle Fusion: Financials > Receivables)
// ============================================================================

/// AR Transaction (Customer Invoice) entity with workflow
/// Oracle Fusion: Receivables > Transactions
pub fn ar_transaction_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("ar_transaction_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("complete", "Complete")
        .working_state("open", "Open")
        .final_state("closed", "Closed")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "complete", "complete")
        .transition("complete", "open", "post")
        .transition("open", "closed", "close")
        .transition("draft", "cancelled", "cancel")
        .transition("complete", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("ar_transactions", "AR Transaction")
        .plural_label("AR Transactions")
        .table_name("fin_ar_transactions")
        .description("Customer receivable transactions (invoices, debit memos, credit memos)")
        .icon("file-invoice")
        .required_string("transaction_number", "Transaction Number")
        .enumeration("transaction_type", "Transaction Type", vec![
            "invoice", "debit_memo", "credit_memo", "chargeback", "deposit", "guarantee"
        ])
        .date("transaction_date", "Transaction Date")
        .date("gl_date", "GL Date")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .string("bill_to_site", "Bill-To Site")
        .string("currency_code", "Currency Code")
        .string("exchange_rate", "Exchange Rate")
        .enumeration("exchange_rate_type", "Exchange Rate Type", vec![
            "daily", "spot", "corporate", "user"
        ])
        .currency("entered_amount", "Entered Amount", "USD")
        .currency("tax_amount", "Tax Amount", "USD")
        .currency("total_amount", "Total Amount", "USD")
        .currency("amount_due_original", "Original Amount Due", "USD")
        .currency("amount_due_remaining", "Remaining Amount Due", "USD")
        .currency("amount_applied", "Amount Applied", "USD")
        .currency("amount_adjusted", "Amount Adjusted", "USD")
        .enumeration("payment_terms", "Payment Terms", vec![
            "immediate", "net_10", "net_15", "net_30", "net_45", "net_60", "net_90", "due_on_receipt"
        ])
        .date("due_date", "Due Date")
        .date("discount_due_date", "Discount Due Date")
        .string("reference_number", "Reference Number")
        .string("purchase_order", "Purchase Order")
        .string("sales_rep", "Sales Representative")
        .enumeration("status", "Status", vec![
            "draft", "complete", "open", "closed", "cancelled"
        ])
        .string("receipt_method", "Receipt Method")
        .rich_text("notes", "Notes")
        .workflow(workflow)
        .build()
}

/// AR Transaction Line entity
/// Oracle Fusion: Receivables > Transaction Lines
pub fn ar_transaction_line_definition() -> EntityDefinition {
    SchemaBuilder::new("ar_transaction_lines", "AR Transaction Line")
        .plural_label("AR Transaction Lines")
        .table_name("fin_ar_transaction_lines")
        .description("Line items on AR transactions")
        .icon("list")
        .reference("transaction_id", "Transaction", "ar_transactions")
        .integer("line_number", "Line Number")
        .string("description", "Description")
        .enumeration("line_type", "Line Type", vec![
            "line", "tax", "freight", "charges"
        ])
        .string("item_code", "Item Code")
        .string("item_description", "Item Description")
        .string("unit_of_measure", "UOM")
        .decimal("quantity", "Quantity", 18, 4)
        .currency("unit_price", "Unit Price", "USD")
        .currency("line_amount", "Line Amount", "USD")
        .currency("tax_amount", "Tax Amount", "USD")
        .string("tax_code", "Tax Code")
        .string("revenue_account", "Revenue Account")
        .string("tax_account", "Tax Account")
        .reference("sales_order_line_id", "Sales Order Line", "sales_order_lines")
        .build()
}

/// AR Receipt entity with workflow
/// Oracle Fusion: Receivables > Receipts
pub fn ar_receipt_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("ar_receipt_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("confirmed", "Confirmed")
        .working_state("applied", "Applied")
        .final_state("deposited", "Deposited")
        .final_state("reversed", "Reversed")
        .transition("draft", "confirmed", "confirm")
        .transition("confirmed", "applied", "apply")
        .transition("applied", "deposited", "deposit")
        .transition("confirmed", "reversed", "reverse")
        .transition("applied", "reversed", "reverse")
        .build();

    SchemaBuilder::new("ar_receipts", "AR Receipt")
        .plural_label("AR Receipts")
        .table_name("fin_ar_receipts")
        .description("Customer payment receipts")
        .icon("money-check-alt")
        .required_string("receipt_number", "Receipt Number")
        .date("receipt_date", "Receipt Date")
        .enumeration("receipt_type", "Receipt Type", vec![
            "cash", "check", "credit_card", "wire_transfer", "ach", "other"
        ])
        .enumeration("receipt_method", "Receipt Method", vec![
            "automatic_receipt", "manual_receipt", "quick_cash", "miscellaneous"
        ])
        .currency("amount", "Amount", "USD")
        .string("currency_code", "Currency Code")
        .string("exchange_rate", "Exchange Rate")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .string("reference_number", "Reference Number")
        .string("bank_account_name", "Bank Account")
        .string("check_number", "Check Number")
        .date("maturity_date", "Maturity Date")
        .enumeration("status", "Status", vec![
            "draft", "confirmed", "applied", "deposited", "reversed"
        ])
        .string("applied_transaction_number", "Applied Transaction")
        .rich_text("notes", "Notes")
        .workflow(workflow)
        .build()
}

/// AR Credit Memo entity with workflow
/// Oracle Fusion: Receivables > Credit Memos
pub fn ar_credit_memo_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("ar_credit_memo_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted for Approval")
        .working_state("approved", "Approved")
        .working_state("applied", "Applied")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "cancelled", "cancel")
        .transition("approved", "applied", "apply")
        .build();

    SchemaBuilder::new("ar_credit_memos", "AR Credit Memo")
        .plural_label("AR Credit Memos")
        .table_name("fin_ar_credit_memos")
        .description("Credit memos issued to customers")
        .icon("minus-circle")
        .required_string("credit_memo_number", "Credit Memo Number")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .reference("transaction_id", "Original Transaction", "ar_transactions")
        .string("transaction_number", "Transaction Number")
        .date("credit_memo_date", "Credit Memo Date")
        .date("gl_date", "GL Date")
        .enumeration("reason_code", "Reason", vec![
            "return", "pricing_error", "damaged", "wrong_item", "discount", "other"
        ])
        .string("reason_description", "Reason Description")
        .currency("amount", "Amount", "USD")
        .currency("tax_amount", "Tax Amount", "USD")
        .currency("total_amount", "Total Amount", "USD")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "approved", "applied", "cancelled"
        ])
        .rich_text("notes", "Notes")
        .workflow(workflow)
        .build()
}

/// AR Adjustment entity
/// Oracle Fusion: Receivables > Adjustments
pub fn ar_adjustment_definition() -> EntityDefinition {
    SchemaBuilder::new("ar_adjustments", "AR Adjustment")
        .plural_label("AR Adjustments")
        .table_name("fin_ar_adjustments")
        .description("Adjustments to customer receivable balances")
        .icon("sliders-h")
        .required_string("adjustment_number", "Adjustment Number")
        .reference("transaction_id", "Transaction", "ar_transactions")
        .string("transaction_number", "Transaction Number")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .date("adjustment_date", "Adjustment Date")
        .date("gl_date", "GL Date")
        .enumeration("adjustment_type", "Adjustment Type", vec![
            "write_off", "write_off_bad_debt", "small_balance_write_off",
            "increase", "decrease", "transfer", "revaluation"
        ])
        .currency("amount", "Amount", "USD")
        .string("receivable_account", "Receivable Account")
        .string("adjustment_account", "Adjustment Account")
        .string("reason_code", "Reason Code")
        .string("reason_description", "Reason Description")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "approved", "rejected", "posted"
        ])
        .reference("approved_by", "Approved By", "employees")
        .rich_text("notes", "Notes")
        .build()
}

// ============================================================================
// Fixed Assets (Oracle Fusion: Financials > Fixed Assets)
// ============================================================================

/// Asset Category entity
/// Oracle Fusion: Fixed Assets > Asset Categories
pub fn asset_category_definition() -> EntityDefinition {
    SchemaBuilder::new("asset_categories", "Asset Category")
        .plural_label("Asset Categories")
        .table_name("fin_asset_categories")
        .description("Categories for classifying fixed assets")
        .icon("folder")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .string("description", "Description")
        .enumeration("default_depreciation_method", "Default Depreciation Method", vec![
            "straight_line", "declining_balance", "sum_of_years_digits"
        ])
        .integer("default_useful_life_months", "Default Useful Life (Months)")
        .decimal("default_salvage_value_percent", "Default Salvage %", 5, 2)
        .string("default_asset_account_code", "Asset Account")
        .string("default_accum_depr_account_code", "Accum Depr Account")
        .string("default_depr_expense_account_code", "Depr Expense Account")
        .string("default_gain_loss_account_code", "Gain/Loss Account")
        .boolean("is_active", "Active")
        .build()
}

/// Asset Book entity
/// Oracle Fusion: Fixed Assets > Asset Books
pub fn asset_book_definition() -> EntityDefinition {
    SchemaBuilder::new("asset_books", "Asset Book")
        .plural_label("Asset Books")
        .table_name("fin_asset_books")
        .description("Depreciation books (corporate or tax)")
        .icon("book-open")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .string("description", "Description")
        .enumeration("book_type", "Book Type", vec![
            "corporate", "tax"
        ])
        .boolean("auto_depreciation", "Auto Depreciation")
        .enumeration("depreciation_calendar", "Depreciation Calendar", vec![
            "monthly", "quarterly", "yearly"
        ])
        .boolean("is_active", "Active")
        .build()
}

/// Fixed Asset entity with lifecycle workflow
/// Oracle Fusion: Fixed Assets > Assets
pub fn fixed_asset_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("fixed_asset_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("acquired", "Acquired")
        .working_state("in_service", "In Service")
        .working_state("under_construction", "Under Construction")
        .final_state("disposed", "Disposed")
        .final_state("retired", "Retired")
        .final_state("transferred", "Transferred")
        .transition("draft", "acquired", "acquire")
        .transition("acquired", "in_service", "place_in_service")
        .transition("draft", "in_service", "place_in_service")
        .transition("in_service", "disposed", "dispose")
        .transition("in_service", "retired", "retire")
        .transition("in_service", "transferred", "transfer")
        .build();

    SchemaBuilder::new("fixed_assets", "Fixed Asset")
        .plural_label("Fixed Assets")
        .table_name("fin_fixed_assets")
        .description("Fixed asset registration with depreciation tracking")
        .icon("building")
        .required_string("asset_number", "Asset Number")
        .required_string("asset_name", "Asset Name")
        .string("description", "Description")
        .reference("category_id", "Category", "asset_categories")
        .string("category_code", "Category Code")
        .reference("book_id", "Book", "asset_books")
        .string("book_code", "Book Code")
        .enumeration("asset_type", "Asset Type", vec![
            "tangible", "intangible", "leased", "cipc"
        ])
        .enumeration("status", "Status", vec![
            "draft", "acquired", "in_service", "under_construction", "disposed", "retired", "transferred"
        ])
        .currency("original_cost", "Original Cost", "USD")
        .currency("current_cost", "Current Cost", "USD")
        .currency("salvage_value", "Salvage Value", "USD")
        .string("salvage_value_percent", "Salvage %")
        .enumeration("depreciation_method", "Depreciation Method", vec![
            "straight_line", "declining_balance", "sum_of_years_digits"
        ])
        .integer("useful_life_months", "Useful Life (Months)")
        .string("declining_balance_rate", "Declining Balance Rate")
        .currency("depreciable_basis", "Depreciable Basis", "USD")
        .currency("accumulated_depreciation", "Accumulated Depreciation", "USD")
        .currency("net_book_value", "Net Book Value", "USD")
        .integer("periods_depreciated", "Periods Depreciated")
        .currency("depreciation_per_period", "Depreciation Per Period", "USD")
        .date("acquisition_date", "Acquisition Date")
        .date("in_service_date", "In Service Date")
        .date("disposal_date", "Disposal Date")
        .date("retirement_date", "Retirement Date")
        .string("location", "Location")
        .reference("department_id", "Department", "departments")
        .string("department_name", "Department Name")
        .reference("custodian_id", "Custodian", "employees")
        .string("custodian_name", "Custodian Name")
        .string("serial_number", "Serial Number")
        .string("tag_number", "Tag Number")
        .string("manufacturer", "Manufacturer")
        .string("model", "Model")
        .date("warranty_expiry", "Warranty Expiry")
        .string("insurance_policy_number", "Insurance Policy")
        .date("insurance_expiry", "Insurance Expiry")
        .string("lease_number", "Lease Number")
        .date("lease_expiry", "Lease Expiry")
        .string("asset_account_code", "Asset Account")
        .string("accum_depr_account_code", "Accum Depr Account")
        .string("depr_expense_account_code", "Depr Expense Account")
        .string("gain_loss_account_code", "Gain/Loss Account")
        .workflow(workflow)
        .build()
}

/// Asset Transfer entity with workflow
/// Oracle Fusion: Fixed Assets > Asset Transfers
pub fn asset_transfer_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("asset_transfer_workflow", "pending")
        .initial_state("pending", "Pending")
        .final_state("approved", "Approved")
        .final_state("rejected", "Rejected")
        .final_state("completed", "Completed")
        .transition("pending", "approved", "approve")
        .transition("pending", "rejected", "reject")
        .transition("approved", "completed", "complete")
        .build();

    SchemaBuilder::new("asset_transfers", "Asset Transfer")
        .plural_label("Asset Transfers")
        .table_name("fin_asset_transfers")
        .description("Asset transfer requests between departments/locations")
        .icon("exchange-alt")
        .required_string("transfer_number", "Transfer Number")
        .reference("asset_id", "Asset", "fixed_assets")
        .string("from_department_name", "From Department")
        .string("from_location", "From Location")
        .string("from_custodian_name", "From Custodian")
        .string("to_department_name", "To Department")
        .string("to_location", "To Location")
        .string("to_custodian_name", "To Custodian")
        .date("transfer_date", "Transfer Date")
        .string("reason", "Reason")
        .enumeration("status", "Status", vec![
            "pending", "approved", "rejected", "completed"
        ])
        .workflow(workflow)
        .build()
}

/// Asset Retirement entity with workflow
/// Oracle Fusion: Fixed Assets > Asset Retirements
pub fn asset_retirement_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("asset_retirement_workflow", "pending")
        .initial_state("pending", "Pending")
        .final_state("approved", "Approved")
        .final_state("completed", "Completed")
        .final_state("cancelled", "Cancelled")
        .transition("pending", "approved", "approve")
        .transition("approved", "completed", "complete")
        .transition("pending", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("asset_retirements", "Asset Retirement")
        .plural_label("Asset Retirements")
        .table_name("fin_asset_retirements")
        .description("Asset retirement/disposal with gain/loss calculation")
        .icon("archive")
        .required_string("retirement_number", "Retirement Number")
        .reference("asset_id", "Asset", "fixed_assets")
        .enumeration("retirement_type", "Retirement Type", vec![
            "sale", "scrap", "donation", "write_off", "casualty"
        ])
        .date("retirement_date", "Retirement Date")
        .currency("proceeds", "Proceeds", "USD")
        .currency("removal_cost", "Removal Cost", "USD")
        .currency("net_book_value", "Net Book Value", "USD")
        .currency("accumulated_depreciation", "Accumulated Depreciation", "USD")
        .currency("gain_loss_amount", "Gain/Loss Amount", "USD")
        .enumeration("gain_loss_type", "Gain/Loss Type", vec![
            "gain", "loss", "none"
        ])
        .string("reference_number", "Reference Number")
        .string("buyer_name", "Buyer Name")
        .enumeration("status", "Status", vec![
            "pending", "approved", "completed", "cancelled"
        ])
        .rich_text("notes", "Notes")
        .workflow(workflow)
        .build()
}

// ============================================================================
// Cost Management (Oracle Fusion: Cost Management > Cost Accounting)
// ============================================================================

/// Cost Book entity
/// Oracle Fusion: Cost Management > Cost Books
pub fn cost_book_definition() -> EntityDefinition {
    SchemaBuilder::new("cost_books", "Cost Book")
        .plural_label("Cost Books")
        .table_name("fin_cost_books")
        .description("Cost books defining costing methods for inventory valuation")
        .icon("book")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .string("description", "Description")
        .enumeration("costing_method", "Costing Method", vec![
            "standard", "average", "fifo", "lifo"
        ])
        .string("currency_code", "Currency Code")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Cost Element entity
/// Oracle Fusion: Cost Management > Cost Elements
pub fn cost_element_definition() -> EntityDefinition {
    SchemaBuilder::new("cost_elements", "Cost Element")
        .plural_label("Cost Elements")
        .table_name("fin_cost_elements")
        .description("Cost elements (material, labor, overhead, etc.)")
        .icon("puzzle-piece")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .string("description", "Description")
        .enumeration("element_type", "Element Type", vec![
            "material", "labor", "overhead", "subcontracting", "expense"
        ])
        .reference("cost_book_id", "Cost Book", "cost_books")
        .decimal("default_rate", "Default Rate", 18, 6)
        .string("rate_uom", "Rate UOM")
        .boolean("is_active", "Active")
        .build()
}

/// Cost Profile entity
/// Oracle Fusion: Cost Management > Cost Profiles
pub fn cost_profile_definition() -> EntityDefinition {
    SchemaBuilder::new("cost_profiles", "Cost Profile")
        .plural_label("Cost Profiles")
        .table_name("fin_cost_profiles")
        .description("Costing configuration for specific items")
        .icon("cog")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .string("description", "Description")
        .reference("cost_book_id", "Cost Book", "cost_books")
        .reference("item_id", "Item", "items")
        .string("item_name", "Item Name")
        .enumeration("cost_type", "Cost Type", vec![
            "standard", "average", "fifo", "lifo"
        ])
        .boolean("lot_level_costing", "Lot Level Costing")
        .boolean("include_landed_costs", "Include Landed Costs")
        .enumeration("overhead_absorption_method", "Overhead Absorption", vec![
            "rate", "amount", "percentage"
        ])
        .build()
}

/// Standard Cost entity
/// Oracle Fusion: Cost Management > Standard Costs
pub fn standard_cost_definition() -> EntityDefinition {
    SchemaBuilder::new("standard_costs", "Standard Cost")
        .plural_label("Standard Costs")
        .table_name("fin_standard_costs")
        .description("Standard cost rates per item and cost element")
        .icon("dollar-sign")
        .reference("cost_book_id", "Cost Book", "cost_books")
        .reference("cost_profile_id", "Cost Profile", "cost_profiles")
        .reference("cost_element_id", "Cost Element", "cost_elements")
        .reference("item_id", "Item", "items")
        .string("item_name", "Item Name")
        .decimal("standard_cost", "Standard Cost", 18, 6)
        .string("currency_code", "Currency")
        .date("effective_date", "Effective Date")
        .enumeration("status", "Status", vec![
            "pending", "active", "superseded"
        ])
        .build()
}

/// Cost Adjustment entity with workflow
/// Oracle Fusion: Cost Management > Cost Adjustments
pub fn cost_adjustment_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("cost_adjustment_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted")
        .working_state("approved", "Approved")
        .final_state("rejected", "Rejected")
        .final_state("posted", "Posted")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "rejected", "reject")
        .transition("approved", "posted", "post")
        .build();

    SchemaBuilder::new("cost_adjustments", "Cost Adjustment")
        .plural_label("Cost Adjustments")
        .table_name("fin_cost_adjustments")
        .description("Cost adjustment requests with approval workflow")
        .icon("tools")
        .required_string("adjustment_number", "Adjustment Number")
        .reference("cost_book_id", "Cost Book", "cost_books")
        .enumeration("adjustment_type", "Adjustment Type", vec![
            "standard_cost_update", "cost_correction", "revaluation", "overhead_adjustment"
        ])
        .string("description", "Description")
        .string("reason", "Reason")
        .string("currency_code", "Currency")
        .currency("total_adjustment_amount", "Total Adjustment Amount", "USD")
        .date("effective_date", "Effective Date")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "approved", "rejected", "posted"
        ])
        .workflow(workflow)
        .build()
}

/// Cost Adjustment Line entity
/// Oracle Fusion: Cost Management > Cost Adjustment Lines
pub fn cost_adjustment_line_definition() -> EntityDefinition {
    SchemaBuilder::new("cost_adjustment_lines", "Cost Adjustment Line")
        .plural_label("Cost Adjustment Lines")
        .table_name("fin_cost_adjustment_lines")
        .description("Individual cost adjustment lines per item/element")
        .icon("list")
        .reference("adjustment_id", "Adjustment", "cost_adjustments")
        .integer("line_number", "Line Number")
        .reference("item_id", "Item", "items")
        .string("item_name", "Item Name")
        .reference("cost_element_id", "Cost Element", "cost_elements")
        .decimal("old_cost", "Old Cost", 18, 6)
        .decimal("new_cost", "New Cost", 18, 6)
        .decimal("adjustment_amount", "Adjustment Amount", 18, 6)
        .string("currency_code", "Currency")
        .date("effective_date", "Effective Date")
        .build()
}

/// Cost Variance entity
/// Oracle Fusion: Cost Management > Variance Analysis
pub fn cost_variance_definition() -> EntityDefinition {
    SchemaBuilder::new("cost_variances", "Cost Variance")
        .plural_label("Cost Variances")
        .table_name("fin_cost_variances")
        .description("Variance analysis between standard and actual costs")
        .icon("chart-bar")
        .reference("cost_book_id", "Cost Book", "cost_books")
        .enumeration("variance_type", "Variance Type", vec![
            "purchase_price", "routing", "overhead", "rate", "usage", "mix"
        ])
        .date("variance_date", "Variance Date")
        .reference("item_id", "Item", "items")
        .string("item_name", "Item Name")
        .reference("cost_element_id", "Cost Element", "cost_elements")
        .enumeration("source_type", "Source Type", vec![
            "purchase_order", "work_order", "transfer_order"
        ])
        .string("source_number", "Source Number")
        .decimal("standard_cost", "Standard Cost", 18, 6)
        .decimal("actual_cost", "Actual Cost", 18, 6)
        .decimal("variance_amount", "Variance Amount", 18, 6)
        .decimal("variance_percent", "Variance %", 8, 4)
        .decimal("quantity", "Quantity", 18, 4)
        .string("currency_code", "Currency")
        .string("accounting_period", "Accounting Period")
        .boolean("is_analyzed", "Analyzed")
        .string("analysis_notes", "Analysis Notes")
        .build()
}

// ============================================================================
// Revenue Recognition (Oracle Fusion: Financials > Revenue Management)
// ASC 606 / IFRS 15 Five-Step Model
// ============================================================================

/// Revenue Policy entity
/// Oracle Fusion: Revenue Management > Revenue Policies
pub fn revenue_policy_definition() -> EntityDefinition {
    SchemaBuilder::new("revenue_policies", "Revenue Policy")
        .plural_label("Revenue Policies")
        .table_name("fin_revenue_policies")
        .description("Revenue recognition policies defining recognition methods and allocation bases")
        .icon("file-contract")
        .required_string("code", "Policy Code")
        .required_string("name", "Policy Name")
        .string("description", "Description")
        .enumeration("recognition_method", "Recognition Method", vec![
            "over_time", "point_in_time",
        ])
        .enumeration("over_time_method", "Over-Time Method", vec![
            "output", "input", "straight_line",
        ])
        .enumeration("allocation_basis", "Allocation Basis", vec![
            "standalone_selling_price", "residual", "equal",
        ])
        .string("currency_code", "Currency Code")
        .boolean("allow_negative_revenue", "Allow Negative Revenue")
        .boolean("auto_recognize", "Auto Recognize")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Revenue Contract entity with workflow
/// Oracle Fusion: Revenue Management > Revenue Contracts
pub fn revenue_contract_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("revenue_contract_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("active", "Active")
        .working_state("modified", "Modified")
        .final_state("completed", "Completed")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "active", "activate")
        .transition("active", "modified", "modify")
        .transition("active", "completed", "complete")
        .transition("active", "cancelled", "cancel")
        .transition("modified", "active", "reactivate")
        .transition("modified", "completed", "complete")
        .build();

    SchemaBuilder::new("revenue_contracts", "Revenue Contract")
        .plural_label("Revenue Contracts")
        .table_name("fin_revenue_contracts")
        .description("Customer contracts for revenue recognition (ASC 606)")
        .icon("handshake")
        .required_string("contract_number", "Contract Number")
        .reference("customer_id", "Customer", "customers")
        .string("customer_name", "Customer Name")
        .date("contract_start_date", "Contract Start Date")
        .date("contract_end_date", "Contract End Date")
        .currency("total_transaction_price", "Transaction Price", "USD")
        .currency("recognized_amount", "Recognized Amount", "USD")
        .currency("deferred_amount", "Deferred Amount", "USD")
        .string("currency_code", "Currency Code")
        .enumeration("status", "Status", vec![
            "draft", "active", "completed", "cancelled", "modified",
        ])
        .string("description", "Description")
        .reference("policy_id", "Revenue Policy", "revenue_policies")
        .workflow(workflow)
        .build()
}

/// Performance Obligation entity
/// Oracle Fusion: Revenue Management > Performance Obligations
pub fn performance_obligation_definition() -> EntityDefinition {
    SchemaBuilder::new("performance_obligations", "Performance Obligation")
        .plural_label("Performance Obligations")
        .table_name("fin_performance_obligations")
        .description("Distinct goods or services within a revenue contract")
        .icon("list-check")
        .reference("contract_id", "Contract", "revenue_contracts")
        .string("contract_number", "Contract Number")
        .required_string("description", "Description")
        .enumeration("satisfaction_method", "Satisfaction Method", vec![
            "over_time", "point_in_time",
        ])
        .enumeration("over_time_method", "Over-Time Method", vec![
            "output", "input", "straight_line",
        ])
        .currency("standalone_selling_price", "Standalone Selling Price", "USD")
        .currency("allocated_amount", "Allocated Amount", "USD")
        .currency("recognized_amount", "Recognized Amount", "USD")
        .date("obligation_start_date", "Start Date")
        .date("obligation_end_date", "End Date")
        .enumeration("status", "Status", vec![
            "pending", "in_progress", "satisfied", "partially_satisfied", "cancelled",
        ])
        .build()
}

/// Revenue Schedule Line entity
/// Oracle Fusion: Revenue Management > Revenue Schedules
pub fn revenue_schedule_line_definition() -> EntityDefinition {
    SchemaBuilder::new("revenue_schedule_lines", "Revenue Schedule Line")
        .plural_label("Revenue Schedule Lines")
        .table_name("fin_revenue_schedule_lines")
        .description("Planned revenue recognition events for performance obligations")
        .icon("calendar-alt")
        .reference("contract_id", "Contract", "revenue_contracts")
        .reference("obligation_id", "Obligation", "performance_obligations")
        .date("recognition_date", "Recognition Date")
        .currency("amount", "Amount", "USD")
        .currency("cumulative_recognized", "Cumulative Recognized", "USD")
        .enumeration("status", "Status", vec![
            "planned", "recognized", "reversed", "cancelled",
        ])
        .string("accounting_period", "Accounting Period")
        .string("revenue_account", "Revenue Account")
        .string("deferred_account", "Deferred Account")
        .build()
}

/// Revenue Contract Modification entity with workflow
/// Oracle Fusion: Revenue Management > Contract Modifications
pub fn revenue_modification_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("revenue_modification_workflow", "draft")
        .initial_state("draft", "Draft")
        .final_state("active", "Active")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "active", "activate")
        .transition("draft", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("revenue_modifications", "Revenue Modification")
        .plural_label("Revenue Modifications")
        .table_name("fin_revenue_modifications")
        .description("Contract modifications (price change, scope change, term extension)")
        .icon("edit")
        .required_string("modification_number", "Modification Number")
        .reference("contract_id", "Contract", "revenue_contracts")
        .enumeration("modification_type", "Type", vec![
            "price_change", "scope_change", "term_extension",
            "termination", "add_obligation", "remove_obligation",
        ])
        .date("effective_date", "Effective Date")
        .currency("original_transaction_price", "Original Price", "USD")
        .currency("new_transaction_price", "New Price", "USD")
        .currency("price_difference", "Price Difference", "USD")
        .string("reason", "Reason")
        .enumeration("status", "Status", vec![
            "draft", "active", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Subledger Accounting (Oracle Fusion: Financials > General Ledger > SLA)
// ============================================================================

/// Accounting Method entity
/// Oracle Fusion: Subledger Accounting > Accounting Methods
pub fn accounting_method_definition() -> EntityDefinition {
    SchemaBuilder::new("accounting_methods", "Accounting Method")
        .plural_label("Accounting Methods")
        .table_name("fin_accounting_methods")
        .description("Accounting methods defining how subledger events are converted to journal entries")
        .icon("cogs")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .string("description", "Description")
        .enumeration("application", "Application", vec![
            "payables", "receivables", "expenses", "assets", "projects", "general",
        ])
        .string("transaction_type", "Transaction Type")
        .enumeration("event_class", "Event Class", vec![
            "create", "update", "cancel", "reverse",
        ])
        .boolean("auto_accounting", "Auto Accounting")
        .boolean("allow_manual_entries", "Allow Manual Entries")
        .boolean("apply_rounding", "Apply Rounding")
        .string("rounding_account_code", "Rounding Account")
        .string("rounding_threshold", "Rounding Threshold")
        .boolean("require_balancing", "Require Balancing")
        .string("intercompany_balancing_account", "IC Balancing Account")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .build()
}

/// Accounting Derivation Rule entity
/// Oracle Fusion: Subledger Accounting > Derivation Rules
pub fn accounting_derivation_rule_definition() -> EntityDefinition {
    SchemaBuilder::new("accounting_derivation_rules", "Derivation Rule")
        .plural_label("Accounting Derivation Rules")
        .table_name("fin_accounting_derivation_rules")
        .description("Rules for deriving account combinations from source values")
        .icon("project-diagram")
        .reference("method_id", "Accounting Method", "accounting_methods")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .enumeration("derivation_type", "Derivation Type", vec![
            "constant", "lookup", "formula",
        ])
        .string("source_field", "Source Field")
        .string("target_segment", "Target Segment")
        .string("constant_value", "Constant Value")
        .string("lookup_table", "Lookup Table")
        .string("formula_expression", "Formula Expression")
        .integer("priority", "Priority")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Subledger Journal Entry entity with workflow
/// Oracle Fusion: Subledger Accounting > Journal Entries
pub fn subledger_journal_entry_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("sla_journal_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("accounted", "Accounted")
        .working_state("posted", "Posted")
        .working_state("transferred", "Transferred to GL")
        .final_state("reversed", "Reversed")
        .final_state("error", "Error")
        .transition("draft", "accounted", "account")
        .transition("accounted", "posted", "post")
        .transition("posted", "transferred", "transfer")
        .transition("accounted", "reversed", "reverse")
        .transition("draft", "error", "error")
        .build();

    SchemaBuilder::new("subledger_journal_entries", "SLA Journal Entry")
        .plural_label("SLA Journal Entries")
        .table_name("fin_sla_journal_entries")
        .description("Subledger journal entries generated from accounting events")
        .icon("file-invoice-dollar")
        .required_string("entry_number", "Entry Number")
        .reference("method_id", "Accounting Method", "accounting_methods")
        .enumeration("application", "Application", vec![
            "payables", "receivables", "expenses", "assets", "projects", "general",
        ])
        .string("transaction_type", "Transaction Type")
        .string("event_type", "Event Type")
        .string("source_entity", "Source Entity")
        .string("source_id", "Source ID")
        .date("accounting_date", "Accounting Date")
        .date("gl_date", "GL Date")
        .enumeration("entry_status", "Status", vec![
            "draft", "accounted", "posted", "transferred", "reversed", "error",
        ])
        .currency("total_debit", "Total Debit", "USD")
        .currency("total_credit", "Total Credit", "USD")
        .string("currency_code", "Currency Code")
        .string("gl_transfer_status", "GL Transfer Status")
        .string("error_message", "Error Message")
        .workflow(workflow)
        .build()
}

/// Subledger Journal Line entity
/// Oracle Fusion: Subledger Accounting > Journal Lines
pub fn subledger_journal_line_definition() -> EntityDefinition {
    SchemaBuilder::new("subledger_journal_lines", "SLA Journal Line")
        .plural_label("SLA Journal Lines")
        .table_name("fin_sla_journal_lines")
        .description("Individual debit/credit lines within a subledger journal entry")
        .icon("list")
        .reference("entry_id", "Journal Entry", "subledger_journal_entries")
        .integer("line_number", "Line Number")
        .enumeration("line_type", "Line Type", vec![
            "debit", "credit", "tax", "discount", "rounding",
        ])
        .string("account_code", "Account Code")
        .string("account_name", "Account Name")
        .string("account_combination", "Account Combination")
        .currency("debit_amount", "Debit Amount", "USD")
        .currency("credit_amount", "Credit Amount", "USD")
        .string("currency_code", "Currency Code")
        .string("entered_dr", "Entered Debit")
        .string("entered_cr", "Entered Credit")
        .string("description", "Description")
        .string("tax_code", "Tax Code")
        .string("source_type", "Source Type")
        .build()
}

// ============================================================================
// Cash Management (Oracle Fusion: Financials > Treasury > Cash Management)
// ============================================================================

/// Cash Position entity
/// Oracle Fusion: Cash Management > Cash Positions
pub fn cash_position_definition() -> EntityDefinition {
    SchemaBuilder::new("cash_positions", "Cash Position")
        .plural_label("Cash Positions")
        .table_name("fin_cash_positions")
        .description("Real-time cash position across bank accounts")
        .icon("money-bill-wave")
        .reference("bank_account_id", "Bank Account", "bank_accounts")
        .string("account_number", "Account Number")
        .string("account_name", "Account Name")
        .string("currency_code", "Currency Code")
        .currency("book_balance", "Book Balance", "USD")
        .currency("available_balance", "Available Balance", "USD")
        .currency("float_amount", "Float Amount", "USD")
        .currency("one_day_float", "1-Day Float", "USD")
        .currency("two_day_float", "2-Day Float", "USD")
        .date("position_date", "Position Date")
        .currency("average_balance", "Average Balance", "USD")
        .currency("prior_day_balance", "Prior Day Balance", "USD")
        .currency("projected_inflows", "Projected Inflows", "USD")
        .currency("projected_outflows", "Projected Outflows", "USD")
        .currency("projected_net", "Projected Net", "USD")
        .boolean("is_reconciled", "Reconciled")
        .build()
}

/// Cash Forecast Template entity
/// Oracle Fusion: Cash Management > Forecast Templates
pub fn cash_forecast_template_definition() -> EntityDefinition {
    SchemaBuilder::new("cash_forecast_templates", "Cash Forecast Template")
        .plural_label("Cash Forecast Templates")
        .table_name("fin_cash_forecast_templates")
        .description("Configurable templates for cash flow forecasting")
        .icon("chart-line")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .string("description", "Description")
        .enumeration("bucket_type", "Bucket Type", vec![
            "daily", "weekly", "monthly",
        ])
        .integer("number_of_buckets", "Number of Buckets")
        .string("currency_code", "Currency Code")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Cash Forecast Source entity
/// Oracle Fusion: Cash Management > Forecast Sources
pub fn cash_forecast_source_definition() -> EntityDefinition {
    SchemaBuilder::new("cash_forecast_sources", "Cash Forecast Source")
        .plural_label("Cash Forecast Sources")
        .table_name("fin_cash_forecast_sources")
        .description("Sources included in cash flow forecasts")
        .icon("database")
        .reference("template_id", "Template", "cash_forecast_templates")
        .enumeration("source_type", "Source Type", vec![
            "accounts_payable", "accounts_receivable", "payroll",
            "purchasing", "manual", "budget", "intercompany",
            "fixed_assets", "tax", "other",
        ])
        .enumeration("cash_flow_direction", "Cash Flow Direction", vec![
            "inflow", "outflow", "both",
        ])
        .boolean("is_included", "Included")
        .integer("display_order", "Display Order")
        .build()
}

/// Cash Forecast entity with workflow
/// Oracle Fusion: Cash Management > Cash Forecasts
pub fn cash_forecast_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("cash_forecast_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("generated", "Generated")
        .working_state("approved", "Approved")
        .final_state("superseded", "Superseded")
        .transition("draft", "generated", "generate")
        .transition("generated", "approved", "approve")
        .transition("approved", "superseded", "supersede")
        .build();

    SchemaBuilder::new("cash_forecasts", "Cash Forecast")
        .plural_label("Cash Forecasts")
        .table_name("fin_cash_forecasts")
        .description("Cash flow forecast with time-bucketed projections")
        .icon("chart-area")
        .required_string("forecast_number", "Forecast Number")
        .reference("template_id", "Template", "cash_forecast_templates")
        .date("forecast_start_date", "Start Date")
        .date("forecast_end_date", "End Date")
        .date("as_of_date", "As-Of Date")
        .string("currency_code", "Currency Code")
        .currency("total_inflows", "Total Inflows", "USD")
        .currency("total_outflows", "Total Outflows", "USD")
        .currency("net_cash_flow", "Net Cash Flow", "USD")
        .currency("opening_balance", "Opening Balance", "USD")
        .currency("closing_balance", "Closing Balance", "USD")
        .enumeration("status", "Status", vec![
            "draft", "generated", "approved", "superseded",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Tax Management (Oracle Fusion: Tax > Tax Configuration)
// ============================================================================

/// Tax Regime entity
/// Oracle Fusion: Tax > Tax Regimes
pub fn tax_regime_definition() -> EntityDefinition {
    SchemaBuilder::new("tax_regimes", "Tax Regime")
        .plural_label("Tax Regimes")
        .table_name("fin_tax_regimes")
        .description("Tax regime configuration defining tax type and rounding rules")
        .icon("landmark")
        .required_string("code", "Regime Code")
        .required_string("name", "Regime Name")
        .string("description", "Description")
        .enumeration("tax_type", "Tax Type", vec![
            "sales_tax", "vat", "gst", "withholding", "excise", "customs",
        ])
        .boolean("default_inclusive", "Tax Inclusive")
        .boolean("allows_recovery", "Allows Recovery")
        .enumeration("rounding_rule", "Rounding Rule", vec![
            "nearest", "up", "down", "none",
        ])
        .integer("rounding_precision", "Rounding Precision")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Tax Jurisdiction entity
/// Oracle Fusion: Tax > Tax Jurisdictions
pub fn tax_jurisdiction_definition() -> EntityDefinition {
    SchemaBuilder::new("tax_jurisdictions", "Tax Jurisdiction")
        .plural_label("Tax Jurisdictions")
        .table_name("fin_tax_jurisdictions")
        .description("Geographic tax jurisdictions")
        .icon("globe")
        .reference("regime_id", "Tax Regime", "tax_regimes")
        .required_string("code", "Jurisdiction Code")
        .required_string("name", "Jurisdiction Name")
        .enumeration("geographic_level", "Geographic Level", vec![
            "country", "state", "county", "city", "region",
        ])
        .string("country_code", "Country Code")
        .string("state_code", "State/Province Code")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Tax Rate entity with workflow
/// Oracle Fusion: Tax > Tax Rates
pub fn tax_rate_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("tax_rate_workflow", "draft")
        .initial_state("draft", "Draft")
        .final_state("active", "Active")
        .final_state("inactive", "Inactive")
        .transition("draft", "active", "activate")
        .transition("active", "inactive", "deactivate")
        .build();

    SchemaBuilder::new("tax_rates", "Tax Rate")
        .plural_label("Tax Rates")
        .table_name("fin_tax_rates")
        .description("Tax rates with percentage or amount calculations")
        .icon("percent")
        .reference("regime_id", "Tax Regime", "tax_regimes")
        .reference("jurisdiction_id", "Jurisdiction", "tax_jurisdictions")
        .required_string("code", "Rate Code")
        .required_string("name", "Rate Name")
        .enumeration("rate_type", "Rate Type", vec![
            "standard", "reduced", "zero", "exempt",
        ])
        .decimal("percentage_rate", "Percentage Rate", 10, 6)
        .currency("flat_amount", "Flat Amount", "USD")
        .string("tax_account_code", "Tax Account")
        .string("recovery_account_code", "Recovery Account")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .enumeration("status", "Status", vec![
            "draft", "active", "inactive",
        ])
        .workflow(workflow)
        .build()
}

/// Tax Determination Rule entity
/// Oracle Fusion: Tax > Tax Determination Rules
pub fn tax_determination_rule_definition() -> EntityDefinition {
    SchemaBuilder::new("tax_determination_rules", "Tax Determination Rule")
        .plural_label("Tax Determination Rules")
        .table_name("fin_tax_determination_rules")
        .description("Rules for automatically determining applicable taxes")
        .icon("gavel")
        .reference("regime_id", "Tax Regime", "tax_regimes")
        .required_string("code", "Rule Code")
        .required_string("name", "Rule Name")
        .string("description", "Description")
        .string("product_type_condition", "Product Type Condition")
        .string("ship_from_condition", "Ship-From Condition")
        .string("ship_to_condition", "Ship-To Condition")
        .string("usage_condition", "Usage Condition")
        .reference("tax_rate_id", "Tax Rate", "tax_rates")
        .integer("priority", "Priority")
        .boolean("is_active", "Active")
        .build()
}

// ============================================================================
// Intercompany (Oracle Fusion: Intercompany > Intercompany Transactions)
// ============================================================================

/// Intercompany Batch entity with workflow
/// Oracle Fusion: Intercompany > Batches
pub fn intercompany_batch_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("ic_batch_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted")
        .working_state("approved", "Approved")
        .final_state("posted", "Posted")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "cancelled", "cancel")
        .transition("approved", "posted", "post")
        .build();

    SchemaBuilder::new("intercompany_batches", "IC Batch")
        .plural_label("Intercompany Batches")
        .table_name("fin_ic_batches")
        .description("Batches of intercompany transactions between legal entities")
        .icon("exchange-alt")
        .required_string("batch_number", "Batch Number")
        .string("description", "Description")
        .reference("from_entity_id", "From Entity", "organizations")
        .string("from_entity_name", "From Entity Name")
        .reference("to_entity_id", "To Entity", "organizations")
        .string("to_entity_name", "To Entity Name")
        .string("currency_code", "Currency Code")
        .date("accounting_date", "Accounting Date")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "approved", "posted", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

/// Intercompany Transaction entity
/// Oracle Fusion: Intercompany > Transactions
pub fn intercompany_transaction_definition() -> EntityDefinition {
    SchemaBuilder::new("intercompany_transactions", "IC Transaction")
        .plural_label("Intercompany Transactions")
        .table_name("fin_ic_transactions")
        .description("Individual intercompany transactions within a batch")
        .icon("arrows-alt-h")
        .reference("batch_id", "Batch", "intercompany_batches")
        .enumeration("transaction_type", "Transaction Type", vec![
            "invoice", "journal_entry", "payment", "charge", "allocation",
        ])
        .date("transaction_date", "Transaction Date")
        .string("description", "Description")
        .currency("amount", "Amount", "USD")
        .string("currency_code", "Currency Code")
        .string("from_account", "From Account")
        .string("to_account", "To Account")
        .enumeration("settlement_method", "Settlement Method", vec![
            "cash", "netting", "offset",
        ])
        .enumeration("status", "Status", vec![
            "draft", "approved", "posted", "settled", "cancelled",
        ])
        .string("reference_number", "Reference Number")
        .build()
}

/// Intercompany Settlement entity
/// Oracle Fusion: Intercompany > Settlements
pub fn intercompany_settlement_definition() -> EntityDefinition {
    SchemaBuilder::new("intercompany_settlements", "IC Settlement")
        .plural_label("Intercompany Settlements")
        .table_name("fin_ic_settlements")
        .description("Settlement of intercompany balances")
        .icon("balance-scale")
        .reference("from_entity_id", "From Entity", "organizations")
        .string("from_entity_name", "From Entity Name")
        .reference("to_entity_id", "To Entity", "organizations")
        .string("to_entity_name", "To Entity Name")
        .enumeration("settlement_method", "Settlement Method", vec![
            "cash", "netting", "offset",
        ])
        .currency("settlement_amount", "Settlement Amount", "USD")
        .string("currency_code", "Currency Code")
        .date("settlement_date", "Settlement Date")
        .date("gl_date", "GL Date")
        .enumeration("status", "Status", vec![
            "pending", "processed", "reversed",
        ])
        .string("reference_number", "Reference Number")
        .build()
}

// ============================================================================
// Period Close (Oracle Fusion: General Ledger > Period Close)
// ============================================================================

/// Accounting Calendar entity
/// Oracle Fusion: General Ledger > Period Close > Calendars
pub fn accounting_calendar_definition() -> EntityDefinition {
    SchemaBuilder::new("accounting_calendars", "Accounting Calendar")
        .plural_label("Accounting Calendars")
        .table_name("fin_accounting_calendars")
        .description("Fiscal calendars defining accounting periods")
        .icon("calendar")
        .required_string("name", "Calendar Name")
        .string("description", "Description")
        .enumeration("calendar_type", "Calendar Type", vec![
            "monthly", "quarterly", "445", "544", "weekly",
        ])
        .integer("fiscal_year_start_month", "FY Start Month")
        .integer("periods_per_year", "Periods Per Year")
        .boolean("has_adjusting_period", "Adjusting Period")
        .integer("current_fiscal_year", "Current Fiscal Year")
        .boolean("is_active", "Active")
        .build()
}

/// Accounting Period entity
/// Oracle Fusion: General Ledger > Period Close > Periods
pub fn accounting_period_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("period_status_workflow", "future")
        .initial_state("future", "Future")
        .working_state("not_opened", "Not Opened")
        .working_state("open", "Open")
        .working_state("pending_close", "Pending Close")
        .final_state("closed", "Closed")
        .final_state("permanently_closed", "Permanently Closed")
        .transition("future", "not_opened", "unlock")
        .transition("not_opened", "open", "open")
        .transition("open", "pending_close", "pending_close")
        .transition("pending_close", "closed", "close")
        .transition("closed", "permanently_closed", "permanent_close")
        .build();

    SchemaBuilder::new("accounting_periods", "Accounting Period")
        .plural_label("Accounting Periods")
        .table_name("fin_accounting_periods")
        .description("Individual accounting periods within a fiscal calendar")
        .icon("calendar-day")
        .reference("calendar_id", "Calendar", "accounting_calendars")
        .required_string("period_name", "Period Name")
        .integer("fiscal_year", "Fiscal Year")
        .integer("period_number", "Period Number")
        .enumeration("period_type", "Period Type", vec![
            "adjusting", "normal", "quarter", "year",
        ])
        .date("start_date", "Start Date")
        .date("end_date", "End Date")
        .enumeration("status", "Status", vec![
            "future", "not_opened", "open", "pending_close", "closed", "permanently_closed",
        ])
        .date("opened_date", "Opened Date")
        .date("closed_date", "Closed Date")
        .string("closed_by", "Closed By")
        .workflow(workflow)
        .build()
}

/// Period Close Checklist Item entity
/// Oracle Fusion: General Ledger > Period Close > Checklist
pub fn period_close_checklist_definition() -> EntityDefinition {
    SchemaBuilder::new("period_close_checklist", "Close Checklist Item")
        .plural_label("Period Close Checklist")
        .table_name("fin_period_close_checklist")
        .description("Checklist items for period close process tracking")
        .icon("tasks")
        .reference("period_id", "Period", "accounting_periods")
        .reference("calendar_id", "Calendar", "accounting_calendars")
        .required_string("task_name", "Task Name")
        .string("description", "Description")
        .enumeration("subledger", "Subledger", vec![
            "gl", "ap", "ar", "fa", "po",
        ])
        .integer("sequence", "Sequence")
        .enumeration("status", "Status", vec![
            "not_started", "in_progress", "completed", "skipped",
        ])
        .string("assigned_to", "Assigned To")
        .date("due_date", "Due Date")
        .date("completed_date", "Completed Date")
        .string("completed_by", "Completed By")
        .build()
}

// ============================================================================
// Lease Accounting (Oracle Fusion: Lease Management, ASC 842 / IFRS 16)
// ============================================================================

/// Lease Contract entity with workflow
/// Oracle Fusion: Lease Management > Lease Contracts
pub fn lease_contract_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("lease_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("active", "Active")
        .working_state("modified", "Modified")
        .working_state("impaired", "Impaired")
        .final_state("terminated", "Terminated")
        .final_state("expired", "Expired")
        .transition("draft", "active", "activate")
        .transition("active", "modified", "modify")
        .transition("active", "impaired", "impair")
        .transition("active", "terminated", "terminate")
        .transition("active", "expired", "expire")
        .transition("modified", "active", "reactivate")
        .transition("impaired", "active", "recover")
        .build();

    SchemaBuilder::new("lease_contracts", "Lease Contract")
        .plural_label("Lease Contracts")
        .table_name("fin_lease_contracts")
        .description("Lease contracts managed under ASC 842 / IFRS 16")
        .icon("file-signature")
        .required_string("lease_number", "Lease Number")
        .string("title", "Title")
        .string("description", "Description")
        .enumeration("classification", "Classification", vec![
            "operating", "finance",
        ])
        .reference("lessor_id", "Lessor", "suppliers")
        .string("lessor_name", "Lessor Name")
        .string("asset_description", "Asset Description")
        .string("location", "Location")
        .reference("department_id", "Department", "departments")
        .string("department_name", "Department Name")
        .date("commencement_date", "Commencement Date")
        .date("end_date", "End Date")
        .integer("lease_term_months", "Term (Months)")
        .boolean("purchase_option_exists", "Purchase Option")
        .boolean("purchase_option_likely", "Purchase Option Likely")
        .boolean("renewal_option_exists", "Renewal Option")
        .integer("renewal_option_months", "Renewal Months")
        .boolean("renewal_option_likely", "Renewal Likely")
        .enumeration("payment_frequency", "Payment Frequency", vec![
            "monthly", "quarterly", "annually",
        ])
        .currency("annual_lease_payment", "Annual Payment", "USD")
        .currency("total_lease_payments", "Total Payments", "USD")
        .decimal("discount_rate", "Discount Rate (IBR)", 10, 8)
        .currency("right_of_use_asset", "ROU Asset", "USD")
        .currency("lease_liability", "Lease Liability", "USD")
        .string("currency_code", "Currency Code")
        .enumeration("status", "Status", vec![
            "draft", "active", "modified", "impaired", "terminated", "expired",
        ])
        .workflow(workflow)
        .build()
}

/// Lease Payment entity
/// Oracle Fusion: Lease Management > Lease Payments
pub fn lease_payment_definition() -> EntityDefinition {
    SchemaBuilder::new("lease_payments", "Lease Payment")
        .plural_label("Lease Payments")
        .table_name("fin_lease_payments")
        .description("Scheduled lease payments with amortization breakdown")
        .icon("credit-card")
        .reference("lease_id", "Lease", "lease_contracts")
        .integer("payment_number", "Payment Number")
        .date("payment_date", "Payment Date")
        .currency("payment_amount", "Payment Amount", "USD")
        .currency("interest_expense", "Interest Expense", "USD")
        .currency("principal_reduction", "Principal Reduction", "USD")
        .currency("lease_liability_balance", "Liability Balance", "USD")
        .currency("rou_asset_balance", "ROU Asset Balance", "USD")
        .currency("accumulated_amortization", "Accum Amortization", "USD")
        .enumeration("status", "Status", vec![
            "scheduled", "paid", "overdue", "cancelled",
        ])
        .build()
}

/// Lease Modification entity with workflow
/// Oracle Fusion: Lease Management > Lease Modifications
pub fn lease_modification_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("lease_modification_workflow", "pending")
        .initial_state("pending", "Pending")
        .final_state("approved", "Approved")
        .final_state("rejected", "Rejected")
        .transition("pending", "approved", "approve")
        .transition("pending", "rejected", "reject")
        .build();

    SchemaBuilder::new("lease_modifications", "Lease Modification")
        .plural_label("Lease Modifications")
        .table_name("fin_lease_modifications")
        .description("Modifications to existing lease contracts")
        .icon("edit")
        .required_string("modification_number", "Modification Number")
        .reference("lease_id", "Lease", "lease_contracts")
        .enumeration("modification_type", "Type", vec![
            "term_extension", "scope_change", "payment_change", "rate_change", "reclassification",
        ])
        .date("effective_date", "Effective Date")
        .currency("original_rou_asset", "Original ROU Asset", "USD")
        .currency("revised_rou_asset", "Revised ROU Asset", "USD")
        .currency("adjustment_amount", "Adjustment Amount", "USD")
        .string("reason", "Reason")
        .enumeration("status", "Status", vec![
            "pending", "approved", "rejected",
        ])
        .workflow(workflow)
        .build()
}

/// Lease Termination entity with workflow
/// Oracle Fusion: Lease Management > Lease Terminations
pub fn lease_termination_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("lease_termination_workflow", "pending")
        .initial_state("pending", "Pending")
        .final_state("approved", "Approved")
        .final_state("cancelled", "Cancelled")
        .transition("pending", "approved", "approve")
        .transition("pending", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("lease_terminations", "Lease Termination")
        .plural_label("Lease Terminations")
        .table_name("fin_lease_terminations")
        .description("Early or end-of-term lease termination processing")
        .icon("ban")
        .required_string("termination_number", "Termination Number")
        .reference("lease_id", "Lease", "lease_contracts")
        .enumeration("termination_type", "Type", vec![
            "early", "end_of_term", "mutual_agreement", "default",
        ])
        .date("termination_date", "Termination Date")
        .currency("termination_penalty", "Termination Penalty", "USD")
        .currency("rou_asset_remaining", "Remaining ROU Asset", "USD")
        .currency("liability_remaining", "Remaining Liability", "USD")
        .currency("gain_loss_amount", "Gain/Loss Amount", "USD")
        .string("reason", "Reason")
        .enumeration("status", "Status", vec![
            "pending", "approved", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Bank Reconciliation (Oracle Fusion: Cash Management > Reconciliation)
// ============================================================================

/// Bank Account entity
/// Oracle Fusion: Cash Management > Bank Accounts
pub fn bank_account_definition() -> EntityDefinition {
    SchemaBuilder::new("bank_accounts", "Bank Account")
        .plural_label("Bank Accounts")
        .table_name("fin_bank_accounts")
        .description("Bank accounts for cash management and reconciliation")
        .icon("university")
        .required_string("account_number", "Account Number")
        .required_string("account_name", "Account Name")
        .required_string("bank_name", "Bank Name")
        .string("bank_code", "Bank Code")
        .string("branch_name", "Branch Name")
        .string("branch_code", "Branch Code")
        .string("gl_account_code", "GL Account")
        .string("currency_code", "Currency Code")
        .enumeration("account_type", "Account Type", vec![
            "checking", "savings", "money_market", "escrow",
        ])
        .boolean("is_active", "Active")
        .build()
}

/// Bank Statement entity
/// Oracle Fusion: Cash Management > Bank Statements
pub fn bank_statement_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("bank_statement_workflow", "imported")
        .initial_state("imported", "Imported")
        .working_state("in_review", "In Review")
        .final_state("reconciled", "Reconciled")
        .final_state("error", "Error")
        .transition("imported", "in_review", "review")
        .transition("in_review", "reconciled", "reconcile")
        .transition("imported", "error", "error")
        .build();

    SchemaBuilder::new("bank_statements", "Bank Statement")
        .plural_label("Bank Statements")
        .table_name("fin_bank_statements")
        .description("Imported bank statements for reconciliation")
        .icon("file-alt")
        .reference("bank_account_id", "Bank Account", "bank_accounts")
        .required_string("statement_number", "Statement Number")
        .date("statement_date", "Statement Date")
        .date("from_date", "From Date")
        .date("to_date", "To Date")
        .currency("opening_balance", "Opening Balance", "USD")
        .currency("closing_balance", "Closing Balance", "USD")
        .currency("total_deposits", "Total Deposits", "USD")
        .currency("total_withdrawals", "Total Withdrawals", "USD")
        .integer("number_of_lines", "Number of Lines")
        .enumeration("status", "Status", vec![
            "imported", "in_review", "reconciled", "error",
        ])
        .string("currency_code", "Currency Code")
        .workflow(workflow)
        .build()
}

/// Bank Statement Line entity
/// Oracle Fusion: Cash Management > Statement Lines
pub fn bank_statement_line_definition() -> EntityDefinition {
    SchemaBuilder::new("bank_statement_lines", "Statement Line")
        .plural_label("Bank Statement Lines")
        .table_name("fin_bank_statement_lines")
        .description("Individual transaction lines from a bank statement")
        .icon("list")
        .reference("statement_id", "Statement", "bank_statements")
        .integer("line_number", "Line Number")
        .date("transaction_date", "Transaction Date")
        .string("transaction_type", "Transaction Type")
        .currency("amount", "Amount", "USD")
        .string("description", "Description")
        .string("reference_number", "Reference Number")
        .string("check_number", "Check Number")
        .enumeration("reconciliation_status", "Recon Status", vec![
            "unmatched", "matched", "reconciled",
        ])
        .build()
}

/// Reconciliation Match entity
/// Oracle Fusion: Cash Management > Reconciliation Matching
pub fn reconciliation_match_definition() -> EntityDefinition {
    SchemaBuilder::new("reconciliation_matches", "Reconciliation Match")
        .plural_label("Reconciliation Matches")
        .table_name("fin_reconciliation_matches")
        .description("Matches between bank statement lines and system transactions")
        .icon("link")
        .reference("statement_id", "Statement", "bank_statements")
        .reference("statement_line_id", "Statement Line", "bank_statement_lines")
        .reference("system_transaction_id", "System Transaction", "ar_receipts")
        .enumeration("match_type", "Match Type", vec![
            "auto_one_to_one", "auto_one_to_many", "auto_many_to_one", "manual",
        ])
        .enumeration("match_method", "Match Method", vec![
            "exact_amount", "reference_number", "date_range", "manual",
        ])
        .currency("matched_amount", "Matched Amount", "USD")
        .date("match_date", "Match Date")
        .string("matched_by", "Matched By")
        .build()
}

// ============================================================================
// Encumbrance Management (Oracle Fusion: Financials > GL > Encumbrances)
// ============================================================================

/// Encumbrance Type entity
/// Oracle Fusion: General Ledger > Encumbrance Types
pub fn encumbrance_type_definition() -> EntityDefinition {
    SchemaBuilder::new("encumbrance_types", "Encumbrance Type")
        .plural_label("Encumbrance Types")
        .table_name("fin_encumbrance_types")
        .description("Types of financial encumbrances (commitments, obligations)")
        .icon("tag")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .enumeration("category", "Category", vec![
            "commitment", "obligation", "preliminary",
        ])
        .string("description", "Description")
        .string("dr_account_code", "Debit Account")
        .string("cr_account_code", "Credit Account")
        .boolean("enable_carry_forward", "Enable Carry-Forward")
        .boolean("is_active", "Active")
        .build()
}

/// Encumbrance Entry entity with workflow
/// Oracle Fusion: General Ledger > Encumbrance Entries
pub fn encumbrance_entry_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("encumbrance_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("active", "Active")
        .working_state("partially_liquidated", "Partially Liquidated")
        .final_state("fully_liquidated", "Fully Liquidated")
        .final_state("cancelled", "Cancelled")
        .final_state("expired", "Expired")
        .transition("draft", "active", "activate")
        .transition("active", "partially_liquidated", "partial_liquidate")
        .transition("active", "fully_liquidated", "full_liquidate")
        .transition("partially_liquidated", "fully_liquidated", "full_liquidate")
        .transition("active", "cancelled", "cancel")
        .transition("active", "expired", "expire")
        .build();

    SchemaBuilder::new("encumbrance_entries", "Encumbrance Entry")
        .plural_label("Encumbrance Entries")
        .table_name("fin_encumbrance_entries")
        .description("Encumbrance entries tracking financial commitments")
        .icon("lock")
        .required_string("entry_number", "Entry Number")
        .reference("encumbrance_type_id", "Encumbrance Type", "encumbrance_types")
        .string("source_entity", "Source Entity")
        .string("source_id", "Source ID")
        .string("description", "Description")
        .currency("encumbered_amount", "Encumbered Amount", "USD")
        .currency("liquidated_amount", "Liquidated Amount", "USD")
        .currency("remaining_amount", "Remaining Amount", "USD")
        .date("encumbrance_date", "Encumbrance Date")
        .date("liquidation_date", "Liquidation Date")
        .date("expiry_date", "Expiry Date")
        .enumeration("status", "Status", vec![
            "draft", "active", "partially_liquidated", "fully_liquidated", "cancelled", "expired",
        ])
        .workflow(workflow)
        .build()
}

/// Encumbrance Liquidation entity
/// Oracle Fusion: General Ledger > Encumbrance Liquidations
pub fn encumbrance_liquidation_definition() -> EntityDefinition {
    SchemaBuilder::new("encumbrance_liquidations", "Encumbrance Liquidation")
        .plural_label("Encumbrance Liquidations")
        .table_name("fin_encumbrance_liquidations")
        .description("Liquidation of encumbrance entries against actual expenditure")
        .icon("unlock")
        .reference("encumbrance_entry_id", "Encumbrance Entry", "encumbrance_entries")
        .enumeration("liquidation_type", "Liquidation Type", vec![
            "full", "partial", "final",
        ])
        .currency("amount", "Liquidation Amount", "USD")
        .date("liquidation_date", "Liquidation Date")
        .string("reference_number", "Reference Number")
        .enumeration("status", "Status", vec![
            "draft", "processed", "reversed",
        ])
        .build()
}

/// Encumbrance Carry-Forward entity
/// Oracle Fusion: General Ledger > Encumbrance Carry-Forward
pub fn encumbrance_carry_forward_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("encumbrance_carry_forward_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("processing", "Processing")
        .final_state("completed", "Completed")
        .final_state("reversed", "Reversed")
        .transition("draft", "processing", "process")
        .transition("processing", "completed", "complete")
        .transition("completed", "reversed", "reverse")
        .build();

    SchemaBuilder::new("encumbrance_carry_forwards", "Encumbrance Carry-Forward")
        .plural_label("Encumbrance Carry-Forwards")
        .table_name("fin_encumbrance_carry_forwards")
        .description("Year-end carry-forward of open encumbrances")
        .icon("fast-forward")
        .required_string("batch_number", "Batch Number")
        .integer("from_fiscal_year", "From Fiscal Year")
        .integer("to_fiscal_year", "To Fiscal Year")
        .currency("total_amount", "Total Carry-Forward Amount", "USD")
        .integer("entry_count", "Number of Entries")
        .enumeration("status", "Status", vec![
            "draft", "processing", "completed", "reversed",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Currency Management (Oracle Fusion: General Ledger > Currency Rates)
// ============================================================================

/// Currency Definition entity
/// Oracle Fusion: General Ledger > Currency Definitions
pub fn currency_definition_entity() -> EntityDefinition {
    SchemaBuilder::new("currencies", "Currency")
        .plural_label("Currencies")
        .table_name("fin_currencies")
        .description("Currency definitions with precision settings")
        .icon("coins")
        .required_string("code", "Currency Code")
        .required_string("name", "Currency Name")
        .string("symbol", "Symbol")
        .integer("precision", "Decimal Precision")
        .boolean("is_base_currency", "Base Currency")
        .build()
}

/// Exchange Rate entity
/// Oracle Fusion: General Ledger > Currency Rates
pub fn exchange_rate_definition() -> EntityDefinition {
    SchemaBuilder::new("exchange_rates", "Exchange Rate")
        .plural_label("Exchange Rates")
        .table_name("fin_exchange_rates")
        .description("Currency exchange rates by type and date")
        .icon("sync-alt")
        .string("from_currency_code", "From Currency")
        .string("to_currency_code", "To Currency")
        .enumeration("rate_type", "Rate Type", vec![
            "daily", "spot", "corporate", "period_average", "period_end", "user", "fixed",
        ])
        .decimal("rate", "Exchange Rate", 18, 10)
        .date("effective_date", "Effective Date")
        .date("inverse_rate", "Inverse Rate")
        .boolean("is_active", "Active")
        .build()
}

// ============================================================================
// Multi-Book Accounting (Oracle Fusion: General Ledger > Multi-Book)
// ============================================================================

/// Accounting Book entity
/// Oracle Fusion: General Ledger > Multi-Book > Books
pub fn accounting_book_definition() -> EntityDefinition {
    SchemaBuilder::new("accounting_books", "Accounting Book")
        .plural_label("Accounting Books")
        .table_name("fin_accounting_books")
        .description("Accounting books for multi-GAAP compliance")
        .icon("book")
        .required_string("code", "Book Code")
        .required_string("name", "Book Name")
        .string("description", "Description")
        .enumeration("book_type", "Book Type", vec![
            "primary", "secondary",
        ])
        .string("chart_of_accounts_id", "Chart of Accounts")
        .string("currency_code", "Currency Code")
        .string("accounting_calendar", "Accounting Calendar")
        .enumeration("status", "Status", vec![
            "draft", "active", "inactive", "suspended",
        ])
        .boolean("auto_propagation", "Auto Propagation")
        .boolean("is_active", "Active")
        .build()
}

/// Account Mapping entity
/// Oracle Fusion: General Ledger > Multi-Book > Account Mappings
pub fn account_mapping_definition() -> EntityDefinition {
    SchemaBuilder::new("account_mappings", "Account Mapping")
        .plural_label("Account Mappings")
        .table_name("fin_account_mappings")
        .description("Mappings between account structures across accounting books")
        .icon("map-signs")
        .reference("source_book_id", "Source Book", "accounting_books")
        .reference("target_book_id", "Target Book", "accounting_books")
        .required_string("source_account_code", "Source Account")
        .required_string("target_account_code", "Target Account")
        .enumeration("mapping_level", "Mapping Level", vec![
            "journal", "subledger",
        ])
        .string("description", "Description")
        .boolean("is_active", "Active")
        .build()
}

/// Book Journal Entry entity
/// Oracle Fusion: General Ledger > Multi-Book > Journal Entries
pub fn book_journal_entry_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("book_journal_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("posted", "Posted")
        .working_state("propagated", "Propagated")
        .final_state("reversed", "Reversed")
        .transition("draft", "posted", "post")
        .transition("posted", "propagated", "propagate")
        .transition("posted", "reversed", "reverse")
        .build();

    SchemaBuilder::new("book_journal_entries", "Book Journal Entry")
        .plural_label("Book Journal Entries")
        .table_name("fin_book_journal_entries")
        .description("Journal entries in a specific accounting book")
        .icon("file-alt")
        .reference("book_id", "Accounting Book", "accounting_books")
        .required_string("entry_number", "Entry Number")
        .reference("source_entry_id", "Source Entry", "journal_entries")
        .string("source_book_code", "Source Book")
        .date("entry_date", "Entry Date")
        .date("gl_date", "GL Date")
        .currency("total_debit", "Total Debit", "USD")
        .currency("total_credit", "Total Credit", "USD")
        .string("currency_code", "Currency Code")
        .enumeration("status", "Status", vec![
            "draft", "posted", "propagated", "reversed",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Financial Consolidation (Oracle Fusion: General Ledger > Consolidation)
// ============================================================================

/// Consolidation Ledger entity
/// Oracle Fusion: General Ledger > Consolidation > Ledgers
pub fn consolidation_ledger_definition() -> EntityDefinition {
    SchemaBuilder::new("consolidation_ledgers", "Consolidation Ledger")
        .plural_label("Consolidation Ledgers")
        .table_name("fin_consolidation_ledgers")
        .description("Consolidation ledgers for multi-entity financial reporting")
        .icon("layer-group")
        .required_string("code", "Code")
        .required_string("name", "Name")
        .string("description", "Description")
        .string("base_currency_code", "Base Currency")
        .enumeration("translation_method", "Translation Method", vec![
            "current_rate", "temporal", "weighted_average",
        ])
        .enumeration("equity_elimination_method", "Equity Elimination", vec![
            "full", "proportional", "equity_method",
        ])
        .enumeration("status", "Status", vec![
            "created", "active", "inactive",
        ])
        .boolean("is_active", "Active")
        .build()
}

/// Consolidation Entity entity
/// Oracle Fusion: General Ledger > Consolidation > Entities
pub fn consolidation_entity_definition() -> EntityDefinition {
    SchemaBuilder::new("consolidation_entities", "Consolidation Entity")
        .plural_label("Consolidation Entities")
        .table_name("fin_consolidation_entities")
        .description("Legal entities included in consolidation")
        .icon("building")
        .reference("ledger_id", "Consolidation Ledger", "consolidation_ledgers")
        .reference("entity_id", "Legal Entity", "organizations")
        .string("entity_name", "Entity Name")
        .string("entity_currency_code", "Entity Currency")
        .enumeration("consolidation_method", "Consolidation Method", vec![
            "full", "proportional", "equity_method",
        ])
        .decimal("ownership_percentage", "Ownership %", 8, 4)
        .boolean("include_in_consolidation", "Include in Consolidation")
        .enumeration("status", "Status", vec![
            "active", "removed",
        ])
        .build()
}

/// Consolidation Scenario entity with workflow
/// Oracle Fusion: General Ledger > Consolidation > Scenarios
pub fn consolidation_scenario_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("consolidation_scenario_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("in_progress", "In Progress")
        .working_state("pending_review", "Pending Review")
        .working_state("approved", "Approved")
        .final_state("posted", "Posted")
        .final_state("reversed", "Reversed")
        .transition("draft", "in_progress", "start")
        .transition("in_progress", "pending_review", "submit_review")
        .transition("pending_review", "approved", "approve")
        .transition("approved", "posted", "post")
        .transition("posted", "reversed", "reverse")
        .build();

    SchemaBuilder::new("consolidation_scenarios", "Consolidation Scenario")
        .plural_label("Consolidation Scenarios")
        .table_name("fin_consolidation_scenarios")
        .description("Consolidation run scenarios for financial statement preparation")
        .icon("sitemap")
        .reference("ledger_id", "Consolidation Ledger", "consolidation_ledgers")
        .required_string("scenario_name", "Scenario Name")
        .integer("fiscal_year", "Fiscal Year")
        .integer("period_number", "Period Number")
        .date("period_start_date", "Period Start")
        .date("period_end_date", "Period End")
        .string("base_currency_code", "Base Currency")
        .integer("entity_count", "Entity Count")
        .enumeration("status", "Status", vec![
            "draft", "in_progress", "pending_review", "approved", "posted", "reversed",
        ])
        .workflow(workflow)
        .build()
}

/// Consolidation Adjustment entity with workflow
/// Oracle Fusion: General Ledger > Consolidation > Adjustments
pub fn consolidation_adjustment_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("consolidation_adjustment_workflow", "draft")
        .initial_state("draft", "Draft")
        .final_state("approved", "Approved")
        .final_state("posted", "Posted")
        .transition("draft", "approved", "approve")
        .transition("approved", "posted", "post")
        .build();

    SchemaBuilder::new("consolidation_adjustments", "Consolidation Adjustment")
        .plural_label("Consolidation Adjustments")
        .table_name("fin_consolidation_adjustments")
        .description("Manual adjustments during consolidation (eliminations, reclassifications)")
        .icon("sliders-h")
        .reference("scenario_id", "Scenario", "consolidation_scenarios")
        .required_string("adjustment_number", "Adjustment Number")
        .enumeration("adjustment_type", "Type", vec![
            "manual", "reclassification", "correction",
        ])
        .string("description", "Description")
        .string("debit_account", "Debit Account")
        .string("credit_account", "Credit Account")
        .currency("amount", "Amount", "USD")
        .string("currency_code", "Currency Code")
        .reference("entity_id", "Entity", "organizations")
        .enumeration("status", "Status", vec![
            "draft", "approved", "posted",
        ])
        .workflow(workflow)
        .build()
}

/// Consolidation Elimination Rule entity
/// Oracle Fusion: General Ledger > Consolidation > Elimination Rules
pub fn consolidation_elimination_rule_definition() -> EntityDefinition {
    SchemaBuilder::new("consolidation_elimination_rules", "Elimination Rule")
        .plural_label("Consolidation Elimination Rules")
        .table_name("fin_consolidation_elimination_rules")
        .description("Rules for automatic intercompany elimination during consolidation")
        .icon("eraser")
        .reference("ledger_id", "Consolidation Ledger", "consolidation_ledgers")
        .required_string("code", "Rule Code")
        .required_string("name", "Rule Name")
        .enumeration("elimination_type", "Elimination Type", vec![
            "intercompany_receivable_payable",
            "intercompany_revenue_expense",
            "investment_equity",
            "intercompany_inventory_profit",
            "other",
        ])
        .string("debit_account_rule", "Debit Account Rule")
        .string("credit_account_rule", "Credit Account Rule")
        .boolean("is_auto_execute", "Auto Execute")
        .boolean("is_active", "Active")
        .build()
}

/// Consolidation Translation Rate entity
/// Oracle Fusion: General Ledger > Consolidation > Translation Rates
pub fn consolidation_translation_rate_definition() -> EntityDefinition {
    SchemaBuilder::new("consolidation_translation_rates", "Translation Rate")
        .plural_label("Consolidation Translation Rates")
        .table_name("fin_consolidation_translation_rates")
        .description("Currency translation rates for consolidation")
        .icon("exchange-alt")
        .reference("ledger_id", "Consolidation Ledger", "consolidation_ledgers")
        .reference("scenario_id", "Scenario", "consolidation_scenarios")
        .string("from_currency_code", "From Currency")
        .string("to_currency_code", "To Currency")
        .enumeration("rate_type", "Rate Type", vec![
            "period_end", "average", "historical", "spot",
        ])
        .decimal("rate", "Rate", 18, 10)
        .date("effective_date", "Effective Date")
        .integer("fiscal_year", "Fiscal Year")
        .integer("period_number", "Period Number")
        .build()
}
