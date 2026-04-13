//! Entity list page - dynamically renders entity records

use leptos::*;

/// Entity list page component
#[component]
pub fn EntityListPage() -> impl IntoView {
    let entity_name = use_route().path().to_string();
    let entity_label = move || {
        match entity_name.trim_start_matches('/') {
            "employees" => "Employees",
            "customers" => "Customers",
            "orders" => "Orders",
            "projects" => "Projects",
            other => other,
        }
    };
    
    view! {
        <div class="page-header">
            <h1>{entity_label}</h1>
            <button class="btn btn-primary">"+ New"</button>
        </div>
        <div class="card">
            <table class="data-table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Status"</th>
                        <th>"Created"</th>
                        <th>"Actions"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td colspan="4">"Loading data..."</td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}
