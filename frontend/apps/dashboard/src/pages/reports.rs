//! Reports page - lists available reports with categories

use leptos::*;

/// Available reports
struct ReportDef {
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    category: &'static str,
    entity: &'static str,
}

/// Reports page component
#[component]
pub fn ReportsPage() -> impl IntoView {
    let reports = vec![
        ReportDef {
            icon: "💰",
            title: "Financial Summary",
            description: "Revenue, expenses, and profit overview",
            category: "Finance",
            entity: "invoices",
        },
        ReportDef {
            icon: "👥",
            title: "Employee Headcount",
            description: "Current employee count by department and status",
            category: "HCM",
            entity: "employees",
        },
        ReportDef {
            icon: "📊",
            title: "Sales Pipeline",
            description: "Open opportunities and conversion rates",
            category: "Sales",
            entity: "orders",
        },
        ReportDef {
            icon: "📦",
            title: "Inventory Status",
            description: "Product stock levels and reorder alerts",
            category: "SCM",
            entity: "products",
        },
        ReportDef {
            icon: "📋",
            title: "Project Status",
            description: "Active projects progress and milestones",
            category: "Projects",
            entity: "projects",
        },
        ReportDef {
            icon: "🧾",
            title: "Outstanding Invoices",
            description: "Unpaid and overdue invoices summary",
            category: "Finance",
            entity: "invoices",
        },
        ReportDef {
            icon: "📈",
            title: "Customer Analytics",
            description: "Customer growth, retention, and lifetime value",
            category: "CRM",
            entity: "customers",
        },
        ReportDef {
            icon: "⏱️",
            title: "Timesheet Report",
            description: "Employee time tracking and utilization",
            category: "HCM",
            entity: "projects",
        },
    ];

    let categories: Vec<&str> = reports.iter().map(|r| r.category).collect::<std::collections::HashSet<_>>().into_iter().collect();

    view! {
        <div>
            <div class="page-header">
                <div class="page-header-left">
                    <h1>"Reports"</h1>
                    <p class="subtitle">"Business intelligence and analytics"</p>
                </div>
            </div>

            // Report cards by category
            {categories.into_iter().map(|category| {
                let category_reports: Vec<&ReportDef> = reports.iter().filter(|r| r.category == category).collect();
                view! {
                    <div class="mb-6">
                        <h2 class="text-lg font-semibold mb-4">{category}</h2>
                        <div class="report-grid">
                            {category_reports.into_iter().map(|report| {
                                view! {
                                    <a href=format!("/{}", report.entity) class="report-card" style="text-decoration: none; color: inherit">
                                        <div class="report-card-icon">{report.icon}</div>
                                        <h3>{report.title}</h3>
                                        <p>{report.description}</p>
                                    </a>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
