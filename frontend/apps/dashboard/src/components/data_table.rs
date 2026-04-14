//! Data table component with sorting, pagination, and search

use leptos::*;
use leptos_router::*;
use serde_json::Value;
use crate::api::client::{fetch_records, PaginationMeta};

/// Data table component
#[component]
pub fn DataTable(
    entity: String,
    entity_label: String,
) -> impl IntoView {
    let navigate = use_navigate();

    // State
    let (records, set_records) = create_signal::<Vec<Value>>(vec![]);
    let (meta, set_meta) = create_signal(PaginationMeta {
        total: 0,
        offset: 0,
        limit: 20,
        has_more: false,
    });
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (search, set_search) = create_signal(String::new());
    let (page, set_page) = create_signal(0i64);

    let per_page: i64 = 20;

    // Fetch data
    let entity_c = entity.clone();
    let load_data = create_action(move |_: &()| {
        let entity = entity_c.clone();
        let search = search.get();
        let page = page.get();
        async move {
            set_loading.set(true);
            set_error.set(None);

            let offset = page * per_page;
            let search_opt: Option<&str> = if search.is_empty() { None } else { Some(&search) };

            match fetch_records(&entity, offset, per_page, search_opt).await {
                Ok(response) => {
                    set_records.set(response.data);
                    set_meta.set(response.meta);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_records.set(vec![]);
                }
            }
            set_loading.set(false);
        }
    });

    // Initial load
    load_data.dispatch(());

    // Refresh handler
    let on_refresh = move |_| {
        load_data.dispatch(());
    };

    // Search handler
    let on_search_change = move |ev| {
        let value = event_target_value(&ev);
        set_search.set(value);
        set_page.set(0);
        load_data.dispatch(());
    };

    // Pagination
    let on_prev = move |_| {
        let p = page.get();
        if p > 0 {
            set_page.set(p - 1);
            load_data.dispatch(());
        }
    };

    let on_next = move |_| {
        if meta.get().has_more {
            set_page.set(page.get() + 1);
            load_data.dispatch(());
        }
    };

    let entity_for_rows = entity.clone();

    view! {
        <div>
            // Header
            <div class="page-header">
                <div class="page-header-left">
                    <h1>{entity_label.clone()}</h1>
                    <p class="subtitle">
                        {move || format!("{} records total", meta.get().total)}
                    </p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-secondary" on:click=on_refresh>
                        "↻ Refresh"
                    </button>
                </div>
            </div>

            // Error state
            {move || error.get().map(|e| view! {
                <div class="alert alert-danger" style="margin-bottom: 1rem">
                    {e}
                </div>
            })}

            // Table card
            <div class="card" style="padding: 0">
                // Toolbar
                <div class="table-toolbar" style="padding: 1rem 1.5rem">
                    <div class="table-search">
                        <span class="table-search-icon">"🔍"</span>
                        <input
                            type="text"
                            placeholder="Search..."
                            on:change=on_search_change
                            prop:value=move || search.get()
                        />
                    </div>
                    <div class="table-filters">
                        <span class="text-sm text-gray-500">
                            {move || format!("Showing {} of {}", records.get().len(), meta.get().total)}
                        </span>
                    </div>
                </div>

                // Loading state
                {move || if loading.get() {
                    view! {
                        <div class="spinner">
                            <div class="spinner-inner"></div>
                        </div>
                    }.into_view()
                } else {
                    ().into_view()
                }}

                // Table
                <div class="data-table-container">
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
                            <TableRows records=records loading=loading entity=entity_for_rows.clone()/>
                        </tbody>
                    </table>
                </div>

                // Pagination
                <div class="pagination" style="padding: 1rem 1.5rem">
                    <div class="pagination-info">
                        {move || {
                            let m = meta.get();
                            let start = if m.total == 0 { 0 } else { m.offset + 1 };
                            let end = m.offset + (records.get().len() as i64);
                            format!("Showing {}-{} of {}", start, end, m.total)
                        }}
                    </div>
                    <div class="pagination-controls">
                        <button class="pagination-btn"
                            disabled=move || page.get() == 0
                            on:click=on_prev>
                            "‹"
                        </button>
                        <button class="pagination-btn active">
                            {move || (page.get() + 1).to_string()}
                        </button>
                        <button class="pagination-btn"
                            disabled=move || !meta.get().has_more
                            on:click=on_next>
                            "›"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Get badge CSS class for a status value
fn status_badge_class(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "active" | "approved" | "completed" | "paid" | "done" => "badge-success",
        "draft" | "pending" | "new" | "open" => "badge-warning",
        "inactive" | "rejected" | "cancelled" | "overdue" | "closed" => "badge-danger",
        "submitted" | "in_progress" | "processing" | "in review" => "badge-info",
        _ => "badge-neutral",
    }
}

/// Table rows subcomponent to handle FnOnce issues
#[component]
fn TableRows(
    records: ReadSignal<Vec<Value>>,
    loading: ReadSignal<bool>,
    entity: String,
) -> impl IntoView {
    view! {
        {move || {
            let recs = records.get();
            let is_loading = loading.get();
            if recs.is_empty() && !is_loading {
                view! {
                    <tr>
                        <td colspan="99" class="empty-state" style="padding: 3rem">
                            <div class="empty-state-icon">"📭"</div>
                            <h3>"No records found"</h3>
                            <p>"Create your first record or adjust your search."</p>
                        </td>
                    </tr>
                }.into_view()
            } else {
                recs.iter().map(|record| {
                    let name_val = record.get("name")
                        .or(record.get("title"))
                        .or(record.get("first_name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("—")
                        .to_string();
                    let status_val = record.get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("—")
                        .to_string();
                    let created_val = record.get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("—")
                        .to_string();
                    let badge_class = status_badge_class(&status_val);

                    view! {
                        <tr>
                            <td>{name_val}</td>
                            <td>
                                <span class=format!("badge {}", badge_class)>
                                    {status_val}
                                </span>
                            </td>
                            <td>{created_val}</td>
                            <td>
                                <button class="btn btn-ghost btn-sm">"View →"</button>
                            </td>
                        </tr>
                    }
                }).collect_view()
            }
        }}
    }
}
