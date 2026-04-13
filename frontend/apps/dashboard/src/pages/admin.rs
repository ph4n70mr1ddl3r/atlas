//! Admin page

use leptos::*;

/// Admin page component  
#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1>"Administration"</h1>
        </div>
        <div class="admin-grid">
            <div class="card">
                <h2>"Schema Manager"</h2>
                <p>"Manage entity definitions, fields, and relationships."</p>
                <button class="btn btn-primary">"Manage Schema"</button>
            </div>
            <div class="card">
                <h2>"Workflow Designer"</h2>
                <p>"Design and modify business process workflows."</p>
                <button class="btn btn-primary">"Manage Workflows"</button>
            </div>
            <div class="card">
                <h2>"User Management"</h2>
                <p>"Manage users, roles, and permissions."</p>
                <button class="btn btn-primary">"Manage Users"</button>
            </div>
            <div class="card">
                <h2>"System Configuration"</h2>
                <p>"Configure system settings and parameters."</p>
                <button class="btn btn-primary">"Configuration"</button>
            </div>
        </div>
    }
}
