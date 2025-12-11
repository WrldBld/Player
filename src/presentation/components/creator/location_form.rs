//! Location Form - Create and edit locations

use dioxus::prelude::*;

use super::asset_gallery::AssetGallery;
use super::suggestion_button::{SuggestionButton, SuggestionContext, SuggestionType};

/// Location types
const LOCATION_TYPES: &[&str] = &[
    "Interior",
    "Exterior",
    "Wilderness",
    "Urban",
    "Dungeon",
    "Castle",
    "Village",
    "City",
    "Forest",
    "Mountain",
    "Cave",
    "Temple",
];

/// Location form for creating/editing locations
#[component]
pub fn LocationForm(location_id: String, on_close: EventHandler<()>) -> Element {
    let is_new = location_id.is_empty();

    // Form state
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut location_type = use_signal(|| "Interior".to_string());
    let mut atmosphere = use_signal(|| String::new());
    let mut notable_features = use_signal(|| String::new());
    let mut hidden_secrets = use_signal(|| String::new());

    // TODO: Load location data if editing existing location

    rsx! {
        div {
            class: "location-form",
            style: "display: flex; flex-direction: column; height: 100%; background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

            // Header
            div {
                class: "form-header",
                style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-bottom: 1px solid #374151;",

                h2 {
                    style: "color: white; margin: 0; font-size: 1.25rem;",
                    if is_new { "New Location" } else { "Edit Location" }
                }

                button {
                    onclick: move |_| on_close.call(()),
                    style: "padding: 0.25rem 0.5rem; background: transparent; color: #9ca3af; border: none; cursor: pointer; font-size: 1.25rem;",
                    "×"
                }
            }

            // Form content (scrollable)
            div {
                class: "form-content",
                style: "flex: 1; overflow-y: auto; padding: 1rem; display: flex; flex-direction: column; gap: 1rem;",

                // Name field with suggest button
                FormField {
                    label: "Name",
                    required: true,
                    children: rsx! {
                        div { style: "display: flex; gap: 0.5rem;",
                            input {
                                r#type: "text",
                                value: "{name}",
                                oninput: move |e| name.set(e.value()),
                                placeholder: "Enter location name...",
                                style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                            }
                            SuggestionButton {
                                suggestion_type: SuggestionType::LocationName,
                                context: SuggestionContext {
                                    entity_type: Some(location_type.read().clone()),
                                    ..Default::default()
                                },
                                on_select: move |value| name.set(value),
                            }
                        }
                    }
                }

                // Location type dropdown
                FormField {
                    label: "Type",
                    required: false,
                    children: rsx! {
                        select {
                            value: "{location_type}",
                            onchange: move |e| location_type.set(e.value()),
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",

                            for lt in LOCATION_TYPES {
                                option { value: "{lt}", "{lt}" }
                            }
                        }
                    }
                }

                // Description field
                FormField {
                    label: "Description",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            textarea {
                                value: "{description}",
                                oninput: move |e| description.set(e.value()),
                                placeholder: "What does this place look like? What stands out?",
                                style: "width: 100%; min-height: 80px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                            }
                            div { style: "display: flex; justify-content: flex-end;",
                                SuggestionButton {
                                    suggestion_type: SuggestionType::LocationDescription,
                                    context: SuggestionContext {
                                        entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                        entity_type: Some(location_type.read().clone()),
                                        ..Default::default()
                                    },
                                    on_select: move |value| description.set(value),
                                }
                            }
                        }
                    }
                }

                // Atmosphere field
                FormField {
                    label: "Atmosphere",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; gap: 0.5rem;",
                            input {
                                r#type: "text",
                                value: "{atmosphere}",
                                oninput: move |e| atmosphere.set(e.value()),
                                placeholder: "The mood and feeling of this place...",
                                style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                            }
                            SuggestionButton {
                                suggestion_type: SuggestionType::LocationAtmosphere,
                                context: SuggestionContext {
                                    entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                    entity_type: Some(location_type.read().clone()),
                                    additional_context: if description.read().is_empty() { None } else { Some(description.read().clone()) },
                                    ..Default::default()
                                },
                                on_select: move |value| atmosphere.set(value),
                            }
                        }
                    }
                }

                // Notable features field
                FormField {
                    label: "Notable Features",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            textarea {
                                value: "{notable_features}",
                                oninput: move |e| notable_features.set(e.value()),
                                placeholder: "Points of interest, interactable objects...",
                                style: "width: 100%; min-height: 60px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                            }
                            div { style: "display: flex; justify-content: flex-end;",
                                SuggestionButton {
                                    suggestion_type: SuggestionType::LocationFeatures,
                                    context: SuggestionContext {
                                        entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                        entity_type: Some(location_type.read().clone()),
                                        hints: if atmosphere.read().is_empty() { None } else { Some(atmosphere.read().clone()) },
                                        ..Default::default()
                                    },
                                    on_select: move |value| notable_features.set(value),
                                }
                            }
                        }
                    }
                }

                // Hidden secrets field
                FormField {
                    label: "Hidden Secrets",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            textarea {
                                value: "{hidden_secrets}",
                                oninput: move |e| hidden_secrets.set(e.value()),
                                placeholder: "Things players might discover with investigation...",
                                style: "width: 100%; min-height: 60px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                            }
                            div { style: "display: flex; justify-content: flex-end;",
                                SuggestionButton {
                                    suggestion_type: SuggestionType::LocationSecrets,
                                    context: SuggestionContext {
                                        entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                        entity_type: Some(location_type.read().clone()),
                                        additional_context: if notable_features.read().is_empty() { None } else { Some(notable_features.read().clone()) },
                                        ..Default::default()
                                    },
                                    on_select: move |value| hidden_secrets.set(value),
                                }
                            }
                        }
                    }
                }

                // Connections section
                div {
                    class: "connections-section",
                    style: "margin-top: 0.5rem;",

                    h3 { style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin-bottom: 0.5rem;", "Connected Locations" }

                    div { style: "padding: 1rem; background: #0f0f23; border-radius: 0.25rem; text-align: center; color: #6b7280; font-size: 0.875rem;",
                        "No connections yet"
                        button {
                            style: "display: block; margin: 0.5rem auto 0; padding: 0.25rem 0.75rem; background: #3b82f6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "+ Add Connection"
                        }
                    }
                }

                // Asset Gallery section
                div {
                    class: "assets-section",
                    style: "margin-top: 1rem;",

                    h3 { style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin-bottom: 0.75rem;", "Assets" }

                    AssetGallery {
                        entity_type: "location".to_string(),
                        entity_id: location_id.clone(),
                    }
                }
            }

            // Footer with action buttons
            div {
                class: "form-footer",
                style: "display: flex; justify-content: flex-end; gap: 0.5rem; padding: 1rem; border-top: 1px solid #374151;",

                button {
                    onclick: move |_| on_close.call(()),
                    style: "padding: 0.5rem 1rem; background: transparent; color: #9ca3af; border: 1px solid #374151; border-radius: 0.25rem; cursor: pointer;",
                    "Cancel"
                }

                button {
                    style: "padding: 0.5rem 1rem; background: #22c55e; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-weight: 500;",
                    if is_new { "Create" } else { "Save" }
                }
            }
        }
    }
}

/// Reusable form field wrapper
#[component]
fn FormField(label: &'static str, required: bool, children: Element) -> Element {
    rsx! {
        div {
            class: "form-field",
            style: "display: flex; flex-direction: column; gap: 0.25rem;",

            label {
                style: "color: #9ca3af; font-size: 0.875rem;",
                "{label}"
                if required {
                    span { style: "color: #ef4444; margin-left: 0.25rem;", "*" }
                }
            }

            {children}
        }
    }
}
