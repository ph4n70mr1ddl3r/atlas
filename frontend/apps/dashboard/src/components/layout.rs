//! App layout component

use leptos::*;
use leptos_router::*;

/// Main application shell with sidebar navigation
#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let (sidebar_open, set_sidebar_open) = create_signal(false);

    view! {
        <div class="app-layout">
            <Sidebar sidebar_open=sidebar_open set_sidebar_open=set_sidebar_open/>
            <main class="main-content">
                <div class="mobile-header">
                    <button class="btn btn-ghost" on:click=move |_| set_sidebar_open.update(|v| *v = !*v)>
                        "☰"
                    </button>
                </div>
                {children()}
            </main>
        </div>
    }
}

#[component]
fn Sidebar(
    sidebar_open: ReadSignal<bool>,
    set_sidebar_open: WriteSignal<bool>,
) -> impl IntoView {
    let user_info = crate::api::client::get_stored_user();

    view! {
        <nav class=format!("sidebar{}", if sidebar_open.get() { " open" } else { "" })>
            <div class="sidebar-header">
                <div class="sidebar-logo">"A"</div>
                <h1>"Atlas ERP"</h1>
            </div>

            <div class="sidebar-section">
                <div class="sidebar-section-title">"Main"</div>
                <div class="sidebar-nav">
                    <A href="/" exact=true>
                        <span class="sidebar-nav-icon">"📊"</span>
                        <span>"Dashboard"</span>
                    </A>
                </div>
            </div>

            <div class="sidebar-section">
                <div class="sidebar-section-title">"Modules"</div>
                <div class="sidebar-nav">
                    <A href="/employees">
                        <span class="sidebar-nav-icon">"👥"</span>
                        <span>"Employees"</span>
                    </A>
                    <A href="/customers">
                        <span class="sidebar-nav-icon">"🤝"</span>
                        <span>"Customers"</span>
                    </A>
                    <A href="/orders">
                        <span class="sidebar-nav-icon">"📦"</span>
                        <span>"Orders"</span>
                    </A>
                    <A href="/products">
                        <span class="sidebar-nav-icon">"🏷️"</span>
                        <span>"Products"</span>
                    </A>
                    <A href="/projects">
                        <span class="sidebar-nav-icon">"📋"</span>
                        <span>"Projects"</span>
                    </A>
                    <A href="/invoices">
                        <span class="sidebar-nav-icon">"🧾"</span>
                        <span>"Invoices"</span>
                    </A>
                </div>
            </div>

            <div class="sidebar-section">
                <div class="sidebar-section-title">"Analytics"</div>
                <div class="sidebar-nav">
                    <A href="/reports">
                        <span class="sidebar-nav-icon">"📈"</span>
                        <span>"Reports"</span>
                    </A>
                </div>
            </div>

            <div class="sidebar-section">
                <div class="sidebar-section-title">"System"</div>
                <div class="sidebar-nav">
                    <A href="/admin">
                        <span class="sidebar-nav-icon">"⚙️"</span>
                        <span>"Admin"</span>
                    </A>
                </div>
            </div>

            <div class="sidebar-footer">
                <div class="sidebar-user">
                    <div class="sidebar-user-avatar">
                        {user_info.as_ref().map(|u| u.username.chars().next().unwrap_or('U').to_uppercase().to_string()).unwrap_or_else(|| "?".to_string())}
                    </div>
                    <div class="sidebar-user-info">
                        <div class="sidebar-user-name">
                            {user_info.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "Not logged in".to_string())}
                        </div>
                        <div class="sidebar-user-role">
                            {user_info.as_ref().map(|u| u.role.clone()).unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    }
}
