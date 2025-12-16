//! Event Chain Editor - Edit chain info and manage events

use dioxus::prelude::*;
use crate::application::services::{EventChainData, CreateEventChainRequest, UpdateEventChainRequest, AddEventRequest};
use crate::presentation::services::use_event_chain_service;

#[derive(Props, Clone, PartialEq)]
pub struct EventChainEditorProps {
    pub chain: Option<EventChainData>,
    pub world_id: String,
    pub on_save: EventHandler<EventChainData>,
    pub on_cancel: EventHandler<()>,
}

/// Event chain editor component for creating/editing chains
#[component]
pub fn EventChainEditor(props: EventChainEditorProps) -> Element {
    let event_chain_service = use_event_chain_service();
    let is_editing = props.chain.is_some();

    let mut name = use_signal(|| props.chain.as_ref().map(|c| c.name.clone()).unwrap_or_default());
    let mut description = use_signal(|| props.chain.as_ref().map(|c| c.description.clone()).unwrap_or_default());
    let mut tags = use_signal(|| props.chain.as_ref().map(|c| c.tags.clone()).unwrap_or_default());
    let mut color = use_signal(|| props.chain.as_ref().and_then(|c| c.color.clone()));
    let mut is_active = use_signal(|| props.chain.as_ref().map(|c| c.is_active).unwrap_or(true));
    let mut new_tag = use_signal(String::new);
    let mut is_saving = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let save_handler = {
        let world_id = props.world_id.clone();
        let service = event_chain_service.clone();
        let chain_id = props.chain.as_ref().map(|c| c.id.clone());
        move || {
            let world_id = world_id.clone();
            let service = service.clone();
            let chain_id = chain_id.clone();
            let name_val = name.read().clone();
            let desc_val = description.read().clone();
            let tags_val = tags.read().clone();
            let color_val = color.read().clone();
            let active_val = *is_active.read();
            let mut saving = is_saving;
            let mut err = error;
            let on_save_handler = props.on_save.clone();
            spawn(async move {
                saving.set(true);
                err.set(None);
                let result = if let Some(id) = chain_id {
                    // Update existing
                    let request = UpdateEventChainRequest {
                        name: Some(name_val),
                        description: Some(desc_val),
                        tags: Some(tags_val),
                        color: color_val,
                        is_active: Some(active_val),
                        events: None,
                        act_id: None,
                    };
                    service.update_chain(&id, &request).await
                } else {
                    // Create new
                    let request = CreateEventChainRequest {
                        name: name_val,
                        description: desc_val,
                        events: Vec::new(),
                        act_id: None,
                        tags: tags_val,
                        color: color_val,
                        is_active: active_val,
                    };
                    service.create_chain(&world_id, &request).await
                };
                match result {
                    Ok(chain) => {
                        on_save_handler.call(chain);
                    }
                    Err(e) => {
                        err.set(Some(e.to_string()));
                    }
                }
                saving.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "event-chain-editor",
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1.5rem; max-width: 600px;",

            h2 {
                style: "color: white; margin: 0 0 1.5rem 0; font-size: 1.25rem;",
                if is_editing { "Edit Event Chain" } else { "Create Event Chain" }
            }

            if let Some(err) = error.read().as_ref() {
                div {
                    style: "padding: 0.75rem; background: #1f2937; border-left: 3px solid #ef4444; border-radius: 0.25rem; margin-bottom: 1rem;",
                    div {
                        style: "color: #ef4444; font-size: 0.875rem;",
                        "Error: {err}"
                    }
                }
            }

            // Form fields
            div {
                style: "display: flex; flex-direction: column; gap: 1rem;",

                // Name
                div {
                    label {
                        style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;",
                        "Chain Name"
                    }
                    input {
                        r#type: "text",
                        value: "{name.read()}",
                        oninput: move |evt| name.set(evt.value()),
                        placeholder: "Enter chain name",
                        style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; font-size: 0.875rem;",
                    }
                }

                // Description
                div {
                    label {
                        style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;",
                        "Description"
                    }
                    textarea {
                        value: "{description.read()}",
                        oninput: move |evt| description.set(evt.value()),
                        placeholder: "Enter chain description",
                        rows: 3,
                        style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; font-size: 0.875rem; resize: vertical;",
                    }
                }

                // Color
                div {
                    label {
                        style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;",
                        "Color (hex)"
                    }
                    input {
                        r#type: "text",
                        value: "{color.read().as_ref().map(|c| c.as_str()).unwrap_or(\"\")}",
                        oninput: move |evt| color.set(if evt.value().is_empty() { None } else { Some(evt.value()) }),
                        placeholder: "#8b5cf6",
                        style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; font-size: 0.875rem;",
                    }
                }

                // Tags
                div {
                    label {
                        style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;",
                        "Tags"
                    }
                    div {
                        style: "display: flex; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.5rem;",
                        for tag in tags.read().iter() {
                            span {
                                style: "display: inline-flex; align-items: center; gap: 0.25rem; padding: 0.25rem 0.5rem; background: #374151; color: white; border-radius: 0.25rem; font-size: 0.75rem;",
                                "{tag}"
                                button {
                                    onclick: {
                                        let tag_to_remove = tag.clone();
                                        move |_| {
                                            let mut tags_list = tags.write();
                                            tags_list.retain(|t| t != &tag_to_remove);
                                        }
                                    },
                                    style: "background: none; border: none; color: white; cursor: pointer; padding: 0; margin-left: 0.25rem;",
                                    "×"
                                }
                            }
                        }
                    }
                    div {
                        style: "display: flex; gap: 0.5rem;",
                        input {
                            r#type: "text",
                            value: "{new_tag.read()}",
                            oninput: move |evt| new_tag.set(evt.value()),
                            onkeydown: move |evt| {
                                if evt.key().to_string() == "Enter" {
                                    evt.prevent_default();
                                    let tag = new_tag.read().trim().to_string();
                                    if !tag.is_empty() && !tags.read().contains(&tag) {
                                        tags.write().push(tag);
                                        new_tag.set(String::new());
                                    }
                                }
                            },
                            placeholder: "Add tag (press Enter)",
                            style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; font-size: 0.875rem;",
                        }
                        button {
                            onclick: move |_| {
                                let tag = new_tag.read().trim().to_string();
                                if !tag.is_empty() && !tags.read().contains(&tag) {
                                    tags.write().push(tag);
                                    new_tag.set(String::new());
                                }
                            },
                            style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.875rem;",
                            "Add"
                        }
                    }
                }

                // Active toggle
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    input {
                        r#type: "checkbox",
                        checked: *is_active.read(),
                        onchange: move |evt| is_active.set(evt.checked()),
                    }
                    label {
                        style: "color: #9ca3af; font-size: 0.875rem; cursor: pointer;",
                        "Active"
                    }
                }
            }

            // Actions
            div {
                style: "display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 1.5rem;",
                button {
                    onclick: move |_| props.on_cancel.call(()),
                    disabled: *is_saving.read(),
                    style: "padding: 0.5rem 1.5rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.875rem;",
                    "Cancel"
                }
                button {
                    onclick: move |_| save_handler(),
                    disabled: *is_saving.read() || name.read().trim().is_empty(),
                    style: format!(
                        "padding: 0.5rem 1.5rem; background: {}; color: white; border: none; border-radius: 0.25rem; cursor: {}; font-size: 0.875rem;",
                        if *is_saving.read() || name.read().trim().is_empty() { "#6b7280" } else { "#8b5cf6" },
                        if *is_saving.read() || name.read().trim().is_empty() { "not-allowed" } else { "pointer" }
                    ),
                    if *is_saving.read() { "Saving..." } else { "Save" }
                }
            }
        }
    }
}

