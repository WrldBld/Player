//! Workflow Slot List Component
//!
//! Displays all available workflow slots organized by category,
//! showing which are configured and allowing selection/configuration.

use dioxus::prelude::*;

use crate::presentation::services::use_workflow_service;
use crate::application::services::{WorkflowSlotStatus, WorkflowSlotCategory};

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

/// List of all workflow slots with their configuration status
#[component]
pub fn WorkflowSlotList(props: WorkflowSlotListProps) -> Element {
    let workflow_service = use_workflow_service();

    // Track loading state
    let mut is_loading = use_signal(|| true);
    // Track error state
    let mut error: Signal<Option<String>> = use_signal(|| None);
    // Store the workflow slot categories (pre-grouped by backend)
    let mut categories: Signal<Vec<WorkflowSlotCategory>> = use_signal(Vec::new);

    // Fetch workflow slots on mount
    use_effect(move || {
        let svc = workflow_service.clone();
        spawn(async move {
            match svc.list_workflows().await {
                Ok(response) => {
                    categories.set(response.categories);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading.set(false);
                }
            }
        });
    });

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

                        p { style: "margin: 0 0 0.5rem 0; font-weight: 600;", "Failed to load workflow slots" }
                        p { style: "margin: 0;", "{err}" }

                        // Help text
                        div {
                            style: "margin-top: 1rem; padding-top: 1rem; border-top: 1px solid rgba(239, 68, 68, 0.3); font-size: 0.75rem; color: #9ca3af;",
                            p { style: "margin: 0 0 0.25rem 0;", "Troubleshooting:" }
                            ul { style: "margin: 0; padding-left: 1.25rem;",
                                li { "Is the Engine server running?" }
                                li { "Check that the Engine API is accessible" }
                            }
                        }
                    }
                } else if categories.read().is_empty() {
                    div {
                        style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 2rem; color: #6b7280; text-align: center;",

                        div { style: "font-size: 2rem; margin-bottom: 0.5rem;", "⚠️" }
                        p { style: "margin: 0 0 0.5rem 0; color: #9ca3af;", "No workflow slots available" }
                        p { style: "margin: 0; font-size: 0.75rem;",
                            "The Engine returned an empty list."
                        }
                    }
                } else {
                    // Render categories directly from backend response - no filtering needed
                    for category in categories.read().iter() {
                        CategorySection {
                            title: category.name.clone(),
                            slots: category.slots.clone(),
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
    title: String,
    slots: Vec<WorkflowSlotStatus>,
    selected_slot: Option<String>,
    on_select: EventHandler<String>,
    on_configure: EventHandler<String>,
}

#[component]
fn CategorySection(props: CategorySectionProps) -> Element {
    if props.slots.is_empty() {
        return rsx! {};
    }

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
                    if let Some(ref config) = props.slot.config {
                        div {
                            style: "color: #22c55e; font-size: 0.75rem; margin-top: 0.25rem; margin-left: 1rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                            "✓ {config.name}"
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

