//! Settings components - Application configuration interface
//!
//! Components for the Settings view, providing workflow configuration,
//! ComfyUI integration settings, skills management, and general application preferences.

pub mod skills_panel;
pub mod workflow_slot_list;
pub mod workflow_config_editor;
pub mod workflow_upload_modal;

pub use skills_panel::SkillsPanel;
pub use workflow_slot_list::WorkflowSlotList;
pub use workflow_config_editor::WorkflowConfigEditor;
pub use workflow_upload_modal::WorkflowUploadModal;

use dioxus::prelude::*;

/// The main Settings container component
#[component]
pub fn SettingsView() -> Element {
    // Track the selected workflow slot for editing
    let mut selected_slot: Signal<Option<String>> = use_signal(|| None);
    // Track whether the upload modal is open
    let mut show_upload_modal = use_signal(|| false);
    // Track which slot we're uploading for
    let mut upload_target_slot: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div {
            class: "settings-view",
            style: "height: 100%; display: grid; grid-template-columns: 320px 1fr; gap: 1rem; padding: 1rem;",

            // Left panel - Workflow slots list
            div {
                class: "left-panel",
                style: "display: flex; flex-direction: column; gap: 1rem; overflow: hidden;",

                WorkflowSlotList {
                    selected_slot: selected_slot.read().clone(),
                    on_select: move |slot: String| selected_slot.set(Some(slot)),
                    on_configure: move |slot: String| {
                        upload_target_slot.set(Some(slot.clone()));
                        show_upload_modal.set(true);
                    },
                }
            }

            // Right panel - Configuration editor
            div {
                class: "editor-panel",
                style: "display: flex; flex-direction: column; gap: 1rem; overflow: hidden;",

                if let Some(slot) = selected_slot.read().clone() {
                    WorkflowConfigEditor {
                        slot: slot.clone(),
                        on_close: move |_| selected_slot.set(None),
                        on_reconfigure: move |_| {
                            upload_target_slot.set(Some(slot.clone()));
                            show_upload_modal.set(true);
                        },
                        on_deleted: move |_| {
                            // Deselect the slot and refresh the list
                            selected_slot.set(None);
                        },
                    }
                } else {
                    EmptyStatePanel {}
                }
            }

            // Upload modal overlay
            if *show_upload_modal.read() {
                if let Some(slot) = upload_target_slot.read().clone() {
                    WorkflowUploadModal {
                        slot: slot.clone(),
                        on_close: move |_| {
                            show_upload_modal.set(false);
                            upload_target_slot.set(None);
                        },
                        on_save: move |_| {
                            show_upload_modal.set(false);
                            // Select the slot we just configured
                            selected_slot.set(upload_target_slot.read().clone());
                            upload_target_slot.set(None);
                        },
                    }
                }
            }
        }
    }
}

/// Empty state panel when no workflow is selected
#[component]
fn EmptyStatePanel() -> Element {
    rsx! {
        div {
            class: "empty-state",
            style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; background: #1a1a2e; border-radius: 0.5rem; color: #6b7280;",

            div {
                style: "text-align: center; max-width: 300px;",

                // Gear icon
                div {
                    style: "font-size: 3rem; margin-bottom: 1rem; opacity: 0.5;",
                    "⚙️"
                }

                h2 {
                    style: "color: #9ca3af; margin-bottom: 0.5rem; font-size: 1.25rem;",
                    "Workflow Configuration"
                }

                p {
                    style: "color: #6b7280; font-size: 0.875rem; line-height: 1.5;",
                    "Select a workflow slot from the left panel to view or edit its configuration. Click 'Configure' to upload a new ComfyUI workflow."
                }
            }
        }
    }
}
