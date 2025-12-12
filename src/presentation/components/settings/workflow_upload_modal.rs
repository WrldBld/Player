//! Workflow Upload Modal Component
//!
//! Modal for uploading or pasting a ComfyUI workflow JSON,
//! analyzing it, and configuring prompt mappings.

use dioxus::prelude::*;

use crate::infrastructure::http_client::HttpClient;

/// Props for the WorkflowUploadModal component
#[derive(Props, Clone, PartialEq)]
pub struct WorkflowUploadModalProps {
    /// The slot to configure
    pub slot: String,
    /// Callback when modal is closed
    pub on_close: EventHandler<()>,
    /// Callback when workflow is saved
    pub on_save: EventHandler<()>,
}

/// Workflow analysis result
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkflowAnalysisResult {
    pub is_valid: bool,
    pub node_count: usize,
    pub input_count: usize,
    pub text_inputs: Vec<TextInputInfo>,
    pub suggested_primary: Option<TextInputInfo>,
    pub suggested_negative: Option<TextInputInfo>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextInputInfo {
    pub node_id: String,
    pub node_title: Option<String>,
    pub input_name: String,
}

/// Upload modal wizard step
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum UploadStep {
    #[default]
    Upload,
    Configure,
    Review,
}

/// Workflow upload modal
#[component]
pub fn WorkflowUploadModal(props: WorkflowUploadModalProps) -> Element {
    // Track wizard step
    let mut current_step = use_signal(|| UploadStep::Upload);
    // Store the workflow name
    let mut workflow_name = use_signal(|| "".to_string());
    // Store the pasted JSON
    let mut workflow_json = use_signal(|| "".to_string());
    // Store analysis result
    let mut analysis: Signal<Option<WorkflowAnalysisResult>> = use_signal(|| None);
    // Track loading state
    let mut is_analyzing = use_signal(|| false);
    let mut is_saving = use_signal(|| false);
    // Track error
    let mut error: Signal<Option<String>> = use_signal(|| None);
    // Store selected mappings
    let mut primary_mapping: Signal<Option<TextInputInfo>> = use_signal(|| None);
    let mut negative_mapping: Signal<Option<TextInputInfo>> = use_signal(|| None);

    // Analyze workflow
    let do_analyze = move |_| {
        let json_text = workflow_json.read().clone();

        spawn(async move {
            is_analyzing.set(true);
            error.set(None);

            match analyze_workflow(&json_text).await {
                Ok(result) => {
                    // Auto-select suggested mappings
                    primary_mapping.set(result.suggested_primary.clone());
                    negative_mapping.set(result.suggested_negative.clone());
                    analysis.set(Some(result));
                    current_step.set(UploadStep::Configure);
                }
                Err(e) => {
                    error.set(Some(e));
                }
            }

            is_analyzing.set(false);
        });
    };

    // Save workflow configuration
    let slot_for_save = props.slot.clone();
    let on_save_handler = props.on_save.clone();
    let do_save = move |_| {
        let json_text = workflow_json.read().clone();
        let name = workflow_name.read().clone();
        let primary = primary_mapping.read().clone();
        let negative = negative_mapping.read().clone();
        let slot = slot_for_save.clone();
        let on_save = on_save_handler.clone();

        spawn(async move {
            is_saving.set(true);
            error.set(None);

            match save_workflow_config(&slot, &name, &json_text, primary, negative).await {
                Ok(_) => {
                    on_save.call(());
                }
                Err(e) => {
                    error.set(Some(e));
                }
            }

            is_saving.set(false);
        });
    };

    rsx! {
        // Modal backdrop
        div {
            class: "modal-backdrop",
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.75); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_close.call(()),

            // Modal content
            div {
                class: "modal-content",
                style: "background: #1a1a2e; border-radius: 0.75rem; width: 90%; max-width: 700px; max-height: 80vh; display: flex; flex-direction: column; overflow: hidden;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 1rem 1.5rem; border-bottom: 1px solid #374151;",

                    div {
                        h2 {
                            style: "color: white; font-size: 1.25rem; margin: 0;",
                            "Configure Workflow"
                        }
                        p {
                            style: "color: #6b7280; font-size: 0.875rem; margin: 0.25rem 0 0 0;",
                            "Slot: {props.slot}"
                        }
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #6b7280; font-size: 1.5rem; cursor: pointer; padding: 0.25rem;",
                        "×"
                    }
                }

                // Progress indicator
                div {
                    style: "display: flex; gap: 0.5rem; padding: 1rem 1.5rem; background: rgba(0, 0, 0, 0.2);",

                    StepIndicator {
                        number: 1,
                        label: "Upload",
                        is_active: *current_step.read() == UploadStep::Upload,
                        is_complete: *current_step.read() != UploadStep::Upload,
                    }
                    StepIndicator {
                        number: 2,
                        label: "Configure",
                        is_active: *current_step.read() == UploadStep::Configure,
                        is_complete: *current_step.read() == UploadStep::Review,
                    }
                    StepIndicator {
                        number: 3,
                        label: "Review",
                        is_active: *current_step.read() == UploadStep::Review,
                        is_complete: false,
                    }
                }

                // Content
                div {
                    style: "flex: 1; overflow-y: auto; padding: 1.5rem;",

                    // Error display
                    if let Some(err) = error.read().as_ref() {
                        div {
                            style: "padding: 0.75rem 1rem; background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; color: #ef4444; margin-bottom: 1rem;",
                            "{err}"
                        }
                    }

                    match *current_step.read() {
                        UploadStep::Upload => rsx! {
                            UploadStepContent {
                                workflow_name: workflow_name.read().clone(),
                                workflow_json: workflow_json.read().clone(),
                                on_name_change: move |name| workflow_name.set(name),
                                on_json_change: move |json| workflow_json.set(json),
                            }
                        },

                        UploadStep::Configure => rsx! {
                            if let Some(result) = analysis.read().as_ref() {
                                ConfigureStepContent {
                                    analysis: result.clone(),
                                    primary_mapping: primary_mapping.read().clone(),
                                    negative_mapping: negative_mapping.read().clone(),
                                    on_primary_change: move |mapping| primary_mapping.set(mapping),
                                    on_negative_change: move |mapping| negative_mapping.set(mapping),
                                }
                            }
                        },

                        UploadStep::Review => rsx! {
                            ReviewStepContent {
                                workflow_name: workflow_name.read().clone(),
                                analysis: analysis.read().clone(),
                                primary_mapping: primary_mapping.read().clone(),
                                negative_mapping: negative_mapping.read().clone(),
                            }
                        },
                    }
                }

                // Footer
                div {
                    style: "display: flex; justify-content: space-between; padding: 1rem 1.5rem; border-top: 1px solid #374151;",

                    // Back button
                    {
                        let step = *current_step.read();
                        if step != UploadStep::Upload {
                            rsx! {
                                button {
                                    onclick: move |_| {
                                        let current = *current_step.read();
                                        match current {
                                            UploadStep::Configure => current_step.set(UploadStep::Upload),
                                            UploadStep::Review => current_step.set(UploadStep::Configure),
                                            _ => {}
                                        }
                                    },
                                    style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                                    "← Back"
                                }
                            }
                        } else {
                            rsx! { div {} } // Spacer
                        }
                    }

                    // Next/Save button
                    {
                        let step = *current_step.read();
                        match step {
                            UploadStep::Upload => {
                                let analyzing = *is_analyzing.read();
                                let name_empty = workflow_name.read().is_empty();
                                let json_empty = workflow_json.read().is_empty();
                                rsx! {
                                    button {
                                        onclick: do_analyze,
                                        disabled: name_empty || json_empty || analyzing,
                                        style: "padding: 0.5rem 1.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 500;",
                                        if analyzing { "Analyzing..." } else { "Analyze Workflow →" }
                                    }
                                }
                            },

                            UploadStep::Configure => rsx! {
                                button {
                                    onclick: move |_| current_step.set(UploadStep::Review),
                                    style: "padding: 0.5rem 1.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 500;",
                                    "Review →"
                                }
                            },

                            UploadStep::Review => {
                                let saving = *is_saving.read();
                                rsx! {
                                    button {
                                        onclick: do_save,
                                        disabled: saving,
                                        style: "padding: 0.5rem 1.5rem; background: #22c55e; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 500;",
                                        if saving { "Saving..." } else { "Save Configuration" }
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Step indicator component
#[component]
fn StepIndicator(number: u8, label: &'static str, is_active: bool, is_complete: bool) -> Element {
    let bg_color = if is_active || is_complete {
        "#3b82f6"
    } else {
        "#374151"
    };
    let text_color = if is_active {
        "white"
    } else if is_complete {
        "#9ca3af"
    } else {
        "#6b7280"
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem;",

            div {
                style: format!(
                    "width: 24px; height: 24px; border-radius: 50%; background: {}; display: flex; align-items: center; justify-content: center; font-size: 0.75rem; color: white; font-weight: 600;",
                    bg_color
                ),
                if is_complete { "✓" } else { "{number}" }
            }

            span {
                style: format!("color: {}; font-size: 0.875rem;", text_color),
                "{label}"
            }

            if number < 3 {
                div {
                    style: "width: 40px; height: 2px; background: #374151; margin: 0 0.5rem;",
                }
            }
        }
    }
}

/// Upload step content
#[derive(Props, Clone, PartialEq)]
struct UploadStepContentProps {
    workflow_name: String,
    workflow_json: String,
    on_name_change: EventHandler<String>,
    on_json_change: EventHandler<String>,
}

#[component]
fn UploadStepContent(props: UploadStepContentProps) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 1rem;",

            // Workflow name input
            div {
                label {
                    style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;",
                    "Workflow Name"
                }
                input {
                    r#type: "text",
                    value: "{props.workflow_name}",
                    oninput: move |e| props.on_name_change.call(e.value()),
                    placeholder: "e.g., SD1.5 Portrait Generator",
                    style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 1rem; box-sizing: border-box;",
                }
            }

            // JSON textarea
            div {
                label {
                    style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;",
                    "Workflow JSON (API Format)"
                }
                p {
                    style: "color: #6b7280; font-size: 0.75rem; margin-bottom: 0.5rem;",
                    "In ComfyUI, use 'Save (API Format)' from the menu to export the workflow in the correct format."
                }
                textarea {
                    value: "{props.workflow_json}",
                    oninput: move |e| props.on_json_change.call(e.value()),
                    placeholder: "Paste your ComfyUI workflow JSON here...",
                    style: "width: 100%; height: 300px; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-family: monospace; font-size: 0.875rem; resize: vertical; box-sizing: border-box;",
                }
            }
        }
    }
}

/// Configure step content
#[derive(Props, Clone, PartialEq)]
struct ConfigureStepContentProps {
    analysis: WorkflowAnalysisResult,
    primary_mapping: Option<TextInputInfo>,
    negative_mapping: Option<TextInputInfo>,
    on_primary_change: EventHandler<Option<TextInputInfo>>,
    on_negative_change: EventHandler<Option<TextInputInfo>>,
}

#[component]
fn ConfigureStepContent(props: ConfigureStepContentProps) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 1.5rem;",

            // Analysis summary
            div {
                style: "display: flex; gap: 1rem; padding: 1rem; background: rgba(34, 197, 94, 0.1); border-radius: 0.5rem;",

                div {
                    style: "color: #22c55e; font-size: 1.5rem;",
                    "✓"
                }
                div {
                    h3 { style: "color: #22c55e; margin: 0 0 0.25rem 0; font-size: 1rem;", "Valid Workflow" }
                    p { style: "color: #9ca3af; margin: 0; font-size: 0.875rem;",
                        "{props.analysis.node_count} nodes, {props.analysis.input_count} configurable inputs"
                    }
                }
            }

            // Primary prompt mapping
            div {
                h4 { style: "color: white; font-size: 0.875rem; margin-bottom: 0.5rem;", "Primary Prompt Mapping" }
                p { style: "color: #6b7280; font-size: 0.75rem; margin-bottom: 0.5rem;",
                    "Select the text input that will receive the main generation prompt."
                }
                TextInputSelector {
                    inputs: props.analysis.text_inputs.clone(),
                    selected: props.primary_mapping.clone(),
                    on_select: props.on_primary_change.clone(),
                }
            }

            // Negative prompt mapping
            div {
                h4 { style: "color: white; font-size: 0.875rem; margin-bottom: 0.5rem;", "Negative Prompt Mapping (Optional)" }
                p { style: "color: #6b7280; font-size: 0.75rem; margin-bottom: 0.5rem;",
                    "Select the text input for negative prompts, if applicable."
                }
                TextInputSelector {
                    inputs: props.analysis.text_inputs.clone(),
                    selected: props.negative_mapping.clone(),
                    on_select: props.on_negative_change.clone(),
                }
            }
        }
    }
}

/// Text input selector dropdown
#[derive(Props, Clone, PartialEq)]
struct TextInputSelectorProps {
    inputs: Vec<TextInputInfo>,
    selected: Option<TextInputInfo>,
    on_select: EventHandler<Option<TextInputInfo>>,
}

#[component]
fn TextInputSelector(props: TextInputSelectorProps) -> Element {
    let selected_key = props.selected.as_ref().map(|i| format!("{}:{}", i.node_id, i.input_name));

    rsx! {
        select {
            value: selected_key.clone().unwrap_or_default(),
            onchange: move |e| {
                let value = e.value();
                if value.is_empty() {
                    props.on_select.call(None);
                } else {
                    let input = props.inputs.iter().find(|i| {
                        format!("{}:{}", i.node_id, i.input_name) == value
                    }).cloned();
                    props.on_select.call(input);
                }
            },
            style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 0.875rem;",

            option { value: "", "(None)" }

            for input in props.inputs.iter() {
                {
                    let node_label = input.node_title.clone().unwrap_or_else(|| format!("Node {}", input.node_id));
                    let key = format!("{}:{}", input.node_id, input.input_name);
                    let display = format!("{} → {}", node_label, input.input_name);
                    rsx! {
                        option {
                            value: "{key}",
                            "{display}"
                        }
                    }
                }
            }
        }
    }
}

/// Review step content
#[derive(Props, Clone, PartialEq)]
struct ReviewStepContentProps {
    workflow_name: String,
    analysis: Option<WorkflowAnalysisResult>,
    primary_mapping: Option<TextInputInfo>,
    negative_mapping: Option<TextInputInfo>,
}

#[component]
fn ReviewStepContent(props: ReviewStepContentProps) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 1rem;",

            h3 { style: "color: white; margin: 0; font-size: 1.125rem;", "Review Configuration" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.75rem; padding: 1rem; background: rgba(0, 0, 0, 0.2); border-radius: 0.5rem;",

                ReviewRow { label: "Workflow Name", value: props.workflow_name.clone() }

                if let Some(analysis) = &props.analysis {
                    ReviewRow { label: "Node Count", value: analysis.node_count.to_string() }
                    ReviewRow { label: "Configurable Inputs", value: analysis.input_count.to_string() }
                }

                ReviewRow {
                    label: "Primary Prompt",
                    value: props.primary_mapping.as_ref()
                        .map(|m| format!("{} → {}", m.node_title.clone().unwrap_or_else(|| format!("Node {}", m.node_id)), m.input_name))
                        .unwrap_or_else(|| "(Not set)".to_string())
                }

                ReviewRow {
                    label: "Negative Prompt",
                    value: props.negative_mapping.as_ref()
                        .map(|m| format!("{} → {}", m.node_title.clone().unwrap_or_else(|| format!("Node {}", m.node_id)), m.input_name))
                        .unwrap_or_else(|| "(Not set)".to_string())
                }
            }

            p {
                style: "color: #6b7280; font-size: 0.875rem;",
                "Click 'Save Configuration' to save this workflow. You can edit input defaults after saving."
            }
        }
    }
}

#[component]
fn ReviewRow(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: space-between;",

            span { style: "color: #9ca3af;", "{label}" }
            span { style: "color: white;", "{value}" }
        }
    }
}

/// Analyze workflow via Engine API
async fn analyze_workflow(json_text: &str) -> Result<WorkflowAnalysisResult, String> {
    // Parse the JSON first
    let workflow_json: serde_json::Value = serde_json::from_str(json_text)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let body = serde_json::json!({ "workflow_json": workflow_json });
    let analysis: AnalyzeWorkflowResponse = HttpClient::post("/api/workflows/analyze", &body)
        .await
        .map_err(|e| e.to_string())?;
    Ok(analysis.into())
}

/// Save workflow configuration via Engine API
async fn save_workflow_config(
    slot: &str,
    name: &str,
    json_text: &str,
    primary_mapping: Option<TextInputInfo>,
    negative_mapping: Option<TextInputInfo>,
) -> Result<(), String> {
    let workflow_json: serde_json::Value = serde_json::from_str(json_text)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let mut prompt_mappings = Vec::new();
    if let Some(primary) = primary_mapping {
        prompt_mappings.push(serde_json::json!({
            "node_id": primary.node_id,
            "input_name": primary.input_name,
            "mapping_type": "primary"
        }));
    }
    if let Some(negative) = negative_mapping {
        prompt_mappings.push(serde_json::json!({
            "node_id": negative.node_id,
            "input_name": negative.input_name,
            "mapping_type": "negative"
        }));
    }

    let body = serde_json::json!({
        "name": name,
        "workflow_json": workflow_json,
        "prompt_mappings": prompt_mappings,
        "input_defaults": [],
        "locked_inputs": []
    });

    let path = format!("/api/workflows/{}", slot);
    HttpClient::post_no_response(&path, &body).await.map_err(|e| e.to_string())
}

/// Response from analyze endpoint
#[derive(Clone, Debug, serde::Deserialize)]
struct AnalyzeWorkflowResponse {
    is_valid: bool,
    analysis: AnalysisData,
    suggested_prompt_mappings: Vec<SuggestedMapping>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct AnalysisData {
    node_count: usize,
    inputs: Vec<InputData>,
    text_inputs: Vec<InputData>,
    #[serde(default)]
    errors: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct InputData {
    node_id: String,
    node_title: Option<String>,
    input_name: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct SuggestedMapping {
    node_id: String,
    input_name: String,
    mapping_type: String,
}

impl From<AnalyzeWorkflowResponse> for WorkflowAnalysisResult {
    fn from(resp: AnalyzeWorkflowResponse) -> Self {
        let text_inputs: Vec<TextInputInfo> = resp.analysis.text_inputs.iter().map(|i| TextInputInfo {
            node_id: i.node_id.clone(),
            node_title: i.node_title.clone(),
            input_name: i.input_name.clone(),
        }).collect();

        let suggested_primary = resp.suggested_prompt_mappings.iter()
            .find(|m| m.mapping_type == "primary")
            .and_then(|m| {
                text_inputs.iter().find(|i| i.node_id == m.node_id && i.input_name == m.input_name).cloned()
            });

        let suggested_negative = resp.suggested_prompt_mappings.iter()
            .find(|m| m.mapping_type == "negative")
            .and_then(|m| {
                text_inputs.iter().find(|i| i.node_id == m.node_id && i.input_name == m.input_name).cloned()
            });

        Self {
            is_valid: resp.is_valid,
            node_count: resp.analysis.node_count,
            input_count: resp.analysis.inputs.len(),
            text_inputs,
            suggested_primary,
            suggested_negative,
            errors: resp.analysis.errors,
        }
    }
}
