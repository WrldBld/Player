//! Skills Panel - Manage skills for a world
//!
//! Displays skills grouped by category with options to:
//! - View default skills from the rule system
//! - Hide/show default skills
//! - Create custom skills
//! - Edit and delete custom skills

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::application::dto::{SkillCategory, SkillData};
use crate::application::services::{CreateSkillRequest, UpdateSkillRequest};
use crate::presentation::services::use_skill_service;

/// Props for SkillsPanel
#[derive(Props, Clone, PartialEq)]
pub struct SkillsPanelProps {
    /// The world ID to manage skills for
    pub world_id: String,
    /// Called when the panel should close
    pub on_close: EventHandler<()>,
}

/// Skills Panel component
#[component]
pub fn SkillsPanel(props: SkillsPanelProps) -> Element {
    let mut skills: Signal<Vec<SkillData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut show_hidden = use_signal(|| false);
    let mut show_add_form = use_signal(|| false);
    let mut editing_skill: Signal<Option<String>> = use_signal(|| None);

    // Clone world_id once for all handlers
    let world_id = props.world_id.clone();
    let world_id_for_effect = world_id.clone();
    let world_id_for_rows = world_id.clone();
    let world_id_for_add = world_id.clone();
    let world_id_for_edit = world_id;

    // Get skill service
    let skill_service = use_skill_service();

    // Load skills on mount
    use_effect(move || {
        let world_id = world_id_for_effect.clone();
        let service = skill_service.clone();
        spawn(async move {
            match service.list_skills(&world_id).await {
                Ok(list) => {
                    skills.set(list);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load skills: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    });

    // Group skills by category - clone the data to avoid borrow issues
    let skills_by_category: HashMap<SkillCategory, Vec<SkillData>> = {
        let skills_read = skills.read();
        let show_hidden_val = *show_hidden.read();
        let mut grouped: HashMap<SkillCategory, Vec<SkillData>> = HashMap::new();
        for skill in skills_read.iter() {
            if !skill.is_hidden || show_hidden_val {
                grouped.entry(skill.category).or_default().push(skill.clone());
            }
        }
        // Sort skills within each category by order
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

    let handle_skill_created = move |skill: SkillData| {
        skills.write().push(skill);
        show_add_form.set(false);
    };

    let handle_skill_updated = move |skill: SkillData| {
        let mut skills_write = skills.write();
        if let Some(existing) = skills_write.iter_mut().find(|s| s.id == skill.id) {
            *existing = skill;
        }
        editing_skill.set(None);
    };

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_close.call(()),

            div {
                style: "background: #1a1a2e; border-radius: 0.5rem; width: 90%; max-width: 800px; max-height: 90vh; overflow: hidden; display: flex; flex-direction: column;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem 1.5rem; border-bottom: 1px solid #374151;",

                    h2 { style: "color: white; margin: 0; font-size: 1.25rem;", "Skills Management" }

                    div { style: "display: flex; gap: 0.5rem; align-items: center;",
                        // Show hidden toggle
                        label { style: "display: flex; align-items: center; gap: 0.5rem; color: #9ca3af; font-size: 0.75rem; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: *show_hidden.read(),
                                onchange: move |e| show_hidden.set(e.checked()),
                            }
                            "Show Hidden"
                        }

                        button {
                            onclick: move |_| props.on_close.call(()),
                            style: "padding: 0.5rem; background: transparent; border: none; color: #9ca3af; cursor: pointer; font-size: 1.25rem;",
                            "X"
                        }
                    }
                }

                // Error message
                if let Some(err) = error.read().as_ref() {
                    div {
                        style: "padding: 0.75rem 1.5rem; background: rgba(239, 68, 68, 0.1); color: #ef4444; font-size: 0.875rem;",
                        "{err}"
                    }
                }

                // Content
                div {
                    style: "flex: 1; overflow-y: auto; padding: 1rem 1.5rem;",

                    if *is_loading.read() {
                        div {
                            style: "text-align: center; color: #6b7280; padding: 2rem;",
                            "Loading skills..."
                        }
                    } else if *show_add_form.read() {
                        AddSkillForm {
                            world_id: world_id_for_add.clone(),
                            on_created: handle_skill_created,
                            on_cancel: move |_| show_add_form.set(false),
                        }
                    } else if let Some(skill_id) = editing_skill.read().clone() {
                        {
                            let skill = skills.read().iter().find(|s| s.id == skill_id).cloned();
                            if let Some(skill) = skill {
                                rsx! {
                                    EditSkillForm {
                                        world_id: world_id_for_edit.clone(),
                                        skill: skill,
                                        on_updated: handle_skill_updated,
                                        on_cancel: move |_| editing_skill.set(None),
                                    }
                                }
                            } else {
                                rsx! { div { "Skill not found" } }
                            }
                        }
                    } else {
                        // Add skill button
                        div { style: "margin-bottom: 1rem;",
                            button {
                                onclick: move |_| show_add_form.set(true),
                                style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.875rem;",
                                "+ Add Custom Skill"
                            }
                        }

                        // Skills by category
                        for category in active_categories.iter() {
                            {
                                let cat_skills = skills_by_category.get(category).cloned().unwrap_or_default();
                                if !cat_skills.is_empty() {
                                    rsx! {
                                        div { style: "margin-bottom: 1.5rem;",
                                            h3 { style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0 0 0.5rem 0;",
                                                "{category.display_name()} ({cat_skills.len()})"
                                            }

                                            div { style: "display: flex; flex-direction: column; gap: 0.25rem;",
                                                for skill in cat_skills.iter() {
                                                    SkillRow {
                                                        key: "{skill.id}",
                                                        world_id: world_id_for_rows.clone(),
                                                        skill: skill.clone(),
                                                        skills_signal: skills,
                                                        error_signal: error,
                                                        on_edit: move |id| editing_skill.set(Some(id)),
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

                        if skills.read().is_empty() {
                            div {
                                style: "text-align: center; color: #6b7280; padding: 2rem;",
                                p { "No skills defined for this world." }
                                p { style: "font-size: 0.875rem;", "Skills are loaded from the rule system preset." }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Individual skill row
#[component]
fn SkillRow(
    world_id: String,
    skill: SkillData,
    skills_signal: Signal<Vec<SkillData>>,
    error_signal: Signal<Option<String>>,
    on_edit: EventHandler<String>,
) -> Element {
    let skill_id_for_toggle = skill.id.clone();
    let skill_id_for_edit = skill.id.clone();
    let skill_id_for_delete = skill.id.clone();
    let is_hidden = skill.is_hidden;
    let is_custom = skill.is_custom;

    let world_id_for_toggle = world_id.clone();
    let world_id_for_delete = world_id.clone();

    // Get skill service
    let skill_service = use_skill_service();

    // Pre-compute styles based on hidden state
    let row_bg = if skill.is_hidden { "rgba(107, 114, 128, 0.2)" } else { "#0f0f23" };
    let icon_color = if skill.is_hidden { "#6b7280" } else { "#10b981" };
    let name_color = if skill.is_hidden { "#6b7280" } else { "white" };
    let row_style = format!("display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem; background: {}; border-radius: 0.25rem;", row_bg);
    let icon_style = format!("padding: 0.25rem; background: transparent; border: none; color: {}; cursor: pointer; font-size: 0.875rem;", icon_color);
    let name_style = format!("color: {}; font-weight: 500;", name_color);

    let handle_toggle = {
        let service = skill_service.clone();
        move |_| {
            let world_id = world_id_for_toggle.clone();
            let skill_id = skill_id_for_toggle.clone();
            let new_hidden = !is_hidden;
            let service = service.clone();
            spawn(async move {
                match service.update_skill_visibility(&world_id, &skill_id, new_hidden).await {
                    Ok(updated) => {
                        let mut skills_write = skills_signal.write();
                        if let Some(skill) = skills_write.iter_mut().find(|s| s.id == skill_id) {
                            skill.is_hidden = updated.is_hidden;
                        }
                    }
                    Err(e) => {
                        error_signal.set(Some(format!("Failed to update skill: {}", e)));
                    }
                }
            });
        }
    };

    let handle_delete = {
        let service = skill_service.clone();
        move |_| {
            let world_id = world_id_for_delete.clone();
            let skill_id = skill_id_for_delete.clone();
            let service = service.clone();
            spawn(async move {
                match service.delete_skill(&world_id, &skill_id).await {
                    Ok(()) => {
                        skills_signal.write().retain(|s| s.id != skill_id);
                    }
                    Err(e) => {
                        error_signal.set(Some(format!("Failed to delete skill: {}", e)));
                    }
                }
            });
        }
    };

    rsx! {
        div {
            style: "{row_style}",

            // Visibility toggle
            button {
                onclick: handle_toggle,
                style: "{icon_style}",
                title: if is_hidden { "Show skill" } else { "Hide skill" },
                if is_hidden { "👁" } else { "👁" }
            }

            // Skill info
            div { style: "flex: 1; min-width: 0;",
                div { style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "{name_style}", "{skill.name}" }
                    if let Some(attr) = &skill.base_attribute {
                        span { style: "color: #8b5cf6; font-size: 0.75rem; background: rgba(139, 92, 246, 0.1); padding: 0.125rem 0.375rem; border-radius: 0.25rem;",
                            "{attr}"
                        }
                    }
                    if is_custom {
                        span { style: "color: #f59e0b; font-size: 0.625rem; background: rgba(245, 158, 11, 0.1); padding: 0.125rem 0.375rem; border-radius: 0.25rem;",
                            "Custom"
                        }
                    }
                }
                if !skill.description.is_empty() {
                    p { style: "color: #6b7280; font-size: 0.75rem; margin: 0.25rem 0 0 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{skill.description}"
                    }
                }
            }

            // Actions
            div { style: "display: flex; gap: 0.25rem;",
                if is_custom {
                    button {
                        onclick: move |_| on_edit.call(skill_id_for_edit.clone()),
                        style: "padding: 0.25rem 0.5rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                        "Edit"
                    }
                    button {
                        onclick: handle_delete,
                        style: "padding: 0.25rem 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                        "X"
                    }
                }
            }
        }
    }
}

/// Form to add a new custom skill
#[component]
fn AddSkillForm(
    world_id: String,
    on_created: EventHandler<SkillData>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut category = use_signal(|| SkillCategory::Custom);
    let mut base_attribute = use_signal(|| String::new());
    let mut is_creating = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Get skill service
    let skill_service = use_skill_service();

    let handle_create = move |_| {
        let name_val = name.read().clone();
        if name_val.trim().is_empty() {
            error.set(Some("Skill name is required".to_string()));
            return;
        }

        let world_id = world_id.clone();
        let desc = description.read().clone();
        let cat = *category.read();
        let attr = base_attribute.read().clone();
        let service = skill_service.clone();

        spawn(async move {
            is_creating.set(true);
            error.set(None);

            let request = CreateSkillRequest {
                name: name_val,
                description: desc,
                category: cat,
                base_attribute: if attr.is_empty() { None } else { Some(attr) },
            };

            match service.create_skill(&world_id, &request).await {
                Ok(skill) => {
                    on_created.call(skill);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to create skill: {}", e)));
                    is_creating.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            style: "padding: 1rem; background: #0f0f23; border-radius: 0.5rem;",

            h3 { style: "color: white; margin: 0 0 1rem 0; font-size: 1rem;", "Add Custom Skill" }

            if let Some(err) = error.read().as_ref() {
                div {
                    style: "padding: 0.5rem; background: rgba(239, 68, 68, 0.1); color: #ef4444; font-size: 0.875rem; margin-bottom: 1rem; border-radius: 0.25rem;",
                    "{err}"
                }
            }

            // Name
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Name *" }
                input {
                    r#type: "text",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    disabled: *is_creating.read(),
                    placeholder: "Skill name",
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                }
            }

            // Description
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Description" }
                textarea {
                    value: "{description}",
                    oninput: move |e| description.set(e.value()),
                    disabled: *is_creating.read(),
                    placeholder: "What this skill is used for...",
                    style: "width: 100%; min-height: 60px; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                }
            }

            // Category
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Category" }
                select {
                    value: "{category.read().display_name()}",
                    onchange: move |e| {
                        let cat = match e.value().as_str() {
                            "Physical" => SkillCategory::Physical,
                            "Mental" => SkillCategory::Mental,
                            "Social" => SkillCategory::Social,
                            "Interpersonal" => SkillCategory::Interpersonal,
                            "Investigation" => SkillCategory::Investigation,
                            "Academic" => SkillCategory::Academic,
                            "Practical" => SkillCategory::Practical,
                            "Combat" => SkillCategory::Combat,
                            "Approach" => SkillCategory::Approach,
                            "Aspect" => SkillCategory::Aspect,
                            "Other" => SkillCategory::Other,
                            _ => SkillCategory::Custom,
                        };
                        category.set(cat);
                    },
                    disabled: *is_creating.read(),
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                    for cat in SkillCategory::all() {
                        option { value: "{cat.display_name()}", "{cat.display_name()}" }
                    }
                }
            }

            // Base Attribute
            div { style: "margin-bottom: 1rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Base Attribute (optional)" }
                input {
                    r#type: "text",
                    value: "{base_attribute}",
                    oninput: move |e| base_attribute.set(e.value()),
                    disabled: *is_creating.read(),
                    placeholder: "e.g., STR, DEX, INT",
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                }
            }

            // Buttons
            div { style: "display: flex; gap: 0.5rem;",
                button {
                    onclick: handle_create,
                    disabled: *is_creating.read(),
                    style: "flex: 1; padding: 0.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                    if *is_creating.read() { "Creating..." } else { "Create Skill" }
                }
                button {
                    onclick: move |_| on_cancel.call(()),
                    disabled: *is_creating.read(),
                    style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                    "Cancel"
                }
            }
        }
    }
}

/// Form to edit an existing skill
#[component]
fn EditSkillForm(
    world_id: String,
    skill: SkillData,
    on_updated: EventHandler<SkillData>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| skill.name.clone());
    let mut description = use_signal(|| skill.description.clone());
    let mut category = use_signal(|| skill.category);
    let mut base_attribute = use_signal(|| skill.base_attribute.clone().unwrap_or_default());
    let mut is_saving = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let skill_id = skill.id.clone();

    // Get skill service
    let skill_service = use_skill_service();

    let handle_save = move |_| {
        let name_val = name.read().clone();
        if name_val.trim().is_empty() {
            error.set(Some("Skill name is required".to_string()));
            return;
        }

        let world_id = world_id.clone();
        let skill_id = skill_id.clone();
        let desc = description.read().clone();
        let cat = *category.read();
        let attr = base_attribute.read().clone();
        let service = skill_service.clone();

        spawn(async move {
            is_saving.set(true);
            error.set(None);

            let request = UpdateSkillRequest {
                name: Some(name_val),
                description: Some(desc),
                category: Some(cat),
                base_attribute: if attr.is_empty() { None } else { Some(attr) },
                is_hidden: None,
            };

            match service.update_skill(&world_id, &skill_id, &request).await {
                Ok(skill) => {
                    on_updated.call(skill);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to update skill: {}", e)));
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            style: "padding: 1rem; background: #0f0f23; border-radius: 0.5rem;",

            h3 { style: "color: white; margin: 0 0 1rem 0; font-size: 1rem;", "Edit Skill" }

            if let Some(err) = error.read().as_ref() {
                div {
                    style: "padding: 0.5rem; background: rgba(239, 68, 68, 0.1); color: #ef4444; font-size: 0.875rem; margin-bottom: 1rem; border-radius: 0.25rem;",
                    "{err}"
                }
            }

            // Name
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Name *" }
                input {
                    r#type: "text",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    disabled: *is_saving.read(),
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                }
            }

            // Description
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Description" }
                textarea {
                    value: "{description}",
                    oninput: move |e| description.set(e.value()),
                    disabled: *is_saving.read(),
                    style: "width: 100%; min-height: 60px; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                }
            }

            // Category
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Category" }
                select {
                    value: "{category.read().display_name()}",
                    onchange: move |e| {
                        let cat = match e.value().as_str() {
                            "Physical" => SkillCategory::Physical,
                            "Mental" => SkillCategory::Mental,
                            "Social" => SkillCategory::Social,
                            "Interpersonal" => SkillCategory::Interpersonal,
                            "Investigation" => SkillCategory::Investigation,
                            "Academic" => SkillCategory::Academic,
                            "Practical" => SkillCategory::Practical,
                            "Combat" => SkillCategory::Combat,
                            "Approach" => SkillCategory::Approach,
                            "Aspect" => SkillCategory::Aspect,
                            "Other" => SkillCategory::Other,
                            _ => SkillCategory::Custom,
                        };
                        category.set(cat);
                    },
                    disabled: *is_saving.read(),
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                    for cat in SkillCategory::all() {
                        option { value: "{cat.display_name()}", "{cat.display_name()}" }
                    }
                }
            }

            // Base Attribute
            div { style: "margin-bottom: 1rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Base Attribute (optional)" }
                input {
                    r#type: "text",
                    value: "{base_attribute}",
                    oninput: move |e| base_attribute.set(e.value()),
                    disabled: *is_saving.read(),
                    placeholder: "e.g., STR, DEX, INT",
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                }
            }

            // Buttons
            div { style: "display: flex; gap: 0.5rem;",
                button {
                    onclick: handle_save,
                    disabled: *is_saving.read(),
                    style: "flex: 1; padding: 0.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                    if *is_saving.read() { "Saving..." } else { "Save Changes" }
                }
                button {
                    onclick: move |_| on_cancel.call(()),
                    disabled: *is_saving.read(),
                    style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                    "Cancel"
                }
            }
        }
    }
}

