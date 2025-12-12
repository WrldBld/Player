//! Workflow Configuration Editor Component
//!
//! Displays and allows editing of a configured workflow's settings,
//! including prompt mappings, input defaults, and locked inputs.

use dioxus::prelude::*;

use crate::infrastructure::http_client::HttpClient;

/// Props for the WorkflowConfigEditor component
#[derive(Props, Clone, PartialEq)]
pub struct WorkflowConfigEditorProps {
    /// The slot ID to show configuration for
    pub slot: String,
    /// Callback when close is clicked
    pub on_close: EventHandler<()>,
    /// Callback when reconfigure is clicked
    pub on_reconfigure: EventHandler<()>,
    /// Callback when workflow is deleted
    pub on_deleted: EventHandler<()>,
}

/// Full workflow configuration data
#[derive(Clone, Debug, Default)]
pub struct WorkflowConfigFull {
    pub id: String,
    pub slot: String,
    pub slot_display_name: String,
    pub name: String,
    pub analysis: WorkflowAnalysisData,
    pub prompt_mappings: Vec<PromptMappingData>,
    pub input_defaults: Vec<InputDefaultData>,
    pub locked_inputs: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default)]
pub struct WorkflowAnalysisData {
    pub node_count: usize,
    pub inputs: Vec<WorkflowInputData>,
    pub text_inputs: Vec<WorkflowInputData>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkflowInputData {
    pub node_id: String,
    pub node_type: String,
    pub node_title: Option<String>,
    pub input_name: String,
    pub input_type: String,
    pub current_value: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PromptMappingData {
    pub node_id: String,
    pub input_name: String,
    pub mapping_type: String, // "primary" or "negative"
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputDefaultData {
    pub node_id: String,
    pub input_name: String,
    pub default_value: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkflowTestResult {
    pub image_url: String,
    pub duration_ms: u64,
}

/// Workflow configuration editor
#[component]
pub fn WorkflowConfigEditor(props: WorkflowConfigEditorProps) -> Element {
    // Track loading state
    let mut is_loading = use_signal(|| true);
    // Track error state
    let mut error: Signal<Option<String>> = use_signal(|| None);
    // Store the config
    let mut config: Signal<Option<WorkflowConfigFull>> = use_signal(|| None);
    // Track which section is expanded
    let mut expanded_section = use_signal(|| "mappings");
    // Track if we're saving
    let mut is_saving = use_signal(|| false);
    // Track edits to defaults
    let mut edited_defaults: Signal<Vec<InputDefaultData>> = use_signal(Vec::new);
    // Track delete confirmation dialog visibility
    let mut show_delete_confirmation = use_signal(|| false);
    // Track if deleting
    let mut is_deleting = use_signal(|| false);
    // Track test modal visibility
    let mut show_test_modal = use_signal(|| false);
    // Track test state
    let mut test_prompt = use_signal(|| "".to_string());
    let mut is_testing = use_signal(|| false);
    let mut test_result: Signal<Option<WorkflowTestResult>> = use_signal(|| None);
    let mut test_error: Signal<Option<String>> = use_signal(|| None);

    let slot_id = props.slot.clone();
    let slot_id_for_effect = slot_id.clone();

    // Fetch config on mount or when slot changes
    use_effect(move || {
        let slot = slot_id_for_effect.clone();
        spawn(async move {
            is_loading.set(true);
            error.set(None);

            match fetch_workflow_config(&slot).await {
                Ok(Some(fetched_config)) => {
                    edited_defaults.set(fetched_config.input_defaults.clone());
                    config.set(Some(fetched_config));
                    is_loading.set(false);
                }
                Ok(None) => {
                    config.set(None);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e));
                    is_loading.set(false);
                }
            }
        });
    });

    let slot_id_for_save = slot_id.clone();
    // Save handler
    let save_config = move |_| {
        let slot = slot_id_for_save.clone();
        let defaults = edited_defaults.read().clone();
        let current_config = config.read().clone();

        spawn(async move {
            is_saving.set(true);

            if let Some(cfg) = current_config {
                match save_workflow_defaults(&slot, &cfg.name, defaults).await {
                    Ok(_) => {
                        // Refresh the config
                        if let Ok(Some(updated)) = fetch_workflow_config(&slot).await {
                            config.set(Some(updated));
                        }
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
            }

            is_saving.set(false);
        });
    };

    let slot_id_for_delete = slot_id.clone();
    // Delete handler
    let on_deleted = props.on_deleted.clone();
    let do_delete = move |_| {
        let slot = slot_id_for_delete.clone();
        let callback = on_deleted.clone();

        spawn(async move {
            is_deleting.set(true);

            match delete_workflow_config(&slot).await {
                Ok(_) => {
                    show_delete_confirmation.set(false);
                    callback.call(());
                }
                Err(e) => {
                    error.set(Some(e));
                    is_deleting.set(false);
                    show_delete_confirmation.set(false);
                }
            }
        });
    };

    let slot_id_for_test = slot_id.clone();
    // Test handler
    let do_test = move |_| {
        let slot = slot_id_for_test.clone();
        let prompt = test_prompt.read().clone();

        spawn(async move {
            is_testing.set(true);
            test_error.set(None);
            test_result.set(None);

            match test_workflow(&slot, &prompt).await {
                Ok(result) => {
                    test_result.set(Some(result));
                }
                Err(e) => {
                    test_error.set(Some(e));
                }
            }

            is_testing.set(false);
        });
    };

    rsx! {
        div {
            class: "workflow-config-editor",
            style: "flex: 1; display: flex; flex-direction: column; background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

            if *is_loading.read() {
                div {
                    style: "flex: 1; display: flex; align-items: center; justify-content: center; color: #6b7280;",
                    "Loading configuration..."
                }
            } else if let Some(err) = error.read().as_ref() {
                div {
                    style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 2rem;",

                    div {
                        style: "padding: 1rem; background: rgba(239, 68, 68, 0.1); border-radius: 0.5rem; color: #ef4444; font-size: 0.875rem; margin-bottom: 1rem;",
                        "Error: {err}"
                    }

                    button {
                        onclick: move |_| props.on_reconfigure.call(()),
                        style: "padding: 0.5rem 1rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                        "Configure Workflow"
                    }
                }
            } else if let Some(cfg) = config.read().as_ref() {
                // Header
                div {
                    style: "padding: 1rem; border-bottom: 1px solid #374151;",

                    div {
                        style: "display: flex; align-items: center; justify-content: space-between;",

                        div {
                            h2 {
                                style: "color: white; font-size: 1.125rem; margin: 0 0 0.25rem 0;",
                                "{cfg.slot_display_name}"
                            }
                            p {
                                style: "color: #22c55e; font-size: 0.875rem; margin: 0;",
                                "✓ {cfg.name}"
                            }
                        }

                        div {
                            style: "display: flex; gap: 0.5rem;",

                            button {
                                onclick: move |_| {
                                    test_prompt.set("".to_string());
                                    test_result.set(None);
                                    test_error.set(None);
                                    show_test_modal.set(true);
                                },
                                style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                                "Test Workflow"
                            }

                            button {
                                onclick: move |_| props.on_reconfigure.call(()),
                                style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                                "Reconfigure"
                            }

                            button {
                                onclick: move |_| show_delete_confirmation.set(true),
                                style: "padding: 0.5rem 1rem; background: #dc2626; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                                "Delete"
                            }
                        }
                    }
                }

                // Stats bar
                div {
                    style: "display: flex; gap: 1rem; padding: 0.75rem 1rem; background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid #374151;",

                    StatBadge { label: "Nodes", value: cfg.analysis.node_count.to_string() }
                    StatBadge { label: "Inputs", value: cfg.analysis.inputs.len().to_string() }
                    StatBadge { label: "Text Inputs", value: cfg.analysis.text_inputs.len().to_string() }
                }

                // Content sections
                div {
                    style: "flex: 1; overflow-y: auto; padding: 1rem;",

                    // Prompt Mappings section
                    CollapsibleSection {
                        title: "Prompt Mappings",
                        is_expanded: *expanded_section.read() == "mappings",
                        on_toggle: move |_| {
                            if *expanded_section.read() == "mappings" {
                                expanded_section.set("");
                            } else {
                                expanded_section.set("mappings");
                            }
                        },

                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",

                            if cfg.prompt_mappings.is_empty() {
                                div {
                                    style: "color: #6b7280; font-size: 0.875rem; padding: 0.5rem;",
                                    "No prompt mappings configured"
                                }
                            } else {
                                for mapping in cfg.prompt_mappings.iter() {
                                    PromptMappingRow {
                                        mapping: mapping.clone(),
                                        inputs: cfg.analysis.text_inputs.clone(),
                                    }
                                }
                            }
                        }
                    }

                    // Input Defaults section
                    CollapsibleSection {
                        title: "Input Defaults",
                        is_expanded: *expanded_section.read() == "defaults",
                        on_toggle: move |_| {
                            if *expanded_section.read() == "defaults" {
                                expanded_section.set("");
                            } else {
                                expanded_section.set("defaults");
                            }
                        },

                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",

                            for input in cfg.analysis.inputs.iter() {
                                InputDefaultRow {
                                    input: input.clone(),
                                    defaults: edited_defaults.read().clone(),
                                    locked: cfg.locked_inputs.contains(&format!("{}:{}", input.node_id, input.input_name)),
                                    on_change: move |new_value: InputDefaultData| {
                                        let mut current = edited_defaults.read().clone();
                                        // Find and update or add
                                        if let Some(existing) = current.iter_mut().find(|d| {
                                            d.node_id == new_value.node_id && d.input_name == new_value.input_name
                                        }) {
                                            existing.default_value = new_value.default_value;
                                        } else {
                                            current.push(new_value);
                                        }
                                        edited_defaults.set(current);
                                    },
                                }
                            }
                        }
                    }

                    // Workflow Info section
                    CollapsibleSection {
                        title: "Workflow Info",
                        is_expanded: *expanded_section.read() == "info",
                        on_toggle: move |_| {
                            if *expanded_section.read() == "info" {
                                expanded_section.set("");
                            } else {
                                expanded_section.set("info");
                            }
                        },

                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.875rem;",

                            InfoRow { label: "ID", value: cfg.id.clone() }
                            InfoRow { label: "Created", value: cfg.created_at.clone() }
                            InfoRow { label: "Updated", value: cfg.updated_at.clone() }
                        }
                    }
                }

                // Footer with save button
                div {
                    style: "padding: 1rem; border-top: 1px solid #374151; display: flex; justify-content: flex-end; gap: 0.5rem;",

                    button {
                        onclick: save_config,
                        disabled: *is_saving.read(),
                        style: "padding: 0.5rem 1.5rem; background: #22c55e; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 500;",
                        if *is_saving.read() { "Saving..." } else { "Save Changes" }
                    }
                }
            } else {
                // Not configured
                div {
                    style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 2rem;",

                    div {
                        style: "text-align: center; max-width: 300px;",

                        div {
                            style: "font-size: 2rem; margin-bottom: 1rem; opacity: 0.5;",
                            "📋"
                        }

                        h3 {
                            style: "color: #9ca3af; margin-bottom: 0.5rem;",
                            "Not Configured"
                        }

                        p {
                            style: "color: #6b7280; font-size: 0.875rem; margin-bottom: 1rem;",
                            "This workflow slot hasn't been configured yet. Upload a ComfyUI workflow to get started."
                        }

                        button {
                            onclick: move |_| props.on_reconfigure.call(()),
                            style: "padding: 0.75rem 1.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 500;",
                            "Configure Workflow"
                        }
                    }
                }

                // Delete confirmation modal
                if *show_delete_confirmation.read() {
                    ConfirmDeleteModal {
                        slot_name: if let Some(cfg) = config.read().as_ref() { cfg.slot_display_name.clone() } else { "Workflow".to_string() },
                        is_deleting: *is_deleting.read(),
                        on_confirm: do_delete,
                        on_cancel: move |_| show_delete_confirmation.set(false),
                    }
                }

                // Test workflow modal
                if *show_test_modal.read() {
                    TestWorkflowModal {
                        slot: props.slot.clone(),
                        test_prompt: test_prompt.read().clone(),
                        is_testing: *is_testing.read(),
                        test_result: test_result.read().clone(),
                        test_error: test_error.read().clone(),
                        on_prompt_change: move |prompt| test_prompt.set(prompt),
                        on_test: do_test,
                        on_close: move |_| {
                            show_test_modal.set(false);
                            test_prompt.set("".to_string());
                            test_result.set(None);
                            test_error.set(None);
                        },
                    }
                }
            }
        }
    }
}

/// Stat badge component
#[component]
fn StatBadge(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem;",

            span { style: "color: #6b7280; font-size: 0.75rem;", "{label}:" }
            span { style: "color: white; font-size: 0.875rem; font-weight: 500;", "{value}" }
        }
    }
}

/// Collapsible section component
#[derive(Props, Clone, PartialEq)]
struct CollapsibleSectionProps {
    title: &'static str,
    is_expanded: bool,
    on_toggle: EventHandler<()>,
    children: Element,
}

#[component]
fn CollapsibleSection(props: CollapsibleSectionProps) -> Element {
    rsx! {
        div {
            class: "collapsible-section",
            style: "margin-bottom: 0.75rem; background: rgba(0, 0, 0, 0.2); border-radius: 0.5rem; overflow: hidden;",

            button {
                onclick: move |_| props.on_toggle.call(()),
                style: "width: 100%; display: flex; align-items: center; justify-content: space-between; padding: 0.75rem 1rem; background: none; border: none; cursor: pointer; color: white;",

                span { style: "font-weight: 500; font-size: 0.875rem;", "{props.title}" }
                span { style: "color: #6b7280;", if props.is_expanded { "▼" } else { "▶" } }
            }

            if props.is_expanded {
                div {
                    style: "padding: 0 1rem 1rem 1rem;",
                    {props.children}
                }
            }
        }
    }
}

/// Prompt mapping row component
#[derive(Props, Clone, PartialEq)]
struct PromptMappingRowProps {
    mapping: PromptMappingData,
    inputs: Vec<WorkflowInputData>,
}

#[component]
fn PromptMappingRow(props: PromptMappingRowProps) -> Element {
    let input_info = props.inputs.iter().find(|i| {
        i.node_id == props.mapping.node_id && i.input_name == props.mapping.input_name
    });

    let node_label = input_info
        .and_then(|i| i.node_title.clone())
        .unwrap_or_else(|| format!("Node {}", props.mapping.node_id));

    let type_color = if props.mapping.mapping_type == "primary" {
        "#22c55e"
    } else {
        "#ef4444"
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem; background: rgba(0, 0, 0, 0.2); border-radius: 0.375rem;",

            span {
                style: format!("padding: 0.125rem 0.5rem; background: {}; color: white; font-size: 0.75rem; border-radius: 0.25rem; text-transform: uppercase;", type_color),
                "{props.mapping.mapping_type}"
            }

            div {
                style: "flex: 1;",

                span { style: "color: white; font-size: 0.875rem;", "{node_label}" }
                span { style: "color: #6b7280; font-size: 0.75rem; margin-left: 0.5rem;", "→ {props.mapping.input_name}" }
            }
        }
    }
}

/// Input default row component
#[derive(Props, Clone, PartialEq)]
struct InputDefaultRowProps {
    input: WorkflowInputData,
    defaults: Vec<InputDefaultData>,
    locked: bool,
    on_change: EventHandler<InputDefaultData>,
}

#[component]
fn InputDefaultRow(props: InputDefaultRowProps) -> Element {
    let current_default = props.defaults.iter().find(|d| {
        d.node_id == props.input.node_id && d.input_name == props.input.input_name
    });

    let display_value = current_default
        .map(|d| format_json_value(&d.default_value))
        .unwrap_or_else(|| format_json_value(&props.input.current_value));

    let node_label = props.input.node_title.clone()
        .unwrap_or_else(|| format!("Node {}", props.input.node_id));

    let input_for_change = props.input.clone();

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: rgba(0, 0, 0, 0.2); border-radius: 0.375rem;",

            // Lock indicator
            if props.locked {
                span { style: "color: #f59e0b; font-size: 0.75rem;", "🔒" }
            }

            // Input info
            div {
                style: "flex: 1; min-width: 0;",

                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",

                    span {
                        style: "color: #9ca3af; font-size: 0.75rem; padding: 0.125rem 0.375rem; background: #374151; border-radius: 0.25rem;",
                        "{props.input.input_type}"
                    }
                    span { style: "color: white; font-size: 0.875rem;", "{props.input.input_name}" }
                }

                span { style: "color: #6b7280; font-size: 0.75rem;", "{node_label}" }
            }

            // Value input
            input {
                r#type: "text",
                value: "{display_value}",
                disabled: props.locked,
                onchange: move |e| {
                    props.on_change.call(InputDefaultData {
                        node_id: input_for_change.node_id.clone(),
                        input_name: input_for_change.input_name.clone(),
                        default_value: parse_input_value(&e.value(), &input_for_change.input_type),
                    });
                },
                style: "width: 120px; padding: 0.375rem 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem;",
            }
        }
    }
}

/// Info row component
#[component]
fn InfoRow(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; padding: 0.25rem 0;",

            span { style: "color: #6b7280;", "{label}" }
            span { style: "color: #9ca3af; font-family: monospace; font-size: 0.8rem;", "{value}" }
        }
    }
}

/// Format JSON value for display
fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

/// Parse input value based on type
fn parse_input_value(value: &str, input_type: &str) -> serde_json::Value {
    match input_type {
        "integer" => value
            .parse::<i64>()
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::String(value.to_string())),
        "float" => value
            .parse::<f64>()
            .map(|n| serde_json::Value::Number(serde_json::Number::from_f64(n).unwrap()))
            .unwrap_or(serde_json::Value::String(value.to_string())),
        "boolean" => serde_json::Value::Bool(value == "true" || value == "1"),
        _ => serde_json::Value::String(value.to_string()),
    }
}

/// Confirm delete modal component
#[derive(Props, Clone, PartialEq)]
struct ConfirmDeleteModalProps {
    slot_name: String,
    is_deleting: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
fn ConfirmDeleteModal(props: ConfirmDeleteModalProps) -> Element {
    rsx! {
        div {
            class: "modal-backdrop",
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.75); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_cancel.call(()),

            div {
                class: "modal-content",
                style: "background: #1a1a2e; border-radius: 0.75rem; width: 90%; max-width: 400px; padding: 1.5rem; overflow: hidden;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem;",

                    div {
                        style: "color: #dc2626; font-size: 1.5rem;",
                        "!"
                    }

                    h2 {
                        style: "color: #dc2626; font-size: 1.125rem; margin: 0;",
                        "Delete Workflow Configuration"
                    }
                }

                // Message
                p {
                    style: "color: #9ca3af; margin: 1rem 0;",
                    "Are you sure you want to delete the configuration for {props.slot_name}? This action cannot be undone."
                }

                // Buttons
                div {
                    style: "display: flex; gap: 0.75rem; justify-content: flex-end; margin-top: 1.5rem;",

                    button {
                        onclick: move |_| props.on_cancel.call(()),
                        disabled: props.is_deleting,
                        style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                        "Cancel"
                    }

                    button {
                        onclick: move |_| props.on_confirm.call(()),
                        disabled: props.is_deleting,
                        style: "padding: 0.5rem 1rem; background: #dc2626; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem; font-weight: 500;",
                        if props.is_deleting { "Deleting..." } else { "Delete Configuration" }
                    }
                }
            }
        }
    }
}

/// Test workflow modal component
#[derive(Props, Clone, PartialEq)]
struct TestWorkflowModalProps {
    slot: String,
    test_prompt: String,
    is_testing: bool,
    test_result: Option<WorkflowTestResult>,
    test_error: Option<String>,
    on_prompt_change: EventHandler<String>,
    on_test: EventHandler<()>,
    on_close: EventHandler<()>,
}

#[component]
fn TestWorkflowModal(props: TestWorkflowModalProps) -> Element {
    let has_result = props.test_result.is_some();

    rsx! {
        div {
            class: "modal-backdrop",
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.75); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_close.call(()),

            div {
                class: "modal-content",
                style: "background: #1a1a2e; border-radius: 0.75rem; width: 90%; max-width: 700px; max-height: 80vh; display: flex; flex-direction: column; overflow: hidden;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 1rem 1.5rem; border-bottom: 1px solid #374151;",

                    h2 {
                        style: "color: white; font-size: 1.25rem; margin: 0;",
                        "Test Workflow"
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #6b7280; font-size: 1.5rem; cursor: pointer; padding: 0.25rem;",
                        "×"
                    }
                }

                // Content
                div {
                    style: "flex: 1; overflow-y: auto; padding: 1.5rem;",

                    if let Some(err) = &props.test_error {
                        div {
                            style: "padding: 0.75rem 1rem; background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; color: #ef4444; margin-bottom: 1rem; font-size: 0.875rem;",
                            "{err}"
                        }
                    }

                    if !has_result {
                        div {
                            style: "display: flex; flex-direction: column; gap: 1rem;",

                            div {
                                label {
                                    style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;",
                                    "Test Prompt"
                                }
                                p {
                                    style: "color: #6b7280; font-size: 0.75rem; margin-bottom: 0.5rem;",
                                    "Enter a test prompt to generate an image and verify the workflow is working correctly."
                                }
                                textarea {
                                    value: "{props.test_prompt}",
                                    oninput: move |e| props.on_prompt_change.call(e.value()),
                                    placeholder: "Enter your test prompt here...",
                                    disabled: props.is_testing,
                                    style: "width: 100%; height: 120px; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-family: sans-serif; font-size: 0.875rem; resize: vertical; box-sizing: border-box;",
                                }
                            }
                        }
                    } else if let Some(result) = &props.test_result {
                        div {
                            style: "display: flex; flex-direction: column; gap: 1rem;",

                            // Success message
                            div {
                                style: "display: flex; gap: 1rem; padding: 0.75rem 1rem; background: rgba(34, 197, 94, 0.1); border-radius: 0.5rem; border: 1px solid #22c55e;",

                                div { style: "color: #22c55e; font-size: 1.5rem;", "✓" }

                                div {
                                    div { style: "color: #22c55e; font-size: 0.875rem; font-weight: 500;", "Generation Successful" }
                                    div { style: "color: #9ca3af; font-size: 0.75rem; margin-top: 0.25rem;", "Time: {result.duration_ms}ms" }
                                }
                            }

                            // Generated image
                            div {
                                h3 { style: "color: white; font-size: 0.875rem; margin: 0 0 0.5rem 0;", "Generated Image" }

                                img {
                                    src: "{result.image_url}",
                                    style: "width: 100%; border-radius: 0.5rem; border: 1px solid #374151; background: #0f0f23;",
                                }
                            }

                            // Prompt display
                            div {
                                h3 { style: "color: white; font-size: 0.875rem; margin: 0 0 0.5rem 0;", "Test Prompt" }

                                div {
                                    style: "padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: #9ca3af; font-size: 0.875rem; word-break: break-word;",
                                    "{props.test_prompt}"
                                }
                            }
                        }
                    }
                }

                // Footer
                div {
                    style: "display: flex; justify-content: flex-end; gap: 0.75rem; padding: 1rem 1.5rem; border-top: 1px solid #374151;",

                    button {
                        onclick: move |_| props.on_close.call(()),
                        disabled: props.is_testing,
                        style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                        "Close"
                    }

                    if !has_result {
                        button {
                            onclick: move |_| props.on_test.call(()),
                            disabled: props.is_testing || props.test_prompt.is_empty(),
                            style: "padding: 0.5rem 1.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem; font-weight: 500;",
                            if props.is_testing { "Generating..." } else { "Generate" }
                        }
                    } else {
                        button {
                            onclick: move |_| {
                                // Reset to test again
                                // This is handled by the parent component
                            },
                            style: "padding: 0.5rem 1.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem; font-weight: 500;",
                            "Test Again"
                        }
                    }
                }
            }
        }
    }
}

/// Fetch workflow configuration from the Engine API
async fn fetch_workflow_config(slot: &str) -> Result<Option<WorkflowConfigFull>, String> {
    let path = format!("/api/workflows/{}", slot);
    let result: Option<WorkflowConfigResponse> = HttpClient::get_optional(&path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.map(|r| r.into()))
}

/// Save workflow defaults
async fn save_workflow_defaults(
    slot: &str,
    name: &str,
    defaults: Vec<InputDefaultData>,
) -> Result<(), String> {
    let base_url = "http://localhost:3000";
    let _ = (slot, name, defaults, base_url);
    // TODO: Implement save - for now just return Ok
    Ok(())
}

/// Response structure from the API
#[derive(Clone, Debug, serde::Deserialize)]
struct WorkflowConfigResponse {
    id: String,
    slot: String,
    slot_display_name: String,
    name: String,
    analysis: WorkflowAnalysisResponse,
    prompt_mappings: Vec<PromptMappingResponse>,
    input_defaults: Vec<InputDefaultResponse>,
    locked_inputs: Vec<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct WorkflowAnalysisResponse {
    node_count: usize,
    inputs: Vec<WorkflowInputResponse>,
    text_inputs: Vec<WorkflowInputResponse>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct WorkflowInputResponse {
    node_id: String,
    node_type: String,
    node_title: Option<String>,
    input_name: String,
    input_type: String,
    current_value: serde_json::Value,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct PromptMappingResponse {
    node_id: String,
    input_name: String,
    mapping_type: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct InputDefaultResponse {
    node_id: String,
    input_name: String,
    default_value: serde_json::Value,
}

impl From<WorkflowConfigResponse> for WorkflowConfigFull {
    fn from(resp: WorkflowConfigResponse) -> Self {
        Self {
            id: resp.id,
            slot: resp.slot,
            slot_display_name: resp.slot_display_name,
            name: resp.name,
            analysis: WorkflowAnalysisData {
                node_count: resp.analysis.node_count,
                inputs: resp.analysis.inputs.into_iter().map(|i| WorkflowInputData {
                    node_id: i.node_id,
                    node_type: i.node_type,
                    node_title: i.node_title,
                    input_name: i.input_name,
                    input_type: i.input_type,
                    current_value: i.current_value,
                }).collect(),
                text_inputs: resp.analysis.text_inputs.into_iter().map(|i| WorkflowInputData {
                    node_id: i.node_id,
                    node_type: i.node_type,
                    node_title: i.node_title,
                    input_name: i.input_name,
                    input_type: i.input_type,
                    current_value: i.current_value,
                }).collect(),
            },
            prompt_mappings: resp.prompt_mappings.into_iter().map(|m| PromptMappingData {
                node_id: m.node_id,
                input_name: m.input_name,
                mapping_type: m.mapping_type,
            }).collect(),
            input_defaults: resp.input_defaults.into_iter().map(|d| InputDefaultData {
                node_id: d.node_id,
                input_name: d.input_name,
                default_value: d.default_value,
            }).collect(),
            locked_inputs: resp.locked_inputs,
            created_at: resp.created_at,
            updated_at: resp.updated_at,
        }
    }
}

/// Delete workflow configuration
async fn delete_workflow_config(slot: &str) -> Result<(), String> {
    let path = format!("/api/workflows/{}", slot);
    HttpClient::delete(&path).await.map_err(|e| e.to_string())
}

/// Test workflow with a prompt
async fn test_workflow(slot: &str, prompt: &str) -> Result<WorkflowTestResult, String> {
    let path = format!("/api/workflows/{}/test", slot);
    let body = serde_json::json!({ "prompt": prompt });
    let result: WorkflowTestResponse = HttpClient::post(&path, &body)
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.into())
}

/// Response from test endpoint
#[derive(Clone, Debug, serde::Deserialize)]
struct WorkflowTestResponse {
    image_url: String,
    #[serde(default)]
    duration_ms: u64,
}

impl From<WorkflowTestResponse> for WorkflowTestResult {
    fn from(resp: WorkflowTestResponse) -> Self {
        Self {
            image_url: resp.image_url,
            duration_ms: resp.duration_ms,
        }
    }
}
