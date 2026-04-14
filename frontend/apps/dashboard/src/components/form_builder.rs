//! Dynamic form builder - renders fields from entity schema

use leptos::*;
use serde_json::{Value, json, Map};
use crate::api::client::{create_record, update_record, fetch_entity_schema, FieldSchema};

/// Dynamic form component driven by schema
#[component]
pub fn FormBuilder(
    entity: String,
    record_id: Option<String>,
    initial_data: Option<Value>,
    on_success: Callback<Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (fields, set_fields) = create_signal::<Vec<FieldSchema>>(vec![]);
    let (form_data, set_form_data) = create_signal::<Map<String, Value>>(
        initial_data
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default()
    );
    let (saving, set_saving) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal::<Option<String>>(None);

    // Load schema
    let entity_for_schema = entity.clone();
    let load_schema = create_action(move |_: &()| {
        let entity = entity_for_schema.clone();
        async move {
            match fetch_entity_schema(&entity).await {
                Ok(schema) => {
                    set_fields.set(schema.fields);
                }
                Err(_) => {
                    set_fields.set(fallback_fields(&entity));
                }
            }
        }
    });
    load_schema.dispatch(());

    let is_edit = record_id.is_some();

    // Handle form submission
    let entity_for_submit = entity.clone();
    let id_for_submit = record_id.clone();
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_saving.set(true);
        set_error_msg.set(None);

        let data = form_data.get();
        let json_data = Value::Object(data);
        let entity = entity_for_submit.clone();
        let id = id_for_submit.clone();

        spawn_local(async move {
            let result = match id {
                Some(id) => update_record(&entity, &id, &json_data).await,
                None => create_record(&entity, &json_data).await,
            };

            match result {
                Ok(saved) => {
                    on_success.call(saved);
                }
                Err(e) => {
                    set_error_msg.set(Some(e));
                }
            }
            set_saving.set(false);
        });
    };

    view! {
        <div>
            {move || error_msg.get().map(|e| view! {
                <div class="alert alert-danger" style="margin-bottom: 1rem">
                    {e}
                </div>
            })}

            <form on:submit=on_submit>
                <div class="form-grid">
                    <For
                        each=move || fields.get()
                        key=|field| field.name.clone()
                        children=move |field: FieldSchema| {
                            let fname = field.name.clone();
                            let flabel = field.label.clone();
                            let ftype = field.field_type.clone();
                            let freq = field.required;
                            let fhelp = field.help_text.clone();
                            let fro = field.is_read_only && is_edit;
                            let ffull = matches!(ftype.as_str(), "rich_text" | "json" | "text" | "attachment");
                            let fconfig = field.type_config.clone();
                            view! {
                                <FormField
                                    name=fname
                                    label=flabel
                                    field_type=ftype
                                    required=freq
                                    help_text=fhelp
                                    read_only=fro
                                    full_width=ffull
                                    type_config=fconfig
                                    form_data=form_data
                                    set_form_data=set_form_data
                                />
                            }
                        }
                    />
                </div>

                // Form actions
                <div class="flex gap-3 mt-6">
                    <button
                        type="submit"
                        class="btn btn-primary"
                        disabled=move || saving.get()
                    >
                        {move || if saving.get() {
                            "Saving...".to_string()
                        } else if is_edit {
                            "Save Changes".to_string()
                        } else {
                            "Create Record".to_string()
                        }}
                    </button>
                    <button
                        type="button"
                        class="btn btn-secondary"
                        on:click=move |_| on_cancel.call(())
                    >
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }
}

/// Individual form field component
#[component]
pub fn FormField(
    name: String,
    label: String,
    field_type: String,
    #[prop(default = false)] required: bool,
    #[prop(default = None)] help_text: Option<String>,
    #[prop(default = false)] read_only: bool,
    #[prop(default = false)] full_width: bool,
    #[prop(default = None)] type_config: Option<Value>,
    form_data: ReadSignal<Map<String, Value>>,
    set_form_data: WriteSignal<Map<String, Value>>,
) -> impl IntoView {
    let name_for_lookup = name.clone();
    let current_val = move || form_data.get()
        .get(&name_for_lookup)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let group_class = format!("form-group{}", if full_width { " full-width" } else { "" });

    view! {
        <div class=group_class>
            <label class="form-label">
                {label.clone()}
                {if required {
                    view! { <span class="required">"*"</span> }.into_view()
                } else {
                    ().into_view()
                }}
            </label>

            {match field_type.as_str() {
                "text" | "rich_text" => {
                    let cv = current_val();
                    let nn = name.clone();
                    view! {
                        <textarea
                            class="form-textarea"
                            placeholder=format!("Enter {}...", label.to_lowercase())
                            required=required
                            disabled=read_only
                            prop:value=cv
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_form_data.update(|d| { d.insert(nn.clone(), json!(val)); });
                            }
                        ></textarea>
                    }.into_view()
                }
                "integer" | "decimal" | "currency" => {
                    let cv = current_val();
                    let nn = name.clone();
                    view! {
                        <input
                            type="number"
                            class="form-input"
                            placeholder=format!("Enter {}...", label.to_lowercase())
                            required=required
                            disabled=read_only
                            step=if field_type == "integer" { "1" } else { "0.01" }
                            prop:value=cv
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_form_data.update(|d| { d.insert(nn.clone(), json!(val)); });
                            }
                        />
                    }.into_view()
                }
                "boolean" => {
                    let cv = current_val();
                    let nn = name.clone();
                    view! {
                        <label class="flex items-center gap-2" style="cursor: pointer">
                            <input
                                type="checkbox"
                                disabled=read_only
                                prop:checked=cv == "true"
                                on:change=move |ev| {
                                    let val = event_target_checked(&ev);
                                    set_form_data.update(|d| { d.insert(nn.clone(), json!(val)); });
                                }
                            />
                            <span class="text-sm">{label.clone()}</span>
                        </label>
                    }.into_view()
                }
                "date" => {
                    let cv = current_val();
                    let nn = name.clone();
                    view! {
                        <input
                            type="date"
                            class="form-input"
                            required=required
                            disabled=read_only
                            prop:value=cv
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_form_data.update(|d| { d.insert(nn.clone(), json!(val)); });
                            }
                        />
                    }.into_view()
                }
                "datetime" => {
                    let cv = current_val();
                    let nn = name.clone();
                    view! {
                        <input
                            type="datetime-local"
                            class="form-input"
                            required=required
                            disabled=read_only
                            prop:value=cv
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_form_data.update(|d| { d.insert(nn.clone(), json!(val)); });
                            }
                        />
                    }.into_view()
                }
                "enum" => {
                    let values = type_config.as_ref()
                        .and_then(|c| c.get("values"))
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
                        .unwrap_or_default();
                    let cv = current_val();
                    let nn = name.clone();
                    view! {
                        <select
                            class="form-select"
                            required=required
                            disabled=read_only
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_form_data.update(|d| { d.insert(nn.clone(), json!(val)); });
                            }
                        >
                            <option value="">"Select..."</option>
                            {values.iter().map(|v| {
                                let selected = v == &cv;
                                view! {
                                    <option value=v.clone() selected=selected>{v.clone()}</option>
                                }
                            }).collect_view()}
                        </select>
                    }.into_view()
                }
                "reference" => {
                    let ref_entity = type_config.as_ref()
                        .and_then(|c| c.get("entity"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let cv = current_val();
                    let nn = name.clone();
                    view! {
                        <div>
                            <input
                                type="text"
                                class="form-input"
                                placeholder=format!("Search {}...", ref_entity)
                                required=required
                                disabled=read_only
                                prop:value=cv
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_form_data.update(|d| { d.insert(nn.clone(), json!(val)); });
                                }
                            />
                            <p class="form-help">{format!("Reference to: {}", ref_entity)}</p>
                        </div>
                    }.into_view()
                }
                // "string", "email", "phone", "url", and default
                _ => {
                    let input_type = if field_type == "email" { "email" }
                        else if field_type == "phone" { "tel" }
                        else if field_type == "url" { "url" }
                        else { "text" };
                    let cv = current_val();
                    let nn = name.clone();
                    view! {
                        <input
                            type=input_type
                            class="form-input"
                            placeholder=format!("Enter {}...", label.to_lowercase())
                            required=required
                            disabled=read_only
                            prop:value=cv
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_form_data.update(|d| { d.insert(nn.clone(), json!(val)); });
                            }
                        />
                    }.into_view()
                }
            }}

            // Help text
            {help_text.map(|h| view! {
                <span class="form-help">{h}</span>
            })}
        </div>
    }
}

/// Fallback field definitions for common entities when schema API is unavailable
fn fallback_fields(entity: &str) -> Vec<FieldSchema> {
    match entity {
        "employees" => vec![
            FieldSchema { name: "first_name".into(), label: "First Name".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "last_name".into(), label: "Last Name".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "email".into(), label: "Email".into(), field_type: "email".into(), required: true, editable: true, visible: true, is_unique: true, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "phone".into(), label: "Phone".into(), field_type: "phone".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "department".into(), label: "Department".into(), field_type: "string".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "position".into(), label: "Position".into(), field_type: "string".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "status".into(), label: "Status".into(), field_type: "enum".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: Some(json!({"values": ["active", "inactive", "on_leave"]})), default_value: Some(json!("active")), help_text: None },
        ],
        "customers" => vec![
            FieldSchema { name: "name".into(), label: "Company Name".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "contact_name".into(), label: "Contact Name".into(), field_type: "string".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "email".into(), label: "Email".into(), field_type: "email".into(), required: true, editable: true, visible: true, is_unique: true, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "phone".into(), label: "Phone".into(), field_type: "phone".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "industry".into(), label: "Industry".into(), field_type: "string".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "status".into(), label: "Status".into(), field_type: "enum".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: Some(json!({"values": ["active", "inactive", "prospect"]})), default_value: Some(json!("active")), help_text: None },
        ],
        "orders" => vec![
            FieldSchema { name: "order_number".into(), label: "Order Number".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: true, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "customer_id".into(), label: "Customer".into(), field_type: "reference".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: Some(json!({"entity": "customers"})), default_value: None, help_text: None },
            FieldSchema { name: "total_amount".into(), label: "Total Amount".into(), field_type: "currency".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: true, type_config: None, default_value: None, help_text: Some("Auto-calculated from order lines".into()) },
            FieldSchema { name: "status".into(), label: "Status".into(), field_type: "enum".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: Some(json!({"values": ["draft", "submitted", "approved", "rejected", "shipped", "delivered", "closed"]})), default_value: Some(json!("draft")), help_text: None },
            FieldSchema { name: "notes".into(), label: "Notes".into(), field_type: "text".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
        ],
        "products" => vec![
            FieldSchema { name: "name".into(), label: "Product Name".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "sku".into(), label: "SKU".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: true, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "description".into(), label: "Description".into(), field_type: "text".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "price".into(), label: "Price".into(), field_type: "currency".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "quantity".into(), label: "Stock Quantity".into(), field_type: "integer".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: Some(json!(0)), help_text: None },
        ],
        "projects" => vec![
            FieldSchema { name: "name".into(), label: "Project Name".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "description".into(), label: "Description".into(), field_type: "text".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "start_date".into(), label: "Start Date".into(), field_type: "date".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "end_date".into(), label: "End Date".into(), field_type: "date".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "status".into(), label: "Status".into(), field_type: "enum".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: Some(json!({"values": ["planning", "in_progress", "on_hold", "completed", "cancelled"]})), default_value: Some(json!("planning")), help_text: None },
        ],
        "invoices" => vec![
            FieldSchema { name: "invoice_number".into(), label: "Invoice Number".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: true, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "customer_id".into(), label: "Customer".into(), field_type: "reference".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: Some(json!({"entity": "customers"})), default_value: None, help_text: None },
            FieldSchema { name: "amount".into(), label: "Amount".into(), field_type: "currency".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "due_date".into(), label: "Due Date".into(), field_type: "date".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "status".into(), label: "Status".into(), field_type: "enum".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: Some(json!({"values": ["draft", "sent", "paid", "overdue", "cancelled"]})), default_value: Some(json!("draft")), help_text: None },
        ],
        _ => vec![
            FieldSchema { name: "name".into(), label: "Name".into(), field_type: "string".into(), required: true, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: None, default_value: None, help_text: None },
            FieldSchema { name: "status".into(), label: "Status".into(), field_type: "enum".into(), required: false, editable: true, visible: true, is_unique: false, is_read_only: false, type_config: Some(json!({"values": ["active", "inactive"]})), default_value: None, help_text: None },
        ],
    }
}
