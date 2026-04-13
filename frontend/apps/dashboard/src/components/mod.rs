//! Shared UI components

use leptos::*;

/// A reusable button component
#[component]
pub fn Button(
    #[prop(optional)] variant: &'static str,
    #[prop(optional)] disabled: bool,
    children: Children,
) -> impl IntoView {
    let class = match variant {
        "primary" => "btn btn-primary",
        "danger" => "btn btn-danger",
        "secondary" => "btn btn-secondary",
        _ => "btn",
    };
    
    view! {
        <button class=class disabled=disabled>
            {children()}
        </button>
    }
}

/// A card container component
#[component]
pub fn Card(
    #[prop(optional)] title: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="card">
            {if !title.is_empty() {
                view! { <h2 class="card-title">{title}</h2> }.into_view()
            } else {
                ().into_view()
            }}
            {children()}
        </div>
    }
}

/// A loading spinner
#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <div class="spinner">
            <div class="spinner-inner"></div>
        </div>
    }
}

/// A notification/alert component
#[component]
pub fn Alert(
    #[prop(optional)] alert_type: &'static str,
    children: Children,
) -> impl IntoView {
    let class = format!("alert alert-{}", alert_type);
    view! {
        <div class=class>
            {children()}
        </div>
    }
}
