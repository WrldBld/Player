//! Asset Gallery - Display and manage entity assets

use dioxus::prelude::*;

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
    let mut selected_asset_type = use_signal(|| "portrait".to_string());
    let mut show_generate_modal = use_signal(|| false);

    // Mock assets - will be replaced with API call
    let assets: Vec<MockAsset> = vec![
        MockAsset {
            id: "asset-1".into(),
            asset_type: "portrait".into(),
            label: Some("Default".into()),
            is_active: true,
        },
        MockAsset {
            id: "asset-2".into(),
            asset_type: "portrait".into(),
            label: Some("Angry".into()),
            is_active: false,
        },
    ];

    let filtered_assets: Vec<_> = assets
        .iter()
        .filter(|a| a.asset_type == *selected_asset_type.read())
        .collect();

    rsx! {
        div {
            class: "asset-gallery",
            style: "background: #0f0f23; border-radius: 0.5rem; padding: 0.75rem;",

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

                if filtered_assets.is_empty() {
                    div {
                        style: "width: 100%; text-align: center; color: #6b7280; font-size: 0.875rem; padding: 1rem;",
                        "No {selected_asset_type} assets yet"
                    }
                } else {
                    for asset in filtered_assets {
                        AssetThumbnail {
                            id: asset.id.clone(),
                            label: asset.label.clone(),
                            is_active: asset.is_active,
                            on_activate: move |id| {
                                // TODO: Call API to activate asset
                                tracing::info!("Activate asset: {}", id);
                            },
                        }
                    }
                }

                // Generate button
                button {
                    onclick: move |_| show_generate_modal.set(true),
                    style: "width: 64px; height: 64px; display: flex; flex-direction: column; align-items: center; justify-content: center; background: rgba(139, 92, 246, 0.2); border: 2px dashed #8b5cf6; border-radius: 0.5rem; cursor: pointer; color: #8b5cf6; font-size: 0.75rem;",
                    span { style: "font-size: 1.5rem;", "+" }
                    span { "Generate" }
                }
            }

            // Generation modal
            if *show_generate_modal.read() {
                GenerateAssetModal {
                    entity_type: entity_type.clone(),
                    entity_id: entity_id.clone(),
                    asset_type: selected_asset_type.read().clone(),
                    on_close: move |_| show_generate_modal.set(false),
                    on_generate: move |_| {
                        // TODO: Queue generation
                        show_generate_modal.set(false);
                    },
                }
            }
        }
    }
}

/// Mock asset structure
struct MockAsset {
    id: String,
    asset_type: String,
    label: Option<String>,
    is_active: bool,
}

/// Individual asset thumbnail
#[component]
fn AssetThumbnail(
    id: String,
    label: Option<String>,
    is_active: bool,
    on_activate: EventHandler<String>,
) -> Element {
    let border = if is_active {
        "2px solid #22c55e"
    } else {
        "2px solid transparent"
    };

    rsx! {
        div {
            onclick: {
                let id = id.clone();
                move |_| on_activate.call(id.clone())
            },
            style: format!(
                "width: 64px; height: 64px; background: #1a1a2e; border: {}; border-radius: 0.5rem; cursor: pointer; position: relative; overflow: hidden;",
                border
            ),

            // Placeholder image
            div {
                style: "width: 100%; height: 100%; background: linear-gradient(135deg, #374151 0%, #1f2937 100%);",
            }

            // Active indicator
            if is_active {
                div {
                    style: "position: absolute; top: 2px; right: 2px; width: 8px; height: 8px; background: #22c55e; border-radius: 50%;",
                }
            }

            // Label
            if let Some(label) = &label {
                div {
                    style: "position: absolute; bottom: 0; left: 0; right: 0; padding: 2px; background: rgba(0,0,0,0.7); color: white; font-size: 0.625rem; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{label}"
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
    let mut prompt = use_signal(|| String::new());
    let mut negative_prompt = use_signal(|| String::new());
    let mut count = use_signal(|| 4u8);

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
                        style: "padding: 0.5rem 1rem; background: transparent; color: #9ca3af; border: 1px solid #374151; border-radius: 0.25rem; cursor: pointer;",
                        "Cancel"
                    }
                    button {
                        onclick: {
                            let entity_type = entity_type.clone();
                            let entity_id = entity_id.clone();
                            let asset_type = asset_type.clone();
                            move |_| {
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
                                });
                            }
                        },
                        style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-weight: 500;",
                        "Generate"
                    }
                }
            }
        }
    }
}

/// Request to generate assets
#[derive(Clone)]
pub struct GenerateRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub asset_type: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub count: u8,
}
