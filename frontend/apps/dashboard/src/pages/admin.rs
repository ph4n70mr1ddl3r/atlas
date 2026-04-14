//! Admin page - system configuration and management

use leptos::*;

/// Admin page component
#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <div>
            <div class="page-header">
                <div class="page-header-left">
                    <h1>"Administration"</h1>
                    <p class="subtitle">"System configuration and management"</p>
                </div>
            </div>

            <div class="admin-grid">
                <AdminCard
                    icon="🏗️"
                    title="Schema Manager"
                    description="Create and modify entity definitions, fields, and relationships dynamically."
                    href="/admin/schema"
                    color="blue"
                />
                <AdminCard
                    icon="🔄"
                    title="Workflow Designer"
                    description="Design and configure business process workflows, states, and transitions."
                    href="/admin/workflows"
                    color="green"
                />
                <AdminCard
                    icon="👥"
                    title="User Management"
                    description="Manage users, roles, and permissions. Configure access control policies."
                    href="/admin/users"
                    color="purple"
                />
                <AdminCard
                    icon="⚙️"
                    title="System Configuration"
                    description="Configure system settings, parameters, and integration endpoints."
                    href="/admin/config"
                    color="orange"
                />
                <AdminCard
                    icon="📊"
                    title="Report Builder"
                    description="Create custom reports and dashboards with the visual report builder."
                    href="/admin/reports"
                    color="blue"
                />
                <AdminCard
                    icon="🔌"
                    title="Integrations"
                    description="Manage external system connections, APIs, and data synchronization."
                    href="/admin/integrations"
                    color="green"
                />
            </div>

            // System info section
            <div class="card mt-6">
                <div class="card-header">
                    <h2 class="card-title">"System Information"</h2>
                </div>
                <div class="form-grid">
                    <div class="form-group">
                        <label class="form-label">"Version"</label>
                        <div class="text-sm">"Atlas ERP v0.1.0"</div>
                    </div>
                    <div class="form-group">
                        <label class="form-label">"API Gateway"</label>
                        <div class="text-sm">
                            <span class="badge badge-success">"Connected"</span>
                        </div>
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Database"</label>
                        <div class="text-sm">
                            <span class="badge badge-success">"PostgreSQL 16"</span>
                        </div>
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Event Bus"</label>
                        <div class="text-sm">
                            <span class="badge badge-success">"NATS"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AdminCard(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    href: &'static str,
    color: &'static str,
) -> impl IntoView {
    let icon_bg = match color {
        "blue" => "background: var(--color-primary-light)",
        "green" => "background: var(--color-success-light)",
        "orange" => "background: var(--color-warning-light)",
        "purple" => "background: #f3e8ff",
        _ => "background: var(--color-gray-100)",
    };

    view! {
        <a href=href class="admin-card" style="text-decoration: none; color: inherit">
            <div class="admin-card-icon" style=icon_bg>
                {icon}
            </div>
            <h3>{title}</h3>
            <p>{description}</p>
            <span class="text-sm font-medium" style="color: var(--color-primary)">
                "Open →"
            </span>
        </a>
    }
}
