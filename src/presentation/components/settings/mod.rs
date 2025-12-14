//! Settings components - Application configuration interface
//!
//! Components for the Settings view, providing workflow configuration,
//! ComfyUI integration settings, skills management, and general application preferences.

pub mod skills_panel;
pub mod workflow_slot_list;
pub mod workflow_config_editor;
pub mod workflow_upload_modal;

use dioxus::prelude::*;
use crate::routes::Route;

/// Props for SettingsView
#[derive(Props, Clone, PartialEq)]
pub struct SettingsViewProps {
    /// World ID for skills management and routing
    pub world_id: String,
    /// Selected sub-tab from route (workflows, skills)
    #[props(default)]
    pub selected_tab: Option<String>,
}

/// The main Settings container component with route-based tabs
#[component]
pub fn SettingsView(props: SettingsViewProps) -> Element {
    // Determine active tab from route, default to workflows
    let active_tab = props.selected_tab.as_deref().unwrap_or("workflows");

    rsx! {
        div {
            class: "settings-view",
            style: "height: 100%; display: flex; flex-direction: column; background: #0f0f23;",

            // Tab bar using router Links
            div {
                class: "settings-tabs",
                style: "display: flex; gap: 0.25rem; padding: 0.75rem 1rem; background: #1a1a2e; border-bottom: 1px solid #374151;",

                SettingsTabLink {
                    label: "Asset Workflows",
                    subtab: "workflows",
                    world_id: props.world_id.clone(),
                    active: active_tab == "workflows",
                }
                SettingsTabLink {
                    label: "Skills Management",
                    subtab: "skills",
                    world_id: props.world_id.clone(),
                    active: active_tab == "skills",
                }
            }

            // Tab content
            div {
                class: "settings-content",
                style: "flex: 1; overflow: hidden;",

                match active_tab {
                    "skills" => rsx! {
                        SkillsManagementTab { world_id: props.world_id.clone() }
                    },
                    _ => rsx! {
                        AssetWorkflowsTab {}
                    },
                }
            }
        }
    }
}

/// Tab link component using router navigation
#[derive(Props, Clone, PartialEq)]
struct SettingsTabLinkProps {
    label: &'static str,
    subtab: &'static str,
    world_id: String,
    active: bool,
}

#[component]
fn SettingsTabLink(props: SettingsTabLinkProps) -> Element {
    let bg_color = if props.active { "#3b82f6" } else { "transparent" };
    let text_color = if props.active { "white" } else { "#9ca3af" };
    let font_weight = if props.active { "500" } else { "400" };

    rsx! {
        Link {
            to: Route::DMSettingsSubTabRoute {
                world_id: props.world_id.clone(),
                subtab: props.subtab.to_string(),
            },
            style: format!(
                "padding: 0.5rem 1rem; background: {}; color: {}; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem; font-weight: {}; transition: all 0.15s; text-decoration: none;",
                bg_color, text_color, font_weight
            ),
            "{props.label}"
        }
    }
}

/// Asset Workflows tab content (original SettingsView content)
#[component]
fn AssetWorkflowsTab() -> Element {
    // Track the selected workflow slot for editing
    let mut selected_slot: Signal<Option<String>> = use_signal(|| None);
    // Track whether the upload modal is open
    let mut show_upload_modal = use_signal(|| false);
    // Track which slot we're uploading for
    let mut upload_target_slot: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div {
            class: "asset-workflows-tab",
            style: "height: 100%; display: grid; grid-template-columns: 320px 1fr; gap: 1rem; padding: 1rem;",

            // Left panel - Workflow slots list
            div {
                class: "left-panel",
                style: "display: flex; flex-direction: column; gap: 1rem; overflow: hidden;",

                workflow_slot_list::WorkflowSlotList {
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
                    workflow_config_editor::WorkflowConfigEditor {
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
                    WorkflowEmptyStatePanel {}
                }
            }

            // Upload modal overlay
            if *show_upload_modal.read() {
                if let Some(slot) = upload_target_slot.read().clone() {
                    workflow_upload_modal::WorkflowUploadModal {
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

/// Skills Management tab content - embedded skills panel
#[derive(Props, Clone, PartialEq)]
struct SkillsManagementTabProps {
    world_id: String,
}

#[component]
fn SkillsManagementTab(props: SkillsManagementTabProps) -> Element {
    use crate::application::services::SkillCategory;
    use crate::presentation::services::use_skill_service;
    use std::collections::HashMap;

    let skill_service = use_skill_service();

    let mut skills: Signal<Vec<crate::application::services::SkillData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut show_hidden = use_signal(|| false);
    let mut show_add_form = use_signal(|| false);
    let _editing_skill: Signal<Option<String>> = use_signal(|| None);

    // Clone world_id for handlers
    let world_id = props.world_id.clone();
    let world_id_for_effect = world_id.clone();

    // Load skills on mount
    use_effect(move || {
        let world_id = world_id_for_effect.clone();
        let svc = skill_service.clone();
        spawn(async move {
            match svc.list_skills(&world_id).await {
                Ok(list) => {
                    skills.set(list);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading.set(false);
                }
            }
        });
    });

    // Group skills by category
    let skills_by_category: HashMap<SkillCategory, Vec<crate::application::services::SkillData>> = {
        let skills_read = skills.read();
        let show_hidden_val = *show_hidden.read();
        let mut grouped: HashMap<SkillCategory, Vec<crate::application::services::SkillData>> = HashMap::new();
        for skill in skills_read.iter() {
            if !skill.is_hidden || show_hidden_val {
                grouped.entry(skill.category).or_default().push(skill.clone());
            }
        }
        for skills_vec in grouped.values_mut() {
            skills_vec.sort_by_key(|s| s.order);
        }
        grouped
    };

    // Get categories that have skills
    let mut active_categories: Vec<SkillCategory> = skills_by_category.keys().cloned().collect();
    active_categories.sort_by_key(|c| match c {
        SkillCategory::Physical => 0,
        SkillCategory::Mental => 1,
        SkillCategory::Social => 2,
        SkillCategory::Interpersonal => 3,
        SkillCategory::Investigation => 4,
        SkillCategory::Academic => 5,
        SkillCategory::Practical => 6,
        SkillCategory::Combat => 7,
        SkillCategory::Approach => 8,
        SkillCategory::Aspect => 9,
        SkillCategory::Other => 10,
        SkillCategory::Custom => 11,
    });

    rsx! {
        div {
            class: "skills-management-tab",
            style: "height: 100%; display: flex; flex-direction: column; padding: 1rem;",

            // Header with controls
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;",

                h2 {
                    style: "color: white; margin: 0; font-size: 1.25rem;",
                    "Skills Management"
                }

                div {
                    style: "display: flex; gap: 1rem; align-items: center;",

                    // Show hidden toggle
                    label {
                        style: "display: flex; align-items: center; gap: 0.5rem; color: #9ca3af; font-size: 0.875rem; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: *show_hidden.read(),
                            onchange: move |e| show_hidden.set(e.checked()),
                        }
                        "Show Hidden"
                    }

                    // Add skill button
                    button {
                        onclick: move |_| show_add_form.set(true),
                        style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem;",
                        "+ Add Custom Skill"
                    }
                }
            }

            // Error message
            if let Some(err) = error.read().as_ref() {
                div {
                    style: "padding: 0.75rem; background: rgba(239, 68, 68, 0.1); color: #ef4444; font-size: 0.875rem; border-radius: 0.375rem; margin-bottom: 1rem;",
                    "{err}"
                }
            }

            // Content area
            div {
                style: "flex: 1; overflow-y: auto; background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                if *is_loading.read() {
                    div {
                        style: "text-align: center; color: #6b7280; padding: 2rem;",
                        "Loading skills..."
                    }
                } else if *show_add_form.read() {
                    // Add form would go here - simplified for now
                    div {
                        style: "padding: 1rem; background: #0f0f23; border-radius: 0.5rem;",
                        p { style: "color: #9ca3af;", "Add skill form (coming soon)" }
                        button {
                            onclick: move |_| show_add_form.set(false),
                            style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                            "Cancel"
                        }
                    }
                } else if skills.read().is_empty() {
                    div {
                        style: "text-align: center; color: #6b7280; padding: 2rem;",
                        div { style: "font-size: 3rem; margin-bottom: 1rem; opacity: 0.5;", "📚" }
                        p { style: "color: #9ca3af; margin-bottom: 0.5rem;", "No skills defined for this world." }
                        p { style: "font-size: 0.875rem;", "Skills are loaded from the rule system preset." }
                    }
                } else {
                    // Skills by category
                    for category in active_categories.iter() {
                        {
                            let cat_skills = skills_by_category.get(category).cloned().unwrap_or_default();
                            if !cat_skills.is_empty() {
                                rsx! {
                                    div {
                                        key: "{category:?}",
                                        style: "margin-bottom: 1.5rem;",

                                        h3 {
                                            style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0 0 0.5rem 0;",
                                            "{category.display_name()} ({cat_skills.len()})"
                                        }

                                        div {
                                            style: "display: flex; flex-direction: column; gap: 0.25rem;",
                                            for skill in cat_skills.iter() {
                                                SkillRowInline {
                                                    key: "{skill.id}",
                                                    skill: skill.clone(),
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Inline skill row for Skills Management tab
#[derive(Props, Clone, PartialEq)]
struct SkillRowInlineProps {
    skill: crate::application::services::SkillData,
}

#[component]
fn SkillRowInline(props: SkillRowInlineProps) -> Element {
    let row_bg = if props.skill.is_hidden { "rgba(107, 114, 128, 0.2)" } else { "#0f0f23" };
    let name_color = if props.skill.is_hidden { "#6b7280" } else { "white" };

    rsx! {
        div {
            style: format!(
                "display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem; background: {}; border-radius: 0.25rem;",
                row_bg
            ),

            // Visibility indicator
            div {
                style: format!(
                    "width: 8px; height: 8px; border-radius: 50%; {}",
                    if props.skill.is_hidden { "background: #6b7280;" } else { "background: #10b981;" }
                ),
            }

            // Skill info
            div {
                style: "flex: 1; min-width: 0;",

                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",

                    span {
                        style: format!("color: {}; font-weight: 500;", name_color),
                        "{props.skill.name}"
                    }

                    if let Some(attr) = &props.skill.base_attribute {
                        span {
                            style: "color: #8b5cf6; font-size: 0.75rem; background: rgba(139, 92, 246, 0.1); padding: 0.125rem 0.375rem; border-radius: 0.25rem;",
                            "{attr}"
                        }
                    }

                    if props.skill.is_custom {
                        span {
                            style: "color: #f59e0b; font-size: 0.625rem; background: rgba(245, 158, 11, 0.1); padding: 0.125rem 0.375rem; border-radius: 0.25rem;",
                            "Custom"
                        }
                    }
                }

                if !props.skill.description.is_empty() {
                    p {
                        style: "color: #6b7280; font-size: 0.75rem; margin: 0.25rem 0 0 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{props.skill.description}"
                    }
                }
            }
        }
    }
}


/// Empty state panel when no workflow is selected
#[component]
fn WorkflowEmptyStatePanel() -> Element {
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
