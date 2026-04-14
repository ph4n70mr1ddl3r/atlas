//! Entity list page - dynamically renders entity records in a data table

use leptos::*;
use leptos_router::*;
use crate::components::DataTable;

/// Entity list page component
#[component]
pub fn EntityListPage() -> impl IntoView {
    let location = use_location();
    let pathname = location.pathname;

    let entity_name = create_memo(move |_| {
        let path = pathname.get();
        path.trim_start_matches('/').to_string()
    });

    let entity_label = create_memo(move |_| {
        match entity_name.get().as_str() {
            "employees" => "Employees".to_string(),
            "customers" => "Customers".to_string(),
            "orders" => "Sales Orders".to_string(),
            "products" => "Products".to_string(),
            "projects" => "Projects".to_string(),
            "invoices" => "Invoices".to_string(),
            other => {
                // Title case
                other.split('_')
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
        }
    });

    view! {
        {move || {
            let entity = entity_name.get();
            let label = entity_label.get();
            view! {
                <DataTable entity=entity entity_label=label />
            }
        }}
    }
}
