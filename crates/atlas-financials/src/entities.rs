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
