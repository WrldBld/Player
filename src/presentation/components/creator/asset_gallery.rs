//! Asset Gallery - Display and manage entity assets

use dioxus::prelude::*;

use crate::application::services::{Asset, GenerateRequest};
use crate::presentation::services::use_asset_service;

/// Asset types that can be generated
const ASSET_TYPES: &[(&str, &str)] = &[
    ("portrait", "Portrait"),
    ("sprite", "Sprite"),
    ("backdrop", "Backdrop"),
    ("emotion_sheet", "Emotions"),
];

/// Asset gallery for an entity
#[component]
pub fn AssetGallery(entity_type: String, entity_id: String) -> Element {
    let asset_service = use_asset_service();
    let mut selected_asset_type = use_signal(|| "portrait".to_string());
    let mut show_generate_modal = use_signal(|| false);
    let mut assets: Signal<Vec<Asset>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Fetch assets on mount (only if entity_id is not empty)
    {
        let entity_type_clone = entity_type.clone();
        let entity_id_clone = entity_id.clone();
        let asset_svc = asset_service.clone();

        use_effect(move || {
            let et = entity_type_clone.clone();
            let ei = entity_id_clone.clone();
            let svc = asset_svc.clone();
            spawn(async move {
                // Skip API call if entity_id is empty (new entity being created)
                if ei.is_empty() {
                    assets.set(Vec::new());
                    is_loading.set(false);
                    return;
                }

                match svc.get_assets(&et, &ei).await {
                    Ok(fetched_assets) => {
                        assets.set(fetched_assets);
                        is_loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                        is_loading.set(false);
                    }
                }
            });
        });
    }

    // Filter assets by selected type
    let selected_type = selected_asset_type.read().clone();
    let filtered_assets: Vec<Asset> = assets
        .read()
        .iter()
        .filter(|a| a.asset_type == selected_type)
        .cloned()
        .collect();

    rsx! {
        div {
            class: "asset-gallery",
            style: "background: #0f0f23; border-radius: 0.5rem; padding: 0.75rem;",

            // Error display
            if let Some(err) = error.read().as_ref() {
                div {
                    style: "padding: 0.75rem; background: rgba(239, 68, 68, 0.1); border-radius: 0.25rem; color: #ef4444; font-size: 0.875rem; margin-bottom: 0.75rem;",
                    "Error: {err}"
                }
            }

            // Asset type tabs
            div {
                class: "asset-tabs",
                style: "display: flex; gap: 0.25rem; margin-bottom: 0.75rem;",

                for (type_id, type_label) in ASSET_TYPES {
                    button {
                        onclick: {
                            let type_id = type_id.to_string();
                            move |_| selected_asset_type.set(type_id.clone())
                        },
                        style: format!(
                            "padding: 0.25rem 0.5rem; font-size: 0.75rem; border-radius: 0.25rem; cursor: pointer; border: none; {}",
                            if *selected_asset_type.read() == *type_id {
                                "background: #3b82f6; color: white;"
                            } else {
                                "background: transparent; color: #9ca3af;"
                            }
                        ),
                        "{type_label}"
                    }
                }
            }

            // Asset grid
            div {
                class: "asset-grid",
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem; min-height: 80px;",

                if entity_id.is_empty() {
                    // New entity - show message about generating assets after creation
                    div {
                        style: "width: 100%; text-align: center; color: #6b7280; font-size: 0.875rem; padding: 1rem; background: rgba(139, 92, 246, 0.1); border-radius: 0.25rem; border: 1px dashed #8b5cf6;",
                        "Save the {entity_type} first to generate assets"
                    }
                } else if *is_loading.read() {
                    div {
                        style: "width: 100%; text-align: center; color: #6b7280; font-size: 0.875rem; padding: 1rem;",
                        "Loading assets..."
                    }
                } else if filtered_assets.is_empty() {
                    div {
                        style: "width: 100%; text-align: center; color: #6b7280; font-size: 0.875rem; padding: 1rem;",
                        "No {selected_asset_type} assets yet"
                    }
                } else {
                    for asset in filtered_assets {
                        {
                            let entity_type_activate = entity_type.clone();
                            let entity_id_activate = entity_id.clone();
                            let entity_type_delete = entity_type.clone();
                            let entity_id_delete = entity_id.clone();
                            let asset_svc_activate = asset_service.clone();
                            let asset_svc_delete = asset_service.clone();
                            rsx! {
                                AssetThumbnail {
                                    id: asset.id.clone(),
                                    label: asset.label.clone(),
                                    is_active: asset.is_active,
                                    style_reference_id: asset.style_reference_id.clone(),
                                    on_activate: move |id: String| {
                                        let entity_type = entity_type_activate.clone();
                                        let entity_id = entity_id_activate.clone();
                                        let svc = asset_svc_activate.clone();
                                        spawn(async move {
                                            if let Err(e) = svc.activate_asset(&entity_type, &entity_id, &id).await {
                                                tracing::error!("Failed to activate asset: {}", e);
                                            }
                                        });
                                    },
                                    on_delete: move |id: String| {
                                        let entity_type = entity_type_delete.clone();
                                        let entity_id = entity_id_delete.clone();
                                        let svc = asset_svc_delete.clone();
                                        spawn(async move {
                                            if let Err(e) = svc.delete_asset(&entity_type, &entity_id, &id).await {
                                                tracing::error!("Failed to delete asset: {}", e);
                                            }
                                        });
                                    },
                                    on_use_as_reference: None, // TODO: Implement "Use as Reference" action
                                }
                            }
                        }
                    }
                }

                // Generate button (only show if entity_id exists)
                if !entity_id.is_empty() {
                button {
                    onclick: move |_| show_generate_modal.set(true),
                    style: "width: 64px; height: 64px; display: flex; flex-direction: column; align-items: center; justify-content: center; background: rgba(139, 92, 246, 0.2); border: 2px dashed #8b5cf6; border-radius: 0.5rem; cursor: pointer; color: #8b5cf6; font-size: 0.75rem;",
                    span { style: "font-size: 1.5rem;", "+" }
                    span { "Generate" }
                    }
                }
            }

            // Generation modal
            if *show_generate_modal.read() {
                GenerateAssetModal {
                    entity_type: entity_type.clone(),
                    entity_id: entity_id.clone(),
                    asset_type: selected_asset_type.read().clone(),
                    on_close: move |_| show_generate_modal.set(false),
                    on_generate: {
                        let asset_svc_gen = asset_service.clone();
                        move |req| {
                            let svc = asset_svc_gen.clone();
                            spawn(async move {
                                if let Err(e) = svc.generate_assets(&req).await {
                                    tracing::error!("Failed to queue generation: {}", e);
                                }
                            });
                            show_generate_modal.set(false);
                        }
                    },
                }
            }
        }
    }
}

/// Props for AssetThumbnail
#[derive(Props, Clone, PartialEq)]
struct AssetThumbnailProps {
    id: String,
    label: Option<String>,
    is_active: bool,
    style_reference_id: Option<String>,
    on_activate: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_use_as_reference: Option<EventHandler<String>>,
}

/// Individual asset thumbnail
#[component]
fn AssetThumbnail(props: AssetThumbnailProps) -> Element {
    let mut show_menu = use_signal(|| false);

    let border = if props.is_active {
        "2px solid #22c55e"
    } else {
        "2px solid transparent"
    };

    let id_for_activate = props.id.clone();
    let id_for_menu_activate = props.id.clone();
    let id_for_delete = props.id.clone();

    rsx! {
        div {
            style: format!(
                "width: 64px; height: 64px; background: #1a1a2e; border: {}; border-radius: 0.5rem; cursor: pointer; position: relative; overflow: hidden;",
                border
            ),
            oncontextmenu: move |e| {
                e.prevent_default();
                show_menu.toggle();
            },

            // Thumbnail click to activate
            div {
                onclick: {
                    let id = id_for_activate.clone();
                    let on_activate = props.on_activate.clone();
                    move |_| {
                        on_activate.call(id.clone());
                        show_menu.set(false);
                    }
                },
                style: "width: 100%; height: 100%; background: linear-gradient(135deg, #374151 0%, #1f2937 100%); display: flex; align-items: center; justify-content: center;",

                // Active indicator
                if props.is_active {
                    div {
                        style: "position: absolute; top: 2px; right: 2px; width: 8px; height: 8px; background: #22c55e; border-radius: 50%;",
                    }
                }
            }

            // Label
            if let Some(label) = &props.label {
                div {
                    style: "position: absolute; bottom: 0; left: 0; right: 0; padding: 2px; background: rgba(0,0,0,0.7); color: white; font-size: 0.625rem; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{label}"
                }
            }

            // Context menu
            if *show_menu.read() {
                div {
                    style: "position: absolute; top: 100%; left: 0; right: 0; background: #1f2937; border: 1px solid #374151; border-radius: 0.25rem; z-index: 100; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);",

                    if !props.is_active {
                        button {
                            onclick: {
                                let id = id_for_menu_activate.clone();
                                let on_activate = props.on_activate.clone();
                                move |_| {
                                    on_activate.call(id.clone());
                                    show_menu.set(false);
                                }
                            },
                            style: "display: block; width: 100%; padding: 0.5rem; text-align: left; background: transparent; color: white; border: none; cursor: pointer; font-size: 0.75rem; border-bottom: 1px solid #374151;",
                            "Activate"
                        }
                    }

                    if let Some(on_use_as_ref) = props.on_use_as_reference.as_ref() {
                        button {
                            onclick: {
                                let id = props.id.clone();
                                let handler = on_use_as_ref.clone();
                                move |_| {
                                    handler.call(id.clone());
                                    show_menu.set(false);
                                }
                            },
                            style: "display: block; width: 100%; padding: 0.5rem; text-align: left; background: transparent; color: #8b5cf6; border: none; cursor: pointer; font-size: 0.75rem; border-bottom: 1px solid #374151;",
                            "Use as Style Reference"
                        }
                    }

                    button {
                        onclick: {
                            let id = id_for_delete.clone();
                            let on_delete = props.on_delete.clone();
                            move |_| {
                                on_delete.call(id.clone());
                                show_menu.set(false);
                            }
                        },
                        style: "display: block; width: 100%; padding: 0.5rem; text-align: left; background: transparent; color: #ef4444; border: none; cursor: pointer; font-size: 0.75rem;",
                        "Delete"
                    }
                }
            }
        }
    }
}

/// Modal for generating new assets
#[component]
fn GenerateAssetModal(
    entity_type: String,
    entity_id: String,
    asset_type: String,
    on_close: EventHandler<()>,
    on_generate: EventHandler<GenerateRequest>,
) -> Element {
    let asset_service = use_asset_service();
    let mut prompt = use_signal(|| String::new());
    let mut negative_prompt = use_signal(|| String::new());
    let mut count = use_signal(|| 4u8);
    let mut workflow_slot = use_signal(|| String::new());
    let mut is_generating = use_signal(|| false);
    let mut style_reference_id: Signal<Option<String>> = use_signal(|| None);
    let mut style_reference_label: Signal<Option<String>> = use_signal(|| None);
    let mut show_style_selector = use_signal(|| false);
    let mut available_assets: Signal<Vec<Asset>> = use_signal(Vec::new);

    // Load available assets for style reference selection
    let entity_type_for_assets = entity_type.clone();
    let entity_id_for_assets = entity_id.clone();
    use_effect(move || {
        let et = entity_type_for_assets.clone();
        let ei = entity_id_for_assets.clone();
        let svc = asset_service.clone();
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
            onclick: move |_| on_close.call(()),

            div {
                class: "modal-content",
                style: "background: #1a1a2e; border-radius: 0.75rem; padding: 1.5rem; width: 90%; max-width: 500px;",
                onclick: move |e| e.stop_propagation(),

                h3 { style: "color: white; margin: 0 0 1rem 0;", "Generate {asset_type}" }

                // Workflow slot field (optional hint text)
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
                                        onmouseenter: move |_| {
                                            // Could add hover effect
                                        },
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

                // Prompt field
                div { style: "margin-bottom: 1rem;",
                    label { style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Prompt" }
                    textarea {
                        value: "{prompt}",
                        oninput: move |e| prompt.set(e.value()),
                        placeholder: "Describe the {asset_type} you want to generate...",
                        style: "width: 100%; min-height: 80px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
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
                        onclick: move |_| on_close.call(()),
                        disabled: *is_generating.read(),
                        style: "padding: 0.5rem 1rem; background: transparent; color: #9ca3af; border: 1px solid #374151; border-radius: 0.25rem; cursor: pointer;",
                        "Cancel"
                    }
                    button {
                        onclick: {
                            let entity_type = entity_type.clone();
                            let entity_id = entity_id.clone();
                            let asset_type = asset_type.clone();
                            move |_| {
                                is_generating.set(true);
                                on_generate.call(GenerateRequest {
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
                                });
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

