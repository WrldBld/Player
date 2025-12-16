//! Director Generate Modal - Quick asset generation from Director Mode
//!
//! Pre-populates prompts from character description and provides
//! quick access to asset generation without switching to Creator Mode.

use dioxus::prelude::*;

use crate::application::services::{Asset, GenerateRequest};
use crate::presentation::services::use_asset_service;

/// Props for DirectorGenerateModal
#[derive(Props, Clone, PartialEq)]
pub struct DirectorGenerateModalProps {
    /// Entity type (e.g., "character")
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Asset type to generate
    pub asset_type: String,
    /// Character name for display
    pub character_name: String,
    /// Initial prompt (pre-populated from character description)
    pub initial_prompt: String,
    /// Handler called when modal should close
    pub on_close: EventHandler<()>,
}

/// Director Mode generation modal with pre-populated prompt
#[component]
pub fn DirectorGenerateModal(props: DirectorGenerateModalProps) -> Element {
    let asset_service = use_asset_service();
    let mut prompt = use_signal(|| props.initial_prompt.clone());
    let mut negative_prompt = use_signal(|| String::new());
    let mut count = use_signal(|| 4u8);
    let mut workflow_slot = use_signal(|| String::new());
    let mut is_generating = use_signal(|| false);
    let mut style_reference_id: Signal<Option<String>> = use_signal(|| None);
    let mut style_reference_label: Signal<Option<String>> = use_signal(|| None);
    let mut show_style_selector = use_signal(|| false);
    let mut available_assets: Signal<Vec<Asset>> = use_signal(Vec::new);

    // Load available assets for style reference selection
    let entity_type_for_assets = props.entity_type.clone();
    let entity_id_for_assets = props.entity_id.clone();
    let asset_service_for_effect = asset_service.clone();
    use_effect(move || {
        let et = entity_type_for_assets.clone();
        let ei = entity_id_for_assets.clone();
        let svc = asset_service_for_effect.clone();
        spawn(async move {
            if let Ok(assets) = svc.get_assets(&et, &ei).await {
                available_assets.set(assets);
            }
        });
    });

    rsx! {
        div {
            class: "modal-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_close.call(()),

            div {
                class: "modal-content",
                style: "background: #1a1a2e; border-radius: 0.75rem; padding: 1.5rem; width: 90%; max-width: 500px; max-height: 90vh; overflow-y: auto;",
                onclick: move |e| e.stop_propagation(),

                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;",
                    h3 { style: "color: white; margin: 0;", "Generate {props.asset_type} for {props.character_name}" }
                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "padding: 0.25rem 0.5rem; background: transparent; color: #9ca3af; border: none; cursor: pointer; font-size: 1.25rem;",
                        "×"
                    }
                }

                // Workflow slot field (optional)
                div { style: "margin-bottom: 1rem;",
                    label { style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Workflow Slot (optional)" }
                    input {
                        r#type: "text",
                        value: "{workflow_slot}",
                        oninput: move |e| workflow_slot.set(e.value()),
                        placeholder: "Leave empty for default workflow...",
                        style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                    }
                }

                // Style Reference field
                div { style: "margin-bottom: 1rem;",
                    label { style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Style Reference (optional)" }
                    if let Some(ref_id) = style_reference_id.read().as_ref() {
                        div {
                            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem;",
                            span {
                                style: "flex: 1; color: white; font-size: 0.875rem;",
                                if let Some(label) = style_reference_label.read().as_ref() {
                                    "{label}"
                                } else {
                                    "Selected asset"
                                }
                            }
                            button {
                                onclick: move |_| {
                                    style_reference_id.set(None);
                                    style_reference_label.set(None);
                                },
                                style: "padding: 0.25rem 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                                "Clear"
                            }
                        }
                    } else {
                        div {
                            style: "display: flex; gap: 0.5rem;",
                            button {
                                onclick: move |_| show_style_selector.set(true),
                                style: "flex: 1; padding: 0.5rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.875rem;",
                                "Select from Gallery..."
                            }
                        }
                    }
                }

                // Style reference selector modal
                if *show_style_selector.read() {
                    div {
                        class: "modal-overlay",
                        style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.9); display: flex; align-items: center; justify-content: center; z-index: 1001;",
                        onclick: move |_| show_style_selector.set(false),
                        div {
                            style: "background: #1a1a2e; border-radius: 0.75rem; padding: 1.5rem; width: 90%; max-width: 600px; max-height: 80vh; overflow-y: auto;",
                            onclick: move |e| e.stop_propagation(),
                            h3 { style: "color: white; margin: 0 0 1rem 0;", "Select Style Reference" }
                            div {
                                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 0.75rem;",
                                for asset in available_assets.read().iter() {
                                    button {
                                        onclick: {
                                            let asset_id = asset.id.clone();
                                            let asset_label = asset.label.clone().or_else(|| Some(asset.id.clone()));
                                            move |_| {
                                                style_reference_id.set(Some(asset_id.clone()));
                                                style_reference_label.set(asset_label.clone());
                                                show_style_selector.set(false);
                                            }
                                        },
                                        style: "display: flex; flex-direction: column; align-items: center; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; cursor: pointer; transition: all 0.2s;",
                                        div {
                                            style: "width: 80px; height: 80px; background: #374151; border-radius: 0.25rem; margin-bottom: 0.5rem; display: flex; align-items: center; justify-content: center;",
                                            span { style: "color: #9ca3af; font-size: 0.75rem;", "📷" }
                                        }
                                        span {
                                            style: "color: white; font-size: 0.75rem; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; width: 100%;",
                                            "{asset.label.as_ref().unwrap_or(&asset.id)}"
                                        }
                                    }
                                }
                            }
                            if available_assets.read().is_empty() {
                                div {
                                    style: "color: #6b7280; text-align: center; padding: 2rem;",
                                    "No assets available for style reference"
                                }
                            }
                        }
                    }
                }

                // Prompt field (pre-populated)
                div { style: "margin-bottom: 1rem;",
                    label { style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Prompt" }
                    textarea {
                        value: "{prompt}",
                        oninput: move |e| prompt.set(e.value()),
                        placeholder: "Describe the {props.asset_type} you want to generate...",
                        style: "width: 100%; min-height: 100px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                    }
                }

                // Negative prompt field
                div { style: "margin-bottom: 1rem;",
                    label { style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Negative Prompt (optional)" }
                    input {
                        r#type: "text",
                        value: "{negative_prompt}",
                        oninput: move |e| negative_prompt.set(e.value()),
                        placeholder: "Things to avoid...",
                        style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                    }
                }

                // Variation count
                div { style: "margin-bottom: 1.5rem;",
                    label { style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Variations: {count}" }
                    input {
                        r#type: "range",
                        min: "1",
                        max: "8",
                        value: "{count}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<u8>() {
                                count.set(v);
                            }
                        },
                        style: "width: 100%;",
                    }
                }

                // Action buttons
                div { style: "display: flex; justify-content: flex-end; gap: 0.5rem;",
                    button {
                        onclick: move |_| props.on_close.call(()),
                        disabled: *is_generating.read(),
                        style: "padding: 0.5rem 1rem; background: transparent; color: #9ca3af; border: 1px solid #374151; border-radius: 0.25rem; cursor: pointer;",
                        "Cancel"
                    }
                    button {
                        onclick: {
                            let entity_type = props.entity_type.clone();
                            let entity_id = props.entity_id.clone();
                            let asset_type = props.asset_type.clone();
                            let svc = asset_service.clone();
                            move |_| {
                                is_generating.set(true);
                                let req = GenerateRequest {
                                    entity_type: entity_type.clone(),
                                    entity_id: entity_id.clone(),
                                    asset_type: asset_type.clone(),
                                    prompt: prompt.read().clone(),
                                    negative_prompt: if negative_prompt.read().is_empty() {
                                        None
                                    } else {
                                        Some(negative_prompt.read().clone())
                                    },
                                    count: *count.read(),
                                    style_reference_id: style_reference_id.read().clone(),
                                };
                                let svc_clone = svc.clone();
                                spawn(async move {
                                    if let Err(e) = svc_clone.generate_assets(&req).await {
                                        tracing::error!("Failed to queue generation: {}", e);
                                    }
                                });
                                props.on_close.call(());
                                is_generating.set(false);
                            }
                        },
                        disabled: *is_generating.read(),
                        style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-weight: 500;",
                        if *is_generating.read() { "Generating..." } else { "Generate" }
                    }
                }
            }
        }
    }
}

