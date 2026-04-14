//! Atlas ERP Dashboard
//!
//! Main dashboard application built with Leptos (CSR/WASM).

use leptos::*;
use leptos_router::*;
use leptos_meta::*;

mod pages;
mod components;
mod api;

use pages::*;
use components::AppLayout;

/// Main application component
#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Atlas ERP"/>
        <AppLayout>
            <Routes>
                <Route path="/" view=DashboardPage/>
                <Route path="/employees" view=EntityListPage/>
                <Route path="/employees/:id" view=EntityDetailPage/>
                <Route path="/customers" view=EntityListPage/>
                <Route path="/customers/:id" view=EntityDetailPage/>
                <Route path="/orders" view=EntityListPage/>
                <Route path="/orders/:id" view=EntityDetailPage/>
                <Route path="/products" view=EntityListPage/>
                <Route path="/products/:id" view=EntityDetailPage/>
                <Route path="/projects" view=EntityListPage/>
                <Route path="/projects/:id" view=EntityDetailPage/>
                <Route path="/invoices" view=EntityListPage/>
                <Route path="/invoices/:id" view=EntityDetailPage/>
                <Route path="/reports" view=ReportsPage/>
                <Route path="/admin" view=AdminPage/>
            </Routes>
        </AppLayout>
    }
}

/// Entry point
fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logging");
    mount_to_body(|| view! { <App/> });
}
