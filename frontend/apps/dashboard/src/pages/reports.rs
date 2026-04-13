//! Reports page

use leptos::*;

/// Reports page component
#[component]
pub fn ReportsPage() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1>"Reports"</h1>
        </div>
        <div class="card">
            <h2>"Available Reports"</h2>
            <ul class="report-list">
                <li>"Financial Summary"</li>
                <li>"Employee Headcount"</li>
                <li>"Sales Pipeline"</li>
                <li>"Inventory Status"</li>
                <li>"Project Status"</li>
            </ul>
        </div>
    }
}
