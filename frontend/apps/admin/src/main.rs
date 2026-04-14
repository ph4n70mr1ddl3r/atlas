//! Atlas Admin Console
//!
//! System administration interface for managing schemas, workflows, and configuration.

use leptos::*;
use leptos_router::*;
use leptos_meta::*;

mod api;
mod components;

use api::client::{fetch_entity_schema, create_entity_schema, EntitySchema, FieldSchema};
use serde_json::json;

/// Main admin app component
#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Atlas Admin"/>
        <div class="app-layout">
            <nav class="sidebar" style="background: #1a1a2e">
                <div class="sidebar-header">
                    <div class="sidebar-logo" style="background: #e94560">"A"</div>
                    <h1>"Atlas Admin"</h1>
                </div>

                <div class="sidebar-section">
                    <div class="sidebar-section-title">"Configuration"</div>
                    <div class="sidebar-nav">
                        <A href="/" exact=true>
                            <span class="sidebar-nav-icon">"🏠"</span>
                            <span>"Overview"</span>
                        </A>
                        <A href="/entities">
                            <span class="sidebar-nav-icon">"🏗️"</span>
                            <span>"Entities"</span>
                        </A>
                        <A href="/workflows">
                            <span class="sidebar-nav-icon">"🔄"</span>
                            <span>"Workflows"</span>
                        </A>
                        <A href="/users">
                            <span class="sidebar-nav-icon">"👥"</span>
                            <span>"Users & Roles"</span>
                        </A>
                        <A href="/config">
                            <span class="sidebar-nav-icon">"⚙️"</span>
                            <span>"Configuration"</span>
                        </A>
                    </div>
                </div>

                <div class="sidebar-section">
                    <div class="sidebar-section-title">"Quick Links"</div>
                    <div class="sidebar-nav">
                        <A href="http://localhost:3000" target="_blank">
                            <span class="sidebar-nav-icon">"📊"</span>
                            <span>"Dashboard App"</span>
                        </A>
                    </div>
                </div>
            </nav>

            <main class="main-content">
                <Routes>
                    <Route path="/" view=AdminOverview/>
                    <Route path="/entities" view=EntityManager/>
                    <Route path="/workflows" view=WorkflowManager/>
                    <Route path="/users" view=UserManager/>
                    <Route path="/config" view=ConfigManager/>
                </Routes>
            </main>
        </div>
    }
}

// ============================================
// Admin Pages
// ============================================

#[component]
fn AdminOverview() -> impl IntoView {
    view! {
        <div class="page-header">
            <div class="page-header-left">
                <h1>"System Overview"</h1>
                <p class="subtitle">"Atlas ERP system health and configuration"</p>
            </div>
        </div>

        <div class="dashboard-grid">
            <div class="card stat-card">
                <div class="stat-icon blue">"🏗️"</div>
                <div class="stat-content">
                    <h3>"Entities"</h3>
                    <p class="stat-value">"8"</p>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon green">"🔄"</div>
                <div class="stat-content">
                    <h3>"Workflows"</h3>
                    <p class="stat-value">"5"</p>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon purple">"👥"</div>
                <div class="stat-content">
                    <h3>"Users"</h3>
                    <p class="stat-value">"3"</p>
                </div>
            </div>
            <div class="card stat-card">
                <div class="stat-icon orange">"📋"</div>
                <div class="stat-content">
                    <h3>"Config Keys"</h3>
                    <p class="stat-value">"12"</p>
                </div>
            </div>
        </div>

        <div class="card mt-6">
            <div class="card-header">
                <h2 class="card-title">"Registered Entities"</h2>
            </div>
            <div class="data-table-container">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>"Entity"</th>
                            <th>"Label"</th>
                            <th>"Fields"</th>
                            <th>"Has Workflow"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {vec![
                            ("employees", "Employees", "7", "Yes"),
                            ("customers", "Customers", "6", "No"),
                            ("orders", "Sales Orders", "5", "Yes"),
                            ("products", "Products", "5", "No"),
                            ("projects", "Projects", "5", "Yes"),
                            ("invoices", "Invoices", "5", "Yes"),
                            ("suppliers", "Suppliers", "4", "No"),
                            ("expense_reports", "Expense Reports", "5", "Yes"),
                        ].into_iter().map(|(name, label, fields, workflow)| {
                            view! {
                                <tr>
                                    <td><code style="font-size: 0.875rem; color: var(--color-primary)">{name}</code></td>
                                    <td>{label}</td>
                                    <td>{fields}</td>
                                    <td>
                                        <span class=format!("badge {}", if workflow == "Yes" { "badge-success" } else { "badge-neutral" })>
                                            {workflow}
                                        </span>
                                    </td>
                                    <td>
                                        <a href=format!("/entities?entity={}", name) class="btn btn-ghost btn-sm">
                                            "Edit"
                                        </a>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn EntityManager() -> impl IntoView {
    let (entities, set_entities) = create_signal::<Vec<String>>(vec![
        "employees".into(), "customers".into(), "orders".into(),
        "products".into(), "projects".into(), "invoices".into(),
    ]);
    let (selected, set_selected) = create_signal::<Option<String>>(None);
    let (schema, set_schema) = create_signal::<Option<EntitySchema>>(None);
    let (loading, set_loading) = create_signal(false);

    let load_schema = create_action(move |entity: &String| {
        let entity = entity.clone();
        async move {
            set_loading.set(true);
            match fetch_entity_schema(&entity).await {
                Ok(s) => set_schema.set(Some(s)),
                Err(_) => set_schema.set(None),
            }
            set_loading.set(false);
        }
    });

    let on_select_entity = move |name: String| {
        set_selected.set(Some(name.clone()));
        load_schema.dispatch(name);
    };

    view! {
        <div class="page-header">
            <div class="page-header-left">
                <h1>"Entity Manager"</h1>
                <p class="subtitle">"Create and modify entity definitions dynamically"</p>
            </div>
            <div class="page-actions">
                <button class="btn btn-primary">"+ New Entity"</button>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 280px 1fr; gap: 1.5rem">
            // Entity list
            <div class="card" style="padding: 0">
                <div class="card-header" style="padding: 1rem 1.5rem">
                    <h3 class="card-title">"Entities"</h3>
                </div>
                <div class="sidebar-nav" style="padding: 0.5rem">
                    {move || entities.get().iter().map(|name| {
                        let is_selected = selected.get() == Some(name.clone());
                        let name_for_click = name.clone();
                        view! {
                            <a href="#"
                                class=format!("sidebar-nav{}", if is_selected { " active" } else { "" })
                                style=if is_selected {
                                    "display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem; border-radius: 0.375rem; background: var(--color-primary-light); color: var(--color-primary-700); text-decoration: none; font-weight: 500; font-size: 0.875rem"
                                } else {
                                    "display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem; border-radius: 0.375rem; text-decoration: none; color: var(--color-gray-600); font-size: 0.875rem"
                                }
                                on:click=move |ev| {
                                    ev.prevent_default();
                                    on_select_entity(name_for_click.clone());
                                }
                            >
                                <span>"📦"</span>
                                <span>{name.clone()}</span>
                            </a>
                        }
                    }).collect_view()}
                </div>
            </div>

            // Schema detail
            <div>
                {move || {
                    if loading.get() {
                        view! {
                            <div class="card">
                                <div class="spinner"><div class="spinner-inner"></div></div>
                            </div>
                        }.into_view()
                    } else if let Some(s) = schema.get() {
                        view! {
                            <div class="card">
                                <div class="card-header">
                                    <div>
                                        <h2 class="card-title">{s.label.clone()}</h2>
                                        <p class="card-subtitle">
                                            {format!("{} ({} fields)", s.name, s.fields.len())}
                                        </p>
                                    </div>
                                    <button class="btn btn-primary btn-sm">"+ Add Field"</button>
                                </div>

                                <div class="data-table-container">
                                    <table class="data-table">
                                        <thead>
                                            <tr>
                                                <th>"Field"</th>
                                                <th>"Type"</th>
                                                <th>"Required"</th>
                                                <th>"Editable"</th>
                                                <th>"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {s.fields.iter().map(|f| {
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <code style="font-size: 0.8rem; color: var(--color-primary)">{f.name.clone()}</code>
                                                            <br/>
                                                            <span class="text-xs text-gray-500">{f.label.clone()}</span>
                                                        </td>
                                                        <td>
                                                            <span class="badge badge-neutral">{f.field_type.clone()}</span>
                                                        </td>
                                                        <td>
                                                            {if f.required { "✓" } else { "—" }}
                                                        </td>
                                                        <td>
                                                            {if f.editable { "✓" } else { "—" }}
                                                        </td>
                                                        <td>
                                                            <button class="btn btn-ghost btn-sm">"Edit"</button>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="card">
                                <div class="empty-state">
                                    <div class="empty-state-icon">"📦"</div>
                                    <h3>"Select an entity"</h3>
                                    <p>"Choose an entity from the list to view and edit its schema."</p>
                                </div>
                            </div>
                        }.into_view()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn WorkflowManager() -> impl IntoView {
    view! {
        <div class="page-header">
            <div class="page-header-left">
                <h1>"Workflow Manager"</h1>
                <p class="subtitle">"Design and configure business process workflows"</p>
            </div>
            <div class="page-actions">
                <button class="btn btn-primary">"+ New Workflow"</button>
            </div>
        </div>

        <div class="card">
            <div class="card-header">
                <h2 class="card-title">"Active Workflows"</h2>
            </div>
            <div class="data-table-container">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>"Entity"</th>
                            <th>"Initial State"</th>
                            <th>"States"</th>
                            <th>"Transitions"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {vec![
                            ("purchase_orders", "draft", "5", "4"),
                            ("expense_reports", "draft", "4", "3"),
                            ("orders", "draft", "7", "6"),
                            ("projects", "planning", "5", "4"),
                            ("invoices", "draft", "5", "4"),
                        ].into_iter().map(|(entity, initial, states, transitions)| {
                            view! {
                                <tr>
                                    <td><code style="color: var(--color-primary)">{entity}</code></td>
                                    <td><span class="badge badge-warning">{initial}</span></td>
                                    <td>{states}</td>
                                    <td>{transitions}</td>
                                    <td>
                                        <button class="btn btn-ghost btn-sm">"Edit"</button>
                                        <button class="btn btn-ghost btn-sm">"Visualize"</button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn UserManager() -> impl IntoView {
    view! {
        <div class="page-header">
            <div class="page-header-left">
                <h1>"User Management"</h1>
                <p class="subtitle">"Manage users, roles, and permissions"</p>
            </div>
            <div class="page-actions">
                <button class="btn btn-primary">"+ New User"</button>
            </div>
        </div>

        <div class="card" style="padding: 0">
            <div class="data-table-container">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>"Username"</th>
                            <th>"Email"</th>
                            <th>"Role"</th>
                            <th>"Status"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {vec![
                            ("admin", "admin@atlas.dev", "system_admin", "active"),
                            ("hr_manager", "hr@atlas.dev", "hr_admin", "active"),
                            ("sales_user", "sales@atlas.dev", "sales_rep", "active"),
                        ].into_iter().map(|(user, email, role, status)| {
                            view! {
                                <tr>
                                    <td class="font-medium">{user}</td>
                                    <td class="text-sm">{email}</td>
                                    <td><code style="font-size: 0.8rem">{role}</code></td>
                                    <td>
                                        <span class="badge badge-success">{status}</span>
                                    </td>
                                    <td>
                                        <button class="btn btn-ghost btn-sm">"Edit"</button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn ConfigManager() -> impl IntoView {
    view! {
        <div class="page-header">
            <div class="page-header-left">
                <h1>"System Configuration"</h1>
                <p class="subtitle">"View and modify system configuration parameters"</p>
            </div>
        </div>

        <div class="card">
            <div class="card-header">
                <h2 class="card-title">"Configuration Keys"</h2>
            </div>
            <div class="data-table-container">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>"Key"</th>
                            <th>"Value"</th>
                            <th>"Type"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {vec![
                            ("app.name", "Atlas ERP", "string"),
                            ("app.version", "0.1.0", "string"),
                            ("auth.jwt_expiry", "3600", "integer"),
                            ("auth.max_login_attempts", "5", "integer"),
                            ("storage.max_upload_size", "10485760", "integer"),
                            ("notifications.enabled", "true", "boolean"),
                            ("audit.enabled", "true", "boolean"),
                            ("cache.ttl", "300", "integer"),
                        ].into_iter().map(|(key, value, typ)| {
                            view! {
                                <tr>
                                    <td><code style="font-size: 0.8rem">{key}</code></td>
                                    <td>{value}</td>
                                    <td><span class="badge badge-neutral">{typ}</span></td>
                                    <td>
                                        <button class="btn btn-ghost btn-sm">"Edit"</button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

/// Entry point
fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logging");
    mount_to_body(|| view! { <App/> });
}
