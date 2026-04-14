//! Dashboard page - main overview with KPI cards and recent activity

use leptos::*;
use crate::api::client::{fetch_dashboard_stats, fetch_records, DashboardStats};

/// Dashboard page component
#[component]
pub fn DashboardPage() -> impl IntoView {
    let (stats, set_stats) = create_signal(DashboardStats {
        employees: 0,
        customers: 0,
        open_orders: 0,
        active_projects: 0,
        recent_activity: vec![],
    });
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);

    // Load dashboard data
    let load_stats = create_action(move |_: &()| async move {
        set_loading.set(true);
        set_error.set(None);

        match fetch_dashboard_stats().await {
            Ok(data) => {
                set_stats.set(data);
            }
            Err(_) => {
                // Try loading counts from individual entities
                let employees: crate::api::client::PaginatedResponse<serde_json::Value> = fetch_records("employees", 0, 1, None).await
                    .unwrap_or_else(|_| crate::api::client::PaginatedResponse { data: vec![], meta: crate::api::client::PaginationMeta { total: 0, offset: 0, limit: 1, has_more: false } });
                let customers: crate::api::client::PaginatedResponse<serde_json::Value> = fetch_records("customers", 0, 1, None).await
                    .unwrap_or_else(|_| crate::api::client::PaginatedResponse { data: vec![], meta: crate::api::client::PaginationMeta { total: 0, offset: 0, limit: 1, has_more: false } });
                let orders: crate::api::client::PaginatedResponse<serde_json::Value> = fetch_records("orders", 0, 1, None).await
                    .unwrap_or_else(|_| crate::api::client::PaginatedResponse { data: vec![], meta: crate::api::client::PaginationMeta { total: 0, offset: 0, limit: 1, has_more: false } });
                let projects: crate::api::client::PaginatedResponse<serde_json::Value> = fetch_records("projects", 0, 1, None).await
                    .unwrap_or_else(|_| crate::api::client::PaginatedResponse { data: vec![], meta: crate::api::client::PaginationMeta { total: 0, offset: 0, limit: 1, has_more: false } });

                set_stats.set(DashboardStats {
                    employees: employees.meta.total,
                    customers: customers.meta.total,
                    open_orders: orders.meta.total,
                    active_projects: projects.meta.total,
                    recent_activity: vec![],
                });
            }
        }
        set_loading.set(false);
    });

    load_stats.dispatch(());

    let on_refresh = move |_| {
        load_stats.dispatch(());
    };

    view! {
        <div>
            <div class="page-header">
                <div class="page-header-left">
                    <h1>"Dashboard"</h1>
                    <p class="subtitle">"Welcome to Atlas ERP — your business at a glance"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-secondary" on:click=on_refresh disabled=move || loading.get()>
                        "↻ Refresh"
                    </button>
                </div>
            </div>

            // Error state
            {move || error.get().map(|e| view! {
                <div class="alert alert-danger">
                    {e}
                </div>
            })}

            // Loading skeleton
            {move || if loading.get() {
                view! {
                    <div class="dashboard-grid">
                        <div class="card"><div class="skeleton skeleton-card"></div></div>
                        <div class="card"><div class="skeleton skeleton-card"></div></div>
                        <div class="card"><div class="skeleton skeleton-card"></div></div>
                        <div class="card"><div class="skeleton skeleton-card"></div></div>
                    </div>
                }.into_view()
            } else {
                ().into_view()
            }}

            // KPI Cards
            {move || if !loading.get() {
                let s = stats.get();
                view! {
                    <div class="dashboard-grid">
                        <div class="card stat-card">
                            <div class="stat-icon blue">"👥"</div>
                            <div class="stat-content">
                                <h3>"Employees"</h3>
                                <p class="stat-value">{s.employees}</p>
                            </div>
                        </div>
                        <div class="card stat-card">
                            <div class="stat-icon green">"🤝"</div>
                            <div class="stat-content">
                                <h3>"Customers"</h3>
                                <p class="stat-value">{s.customers}</p>
                            </div>
                        </div>
                        <div class="card stat-card">
                            <div class="stat-icon orange">"📦"</div>
                            <div class="stat-content">
                                <h3>"Open Orders"</h3>
                                <p class="stat-value">{s.open_orders}</p>
                            </div>
                        </div>
                        <div class="card stat-card">
                            <div class="stat-icon purple">"📋"</div>
                            <div class="stat-content">
                                <h3>"Active Projects"</h3>
                                <p class="stat-value">{s.active_projects}</p>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                ().into_view()
            }}

            // Dashboard sections
            <div class="dashboard-charts">
                // Recent Activity
                <div class="dashboard-section">
                    <div class="card">
                        <div class="card-header">
                            <h2 class="card-title">"Recent Activity"</h2>
                        </div>
                        {move || {
                            let activities = stats.get().recent_activity.clone();
                            if activities.is_empty() {
                                view! {
                                    <div class="empty-state" style="padding: 2rem">
                                        <p class="text-sm text-gray-500">
                                            "Activity feed will appear here when records are created or updated."
                                        </p>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="activity-feed">
                                        {activities.iter().map(|a| {
                                            let dot_class = match a.action.as_str() {
                                                "created" => "created",
                                                "updated" => "updated",
                                                "deleted" => "deleted",
                                                _ => "workflow",
                                            };
                                            view! {
                                                <div class="activity-item">
                                                    <div class=format!("activity-dot {}", dot_class)></div>
                                                    <div class="activity-text">
                                                        {a.description.clone()}
                                                    </div>
                                                    <div class="activity-time">
                                                        {a.timestamp.clone()}
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }
                        }}
                    </div>
                </div>

                // Quick Actions
                <div class="dashboard-section">
                    <div class="card">
                        <div class="card-header">
                            <h2 class="card-title">"Quick Actions"</h2>
                        </div>
                        <div class="flex flex-col gap-3">
                            <QuickAction icon="👤" label="Add Employee" href="/employees?new=true" />
                            <QuickAction icon="🏢" label="Add Customer" href="/customers?new=true" />
                            <QuickAction icon="📦" label="New Order" href="/orders?new=true" />
                            <QuickAction icon="📋" label="New Project" href="/projects?new=true" />
                            <QuickAction icon="📈" label="View Reports" href="/reports" />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn QuickAction(icon: &'static str, label: &'static str, href: &'static str) -> impl IntoView {
    view! {
        <a href=href class="flex items-center gap-3 p-3 rounded-lg hover-bg" style="
            text-decoration: none;
            color: var(--color-gray-700);
            border: 1px solid var(--color-gray-200);
            border-radius: var(--radius-lg);
            transition: all 0.15s ease;
        ">
            <span style="font-size: 1.25rem">{icon}</span>
            <span class="text-sm font-medium">{label}</span>
            <span style="margin-left: auto; color: var(--color-gray-400)">"→"</span>
        </a>
    }
}
