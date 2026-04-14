//! Modal component

use leptos::*;

/// Modal dialog component
#[component]
pub fn Modal(
    #[prop(optional)] size: &'static str,
    title: String,
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let size_class = match size {
        "sm" => "modal-sm",
        "lg" => "modal-lg",
        "xl" => "modal-xl",
        _ => "modal-md",
    };

    let on_backdrop_click = move |_| {
        set_open.set(false);
    };

    let on_close = move |_| {
        set_open.set(false);
    };

    view! {
        <Show when=move || open.get()>
            <div class="modal-backdrop" on:click=on_backdrop_click>
                <div class=format!("modal {}", size_class) on:click=|ev| ev.stop_propagation()>
                    <div class="modal-header">
                        <h2>{title.clone()}</h2>
                        <button class="modal-close" on:click=on_close>"✕"</button>
                    </div>
                    <div class="modal-body">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}
