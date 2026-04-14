//! Entity detail page - shows record details with tabs for info, history, and edit

use leptos::*;
use leptos_router::*;
use serde_json::Value;
use crate::api::client::{
    fetch_record, delete_record,
    fetch_transitions, fetch_record_history, fetch_entity_schema,
    AuditEntry, TransitionInfo, EntitySchema,
};
use crate::components::{FormBuilder, WorkflowBar, StatusBadge};

/// Entity detail page component
#[component]
pub fn EntityDetailPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let location = use_location();

    // Determine entity and ID from URL
    let path_info = create_memo(move |_| {
        let path = location.pathname.get();
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let entity = parts.first().unwrap_or(&"").to_string();
        let id = parts.get(1).unwrap_or(&"").to_string();
        (entity, id)
    });

    let (active_tab, set_active_tab) = create_signal("details".to_string());

    // Record data
    let (record, set_record) = create_signal::<Option<Value>>(None);
    let (schema, set_schema) = create_signal::<Option<EntitySchema>>(None);
    let (transitions, set_transitions) = create_signal::<Option<TransitionInfo>>(None);
    let (history, set_history) = create_signal::<Vec<AuditEntry>>(vec![]);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (show_delete_modal, set_show_delete_modal) = create_signal(false);
    let (deleting, set_deleting) = create_signal(false);

    // Load record data
    let load_record = create_action(move |_: &()| {
        let (entity, id) = path_info.get();
        async move {
            set_loading.set(true);
            set_error.set(None);

            // Fetch record
            match fetch_record(&entity, &id).await {
                Ok(data) => {
                    set_record.set(Some(data));
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }

            // Fetch schema
            let entity_s = entity.clone();
            match fetch_entity_schema(&entity_s).await {
                Ok(s) => set_schema.set(Some(s)),
                Err(_) => {}
            }

            // Fetch transitions
            let entity_t = entity.clone();
            let id_t = id.clone();
            match fetch_transitions(&entity_t, &id_t).await {
                Ok(t) => set_transitions.set(Some(t)),
                Err(_) => {}
            }

            // Fetch history
            let entity_h = entity.clone();
            let id_h = id.clone();
            match fetch_record_history(&entity_h, &id_h).await {
                Ok(h) => set_history.set(h),
                Err(_) => {}
            }

            set_loading.set(false);
        }
    });

    load_record.dispatch(());

    // Handle workflow action completion - reload record
    let on_action_completed = Callback::new(move |_: Value| {
        load_record.dispatch(());
    });

    // Handle save
    let on_save_success = Callback::new(move |data: Value| {
        set_record.set(Some(data));
        set_active_tab.set("details".to_string());
    });

    // Handle cancel
    let on_cancel = Callback::new(move |()| {
        set_active_tab.set("details".to_string());
    });

    // Get entity label
    let entity_label = create_memo(move |_| {
        match path_info.get().0.as_str() {
            "employees" => "Employee".to_string(),
            "customers" => "Customer".to_string(),
            "orders" => "Order".to_string(),
            "products" => "Product".to_string(),
            "projects" => "Project".to_string(),
            "invoices" => "Invoice".to_string(),
            other => other.to_string(),
        }
    });

    // Get display name for the record
    let record_name = create_memo(move |_| {
        record.get().map(|r| {
            r.get("name").or(r.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Record")
                .to_string()
        }).unwrap_or_else(|| "Loading...".to_string())
    });

    let current_status = create_memo(move |_| {
        record.get()
            .and_then(|r| r.get("status").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_default()
    });

    // Delete handler using spawn_local to avoid FnOnce issues
    let entity_for_nav = path_info.get().0.clone();

    view! {
        <div>
            // Breadcrumbs
            <div class="breadcrumbs">
                <a href="/">"Home"</a>
                <span class="breadcrumb-separator">"/"</span>
                <a href=format!("/{}", path_info.get().0)>
                    {path_info.get().0}
                </a>
                <span class="breadcrumb-separator">"/"</span>
                <span class="text-gray-600">{record_name.get()}</span>
            </div>

            // Header
            <div class="page-header">
                <div class="page-header-left">
                    <h1>
                        {record_name.get()}
                        {move || if !current_status.get().is_empty() {
                            view! {
                                <span style="margin-left: 0.75rem">
                                    <StatusBadge status=current_status.get()/>
                                </span>
                            }.into_view()
                        } else {
                            ().into_view()
                        }}
                    </h1>
                    <p class="subtitle">
                        {move || format!("{} • ID: {}", entity_label.get(), path_info.get().1)}
                    </p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-secondary"
                        on:click={
                            let nav = navigate.clone();
                            let nav_entity = path_info.get().0.clone();
                            move |_| nav(&format!("/{}", nav_entity), Default::default())
                        }>
                        "← Back"
                    </button>
                    <button class="btn btn-secondary"
                        on:click=move |_| set_active_tab.set("edit".to_string())>
                        "✎ Edit"
                    </button>
                    <button class="btn btn-danger"
                        on:click=move |_| set_show_delete_modal.set(true)>
                        "🗑 Delete"
                    </button>
                </div>
            </div>

            // Error
            {move || error.get().map(|e| view! {
                <div class="alert alert-danger" style="margin-bottom: 1rem">
                    {e}
                </div>
            })}

            // Loading
            {move || if loading.get() {
                view! {
                    <div class="card">
                        <div class="spinner"><div class="spinner-inner"></div></div>
                    </div>
                }.into_view()
            } else {
                ().into_view()
            }}

            // Content
            {move || if !loading.get() {
                let (entity, id) = path_info.get();
                let cs = current_status.get();
                let wf: Option<crate::api::client::WorkflowDefinition> = None;
                let tr = transitions.get();
                let rec = record.get();
                let tab = active_tab.get();
                let hist = history.get();

                view! {
                    <div>
                        // Workflow bar (only if there's a status)
                        {if !cs.is_empty() {
                            view! {
                                <WorkflowBar
                                    entity=entity.clone()
                                    record_id=id.clone()
                                    current_status=cs
                                    workflow=None
                                    transitions=tr
                                    on_action_completed=on_action_completed
                                />
                            }.into_view()
                        } else {
                            ().into_view()
                        }}

                        // Tabs
                        <div class="card" style="padding: 0">
                            <div class="tabs" style="padding: 0 1.5rem">
                                <button
                                    class=format!("tab{}", if tab == "details" { " active" } else { "" })
                                    on:click=move |_| set_active_tab.set("details".to_string())
                                >
                                    "Details"
                                </button>
                                <button
                                    class=format!("tab{}", if tab == "edit" { " active" } else { "" })
                                    on:click=move |_| set_active_tab.set("edit".to_string())
                                >
                                    "Edit"
                                </button>
                                <button
                                    class=format!("tab{}", if tab == "history" { " active" } else { "" })
                                    on:click=move |_| set_active_tab.set("history".to_string())
                                >
                                    "History"
                                </button>
                            </div>

                            <div class="tab-content" style="padding: 1.5rem">
                                {if tab == "details" {
                                    match rec {
                                        Some(data) => view! {
                                            <RecordDetails data=data schema=None/>
                                        }.into_view(),
                                        None => view! {
                                            <div class="empty-state">
                                                <p>"No data available"</p>
                                            </div>
                                        }.into_view(),
                                    }
                                } else if tab == "edit" {
                                    match rec {
                                        Some(data) => view! {
                                            <FormBuilder
                                                entity=entity.clone()
                                                record_id=Some(id.clone())
                                                initial_data=Some(data)
                                                on_success=on_save_success
                                                on_cancel=on_cancel
                                            />
                                        }.into_view(),
                                        None => ().into_view(),
                                    }
                                } else if tab == "history" {
                                    view! {
                                        <HistoryTimeline history=hist/>
                                    }.into_view()
                                } else {
                                    ().into_view()
                                }}
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                ().into_view()
            }}

            // Delete confirmation modal
            <DeleteModal
                show=show_delete_modal
                set_show=set_show_delete_modal
                deleting=deleting
                entity_label=entity_label.get()
                path_info=path_info
                on_error=set_error
                on_navigate=Callback::new({
                    let nav = navigate.clone();
                    move |path: String| {
                        nav(&path, Default::default());
                    }
                })
            />
        </div>
    }
}

/// Record details display
#[component]
fn RecordDetails(data: Value, schema: Option<EntitySchema>) -> impl IntoView {
    let fields = schema.map(|s| s.fields).unwrap_or_default();

    view! {
        <div class="form-grid">
            {if fields.is_empty() {
                // No schema - render all fields from data
                data.as_object()
                    .map(|obj| {
                        obj.iter()
                            .filter(|(k, _)| *k != "id" && *k != "organization_id")
                            .map(|(key, value)| {
                                let label = key.split('_')
                                    .map(|w| {
                                        let mut chars = w.chars();
                                        match chars.next() {
                                            None => String::new(),
                                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                let display = match value {
                                    Value::String(s) => s.clone(),
                                    Value::Number(n) => n.to_string(),
                                    Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
                                    Value::Null => "—".to_string(),
                                    other => other.to_string(),
                                };
                                view! {
                                    <div class="form-group">
                                        <label class="form-label">{label}</label>
                                        <div class="text-sm" style="padding: 0.5rem 0">{display}</div>
                                    </div>
                                }
                            })
                            .collect_view()
                    })
                    .unwrap_or_else(|| view! { <p>"No data"</p> }.into_view())
            } else {
                // Render from schema
                fields.iter()
                    .filter(|f| f.visible)
                    .map(|field| {
                        let value = data.get(&field.name);
                        let display = match value {
                            Some(Value::String(s)) => s.clone(),
                            Some(Value::Number(n)) => n.to_string(),
                            Some(Value::Bool(b)) => if *b { "Yes" } else { "No" }.to_string(),
                            Some(Value::Null) | None => "—".to_string(),
                            Some(other) => other.to_string(),
                        };
                        let is_status_field = field.field_type == "status" || field.name == "status";
                        view! {
                            <div class="form-group">
                                <label class="form-label">{field.label.clone()}</label>
                                <div class="text-sm" style="padding: 0.5rem 0">
                                    {if is_status_field {
                                        view! {
                                            <StatusBadge status=display/>
                                        }.into_view()
                                    } else {
                                        view! { <span>{display}</span> }.into_view()
                                    }}
                                </div>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}

/// Audit history timeline
#[component]
fn HistoryTimeline(history: Vec<AuditEntry>) -> impl IntoView {
    view! {
        <div>
            {if history.is_empty() {
                view! {
                    <div class="empty-state" style="padding: 2rem">
                        <p class="text-sm text-gray-500">"No history available for this record."</p>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="activity-feed">
                        {history.iter().map(|entry| {
                            let dot_class = match entry.action.as_str() {
                                "create" => "created",
                                "update" => "updated",
                                "delete" => "deleted",
                                _ => "workflow",
                            };
                            let changed_by = entry.changed_by.clone().unwrap_or_default();
                            let new_data = entry.new_data.clone();
                            let changes = new_data.as_ref()
                                .and_then(|d| d.as_object())
                                .map(|obj| {
                                    obj.keys().take(3).cloned().collect::<Vec<_>>().join(", ")
                                })
                                .unwrap_or_default();
                            view! {
                                <div class="activity-item">
                                    <div class=format!("activity-dot {}", dot_class)></div>
                                    <div class="activity-text">
                                        <strong>{entry.action.clone()}</strong>
                                        {if changed_by.is_empty() { String::new() } else { format!(" by {}", changed_by) }}
                                        {if changes.is_empty() { String::new() } else { format!(" — changed: {}", changes) }}
                                    </div>
                                    <div class="activity-time">{entry.changed_at.clone()}</div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_view()
            }}
        </div>
    }
}

/// Delete confirmation modal component
#[component]
fn DeleteModal(
    show: ReadSignal<bool>,
    set_show: WriteSignal<bool>,
    deleting: ReadSignal<bool>,
    entity_label: String,
    path_info: Memo<(String, String)>,
    on_error: WriteSignal<Option<String>>,
    on_navigate: Callback<String>,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="modal-backdrop" on:click=move |_| set_show.set(false)>
                <div class="modal modal-sm" on:click=|ev| ev.stop_propagation()>
                    <div class="modal-header">
                        <h2>"Confirm Delete"</h2>
                        <button class="modal-close" on:click=move |_| set_show.set(false)>"✕"</button>
                    </div>
                    <div class="modal-body">
                        <p>
                            {"Are you sure you want to delete this "}
                            <strong>{entity_label.clone()}</strong>
                            {"? This action cannot be undone."}
                        </p>
                    </div>
                    <div class="modal-footer">
                        <button class="btn btn-secondary"
                            on:click=move |_| set_show.set(false)>
                            "Cancel"
                        </button>
                        <button class="btn btn-danger"
                            disabled=move || deleting.get()
                            on:click=move |_| {
                                let (ent, id) = path_info.get();
                                spawn_local(async move {
                                    match delete_record(&ent, &id).await {
                                        Ok(_) => {
                                            on_navigate.call(format!("/{}", ent));
                                        }
                                        Err(err) => {
                                            on_error.set(Some(err));
                                            set_show.set(false);
                                        }
                                    }
                                });
                            }>
                            {move || if deleting.get() { "Deleting..." } else { "Delete" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
