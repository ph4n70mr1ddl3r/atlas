//! Dashboard page

use leptos::*;

/// Dashboard page component
#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1>"Dashboard"</h1>
            <p class="subtitle">"Welcome to Atlas ERP"</p>
        </div>
        <div class="dashboard-grid">
            <div class="card stat-card">
                <div class="stat-icon">"👥"</div>
                <div class="stat-content">
                    <h3>"Employees"</h3>
                    <p class="stat-value">"--"</p>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon">"🤝"</div>
                <div class="stat-content">
                    <h3>"Customers"</h3>
                    <p class="stat-value">"--"</p>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon">"📦"</div>
                <div class="stat-content">
                    <h3>"Open Orders"</h3>
                    <p class="stat-value">"--"</p>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon">"📋"</div>
                <div class="stat-content">
                    <h3>"Active Projects"</h3>
                    <p class="stat-value">"--"</p>
                </div>
            </div>
        </div>
        <div class="dashboard-section">
            <div class="card">
                <h2>"Recent Activity"</h2>
                <p>"Activity feed will appear here once connected to the API."</p>
            </div>
        </div>
    }
}
