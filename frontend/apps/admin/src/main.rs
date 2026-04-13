//! Atlas Admin Console
//!
//! System administration interface for managing schemas, workflows, and configuration.

use leptos::*;
use leptos_router::*;
use leptos_meta::*;

/// Main admin app component
#[component]
fn App() -> impl IntoView {
    provide_meta_context();
    
    view! {
        <Title text="Atlas Admin"/>
        <Router>
            <nav class="admin-nav">
                <h1>"Atlas Admin"</h1>
                <ul>
                    <li><A href="/" exact=true>"Overview"</A></li>
                    <li><A href="/entities">"Entities"</A></li>
                    <li><A href="/workflows">"Workflows"</A></li>
                    <li><A href="/users">"Users"</A></li>
                    <li><A href="/config">"Config"</A></li>
                </ul>
            </nav>
            <main>
                <Routes>
                    <Route path="/" view=AdminOverview/>
                    <Route path="/entities" view=EntityManager/>
                    <Route path="/workflows" view=WorkflowManager/>
                    <Route path="/users" view=UserManager/>
                    <Route path="/config" view=ConfigManager/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn AdminOverview() -> impl IntoView {
    view! {
        <h1>"System Overview"</h1>
        <p>"Manage your Atlas ERP system configuration."</p>
    }
}

#[component]
fn EntityManager() -> impl IntoView {
    view! {
        <h1>"Entity Manager"</h1>
        <p>"Create and modify entity definitions dynamically."</p>
    }
}

#[component]
fn WorkflowManager() -> impl IntoView {
    view! {
        <h1>"Workflow Manager"</h1>
        <p>"Design and configure business process workflows."</p>
    }
}

#[component]
fn UserManager() -> impl IntoView {
    view! {
        <h1>"User Management"</h1>
        <p>"Manage users, roles, and permissions."</p>
    }
}

#[component]
fn ConfigManager() -> impl IntoView {
    view! {
        <h1>"System Configuration"</h1>
        <p>"View and modify system configuration parameters."</p>
    }
}

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logging");
    mount_to_body(|| view! { <App/> });
}
