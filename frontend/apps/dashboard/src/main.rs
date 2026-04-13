//! Atlas ERP Dashboard
//!
//! Main dashboard application built with Leptos.

use leptos::*;
use leptos_router::*;
use leptos_meta::*;

mod pages;
mod components;
mod api;

use pages::*;

/// Main application component
#[component]
fn App() -> impl IntoView {
    provide_meta_context();
    
    view! {
        <Stylesheet id="atlas" href="/static/atlas.css"/>
        <Title text="Atlas ERP"/>
        <Router>
            <nav class="sidebar">
                <div class="sidebar-header">
                    <h1>"Atlas ERP"</h1>
                </div>
                <ul class="sidebar-nav">
                    <li><A href="/" exact=true>"📊 Dashboard"</A></li>
                    <li><A href="/employees">"👥 Employees"</A></li>
                    <li><A href="/customers">"🤝 Customers"</A></li>
                    <li><A href="/orders">"📦 Orders"</A></li>
                    <li><A href="/projects">"📋 Projects"</A></li>
                    <li><A href="/reports">"📈 Reports"</A></li>
                    <li><A href="/admin">"⚙️ Admin"</A></li>
                </ul>
            </nav>
            <main class="main-content">
                <Routes>
                    <Route path="/" view=DashboardPage/>
                    <Route path="/employees" view=EntityListPage/>
                    <Route path="/employees/:id" view=EntityDetailPage/>
                    <Route path="/customers" view=EntityListPage/>
                    <Route path="/customers/:id" view=EntityDetailPage/>
                    <Route path="/orders" view=EntityListPage/>
                    <Route path="/projects" view=EntityListPage/>
                    <Route path="/reports" view=ReportsPage/>
                    <Route path="/admin" view=AdminPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Entry point
fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logging");
    mount_to_body(|| view! { <App/> });
}
