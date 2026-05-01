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

// ============================================================================
// Collections Management (Oracle Fusion: Financials > Collections)
// ============================================================================

/// Customer Credit Profile entity (Collections context)
/// Oracle Fusion: Collections > Customer Credit Profiles
pub fn customer_credit_profile_definition() -> EntityDefinition {
    SchemaBuilder::new("customer_credit_profiles", "Customer Credit Profile")
        .plural_label("Customer Credit Profiles")
        .table_name("fin_customer_credit_profiles")
        .description("Customer credit profiles with limits, scoring, and risk classification")
        .icon("user-shield")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .currency("credit_limit", "Credit Limit", "USD")
        .currency("credit_used", "Credit Used", "USD")
        .currency("credit_available", "Credit Available", "USD")
        .enumeration("risk_classification", "Risk Classification", vec![
            "low", "medium", "high", "very_high", "defaulted",
        ])
        .integer("credit_score", "Credit Score (0-1000)")
        .string("external_credit_rating", "External Credit Rating")
        .string("external_rating_agency", "Rating Agency")
        .date("external_rating_date", "Rating Date")
        .enumeration("payment_terms", "Payment Terms", vec![
            "net_15", "net_30", "net_45", "net_60", "due_on_receipt", "cod",
        ])
        .decimal("average_days_to_pay", "Avg Days to Pay", 8, 2)
        .integer("overdue_invoice_count", "Overdue Invoices")
        .currency("total_overdue_amount", "Total Overdue", "USD")
        .date("oldest_overdue_date", "Oldest Overdue Date")
        .boolean("credit_hold", "Credit Hold")
        .string("credit_hold_reason", "Hold Reason")
        .date("last_review_date", "Last Review Date")
        .date("next_review_date", "Next Review Date")
        .enumeration("status", "Status", vec![
            "active", "inactive", "blocked",
        ])
        .build()
}

/// Collection Strategy entity
/// Oracle Fusion: Collections > Collection Strategies
pub fn collection_strategy_definition() -> EntityDefinition {
    SchemaBuilder::new("collection_strategies", "Collection Strategy")
        .plural_label("Collection Strategies")
        .table_name("fin_collection_strategies")
        .description("Automated collection strategies triggered by aging and risk")
        .icon("chess")
        .required_string("code", "Strategy Code")
        .required_string("name", "Strategy Name")
        .string("description", "Description")
        .enumeration("strategy_type", "Strategy Type", vec![
            "automatic", "manual",
        ])
        .json("applicable_risk_classifications", "Applicable Risk Classifications")
        .json("trigger_aging_buckets", "Trigger Aging Buckets")
        .currency("overdue_amount_threshold", "Overdue Amount Threshold", "USD")
        .json("actions", "Collection Actions")
        .integer("priority", "Priority")
        .boolean("is_active", "Active")
        .build()
}

/// Collection Case entity with workflow
/// Oracle Fusion: Collections > Collection Cases
pub fn collection_case_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("collection_case_workflow", "open")
        .initial_state("open", "Open")
        .working_state("in_progress", "In Progress")
        .working_state("escalated", "Escalated")
        .final_state("resolved", "Resolved")
        .final_state("closed", "Closed")
        .final_state("written_off", "Written Off")
        .transition("open", "in_progress", "start_work")
        .transition("in_progress", "escalated", "escalate")
        .transition("in_progress", "resolved", "resolve")
        .transition("escalated", "resolved", "resolve")
        .transition("resolved", "closed", "close")
        .transition("in_progress", "written_off", "write_off")
        .build();

    SchemaBuilder::new("collection_cases", "Collection Case")
        .plural_label("Collection Cases")
        .table_name("fin_collection_cases")
        .description("Collection cases for managing overdue customer receivables")
        .icon("folder-open")
        .required_string("case_number", "Case Number")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .reference("strategy_id", "Strategy", "collection_strategies")
        .reference("assigned_to", "Assigned To", "employees")
        .string("assigned_to_name", "Assignee Name")
        .enumeration("case_type", "Case Type", vec![
            "collection", "dispute", "bankruptcy", "skip_trace",
        ])
        .enumeration("priority", "Priority", vec![
            "low", "medium", "high", "critical",
        ])
        .currency("total_overdue_amount", "Total Overdue", "USD")
        .currency("total_disputed_amount", "Total Disputed", "USD")
        .currency("total_invoiced_amount", "Total Invoiced", "USD")
        .integer("overdue_invoice_count", "Overdue Invoice Count")
        .date("oldest_overdue_date", "Oldest Overdue Date")
        .integer("current_step", "Current Strategy Step")
        .date("opened_date", "Opened Date")
        .date("target_resolution_date", "Target Resolution Date")
        .date("resolved_date", "Resolved Date")
        .date("closed_date", "Closed Date")
        .date("next_action_date", "Next Action Date")
        .enumeration("resolution_type", "Resolution Type", vec![
            "full_payment", "partial_payment", "payment_plan",
            "write_off", "dispute_resolved", "uncollectible", "other",
        ])
        .string("resolution_notes", "Resolution Notes")
        .json("related_invoice_ids", "Related Invoice IDs")
        .enumeration("status", "Status", vec![
            "open", "in_progress", "resolved", "closed", "escalated", "written_off",
        ])
        .workflow(workflow)
        .build()
}

/// Customer Interaction entity
/// Oracle Fusion: Collections > Customer Interactions
pub fn customer_interaction_definition() -> EntityDefinition {
    SchemaBuilder::new("customer_interactions", "Customer Interaction")
        .plural_label("Customer Interactions")
        .table_name("fin_customer_interactions")
        .description("Customer interactions (calls, emails, meetings) for collections")
        .icon("comments")
        .reference("case_id", "Case", "collection_cases")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .enumeration("interaction_type", "Interaction Type", vec![
            "phone_call", "email", "letter", "meeting", "note", "sms",
        ])
        .enumeration("direction", "Direction", vec![
            "outbound", "inbound",
        ])
        .string("contact_name", "Contact Name")
        .string("contact_role", "Contact Role")
        .string("contact_phone", "Contact Phone")
        .string("contact_email", "Contact Email")
        .string("subject", "Subject")
        .rich_text("body", "Body")
        .enumeration("outcome", "Outcome", vec![
            "contacted", "left_message", "no_answer", "promised_to_pay",
            "disputed", "refused", "agreed_payment_plan", "escalated", "no_action",
        ])
        .date("follow_up_date", "Follow-Up Date")
        .string("follow_up_notes", "Follow-Up Notes")
        .reference("performed_by", "Performed By", "employees")
        .string("performed_by_name", "Performer Name")
        .integer("duration_minutes", "Duration (Minutes)")
        .build()
}

/// Promise to Pay entity
/// Oracle Fusion: Collections > Promises to Pay
pub fn promise_to_pay_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("promise_to_pay_workflow", "pending")
        .initial_state("pending", "Pending")
        .working_state("partially_kept", "Partially Kept")
        .final_state("kept", "Kept")
        .final_state("broken", "Broken")
        .final_state("cancelled", "Cancelled")
        .transition("pending", "partially_kept", "partial_payment")
        .transition("pending", "kept", "full_payment")
        .transition("pending", "broken", "break")
        .transition("pending", "cancelled", "cancel")
        .transition("partially_kept", "kept", "full_payment")
        .transition("partially_kept", "broken", "break")
        .build();

    SchemaBuilder::new("promise_to_pay", "Promise to Pay")
        .plural_label("Promises to Pay")
        .table_name("fin_promise_to_pay")
        .description("Customer promises to pay overdue amounts")
        .icon("handshake")
        .reference("case_id", "Case", "collection_cases")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .enumeration("promise_type", "Promise Type", vec![
            "single_payment", "installment", "full_balance",
        ])
        .currency("promised_amount", "Promised Amount", "USD")
        .currency("paid_amount", "Paid Amount", "USD")
        .currency("remaining_amount", "Remaining Amount", "USD")
        .date("promise_date", "Promise Date")
        .integer("installment_count", "Installment Count")
        .enumeration("installment_frequency", "Installment Frequency", vec![
            "weekly", "biweekly", "monthly",
        ])
        .date("broken_date", "Broken Date")
        .string("broken_reason", "Broken Reason")
        .json("related_invoice_ids", "Related Invoice IDs")
        .string("promised_by_name", "Promised By")
        .string("promised_by_role", "Promised By Role")
        .rich_text("notes", "Notes")
        .reference("recorded_by", "Recorded By", "employees")
        .enumeration("status", "Status", vec![
            "pending", "partially_kept", "kept", "broken", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

/// Dunning Campaign entity with workflow
/// Oracle Fusion: Collections > Dunning Management
pub fn dunning_campaign_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("dunning_campaign_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("scheduled", "Scheduled")
        .working_state("in_progress", "In Progress")
        .final_state("completed", "Completed")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "scheduled", "schedule")
        .transition("scheduled", "in_progress", "start")
        .transition("in_progress", "completed", "complete")
        .transition("draft", "cancelled", "cancel")
        .transition("scheduled", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("dunning_campaigns", "Dunning Campaign")
        .plural_label("Dunning Campaigns")
        .table_name("fin_dunning_campaigns")
        .description("Dunning campaigns for escalating overdue customer communications")
        .icon("bullhorn")
        .required_string("campaign_number", "Campaign Number")
        .required_string("name", "Campaign Name")
        .string("description", "Description")
        .enumeration("dunning_level", "Dunning Level", vec![
            "reminder", "first_notice", "second_notice",
            "final_notice", "pre_legal", "legal",
        ])
        .enumeration("communication_method", "Communication Method", vec![
            "email", "letter", "sms", "phone",
        ])
        .string("template_name", "Template Name")
        .integer("min_overdue_days", "Min Overdue Days")
        .currency("min_overdue_amount", "Min Overdue Amount", "USD")
        .json("target_risk_classifications", "Target Risk Classifications")
        .boolean("exclude_active_cases", "Exclude Active Cases")
        .date("scheduled_date", "Scheduled Date")
        .date("sent_date", "Sent Date")
        .integer("target_customer_count", "Target Customer Count")
        .integer("sent_count", "Sent Count")
        .integer("failed_count", "Failed Count")
        .enumeration("status", "Status", vec![
            "draft", "scheduled", "in_progress", "completed", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

/// Dunning Letter entity
/// Oracle Fusion: Collections > Dunning Letters
pub fn dunning_letter_definition() -> EntityDefinition {
    SchemaBuilder::new("dunning_letters", "Dunning Letter")
        .plural_label("Dunning Letters")
        .table_name("fin_dunning_letters")
        .description("Individual dunning letters sent to customers")
        .icon("envelope")
        .reference("campaign_id", "Campaign", "dunning_campaigns")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .string("customer_email", "Customer Email")
        .enumeration("dunning_level", "Dunning Level", vec![
            "reminder", "first_notice", "second_notice",
            "final_notice", "pre_legal", "legal",
        ])
        .enumeration("communication_method", "Communication Method", vec![
            "email", "letter", "sms", "phone",
        ])
        .currency("total_overdue_amount", "Total Overdue", "USD")
        .integer("overdue_invoice_count", "Overdue Invoice Count")
        .date("oldest_overdue_date", "Oldest Overdue Date")
        .currency("aging_current", "Current", "USD")
        .currency("aging_1_30", "1-30 Days", "USD")
        .currency("aging_31_60", "31-60 Days", "USD")
        .currency("aging_61_90", "61-90 Days", "USD")
        .currency("aging_91_120", "91-120 Days", "USD")
        .currency("aging_121_plus", "121+ Days", "USD")
        .enumeration("status", "Status", vec![
            "pending", "sent", "delivered", "bounced", "failed", "viewed",
        ])
        .string("failure_reason", "Failure Reason")
        .json("invoice_details", "Invoice Details")
        .build()
}

/// Receivables Aging Snapshot entity
/// Oracle Fusion: Collections > Aging Analysis
pub fn receivables_aging_snapshot_definition() -> EntityDefinition {
    SchemaBuilder::new("receivables_aging_snapshots", "Aging Snapshot")
        .plural_label("Receivables Aging Snapshots")
        .table_name("fin_receivables_aging_snapshots")
        .description("Receivables aging analysis snapshots by customer")
        .icon("chart-bar")
        .date("snapshot_date", "Snapshot Date")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .currency("total_outstanding", "Total Outstanding", "USD")
        .currency("aging_current", "Current", "USD")
        .currency("aging_1_30", "1-30 Days", "USD")
        .currency("aging_31_60", "31-60 Days", "USD")
        .currency("aging_61_90", "61-90 Days", "USD")
        .currency("aging_91_120", "91-120 Days", "USD")
        .currency("aging_121_plus", "121+ Days", "USD")
        .integer("count_current", "Count Current")
        .integer("count_1_30", "Count 1-30")
        .integer("count_31_60", "Count 31-60")
        .integer("count_61_90", "Count 61-90")
        .integer("count_91_120", "Count 91-120")
        .integer("count_121_plus", "Count 121+")
        .decimal("weighted_average_days_overdue", "Wtd Avg Days Overdue", 8, 2)
        .decimal("overdue_percent", "Overdue %", 5, 2)
        .build()
}

/// Write-Off Request entity with workflow
/// Oracle Fusion: Collections > Write-Off Management
pub fn write_off_request_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("write_off_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted")
        .working_state("approved", "Approved")
        .final_state("rejected", "Rejected")
        .final_state("processed", "Processed")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "rejected", "reject")
        .transition("approved", "processed", "process")
        .transition("draft", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("write_off_requests", "Write-Off Request")
        .plural_label("Write-Off Requests")
        .table_name("fin_write_off_requests")
        .description("Write-off requests for uncollectible receivables")
        .icon("eraser")
        .required_string("request_number", "Request Number")
        .reference("customer_id", "Customer", "customers")
        .string("customer_number", "Customer Number")
        .string("customer_name", "Customer Name")
        .enumeration("write_off_type", "Write-Off Type", vec![
            "bad_debt", "small_balance", "dispute", "adjustment",
        ])
        .currency("write_off_amount", "Write-Off Amount", "USD")
        .string("write_off_account_code", "Write-Off Account")
        .string("reason", "Reason")
        .json("related_invoice_ids", "Related Invoice IDs")
        .reference("case_id", "Case", "collection_cases")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "approved", "rejected", "processed", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Credit Management (Oracle Fusion: Financials > Credit Management)
// ============================================================================

/// Credit Scoring Model entity
/// Oracle Fusion: Credit Management > Credit Scoring Models
pub fn credit_scoring_model_definition() -> EntityDefinition {
    SchemaBuilder::new("credit_scoring_models", "Credit Scoring Model")
        .plural_label("Credit Scoring Models")
        .table_name("fin_credit_scoring_models")
        .description("Models for assessing customer creditworthiness")
        .icon("star-half-alt")
        .required_string("code", "Model Code")
        .required_string("name", "Model Name")
        .string("description", "Description")
        .enumeration("model_type", "Model Type", vec![
            "manual", "scorecard", "risk_category", "external",
        ])
        .json("scoring_criteria", "Scoring Criteria")
        .json("score_ranges", "Score Ranges")
        .boolean("is_active", "Active")
        .build()
}

/// Credit Profile entity (Credit Management context)
/// Oracle Fusion: Credit Management > Credit Profiles
pub fn credit_profile_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("credit_profile_workflow", "active")
        .initial_state("active", "Active")
        .working_state("suspended", "Suspended")
        .final_state("blocked", "Blocked")
        .final_state("inactive", "Inactive")
        .transition("active", "suspended", "suspend")
        .transition("suspended", "active", "reactivate")
        .transition("active", "blocked", "block")
        .transition("active", "inactive", "deactivate")
        .transition("suspended", "blocked", "block")
        .build();

    SchemaBuilder::new("credit_profiles", "Credit Profile")
        .plural_label("Credit Profiles")
        .table_name("fin_credit_profiles")
        .description("Customer credit profiles with scoring and risk assessment")
        .icon("id-card")
        .required_string("profile_number", "Profile Number")
        .required_string("profile_name", "Profile Name")
        .string("description", "Description")
        .enumeration("profile_type", "Profile Type", vec![
            "customer", "customer_group", "global",
        ])
        .reference("customer_id", "Customer", "customers")
        .string("customer_name", "Customer Name")
        .reference("scoring_model_id", "Scoring Model", "credit_scoring_models")
        .decimal("credit_score", "Credit Score", 10, 2)
        .string("credit_rating", "Credit Rating")
        .enumeration("risk_level", "Risk Level", vec![
            "low", "medium", "high", "very_high", "blocked",
        ])
        .integer("review_frequency_days", "Review Frequency (Days)")
        .date("last_review_date", "Last Review Date")
        .date("next_review_date", "Next Review Date")
        .enumeration("status", "Status", vec![
            "active", "inactive", "suspended", "blocked",
        ])
        .workflow(workflow)
        .build()
}

/// Credit Limit entity
/// Oracle Fusion: Credit Management > Credit Limits
pub fn credit_limit_definition() -> EntityDefinition {
    SchemaBuilder::new("credit_limits", "Credit Limit")
        .plural_label("Credit Limits")
        .table_name("fin_credit_limits")
        .description("Credit limits per profile with multi-currency support")
        .icon("dollar-sign")
        .reference("profile_id", "Credit Profile", "credit_profiles")
        .enumeration("limit_type", "Limit Type", vec![
            "overall", "order", "delivery", "currency",
        ])
        .string("currency_code", "Currency Code")
        .currency("credit_limit", "Credit Limit", "USD")
        .currency("temp_limit_increase", "Temp Limit Increase", "USD")
        .date("temp_limit_expiry", "Temp Limit Expiry")
        .currency("used_amount", "Used Amount", "USD")
        .currency("available_amount", "Available Amount", "USD")
        .currency("hold_amount", "Hold Amount", "USD")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Credit Check Rule entity
/// Oracle Fusion: Credit Management > Credit Check Rules
pub fn credit_check_rule_definition() -> EntityDefinition {
    SchemaBuilder::new("credit_check_rules", "Credit Check Rule")
        .plural_label("Credit Check Rules")
        .table_name("fin_credit_check_rules")
        .description("Rules defining when and how credit checks are triggered")
        .icon("search-dollar")
        .required_string("name", "Rule Name")
        .string("description", "Description")
        .enumeration("check_point", "Check Point", vec![
            "order_entry", "shipment", "invoice", "delivery", "payment",
        ])
        .enumeration("check_type", "Check Type", vec![
            "automatic", "manual",
        ])
        .json("condition", "Condition")
        .enumeration("action_on_failure", "Action on Failure", vec![
            "hold", "warn", "reject", "notify",
        ])
        .integer("priority", "Priority")
        .boolean("is_active", "Active")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .build()
}

/// Credit Exposure entity
/// Oracle Fusion: Credit Management > Credit Exposure
pub fn credit_exposure_definition() -> EntityDefinition {
    SchemaBuilder::new("credit_exposure", "Credit Exposure")
        .plural_label("Credit Exposures")
        .table_name("fin_credit_exposure")
        .description("Real-time credit exposure tracking per profile")
        .icon("tachometer-alt")
        .reference("profile_id", "Credit Profile", "credit_profiles")
        .date("exposure_date", "Exposure Date")
        .currency("open_receivables", "Open Receivables", "USD")
        .currency("open_orders", "Open Orders", "USD")
        .currency("open_shipments", "Open Shipments", "USD")
        .currency("open_invoices", "Open Invoices", "USD")
        .currency("unapplied_cash", "Unapplied Cash", "USD")
        .currency("on_hold_amount", "On Hold Amount", "USD")
        .currency("total_exposure", "Total Exposure", "USD")
        .currency("credit_limit", "Credit Limit", "USD")
        .currency("available_credit", "Available Credit", "USD")
        .decimal("utilization_percent", "Utilization %", 8, 4)
        .string("currency_code", "Currency Code")
        .build()
}

/// Credit Hold entity with workflow
/// Oracle Fusion: Credit Management > Credit Holds
pub fn credit_hold_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("credit_hold_workflow", "active")
        .initial_state("active", "Active")
        .final_state("released", "Released")
        .final_state("overridden", "Overridden")
        .final_state("cancelled", "Cancelled")
        .transition("active", "released", "release")
        .transition("active", "overridden", "override")
        .transition("active", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("credit_holds", "Credit Hold")
        .plural_label("Credit Holds")
        .table_name("fin_credit_holds")
        .description("Credit holds placed on transactions when limits are exceeded")
        .icon("lock")
        .reference("profile_id", "Credit Profile", "credit_profiles")
        .required_string("hold_number", "Hold Number")
        .enumeration("hold_type", "Hold Type", vec![
            "credit_limit", "overdue", "review", "manual", "scoring",
        ])
        .string("entity_type", "Entity Type")
        .string("entity_number", "Entity Number")
        .currency("hold_amount", "Hold Amount", "USD")
        .string("reason", "Reason")
        .string("release_reason", "Release Reason")
        .string("override_reason", "Override Reason")
        .enumeration("status", "Status", vec![
            "active", "released", "overridden", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

/// Credit Review entity with workflow
/// Oracle Fusion: Credit Management > Credit Reviews
pub fn credit_review_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("credit_review_workflow", "pending")
        .initial_state("pending", "Pending")
        .working_state("in_review", "In Review")
        .final_state("completed", "Completed")
        .final_state("cancelled", "Cancelled")
        .transition("pending", "in_review", "start_review")
        .transition("in_review", "completed", "complete")
        .transition("pending", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("credit_reviews", "Credit Review")
        .plural_label("Credit Reviews")
        .table_name("fin_credit_reviews")
        .description("Periodic or triggered credit profile reviews")
        .icon("clipboard-check")
        .reference("profile_id", "Credit Profile", "credit_profiles")
        .required_string("review_number", "Review Number")
        .enumeration("review_type", "Review Type", vec![
            "periodic", "triggered", "ad_hoc", "escalation",
        ])
        .currency("previous_credit_limit", "Previous Credit Limit", "USD")
        .currency("recommended_credit_limit", "Recommended Credit Limit", "USD")
        .currency("approved_credit_limit", "Approved Credit Limit", "USD")
        .decimal("previous_score", "Previous Score", 10, 2)
        .decimal("new_score", "New Score", 10, 2)
        .string("previous_rating", "Previous Rating")
        .string("new_rating", "New Rating")
        .rich_text("findings", "Findings")
        .rich_text("recommendations", "Recommendations")
        .reference("reviewer_id", "Reviewer", "employees")
        .string("reviewer_name", "Reviewer Name")
        .reference("approver_id", "Approver", "employees")
        .string("approver_name", "Approver Name")
        .string("rejected_reason", "Rejected Reason")
        .date("due_date", "Due Date")
        .enumeration("status", "Status", vec![
            "pending", "in_review", "completed", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Withholding Tax (Oracle Fusion: Payables > Withholding Tax)
// ============================================================================

/// Withholding Tax Code entity
/// Oracle Fusion: Payables > Withholding Tax > Tax Codes
pub fn withholding_tax_code_definition() -> EntityDefinition {
    SchemaBuilder::new("withholding_tax_codes", "Withholding Tax Code")
        .plural_label("Withholding Tax Codes")
        .table_name("fin_withholding_tax_codes")
        .description("Withholding tax codes with rates and thresholds")
        .icon("file-invoice-dollar")
        .required_string("code", "Tax Code")
        .required_string("name", "Tax Code Name")
        .string("description", "Description")
        .enumeration("tax_type", "Tax Type", vec![
            "income_tax", "vat", "service_tax", "contract_tax",
            "royalty", "dividend", "interest", "other",
        ])
        .decimal("rate_percentage", "Rate %", 12, 4)
        .currency("threshold_amount", "Threshold Amount", "USD")
        .boolean("threshold_is_cumulative", "Cumulative Threshold")
        .string("withholding_account_code", "Withholding Account")
        .string("expense_account_code", "Expense Account")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Withholding Tax Group entity
/// Oracle Fusion: Payables > Withholding Tax > Tax Groups
pub fn withholding_tax_group_definition() -> EntityDefinition {
    SchemaBuilder::new("withholding_tax_groups", "Withholding Tax Group")
        .plural_label("Withholding Tax Groups")
        .table_name("fin_withholding_tax_groups")
        .description("Groups of withholding tax codes assignable to suppliers")
        .icon("layer-group")
        .required_string("code", "Group Code")
        .required_string("name", "Group Name")
        .string("description", "Description")
        .boolean("is_active", "Active")
        .build()
}

/// Supplier Withholding Assignment entity
/// Oracle Fusion: Payables > Withholding Tax > Supplier Assignments
pub fn supplier_withholding_assignment_definition() -> EntityDefinition {
    SchemaBuilder::new("supplier_withholding_assignments", "Supplier WHT Assignment")
        .plural_label("Supplier Withholding Assignments")
        .table_name("fin_supplier_withholding_assignments")
        .description("Supplier assignments to withholding tax groups")
        .icon("link")
        .reference("supplier_id", "Supplier", "suppliers")
        .string("supplier_number", "Supplier Number")
        .string("supplier_name", "Supplier Name")
        .reference("tax_group_id", "Tax Group", "withholding_tax_groups")
        .boolean("is_exempt", "Exempt")
        .string("exemption_reason", "Exemption Reason")
        .string("exemption_certificate", "Exemption Certificate")
        .date("exemption_valid_until", "Exemption Valid Until")
        .boolean("is_active", "Active")
        .build()
}

/// Withholding Tax Line entity
/// Oracle Fusion: Payables > Withholding Tax > Tax Lines
pub fn withholding_tax_line_definition() -> EntityDefinition {
    SchemaBuilder::new("withholding_tax_lines", "Withholding Tax Line")
        .plural_label("Withholding Tax Lines")
        .table_name("fin_withholding_tax_lines")
        .description("Actual withholding tax lines from supplier payments")
        .icon("receipt")
        .reference("payment_id", "Payment", "ap_payments")
        .string("payment_number", "Payment Number")
        .reference("invoice_id", "Invoice", "ap_invoices")
        .string("invoice_number", "Invoice Number")
        .reference("supplier_id", "Supplier", "suppliers")
        .string("supplier_name", "Supplier Name")
        .reference("tax_code_id", "Tax Code", "withholding_tax_codes")
        .string("tax_code", "Tax Code")
        .string("tax_type", "Tax Type")
        .decimal("rate_percentage", "Rate %", 12, 4)
        .currency("taxable_amount", "Taxable Amount", "USD")
        .currency("withheld_amount", "Withheld Amount", "USD")
        .string("withholding_account_code", "Withholding Account")
        .enumeration("status", "Status", vec![
            "pending", "withheld", "remitted", "refunded",
        ])
        .date("remittance_date", "Remittance Date")
        .string("remittance_reference", "Remittance Reference")
        .build()
}

/// Withholding Certificate entity with workflow
/// Oracle Fusion: Payables > Withholding Tax > Certificates
pub fn withholding_certificate_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("withholding_certificate_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("issued", "Issued")
        .final_state("acknowledged", "Acknowledged")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "issued", "issue")
        .transition("issued", "acknowledged", "acknowledge")
        .transition("draft", "cancelled", "cancel")
        .transition("issued", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("withholding_certificates", "Withholding Certificate")
        .plural_label("Withholding Certificates")
        .table_name("fin_withholding_certificates")
        .description("Withholding tax certificates issued to suppliers")
        .icon("certificate")
        .required_string("certificate_number", "Certificate Number")
        .reference("supplier_id", "Supplier", "suppliers")
        .string("supplier_number", "Supplier Number")
        .string("supplier_name", "Supplier Name")
        .string("tax_type", "Tax Type")
        .reference("tax_code_id", "Tax Code", "withholding_tax_codes")
        .string("tax_code", "Tax Code")
        .date("period_start", "Period Start")
        .date("period_end", "Period End")
        .currency("total_invoice_amount", "Total Invoice Amount", "USD")
        .currency("total_withheld_amount", "Total Withheld Amount", "USD")
        .decimal("rate_percentage", "Rate %", 12, 4)
        .json("payment_ids", "Payment IDs")
        .string("notes", "Notes")
        .enumeration("status", "Status", vec![
            "draft", "issued", "acknowledged", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Project Billing (Oracle Fusion: Project Management > Project Billing)
// ============================================================================

/// Bill Rate Schedule entity with workflow
/// Oracle Fusion: Project Billing > Bill Rate Schedules
pub fn bill_rate_schedule_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("bill_rate_schedule_workflow", "draft")
        .initial_state("draft", "Draft")
        .final_state("active", "Active")
        .final_state("inactive", "Inactive")
        .transition("draft", "active", "activate")
        .transition("active", "inactive", "deactivate")
        .build();

    SchemaBuilder::new("bill_rate_schedules", "Bill Rate Schedule")
        .plural_label("Bill Rate Schedules")
        .table_name("fin_bill_rate_schedules")
        .description("Billable rate schedules by role and labor category")
        .icon("clock")
        .required_string("schedule_number", "Schedule Number")
        .required_string("name", "Schedule Name")
        .string("description", "Description")
        .enumeration("schedule_type", "Schedule Type", vec![
            "standard", "overtime", "holiday", "custom",
        ])
        .string("currency_code", "Currency Code")
        .date("effective_start", "Effective Start")
        .date("effective_end", "Effective End")
        .decimal("default_markup_pct", "Default Markup %", 8, 4)
        .enumeration("status", "Status", vec![
            "draft", "active", "inactive",
        ])
        .workflow(workflow)
        .build()
}

/// Bill Rate Line entity
/// Oracle Fusion: Project Billing > Bill Rate Lines
pub fn bill_rate_line_definition() -> EntityDefinition {
    SchemaBuilder::new("bill_rate_lines", "Bill Rate Line")
        .plural_label("Bill Rate Lines")
        .table_name("fin_bill_rate_lines")
        .description("Individual billable rates within a schedule")
        .icon("list")
        .reference("schedule_id", "Schedule", "bill_rate_schedules")
        .required_string("role_name", "Role Name")
        .reference("project_id", "Project", "projects")
        .currency("bill_rate", "Bill Rate", "USD")
        .string("unit_of_measure", "UOM")
        .date("effective_start", "Effective Start")
        .date("effective_end", "Effective End")
        .decimal("markup_pct", "Markup %", 8, 4)
        .build()
}

/// Project Billing Config entity with workflow
/// Oracle Fusion: Project Billing > Billing Configuration
pub fn project_billing_config_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("project_billing_config_workflow", "draft")
        .initial_state("draft", "Draft")
        .final_state("active", "Active")
        .final_state("completed", "Completed")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "active", "activate")
        .transition("active", "completed", "complete")
        .transition("active", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("project_billing_configs", "Project Billing Config")
        .plural_label("Project Billing Configs")
        .table_name("fin_project_billing_configs")
        .description("Billing arrangement configuration per project")
        .icon("cog")
        .reference("project_id", "Project", "projects")
        .enumeration("billing_method", "Billing Method", vec![
            "time_and_materials", "fixed_price", "milestone", "cost_plus", "retention",
        ])
        .reference("bill_rate_schedule_id", "Bill Rate Schedule", "bill_rate_schedules")
        .currency("contract_amount", "Contract Amount", "USD")
        .string("currency_code", "Currency Code")
        .enumeration("invoice_format", "Invoice Format", vec![
            "detailed", "summary", "consolidated",
        ])
        .enumeration("billing_cycle", "Billing Cycle", vec![
            "weekly", "biweekly", "monthly", "milestone",
        ])
        .integer("payment_terms_days", "Payment Terms (Days)")
        .decimal("retention_pct", "Retention %", 8, 4)
        .currency("retention_amount_cap", "Retention Cap", "USD")
        .reference("customer_id", "Customer", "customers")
        .string("customer_name", "Customer Name")
        .string("customer_po_number", "Customer PO Number")
        .string("contract_number", "Contract Number")
        .enumeration("status", "Status", vec![
            "draft", "active", "completed", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

/// Billing Event entity with workflow
/// Oracle Fusion: Project Billing > Billing Events
pub fn billing_event_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("billing_event_workflow", "planned")
        .initial_state("planned", "Planned")
        .working_state("ready", "Ready")
        .working_state("invoiced", "Invoiced")
        .final_state("partially_invoiced", "Partially Invoiced")
        .final_state("cancelled", "Cancelled")
        .transition("planned", "ready", "mark_ready")
        .transition("ready", "invoiced", "invoice")
        .transition("ready", "partially_invoiced", "partial_invoice")
        .transition("planned", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("billing_events", "Billing Event")
        .plural_label("Billing Events")
        .table_name("fin_billing_events")
        .description("Milestones and progress markers for project billing")
        .icon("flag")
        .reference("project_id", "Project", "projects")
        .required_string("event_number", "Event Number")
        .required_string("event_name", "Event Name")
        .string("description", "Description")
        .enumeration("event_type", "Event Type", vec![
            "milestone", "progress", "completion", "retention_release",
        ])
        .currency("billing_amount", "Billing Amount", "USD")
        .string("currency_code", "Currency Code")
        .decimal("completion_pct", "Completion %", 8, 4)
        .date("planned_date", "Planned Date")
        .date("actual_date", "Actual Date")
        .reference("task_id", "Task", "tasks")
        .string("task_name", "Task Name")
        .enumeration("status", "Status", vec![
            "planned", "ready", "invoiced", "partially_invoiced", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

/// Project Invoice Header entity with workflow
/// Oracle Fusion: Project Billing > Project Invoices
pub fn project_invoice_header_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("project_invoice_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted")
        .working_state("approved", "Approved")
        .final_state("rejected", "Rejected")
        .final_state("posted", "Posted")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "rejected", "reject")
        .transition("approved", "posted", "post")
        .transition("draft", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("project_invoice_headers", "Project Invoice")
        .plural_label("Project Invoices")
        .table_name("fin_project_invoice_headers")
        .description("Project invoices for client billing")
        .icon("file-invoice-dollar")
        .required_string("invoice_number", "Invoice Number")
        .reference("project_id", "Project", "projects")
        .string("project_number", "Project Number")
        .string("project_name", "Project Name")
        .enumeration("invoice_type", "Invoice Type", vec![
            "progress", "milestone", "t_and_m", "retention_release",
            "debit_memo", "credit_memo",
        ])
        .reference("customer_id", "Customer", "customers")
        .string("customer_name", "Customer Name")
        .currency("invoice_amount", "Invoice Amount", "USD")
        .currency("tax_amount", "Tax Amount", "USD")
        .currency("retention_held", "Retention Held", "USD")
        .currency("total_amount", "Total Amount", "USD")
        .string("currency_code", "Currency Code")
        .date("billing_period_start", "Billing Period Start")
        .date("billing_period_end", "Billing Period End")
        .date("invoice_date", "Invoice Date")
        .date("due_date", "Due Date")
        .reference("billing_event_id", "Billing Event", "billing_events")
        .string("customer_po_number", "Customer PO Number")
        .string("contract_number", "Contract Number")
        .boolean("gl_posted_flag", "GL Posted")
        .string("rejected_reason", "Rejected Reason")
        .enumeration("payment_status", "Payment Status", vec![
            "unpaid", "partially_paid", "paid",
        ])
        .rich_text("notes", "Notes")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "approved", "rejected", "posted", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

/// Project Invoice Line entity
/// Oracle Fusion: Project Billing > Invoice Lines
pub fn project_invoice_line_definition() -> EntityDefinition {
    SchemaBuilder::new("project_invoice_lines", "Project Invoice Line")
        .plural_label("Project Invoice Lines")
        .table_name("fin_project_invoice_lines")
        .description("Individual line items on a project invoice")
        .icon("list")
        .reference("invoice_header_id", "Invoice", "project_invoice_headers")
        .integer("line_number", "Line Number")
        .enumeration("line_source", "Line Source", vec![
            "expenditure_item", "billing_event", "retention", "manual",
        ])
        .reference("billing_event_id", "Billing Event", "billing_events")
        .reference("task_id", "Task", "tasks")
        .string("task_number", "Task Number")
        .string("task_name", "Task Name")
        .string("description", "Description")
        .reference("employee_id", "Employee", "employees")
        .string("employee_name", "Employee Name")
        .string("role_name", "Role Name")
        .string("expenditure_type", "Expenditure Type")
        .decimal("quantity", "Quantity", 18, 4)
        .string("unit_of_measure", "UOM")
        .currency("bill_rate", "Bill Rate", "USD")
        .currency("raw_cost_amount", "Raw Cost", "USD")
        .currency("bill_amount", "Bill Amount", "USD")
        .currency("markup_amount", "Markup Amount", "USD")
        .currency("retention_amount", "Retention Amount", "USD")
        .currency("tax_amount", "Tax Amount", "USD")
        .date("transaction_date", "Transaction Date")
        .build()
}

// ============================================================================
// Payment Terms Engine (Oracle Fusion: Financials > Payment Terms)
// ============================================================================

/// Payment Term entity with discount scheduling
/// Oracle Fusion: Financials > Payment Terms > Define Payment Terms
pub fn payment_term_definition() -> EntityDefinition {
    SchemaBuilder::new("payment_terms", "Payment Term")
        .plural_label("Payment Terms")
        .table_name("fin_payment_terms")
        .description("Payment terms with discount schedules for AP and AR")
        .icon("calendar-check")
        .required_string("code", "Term Code")
        .required_string("name", "Term Name")
        .string("description", "Description")
        .enumeration("term_type", "Term Type", vec![
            "immediate", "net_days", "discount_net", "milestone", "installment",
        ])
        .integer("net_due_days", "Net Due Days")
        .integer("discount_days", "Discount Days")
        .decimal("discount_percentage", "Discount %", 8, 4)
        .integer("discount_days_2", "Second Discount Days")
        .decimal("discount_percentage_2", "Second Discount %", 8, 4)
        .enumeration("day_of_month", "Due Day of Month", vec![
            "any", "1", "5", "10", "15", "20", "25",
        ])
        .integer("cutoff_day", "Cutoff Day")
        .boolean("is_active", "Active")
        .build()
}

/// Payment Schedule entity for installment terms
/// Oracle Fusion: Financials > Payment Terms > Payment Schedules
pub fn payment_schedule_definition() -> EntityDefinition {
    SchemaBuilder::new("payment_schedules", "Payment Schedule")
        .plural_label("Payment Schedules")
        .table_name("fin_payment_schedules")
        .description("Installment payment schedules with multiple due dates")
        .icon("tasks")
        .reference("payment_term_id", "Payment Term", "payment_terms")
        .integer("sequence", "Sequence")
        .integer("due_days", "Due Days")
        .decimal("percentage", "Percentage", 8, 4)
        .integer("discount_days", "Discount Days")
        .decimal("discount_percentage", "Discount %", 8, 4)
        .string("description", "Description")
        .build()
}

// ============================================================================
// Financial Statement Generation (Oracle Fusion: Financial Reporting Center)
// ============================================================================

/// Financial Report Template entity
/// Oracle Fusion: Financial Reporting > Report Templates
pub fn financial_report_template_definition() -> EntityDefinition {
    SchemaBuilder::new("financial_report_templates", "Report Template")
        .plural_label("Financial Report Templates")
        .table_name("fin_report_templates")
        .description("Templates for generating standard financial statements")
        .icon("file-alt")
        .required_string("code", "Template Code")
        .required_string("name", "Template Name")
        .enumeration("report_type", "Report Type", vec![
            "balance_sheet", "income_statement", "cash_flow", "trial_balance", "custom",
        ])
        .string("description", "Description")
        .string("base_currency_code", "Base Currency")
        .boolean("include_zero_balances", "Include Zero Balances")
        .boolean("show_beginning_balance", "Show Beginning Balance")
        .boolean("show_period_activity", "Show Period Activity")
        .boolean("show_ending_balance", "Show Ending Balance")
        .boolean("is_active", "Active")
        .build()
}

/// Financial Report Row Definition entity
/// Oracle Fusion: Financial Reporting > Row Definitions
pub fn financial_report_row_definition() -> EntityDefinition {
    SchemaBuilder::new("financial_report_rows", "Report Row")
        .plural_label("Financial Report Rows")
        .table_name("fin_report_rows")
        .description("Row definitions for financial statement line items")
        .icon("list")
        .reference("template_id", "Template", "financial_report_templates")
        .integer("sequence", "Sequence")
        .required_string("label", "Row Label")
        .enumeration("row_type", "Row Type", vec![
            "header", "account_range", "calculated", "total", "subtotal", "text",
        ])
        .string("account_range_from", "Account Range From")
        .string("account_range_to", "Account Range To")
        .string("calculation_formula", "Calculation Formula")
        .string("normal_balance", "Normal Balance")
        .boolean("show_on_report", "Show on Report")
        .integer("indent_level", "Indent Level")
        .build()
}

/// Generated Financial Report entity with workflow
/// Oracle Fusion: Financial Reporting > Generated Reports
pub fn generated_financial_report_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("financial_report_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("generated", "Generated")
        .working_state("reviewed", "Reviewed")
        .final_state("published", "Published")
        .final_state("archived", "Archived")
        .transition("draft", "generated", "generate")
        .transition("generated", "reviewed", "review")
        .transition("reviewed", "published", "publish")
        .transition("published", "archived", "archive")
        .build();

    SchemaBuilder::new("generated_financial_reports", "Generated Financial Report")
        .plural_label("Generated Financial Reports")
        .table_name("fin_generated_reports")
        .description("Generated financial statement reports")
        .icon("chart-bar")
        .required_string("report_number", "Report Number")
        .reference("template_id", "Template", "financial_report_templates")
        .enumeration("report_type", "Report Type", vec![
            "balance_sheet", "income_statement", "cash_flow", "trial_balance", "custom",
        ])
        .integer("fiscal_year", "Fiscal Year")
        .integer("period_number", "Period Number")
        .date("period_start_date", "Period Start")
        .date("period_end_date", "Period End")
        .string("base_currency_code", "Base Currency")
        .json("report_data", "Report Data")
        .json("row_results", "Row Results")
        .boolean("is_balanced", "Balanced")
        .string("reviewed_by", "Reviewed By")
        .date("reviewed_date", "Reviewed Date")
        .enumeration("status", "Status", vec![
            "draft", "generated", "reviewed", "published", "archived",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Tax Return & Filing (Oracle Fusion: Tax > Tax Filing)
// ============================================================================

/// Tax Filing Obligation entity
/// Oracle Fusion: Tax > Tax Filing > Filing Obligations
pub fn tax_filing_obligation_definition() -> EntityDefinition {
    SchemaBuilder::new("tax_filing_obligations", "Tax Filing Obligation")
        .plural_label("Tax Filing Obligations")
        .table_name("fin_tax_filing_obligations")
        .description("Tax filing obligations by jurisdiction and period")
        .icon("gavel")
        .reference("regime_id", "Tax Regime", "tax_regimes")
        .reference("jurisdiction_id", "Jurisdiction", "tax_jurisdictions")
        .required_string("obligation_code", "Obligation Code")
        .required_string("name", "Obligation Name")
        .enumeration("filing_frequency", "Filing Frequency", vec![
            "monthly", "quarterly", "semi_annually", "annually",
        ])
        .enumeration("filing_method", "Filing Method", vec![
            "electronic", "paper", "both",
        ])
        .integer("due_day_of_month", "Due Day of Month")
        .integer("due_days_after_period", "Due Days After Period")
        .string("tax_authority", "Tax Authority")
        .string("tax_authority_code", "Tax Authority Code")
        .string("filing_form", "Filing Form")
        .boolean("requires_payment", "Requires Payment")
        .boolean("is_active", "Active")
        .build()
}

/// Tax Return entity with workflow
/// Oracle Fusion: Tax > Tax Filing > Tax Returns
pub fn tax_return_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("tax_return_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("calculated", "Calculated")
        .working_state("reviewed", "Reviewed")
        .working_state("approved", "Approved")
        .final_state("filed", "Filed")
        .final_state("amended", "Amended")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "calculated", "calculate")
        .transition("calculated", "reviewed", "review")
        .transition("reviewed", "approved", "approve")
        .transition("approved", "filed", "file")
        .transition("filed", "amended", "amend")
        .transition("draft", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("tax_returns", "Tax Return")
        .plural_label("Tax Returns")
        .table_name("fin_tax_returns")
        .description("Tax returns prepared for filing with tax authorities")
        .icon("file-signature")
        .required_string("return_number", "Return Number")
        .reference("obligation_id", "Filing Obligation", "tax_filing_obligations")
        .reference("regime_id", "Tax Regime", "tax_regimes")
        .reference("jurisdiction_id", "Jurisdiction", "tax_jurisdictions")
        .integer("fiscal_year", "Fiscal Year")
        .integer("period_number", "Period Number")
        .date("period_start", "Period Start")
        .date("period_end", "Period End")
        .date("filing_due_date", "Filing Due Date")
        .date("filed_date", "Filed Date")
        .string("filing_confirmation", "Filing Confirmation")
        .currency("total_taxable_amount", "Total Taxable Amount", "USD")
        .currency("total_tax_amount", "Total Tax Amount", "USD")
        .currency("total_tax_payable", "Tax Payable", "USD")
        .currency("total_tax_refund", "Tax Refund", "USD")
        .currency("penalty_amount", "Penalty Amount", "USD")
        .currency("interest_amount", "Interest Amount", "USD")
        .string("tax_authority_reference", "Tax Authority Reference")
        .json("line_details", "Line Details")
        .reference("prepared_by", "Prepared By", "employees")
        .reference("reviewed_by", "Reviewed By", "employees")
        .reference("approved_by", "Approved By", "employees")
        .string("amendment_reason", "Amendment Reason")
        .reference("original_return_id", "Original Return", "tax_returns")
        .rich_text("notes", "Notes")
        .enumeration("status", "Status", vec![
            "draft", "calculated", "reviewed", "approved", "filed", "amended", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

/// Tax Payment entity
/// Oracle Fusion: Tax > Tax Filing > Tax Payments
pub fn tax_payment_definition() -> EntityDefinition {
    SchemaBuilder::new("tax_payments", "Tax Payment")
        .plural_label("Tax Payments")
        .table_name("fin_tax_payments")
        .description("Tax payments made to tax authorities")
        .icon("money-check")
        .reference("tax_return_id", "Tax Return", "tax_returns")
        .required_string("payment_number", "Payment Number")
        .reference("bank_account_id", "Bank Account", "bank_accounts")
        .date("payment_date", "Payment Date")
        .currency("payment_amount", "Payment Amount", "USD")
        .string("currency_code", "Currency Code")
        .enumeration("payment_method", "Payment Method", vec![
            "wire", "ach", "check", "electronic",
        ])
        .string("tax_authority_reference", "Tax Authority Reference")
        .string("confirmation_number", "Confirmation Number")
        .enumeration("status", "Status", vec![
            "pending", "processed", "confirmed", "reversed",
        ])
        .build()
}

// ============================================================================
// ============================================================================
// Recurring Journals (Oracle Fusion: General Ledger > Recurring Journals)
// ============================================================================

/// Recurring Journal Template entity
/// Oracle Fusion: GL > Journals > Recurring Journals > Define Template
pub fn recurring_journal_template_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("recurring_journal_workflow", "draft")
        .initial_state("draft", "Draft")
        .final_state("active", "Active")
        .final_state("inactive", "Inactive")
        .transition("draft", "active", "activate")
        .transition("active", "inactive", "deactivate")
        .transition("inactive", "active", "reactivate")
        .build();

    SchemaBuilder::new("recurring_journal_templates", "Recurring Journal Template")
        .plural_label("Recurring Journal Templates")
        .table_name("fin_recurring_journal_templates")
        .description("Templates for automatically generating recurring journal entries")
        .icon("redo")
        .required_string("template_number", "Template Number")
        .required_string("name", "Template Name")
        .string("description", "Description")
        .enumeration("recurrence_type", "Recurrence Type", vec![
            "daily", "weekly", "monthly", "quarterly", "yearly",
        ])
        .integer("recurrence_interval", "Recurrence Interval")
        .enumeration("journal_type", "Journal Type", vec![
            "standard", "statistical", "budget",
        ])
        .enumeration("amount_type", "Amount Type", vec![
            "fixed", "variable", "calculated",
        ])
        .currency("fixed_amount", "Fixed Amount", "USD")
        .string("calculation_formula", "Calculation Formula")
        .string("currency_code", "Currency Code")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .date("last_generated_date", "Last Generated Date")
        .date("next_generation_date", "Next Generation Date")
        .integer("times_generated", "Times Generated")
        .integer("max_generations", "Max Generations")
        .boolean("auto_post", "Auto Post")
        .boolean("allow_edit_before_post", "Allow Edit Before Post")
        .enumeration("status", "Status", vec![
            "draft", "active", "inactive",
        ])
        .workflow(workflow)
        .build()
}

/// Recurring Journal Line entity
/// Oracle Fusion: GL > Journals > Recurring Journal Lines
pub fn recurring_journal_line_definition() -> EntityDefinition {
    SchemaBuilder::new("recurring_journal_lines", "Recurring Journal Line")
        .plural_label("Recurring Journal Lines")
        .table_name("fin_recurring_journal_lines")
        .description("Line items within a recurring journal template")
        .icon("list")
        .reference("template_id", "Template", "recurring_journal_templates")
        .integer("line_number", "Line Number")
        .string("account_code", "Account Code")
        .string("account_name", "Account Name")
        .enumeration("line_type", "Line Type", vec![
            "debit", "credit",
        ])
        .enumeration("amount_type", "Amount Type", vec![
            "fixed", "variable", "calculated",
        ])
        .currency("fixed_amount", "Fixed Amount", "USD")
        .string("calculation_rule", "Calculation Rule")
        .string("description", "Description")
        .string("cost_center", "Cost Center")
        .string("department", "Department")
        .reference("project_id", "Project", "projects")
        .build()
}

// ============================================================================
// Allocations / Mass Allocations (Oracle Fusion: GL > Allocations)
// ============================================================================

/// Allocation Rule entity with workflow
/// Oracle Fusion: GL > Allocations > Define Allocation Rule
pub fn allocation_rule_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("allocation_rule_workflow", "draft")
        .initial_state("draft", "Draft")
        .final_state("active", "Active")
        .final_state("inactive", "Inactive")
        .transition("draft", "active", "activate")
        .transition("active", "inactive", "deactivate")
        .transition("inactive", "active", "reactivate")
        .build();

    SchemaBuilder::new("allocation_rules", "Allocation Rule")
        .plural_label("Allocation Rules")
        .table_name("fin_allocation_rules")
        .description("Rules for allocating costs across cost centers, projects, or departments")
        .icon("project-diagram")
        .required_string("rule_number", "Rule Number")
        .required_string("name", "Rule Name")
        .string("description", "Description")
        .enumeration("allocation_type", "Allocation Type", vec![
            "mass_allocation", "recurring_allocation", "statistical_allocation",
        ])
        .enumeration("allocation_basis", "Allocation Basis", vec![
            "fixed_percentage", "statistical", "ratio", "equal_share",
        ])
        .string("source_pool_account", "Source Pool Account")
        .string("target_account_prefix", "Target Account Prefix")
        .string("offset_account", "Offset Account")
        .string("currency_code", "Currency Code")
        .enumeration("recurrence", "Recurrence", vec![
            "manual", "monthly", "quarterly", "yearly",
        ])
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .date("last_run_date", "Last Run Date")
        .integer("times_run", "Times Run")
        .boolean("auto_post", "Auto Post")
        .enumeration("status", "Status", vec![
            "draft", "active", "inactive",
        ])
        .workflow(workflow)
        .build()
}

/// Allocation Line entity
/// Oracle Fusion: GL > Allocations > Allocation Lines
pub fn allocation_line_definition() -> EntityDefinition {
    SchemaBuilder::new("allocation_lines", "Allocation Line")
        .plural_label("Allocation Lines")
        .table_name("fin_allocation_lines")
        .description("Individual allocation targets with basis and percentage")
        .icon("list")
        .reference("rule_id", "Rule", "allocation_rules")
        .integer("line_number", "Line Number")
        .string("target_account_code", "Target Account")
        .string("target_account_name", "Target Account Name")
        .string("target_cost_center", "Target Cost Center")
        .string("target_department", "Target Department")
        .decimal("percentage", "Percentage", 10, 6)
        .string("basis_value_source", "Basis Value Source")
        .currency("basis_amount", "Basis Amount", "USD")
        .string("description", "Description")
        .build()
}

// ============================================================================
// Funds Reservation / Budgetary Control (Oracle Fusion: Budgetary Control)
// ============================================================================

/// Funds Reservation entity with workflow
/// Oracle Fusion: General Ledger > Budgetary Control > Funds Reservation
pub fn funds_reservation_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("funds_reservation_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("reserved", "Reserved")
        .working_state("partially_consumed", "Partially Consumed")
        .final_state("fully_consumed", "Fully Consumed")
        .final_state("cancelled", "Cancelled")
        .final_state("expired", "Expired")
        .transition("draft", "reserved", "reserve")
        .transition("reserved", "partially_consumed", "partial_consume")
        .transition("reserved", "fully_consumed", "full_consume")
        .transition("reserved", "cancelled", "cancel")
        .transition("partially_consumed", "fully_consumed", "full_consume")
        .transition("reserved", "expired", "expire")
        .build();

    SchemaBuilder::new("funds_reservations", "Funds Reservation")
        .plural_label("Funds Reservations")
        .table_name("fin_funds_reservations")
        .description("Budget funds reservations for expenditure control")
        .icon("shield-alt")
        .required_string("reservation_number", "Reservation Number")
        .string("description", "Description")
        .string("budget_code", "Budget Code")
        .string("account_code", "Account Code")
        .string("cost_center", "Cost Center")
        .string("department", "Department")
        .string("fund_code", "Fund Code")
        .currency("reserved_amount", "Reserved Amount", "USD")
        .currency("consumed_amount", "Consumed Amount", "USD")
        .currency("remaining_amount", "Remaining Amount", "USD")
        .date("reservation_date", "Reservation Date")
        .date("expiry_date", "Expiry Date")
        .string("source_entity", "Source Entity")
        .string("source_id", "Source ID")
        .reference("requested_by", "Requested By", "employees")
        .reference("approved_by", "Approved By", "employees")
        .enumeration("status", "Status", vec![
            "draft", "reserved", "partially_consumed", "fully_consumed", "cancelled", "expired",
        ])
        .workflow(workflow)
        .build()
}

/// Funds Check Result entity
/// Oracle Fusion: Budgetary Control > Funds Check Results
pub fn funds_check_result_definition() -> EntityDefinition {
    SchemaBuilder::new("funds_check_results", "Funds Check Result")
        .plural_label("Funds Check Results")
        .table_name("fin_funds_check_results")
        .description("Results of budgetary funds availability checks")
        .icon("check-circle")
        .string("check_type", "Check Type")
        .string("entity_type", "Entity Type")
        .string("entity_id", "Entity ID")
        .string("account_code", "Account Code")
        .string("budget_code", "Budget Code")
        .currency("requested_amount", "Requested Amount", "USD")
        .currency("budget_amount", "Budget Amount", "USD")
        .currency("reserved_amount", "Reserved Amount", "USD")
        .currency("consumed_amount", "Consumed Amount", "USD")
        .currency("available_amount", "Available Amount", "USD")
        .enumeration("result", "Result", vec![
            "pass", "warning", "fail",
        ])
        .string("message", "Message")
        .date("check_date", "Check Date")
        .build()
}

// ============================================================================
// Journal Import (Oracle Fusion: GL > Journal Import)
// ============================================================================

/// Journal Import Request entity with workflow
/// Oracle Fusion: GL > Journal Import > Import Journals
pub fn journal_import_request_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("journal_import_workflow", "uploaded")
        .initial_state("uploaded", "Uploaded")
        .working_state("validating", "Validating")
        .working_state("validated", "Validated")
        .working_state("importing", "Importing")
        .final_state("completed", "Completed")
        .final_state("failed", "Failed")
        .final_state("cancelled", "Cancelled")
        .transition("uploaded", "validating", "validate")
        .transition("validating", "validated", "validation_pass")
        .transition("validating", "failed", "validation_fail")
        .transition("validated", "importing", "import")
        .transition("importing", "completed", "complete")
        .transition("importing", "failed", "fail")
        .transition("uploaded", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("journal_import_requests", "Journal Import Request")
        .plural_label("Journal Import Requests")
        .table_name("fin_journal_import_requests")
        .description("Requests to import journal entries from external systems")
        .icon("upload")
        .required_string("import_number", "Import Number")
        .required_string("source", "Source")
        .enumeration("import_format", "Format", vec![
            "csv", "xml", "json", "flat_file", "api",
        ])
        .string("ledger_code", "Ledger Code")
        .string("currency_code", "Currency Code")
        .date("accounting_date", "Accounting Date")
        .date("gl_date", "GL Date")
        .integer("total_rows", "Total Rows")
        .integer("valid_rows", "Valid Rows")
        .integer("imported_rows", "Imported Rows")
        .integer("error_rows", "Error Rows")
        .integer("skipped_rows", "Skipped Rows")
        .boolean("auto_post", "Auto Post")
        .boolean("stop_on_error", "Stop on Error")
        .string("original_filename", "Original Filename")
        .string("field_mapping", "Field Mapping")
        .json("validation_errors", "Validation Errors")
        .json("import_errors", "Import Errors")
        .reference("submitted_by", "Submitted By", "employees")
        .enumeration("status", "Status", vec![
            "uploaded", "validating", "validated", "importing", "completed", "failed", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Landed Cost Management (Oracle Fusion: Cost Management > Landed Cost)
// ============================================================================

/// Landed Cost Template entity
/// Oracle Fusion: Cost Management > Landed Cost > Templates
pub fn landed_cost_template_definition() -> EntityDefinition {
    SchemaBuilder::new("landed_cost_templates", "Landed Cost Template")
        .plural_label("Landed Cost Templates")
        .table_name("fin_landed_cost_templates")
        .description("Templates defining landed cost components for imported goods")
        .icon("truck")
        .required_string("code", "Template Code")
        .required_string("name", "Template Name")
        .string("description", "Description")
        .string("currency_code", "Currency Code")
        .boolean("is_active", "Active")
        .build()
}

/// Landed Cost Component entity
/// Oracle Fusion: Cost Management > Landed Cost > Cost Components
pub fn landed_cost_component_definition() -> EntityDefinition {
    SchemaBuilder::new("landed_cost_components", "Landed Cost Component")
        .plural_label("Landed Cost Components")
        .table_name("fin_landed_cost_components")
        .description("Individual cost components (freight, insurance, duty, etc.)")
        .icon("puzzle-piece")
        .reference("template_id", "Template", "landed_cost_templates")
        .required_string("code", "Component Code")
        .required_string("name", "Component Name")
        .enumeration("component_type", "Component Type", vec![
            "freight", "insurance", "duty", "customs_fee",
            "handling", "storage", "brokerage", "other",
        ])
        .enumeration("allocation_method", "Allocation Method", vec![
            "quantity", "weight", "volume", "value", "equal",
        ])
        .decimal("rate_percentage", "Rate %", 10, 6)
        .currency("flat_amount", "Flat Amount", "USD")
        .string("charge_account_code", "Charge Account")
        .integer("priority", "Priority")
        .boolean("is_active", "Active")
        .build()
}

/// Landed Cost Assignment entity with workflow
/// Oracle Fusion: Cost Management > Landed Cost > Assignments
pub fn landed_cost_assignment_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("landed_cost_assignment_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("estimated", "Estimated")
        .working_state("actualized", "Actualized")
        .final_state("posted", "Posted")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "estimated", "estimate")
        .transition("estimated", "actualized", "actualize")
        .transition("actualized", "posted", "post")
        .transition("draft", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("landed_cost_assignments", "Landed Cost Assignment")
        .plural_label("Landed Cost Assignments")
        .table_name("fin_landed_cost_assignments")
        .description("Assignment of landed costs to receipt lines")
        .icon("link")
        .required_string("assignment_number", "Assignment Number")
        .reference("template_id", "Template", "landed_cost_templates")
        .string("receipt_number", "Receipt Number")
        .string("purchase_order_number", "PO Number")
        .reference("item_id", "Item", "items")
        .string("item_name", "Item Name")
        .currency("item_value", "Item Value", "USD")
        .currency("total_landed_cost", "Total Landed Cost", "USD")
        .currency("estimated_cost", "Estimated Cost", "USD")
        .currency("actual_cost", "Actual Cost", "USD")
        .currency("variance_amount", "Variance Amount", "USD")
        .string("currency_code", "Currency Code")
        .enumeration("status", "Status", vec![
            "draft", "estimated", "actualized", "posted", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Transfer Pricing (Oracle Fusion: Intercompany > Transfer Pricing)
// ============================================================================

/// Transfer Pricing Policy entity
/// Oracle Fusion: Intercompany > Transfer Pricing > Policies
pub fn transfer_pricing_policy_definition() -> EntityDefinition {
    SchemaBuilder::new("transfer_pricing_policies", "Transfer Pricing Policy")
        .plural_label("Transfer Pricing Policies")
        .table_name("fin_transfer_pricing_policies")
        .description("Policies governing intercompany transfer pricing (arm's length compliance)")
        .icon("balance-scale-left")
        .required_string("code", "Policy Code")
        .required_string("name", "Policy Name")
        .string("description", "Description")
        .enumeration("pricing_method", "Pricing Method", vec![
            "comparable_uncontrolled", "resale_price", "cost_plus",
            "profit_split", "tnmm", "other",
        ])
        .decimal("standard_margin_pct", "Standard Margin %", 8, 4)
        .string("currency_code", "Currency Code")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Transfer Pricing Transaction entity
/// Oracle Fusion: Intercompany > Transfer Pricing > Transactions
pub fn transfer_pricing_transaction_definition() -> EntityDefinition {
    SchemaBuilder::new("transfer_pricing_transactions", "Transfer Pricing Transaction")
        .plural_label("Transfer Pricing Transactions")
        .table_name("fin_transfer_pricing_transactions")
        .description("Individual intercompany transactions with transfer pricing")
        .icon("exchange-alt")
        .required_string("transaction_number", "Transaction Number")
        .reference("policy_id", "Policy", "transfer_pricing_policies")
        .reference("from_entity_id", "From Entity", "organizations")
        .string("from_entity_name", "From Entity Name")
        .reference("to_entity_id", "To Entity", "organizations")
        .string("to_entity_name", "To Entity Name")
        .reference("item_id", "Item", "items")
        .string("item_name", "Item Name")
        .decimal("quantity", "Quantity", 18, 4)
        .currency("unit_price", "Unit Price", "USD")
        .currency("transfer_price", "Transfer Price", "USD")
        .currency("total_amount", "Total Amount", "USD")
        .string("currency_code", "Currency Code")
        .enumeration("arm_length_result", "Arm's Length Result", vec![
            "within_range", "below_range", "above_range",
        ])
        .string("benchmark_study_reference", "Benchmark Study")
        .date("transaction_date", "Transaction Date")
        .enumeration("status", "Status", vec![
            "pending", "approved", "disputed", "completed",
        ])
        .build()
}

// ============================================================================
// AutoInvoice (Oracle Fusion: AR > AutoInvoice)
// ============================================================================

/// AutoInvoice Rule entity
/// Oracle Fusion: Receivables > AutoInvoice > Transaction Sources
pub fn autoinvoice_rule_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("autoinvoice_rule_workflow", "draft")
        .initial_state("draft", "Draft")
        .final_state("active", "Active")
        .final_state("inactive", "Inactive")
        .transition("draft", "active", "activate")
        .transition("active", "inactive", "deactivate")
        .build();

    SchemaBuilder::new("autoinvoice_rules", "AutoInvoice Rule")
        .plural_label("AutoInvoice Rules")
        .table_name("fin_autoinvoice_rules")
        .description("Rules for automatically generating AR invoices from source transactions")
        .icon("magic")
        .required_string("code", "Rule Code")
        .required_string("name", "Rule Name")
        .string("description", "Description")
        .enumeration("source_type", "Source Type", vec![
            "sales_order", "service_completion", "project_milestone",
            "recurring_contract", "usage_based",
        ])
        .enumeration("invoice_type", "Invoice Type", vec![
            "invoice", "debit_memo", "credit_memo",
        ])
        .boolean("group_by_customer", "Group by Customer")
        .boolean("group_by_project", "Group by Project")
        .string("default_payment_terms", "Default Payment Terms")
        .string("default_revenue_account", "Default Revenue Account")
        .string("default_tax_code", "Default Tax Code")
        .string("currency_code", "Currency Code")
        .boolean("auto_post_to_gl", "Auto Post to GL")
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .enumeration("status", "Status", vec![
            "draft", "active", "inactive",
        ])
        .workflow(workflow)
        .build()
}

/// AutoInvoice Run entity with workflow
/// Oracle Fusion: Receivables > AutoInvoice > Process
pub fn autoinvoice_run_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("autoinvoice_run_workflow", "pending")
        .initial_state("pending", "Pending")
        .working_state("processing", "Processing")
        .final_state("completed", "Completed")
        .final_state("failed", "Failed")
        .final_state("cancelled", "Cancelled")
        .transition("pending", "processing", "process")
        .transition("processing", "completed", "complete")
        .transition("processing", "failed", "fail")
        .transition("pending", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("autoinvoice_runs", "AutoInvoice Run")
        .plural_label("AutoInvoice Runs")
        .table_name("fin_autoinvoice_runs")
        .description("Execution runs of the AutoInvoice process")
        .icon("play-circle")
        .required_string("run_number", "Run Number")
        .reference("rule_id", "Rule", "autoinvoice_rules")
        .date("run_date", "Run Date")
        .date("invoice_date", "Invoice Date")
        .integer("source_transactions_processed", "Sources Processed")
        .integer("invoices_generated", "Invoices Generated")
        .integer("invoices_failed", "Invoices Failed")
        .integer("lines_generated", "Lines Generated")
        .currency("total_amount_generated", "Total Generated", "USD")
        .string("currency_code", "Currency Code")
        .json("errors", "Errors")
        .reference("submitted_by", "Submitted By", "employees")
        .enumeration("status", "Status", vec![
            "pending", "processing", "completed", "failed", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Currency Revaluation (Oracle Fusion: GL > Currency Revaluation)
// ============================================================================

/// Currency Revaluation entity with workflow
/// Oracle Fusion: GL > Currency > Revaluation
pub fn currency_revaluation_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("currency_revaluation_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("calculated", "Calculated")
        .working_state("reviewed", "Reviewed")
        .final_state("posted", "Posted")
        .final_state("reversed", "Reversed")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "calculated", "calculate")
        .transition("calculated", "reviewed", "review")
        .transition("reviewed", "posted", "post")
        .transition("posted", "reversed", "reverse")
        .transition("draft", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("currency_revaluations", "Currency Revaluation")
        .plural_label("Currency Revaluations")
        .table_name("fin_currency_revaluations")
        .description("Month-end currency revaluation of foreign currency balances")
        .icon("sync")
        .required_string("revaluation_number", "Revaluation Number")
        .string("currency_code", "Revalued Currency")
        .string("base_currency_code", "Base Currency")
        .enumeration("rate_type", "Rate Type", vec![
            "period_end", "spot", "daily", "corporate",
        ])
        .decimal("revaluation_rate", "Revaluation Rate", 18, 10)
        .date("revaluation_date", "Revaluation Date")
        .date("gl_date", "GL Date")
        .integer("fiscal_year", "Fiscal Year")
        .integer("period_number", "Period Number")
        .integer("accounts_revalued", "Accounts Revalued")
        .currency("total_unrealized_gain", "Total Unrealized Gain", "USD")
        .currency("total_unrealized_loss", "Total Unrealized Loss", "USD")
        .string("unrealized_gain_account", "Gain Account")
        .string("unrealized_loss_account", "Loss Account")
        .json("revaluation_details", "Revaluation Details")
        .enumeration("status", "Status", vec![
            "draft", "calculated", "reviewed", "posted", "reversed", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Netting (Oracle Fusion: Treasury > Netting)
// ============================================================================

/// Netting Agreement entity
/// Oracle Fusion: Treasury > Netting > Agreements
pub fn netting_agreement_definition() -> EntityDefinition {
    SchemaBuilder::new("netting_agreements", "Netting Agreement")
        .plural_label("Netting Agreements")
        .table_name("fin_netting_agreements")
        .description("Bilateral netting agreements between business partners")
        .icon("handshake")
        .required_string("agreement_number", "Agreement Number")
        .reference("party_a_id", "Party A", "organizations")
        .string("party_a_name", "Party A Name")
        .reference("party_b_id", "Party B", "organizations")
        .string("party_b_name", "Party B Name")
        .enumeration("netting_type", "Netting Type", vec![
            "bilateral", "multilateral",
        ])
        .enumeration("settlement_currency", "Settlement Currency", vec![
            "USD", "EUR", "GBP", "JPY",
        ])
        .string("settlement_account_code", "Settlement Account")
        .enumeration("frequency", "Frequency", vec![
            "daily", "weekly", "monthly",
        ])
        .date("effective_from", "Effective From")
        .date("effective_to", "Effective To")
        .boolean("is_active", "Active")
        .build()
}

/// Netting Batch entity with workflow
/// Oracle Fusion: Treasury > Netting > Batches
pub fn netting_batch_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("netting_batch_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("calculated", "Calculated")
        .working_state("approved", "Approved")
        .final_state("settled", "Settled")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "calculated", "calculate")
        .transition("calculated", "approved", "approve")
        .transition("approved", "settled", "settle")
        .transition("draft", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("netting_batches", "Netting Batch")
        .plural_label("Netting Batches")
        .table_name("fin_netting_batches")
        .description("Netting batches for offsetting payables and receivables")
        .icon("compress-arrows-alt")
        .required_string("batch_number", "Batch Number")
        .reference("agreement_id", "Agreement", "netting_agreements")
        .date("netting_date", "Netting Date")
        .date("settlement_date", "Settlement Date")
        .string("settlement_currency", "Settlement Currency")
        .currency("total_payables", "Total Payables", "USD")
        .currency("total_receivables", "Total Receivables", "USD")
        .currency("net_amount", "Net Amount", "USD")
        .enumeration("net_direction", "Net Direction", vec![
            "party_a_owes", "party_b_owes", "balanced",
        ])
        .integer("transactions_included", "Transactions Included")
        .reference("approved_by", "Approved By", "employees")
        .enumeration("status", "Status", vec![
            "draft", "calculated", "approved", "settled", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Subscription Management (Oracle Fusion: Revenue > Subscription Management)
// ============================================================================

/// Subscription Product entity
/// Oracle Fusion: Subscription Management > Products
pub fn subscription_product_definition() -> EntityDefinition {
    SchemaBuilder::new("subscription_products", "Subscription Product")
        .plural_label("Subscription Products")
        .table_name("fin_subscription_products")
        .description("Subscription product/service definitions with billing plans")
        .icon("box")
        .required_string("code", "Product Code")
        .required_string("name", "Product Name")
        .string("description", "Description")
        .enumeration("billing_frequency", "Billing Frequency", vec![
            "monthly", "quarterly", "semi_annually", "annually",
        ])
        .enumeration("pricing_model", "Pricing Model", vec![
            "flat_rate", "per_unit", "tiered", "volume", "usage",
        ])
        .currency("base_price", "Base Price", "USD")
        .string("currency_code", "Currency Code")
        .integer("minimum_term_months", "Minimum Term (Months)")
        .boolean("auto_renew", "Auto Renew")
        .integer("renewal_term_months", "Renewal Term (Months)")
        .boolean("is_active", "Active")
        .build()
}

/// Subscription Contract entity with workflow
/// Oracle Fusion: Subscription Management > Subscriptions
pub fn subscription_contract_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("subscription_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("active", "Active")
        .working_state("suspended", "Suspended")
        .working_state("in_renewal", "In Renewal")
        .final_state("cancelled", "Cancelled")
        .final_state("expired", "Expired")
        .final_state("terminated", "Terminated")
        .transition("draft", "active", "activate")
        .transition("active", "suspended", "suspend")
        .transition("suspended", "active", "reactivate")
        .transition("active", "in_renewal", "start_renewal")
        .transition("in_renewal", "active", "renew")
        .transition("active", "cancelled", "cancel")
        .transition("active", "expired", "expire")
        .transition("active", "terminated", "terminate")
        .build();

    SchemaBuilder::new("subscription_contracts", "Subscription Contract")
        .plural_label("Subscription Contracts")
        .table_name("fin_subscription_contracts")
        .description("Customer subscription contracts with recurring billing (ASC 606)")
        .icon("file-contract")
        .required_string("contract_number", "Contract Number")
        .reference("customer_id", "Customer", "customers")
        .string("customer_name", "Customer Name")
        .reference("product_id", "Product", "subscription_products")
        .string("product_name", "Product Name")
        .enumeration("pricing_model", "Pricing Model", vec![
            "flat_rate", "per_unit", "tiered", "volume", "usage",
        ])
        .currency("contract_value", "Contract Value", "USD")
        .currency("monthly_recurring_revenue", "Monthly Recurring Revenue", "USD")
        .integer("quantity", "Quantity")
        .string("currency_code", "Currency Code")
        .date("start_date", "Start Date")
        .date("end_date", "End Date")
        .date("renewal_date", "Renewal Date")
        .date("cancellation_date", "Cancellation Date")
        .date("termination_date", "Termination Date")
        .integer("term_months", "Term (Months)")
        .boolean("auto_renew", "Auto Renew")
        .enumeration("revenue_recognition_method", "Revenue Method", vec![
            "straight_line", "over_time", "point_in_time",
        ])
        .currency("recognized_revenue", "Recognized Revenue", "USD")
        .currency("deferred_revenue", "Deferred Revenue", "USD")
        .reference("revenue_policy_id", "Revenue Policy", "revenue_policies")
        .enumeration("status", "Status", vec![
            "draft", "active", "suspended", "in_renewal", "cancelled", "expired", "terminated",
        ])
        .workflow(workflow)
        .build()
}

/// Subscription Billing Event entity with workflow
/// Oracle Fusion: Subscription Management > Billing Events
pub fn subscription_billing_event_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("subscription_billing_workflow", "scheduled")
        .initial_state("scheduled", "Scheduled")
        .working_state("invoiced", "Invoiced")
        .working_state("partially_invoiced", "Partially Invoiced")
        .final_state("completed", "Completed")
        .final_state("cancelled", "Cancelled")
        .transition("scheduled", "invoiced", "invoice")
        .transition("scheduled", "partially_invoiced", "partial_invoice")
        .transition("invoiced", "completed", "complete")
        .transition("partially_invoiced", "completed", "complete")
        .transition("scheduled", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("subscription_billing_events", "Subscription Billing Event")
        .plural_label("Subscription Billing Events")
        .table_name("fin_subscription_billing_events")
        .description("Recurring billing events for subscription contracts")
        .icon("calendar-alt")
        .reference("contract_id", "Contract", "subscription_contracts")
        .reference("product_id", "Product", "subscription_products")
        .integer("billing_period_number", "Billing Period")
        .date("billing_start_date", "Billing Start")
        .date("billing_end_date", "Billing End")
        .date("billing_date", "Billing Date")
        .currency("billing_amount", "Billing Amount", "USD")
        .currency("recognized_revenue", "Recognized Revenue", "USD")
        .currency("deferred_revenue", "Deferred Revenue", "USD")
        .string("currency_code", "Currency Code")
        .string("invoice_number", "Invoice Number")
        .enumeration("status", "Status", vec![
            "scheduled", "invoiced", "partially_invoiced", "completed", "cancelled",
        ])
        .workflow(workflow)
        .build()
}

// ============================================================================
// Journal Reversal (Oracle Fusion: General Ledger > Journal Reversal)
// ============================================================================

/// Journal Reversal Request entity with workflow
/// Oracle Fusion: GL > Journals > Reverse Journals
pub fn journal_reversal_request_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("journal_reversal_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted")
        .working_state("approved", "Approved")
        .final_state("processed", "Processed")
        .final_state("rejected", "Rejected")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "rejected", "reject")
        .transition("approved", "processed", "process")
        .transition("draft", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("journal_reversal_requests", "Journal Reversal Request")
        .plural_label("Journal Reversal Requests")
        .table_name("fin_journal_reversal_requests")
        .description("Requests to reverse posted journal entries with full audit trail")
        .icon("undo")
        .required_string("reversal_number", "Reversal Number")
        .reference("original_entry_id", "Original Journal Entry", "journal_entries")
        .string("original_entry_number", "Original Entry Number")
        .date("original_entry_date", "Original Entry Date")
        .date("reversal_date", "Reversal Date")
        .date("reversal_gl_date", "Reversal GL Date")
        .enumeration("reversal_method", "Reversal Method", vec![
            "switch_dr_cr", "sign_reverse", "switch_signs",
        ])
        .enumeration("reversal_reason", "Reversal Reason", vec![
            "error_correction", "period_adjustment", "duplicate_entry",
            "reclassification", "management_decision", "other",
        ])
        .string("reason_description", "Reason Description")
        .currency("total_debit", "Total Debit", "USD")
        .currency("total_credit", "Total Credit", "USD")
        .reference("requested_by", "Requested By", "employees")
        .date("requested_date", "Requested Date")
        .reference("approved_by", "Approved By", "employees")
        .date("approved_date", "Approved Date")
        .string("approved_reason", "Approval Notes")
        .reference("processed_by", "Processed By", "employees")
        .date("processed_date", "Processed Date")
        .string("reversal_entry_number", "Reversal Entry Number")
        .reference("reversal_entry_id", "Reversal Entry", "journal_entries")
        .enumeration("status", "Status", vec![
            "draft", "submitted", "approved", "processed", "rejected", "cancelled",
        ])
        .rich_text("notes", "Notes")
        .workflow(workflow)
        .build()
}
