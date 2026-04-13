//! Entity detail page

use leptos::*;

/// Entity detail page component
#[component]
pub fn EntityDetailPage() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1>"Record Detail"</h1>
            <div class="page-actions">
                <button class="btn">"Back"</button>
                <button class="btn btn-primary">"Save"</button>
            </div>
        </div>
        <div class="card">
            <p>"Record details will be loaded dynamically from the API."</p>
        </div>
    }
}
