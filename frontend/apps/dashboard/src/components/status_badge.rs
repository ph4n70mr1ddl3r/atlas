//! Status badge component

use leptos::*;

/// A colored badge for displaying status values
#[component]
pub fn StatusBadge(status: String) -> impl IntoView {
    let class = match status.to_lowercase().as_str() {
        "active" | "approved" | "completed" | "paid" | "done" | "delivered" | "shipped" => "badge-success",
        "draft" | "pending" | "new" | "open" | "planning" | "prospect" => "badge-warning",
        "inactive" | "rejected" | "cancelled" | "overdue" | "closed" => "badge-danger",
        "submitted" | "in_progress" | "sent" | "processing" | "in review" | "on_hold" => "badge-info",
        _ => "badge-neutral",
    };

    view! {
        <span class=format!("badge {}", class)>
            {status}
        </span>
    }
}
