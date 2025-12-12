//! Workflow Slot List Component
//!
//! Displays all available workflow slots organized by category,
//! showing which are configured and allowing selection/configuration.

use dioxus::prelude::*;

/// Props for the WorkflowSlotList component
#[derive(Props, Clone, PartialEq)]
pub struct WorkflowSlotListProps {
    /// Currently selected slot ID
    pub selected_slot: Option<String>,
    /// Callback when a slot is selected for viewing
    pub on_select: EventHandler<String>,
    /// Callback when configure button is clicked
    pub on_configure: EventHandler<String>,
}

/// Workflow slot status data
#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowSlotStatus {
    pub slot: String,
    pub display_name: String,
    pub category: String,
    pub default_width: u32,
    pub default_height: u32,
    pub configured: bool,
    pub workflow_name: Option<String>,
}

/// List of all workflow slots with their configuration status
#[component]
pub fn WorkflowSlotList(props: WorkflowSlotListProps) -> Element {
    // Track loading state
    let mut is_loading = use_signal(|| true);
    // Track error state
    let mut error: Signal<Option<String>> = use_signal(|| None);
    // Store the workflow slots
    let mut slots: Signal<Vec<WorkflowSlotStatus>> = use_signal(Vec::new);

    // Fetch workflow slots on mount
    use_effect(move || {
        spawn(async move {
            match fetch_workflow_slots().await {
                Ok(fetched_slots) => {
                    slots.set(fetched_slots);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e));
                    is_loading.set(false);
                }
            }
        });
    });

    // Group slots by category
    let character_slots: Vec<_> = slots
        .read()
        .iter()
        .filter(|s| s.category == "Character")
        .cloned()
        .collect();
    let location_slots: Vec<_> = slots
        .read()
        .iter()
        .filter(|s| s.category == "Location")
        .cloned()
        .collect();
    let item_slots: Vec<_> = slots
        .read()
        .iter()
        .filter(|s| s.category == "Item")
        .cloned()
        .collect();

    rsx! {
        div {
            class: "workflow-slot-list",
            style: "flex: 1; display: flex; flex-direction: column; background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

            // Header
            div {
                style: "padding: 1rem; border-bottom: 1px solid #374151;",

                h3 {
                    style: "color: white; font-size: 1rem; margin: 0 0 0.25rem 0;",
                    "Workflow Slots"
                }
                p {
                    style: "color: #6b7280; font-size: 0.75rem; margin: 0;",
                    "Configure ComfyUI workflows for asset generation"
                }
            }

            // Content
            div {
                style: "flex: 1; overflow-y: auto; padding: 0.5rem;",

                if *is_loading.read() {
                    div {
                        style: "display: flex; align-items: center; justify-content: center; padding: 2rem; color: #6b7280;",
                        "Loading workflows..."
                    }
                } else if let Some(err) = error.read().as_ref() {
                    div {
                        style: "padding: 1rem; background: rgba(239, 68, 68, 0.1); border-radius: 0.5rem; color: #ef4444; font-size: 0.875rem;",
                        "Error: {err}"
                    }
                } else {
                    // Character workflows
                    if !character_slots.is_empty() {
                        CategorySection {
                            title: "Character Assets",
                            slots: character_slots,
                            selected_slot: props.selected_slot.clone(),
                            on_select: props.on_select.clone(),
                            on_configure: props.on_configure.clone(),
                        }
                    }

                    // Location workflows
                    if !location_slots.is_empty() {
                        CategorySection {
                            title: "Location Assets",
                            slots: location_slots,
                            selected_slot: props.selected_slot.clone(),
                            on_select: props.on_select.clone(),
                            on_configure: props.on_configure.clone(),
                        }
                    }

                    // Item workflows
                    if !item_slots.is_empty() {
                        CategorySection {
                            title: "Item Assets",
                            slots: item_slots,
                            selected_slot: props.selected_slot.clone(),
                            on_select: props.on_select.clone(),
                            on_configure: props.on_configure.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Category section with slots
#[derive(Props, Clone, PartialEq)]
struct CategorySectionProps {
    title: &'static str,
    slots: Vec<WorkflowSlotStatus>,
    selected_slot: Option<String>,
    on_select: EventHandler<String>,
    on_configure: EventHandler<String>,
}

#[component]
fn CategorySection(props: CategorySectionProps) -> Element {
    rsx! {
        div {
            class: "category-section",
            style: "margin-bottom: 1rem;",

            h4 {
                style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; margin: 0 0 0.5rem 0.5rem;",
                "{props.title}"
            }

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",

                for slot in props.slots.iter() {
                    SlotCard {
                        slot: slot.clone(),
                        is_selected: props.selected_slot.as_ref() == Some(&slot.slot),
                        on_select: props.on_select.clone(),
                        on_configure: props.on_configure.clone(),
                    }
                }
            }
        }
    }
}

/// Individual slot card
#[derive(Props, Clone, PartialEq)]
struct SlotCardProps {
    slot: WorkflowSlotStatus,
    is_selected: bool,
    on_select: EventHandler<String>,
    on_configure: EventHandler<String>,
}

#[component]
fn SlotCard(props: SlotCardProps) -> Element {
    let bg_color = if props.is_selected {
        "rgba(59, 130, 246, 0.2)"
    } else {
        "rgba(0, 0, 0, 0.2)"
    };
    let border = if props.is_selected {
        "1px solid #3b82f6"
    } else {
        "1px solid transparent"
    };

    let slot_id = props.slot.slot.clone();
    let slot_id_for_configure = props.slot.slot.clone();

    rsx! {
        div {
            class: "slot-card",
            style: format!(
                "display: flex; align-items: center; justify-content: space-between; padding: 0.75rem; background: {}; border: {}; border-radius: 0.5rem; cursor: pointer; transition: all 0.2s;",
                bg_color, border
            ),
            onclick: move |_| props.on_select.call(slot_id.clone()),

            // Slot info
            div {
                style: "flex: 1; min-width: 0;",

                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",

                    // Status indicator
                    div {
                        style: format!(
                            "width: 8px; height: 8px; border-radius: 50%; {}",
                            if props.slot.configured {
                                "background: #22c55e;"
                            } else {
                                "background: #6b7280;"
                            }
                        ),
                    }

                    // Name
                    span {
                        style: "color: white; font-size: 0.875rem; font-weight: 500;",
                        "{props.slot.display_name}"
                    }
                }

                // Dimensions
                div {
                    style: "color: #6b7280; font-size: 0.75rem; margin-top: 0.25rem; margin-left: 1rem;",
                    "{props.slot.default_width}×{props.slot.default_height}"
                }

                // Workflow name if configured
                if props.slot.configured {
                    if let Some(name) = &props.slot.workflow_name {
                        div {
                            style: "color: #22c55e; font-size: 0.75rem; margin-top: 0.25rem; margin-left: 1rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                            "✓ {name}"
                        }
                    }
                }
            }

            // Configure button
            button {
                onclick: move |e| {
                    e.stop_propagation();
                    props.on_configure.call(slot_id_for_configure.clone());
                },
                style: "padding: 0.375rem 0.75rem; background: #374151; color: white; border: none; border-radius: 0.375rem; font-size: 0.75rem; cursor: pointer;",
                if props.slot.configured { "Edit" } else { "Configure" }
            }
        }
    }
}

/// Fetch workflow slots from the Engine API
async fn fetch_workflow_slots() -> Result<Vec<WorkflowSlotStatus>, String> {
    // Get the server URL from session state or use default
    let base_url = get_engine_http_url();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, Response};

        let opts = RequestInit::new();
        opts.set_method("GET");

        let url = format!("{}/api/workflows", base_url);
        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;

        let window = web_sys::window().ok_or("No window object")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let resp: Response = resp_value
            .dyn_into()
            .map_err(|_| "Response cast failed")?;

        if !resp.ok() {
            return Err(format!("Server error: {}", resp.status()));
        }

        let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
            .await
            .map_err(|e| format!("JSON await failed: {:?}", e))?;

        let slots: Vec<WorkflowSlotStatusResponse> = serde_wasm_bindgen::from_value(json)
            .map_err(|e| format!("Deserialize error: {:?}", e))?;

        Ok(slots.into_iter().map(|s| s.into()).collect())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let url = format!("{}/api/workflows", base_url);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let slots: Vec<WorkflowSlotStatusResponse> = response
            .json()
            .await
            .map_err(|e| format!("Deserialize error: {}", e))?;

        Ok(slots.into_iter().map(|s| s.into()).collect())
    }
}

/// Get the HTTP URL for the Engine API
fn get_engine_http_url() -> String {
    // Default to localhost:3000
    "http://localhost:3000".to_string()
}

/// Response structure from the API
#[derive(Clone, Debug, serde::Deserialize)]
struct WorkflowSlotStatusResponse {
    slot: String,
    display_name: String,
    category: String,
    default_width: u32,
    default_height: u32,
    configured: bool,
    config: Option<WorkflowConfigBrief>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct WorkflowConfigBrief {
    name: String,
}

impl From<WorkflowSlotStatusResponse> for WorkflowSlotStatus {
    fn from(resp: WorkflowSlotStatusResponse) -> Self {
        Self {
            slot: resp.slot,
            display_name: resp.display_name,
            category: resp.category,
            default_width: resp.default_width,
            default_height: resp.default_height,
            configured: resp.configured,
            workflow_name: resp.config.map(|c| c.name),
        }
    }
}
