//! Workflow status visualization component

use leptos::*;
use serde_json::Value;
use crate::api::client::{WorkflowDefinition, TransitionInfo, execute_action};

/// Workflow status bar showing state progression
#[component]
pub fn WorkflowBar(
    entity: String,
    record_id: String,
    current_status: String,
    workflow: Option<WorkflowDefinition>,
    transitions: Option<TransitionInfo>,
    on_action_completed: Callback<Value>,
) -> impl IntoView {
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let entity_clone = entity.clone();
    let record_id_clone = record_id.clone();
    let current_status_for_bar = current_status.clone();

    // Build the state steps from the workflow definition
    let states: Vec<(String, String, bool, bool)> = workflow.as_ref().map(|wf| {
        let cs = &current_status;
        wf.states.iter().map(|s| {
            let is_current = s.name == *cs;
            let is_completed = is_state_before(wf, &s.name, cs);
            (s.name.clone(), s.label.clone(), is_completed, is_current)
        }).collect()
    }).unwrap_or_default();

    let available = transitions.map(|t| t.available_transitions).unwrap_or_default();

    view! {
        <div class="card" style="margin-bottom: 1.5rem">
            <div class="card-header">
                <h3 class="card-title">"Workflow Status"</h3>
                <span class=format!("badge {}", badge_for_status(&current_status_for_bar))>
                    {current_status_for_bar}
                </span>
            </div>

            // State progression bar
            {if !states.is_empty() {
                view! {
                    <div class="workflow-status">
                        {states.iter().enumerate().map(|(i, (name, label, completed, current))| {
                            let dot_class = if *current { "current" }
                                else if *completed { "completed" }
                                else { "" };
                            let connector_completed = *completed || *current;
                            view! {
                                <div class="workflow-step">
                                    <div class=format!("workflow-step-dot {}", dot_class)>
                                        {if *completed { "✓".to_string() }
                                         else { (i + 1).to_string() }}
                                    </div>
                                    <span class="workflow-step-label">{label}</span>
                                    {if i < states.len() - 1 {
                                        view! {
                                            <div class=format!("workflow-connector{}", if connector_completed { " completed" } else { "" })></div>
                                        }.into_view()
                                    } else {
                                        ().into_view()
                                    }}
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_view()
            } else {
                ().into_view()
            }}

            // Available actions
            {if !available.is_empty() {
                view! {
                    <div class="workflow-actions">
                        <span class="text-sm text-gray-500" style="margin-right: 0.5rem">"Actions:"</span>
                        {available.iter().map(|t| {
                            let label = t.label.clone().unwrap_or_else(|| title_case(&t.action));
                            let btn_class = if t.to_state.contains("reject") || t.to_state.contains("cancel") {
                                "btn btn-danger btn-sm"
                            } else if t.to_state.contains("approve") || t.to_state.contains("complete") {
                                "btn btn-primary btn-sm"
                            } else {
                                "btn btn-secondary btn-sm"
                            };
                            view! {
                                <button
                                    class=btn_class
                                    disabled=move || loading.get()
                                >
                                    {label}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                }.into_view()
            } else {
                ().into_view()
            }}

            // Error
            {move || error.get().map(|e| view! {
                <div class="alert alert-danger" style="margin-top: 1rem">
                    {e}
                </div>
            })}

            // Loading
            {move || if loading.get() {
                view! { <div class="spinner spinner-sm" style="margin-top: 1rem"><div class="spinner-inner"></div></div> }.into_view()
            } else {
                ().into_view()
            }}
        </div>
    }
}

/// Check if a state comes before the current state in the workflow
fn is_state_before(workflow: &WorkflowDefinition, state: &str, current: &str) -> bool {
    if state == current {
        return false;
    }

    let mut visited = std::collections::HashSet::new();
    let mut queue = vec![workflow.initial_state.clone()];
    visited.insert(workflow.initial_state.clone());

    while let Some(s) = queue.pop() {
        if s == current {
            return visited.contains(state);
        }
        for t in &workflow.transitions {
            if t.from == s && !visited.contains(&t.to) {
                visited.insert(t.to.clone());
                queue.push(t.to.clone());
            }
        }
    }

    visited.contains(state) && !visited.contains(current)
}

fn badge_for_status(status: &str) -> &'static str {
    match status {
        "active" | "approved" | "completed" | "paid" | "delivered" | "shipped" => "badge-success",
        "draft" | "pending" | "planning" | "new" => "badge-warning",
        "inactive" | "rejected" | "cancelled" | "overdue" | "closed" => "badge-danger",
        "submitted" | "in_progress" | "sent" | "processing" | "on_hold" => "badge-info",
        _ => "badge-neutral",
    }
}

fn title_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
