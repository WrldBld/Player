//! Character Form - Create and edit characters

use dioxus::prelude::*;

use super::asset_gallery::AssetGallery;
use super::suggestion_button::{SuggestionButton, SuggestionContext, SuggestionType};

/// Character archetypes
const ARCHETYPES: &[&str] = &[
    "Hero",
    "Mentor",
    "Threshold Guardian",
    "Herald",
    "Shapeshifter",
    "Shadow",
    "Ally",
    "Trickster",
];

/// Character form for creating/editing characters
#[component]
pub fn CharacterForm(character_id: String, on_close: EventHandler<()>) -> Element {
    let is_new = character_id.is_empty();

    // Form state
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut archetype = use_signal(|| "Hero".to_string());
    let mut wants = use_signal(|| String::new());
    let mut fears = use_signal(|| String::new());
    let mut backstory = use_signal(|| String::new());

    // TODO: Load character data if editing existing character
    // useEffect to fetch character by ID

    rsx! {
        div {
            class: "character-form",
            style: "display: flex; flex-direction: column; height: 100%; background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

            // Header
            div {
                class: "form-header",
                style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-bottom: 1px solid #374151;",

                h2 {
                    style: "color: white; margin: 0; font-size: 1.25rem;",
                    if is_new { "New Character" } else { "Edit Character" }
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
                                placeholder: "Enter character name...",
                                style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                            }
                            SuggestionButton {
                                suggestion_type: SuggestionType::CharacterName,
                                context: SuggestionContext {
                                    hints: Some(archetype.read().clone()),
                                    ..Default::default()
                                },
                                on_select: move |value| name.set(value),
                            }
                        }
                    }
                }

                // Archetype dropdown
                FormField {
                    label: "Archetype",
                    required: false,
                    children: rsx! {
                        select {
                            value: "{archetype}",
                            onchange: move |e| archetype.set(e.value()),
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",

                            for arch in ARCHETYPES {
                                option { value: "{arch}", "{arch}" }
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
                                placeholder: "Physical appearance, mannerisms, voice...",
                                style: "width: 100%; min-height: 80px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                            }
                            div { style: "display: flex; justify-content: flex-end;",
                                SuggestionButton {
                                    suggestion_type: SuggestionType::CharacterDescription,
                                    context: SuggestionContext {
                                        entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                        hints: Some(archetype.read().clone()),
                                        ..Default::default()
                                    },
                                    on_select: move |value| description.set(value),
                                }
                            }
                        }
                    }
                }

                // Wants field
                FormField {
                    label: "Wants",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; gap: 0.5rem;",
                            input {
                                r#type: "text",
                                value: "{wants}",
                                oninput: move |e| wants.set(e.value()),
                                placeholder: "What does this character desire?",
                                style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                            }
                            SuggestionButton {
                                suggestion_type: SuggestionType::CharacterWants,
                                context: SuggestionContext {
                                    entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                    hints: Some(archetype.read().clone()),
                                    additional_context: if description.read().is_empty() { None } else { Some(description.read().clone()) },
                                    ..Default::default()
                                },
                                on_select: move |value| wants.set(value),
                            }
                        }
                    }
                }

                // Fears field
                FormField {
                    label: "Fears",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; gap: 0.5rem;",
                            input {
                                r#type: "text",
                                value: "{fears}",
                                oninput: move |e| fears.set(e.value()),
                                placeholder: "What does this character fear?",
                                style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                            }
                            SuggestionButton {
                                suggestion_type: SuggestionType::CharacterFears,
                                context: SuggestionContext {
                                    entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                    hints: Some(archetype.read().clone()),
                                    additional_context: if wants.read().is_empty() { None } else { Some(wants.read().clone()) },
                                    ..Default::default()
                                },
                                on_select: move |value| fears.set(value),
                            }
                        }
                    }
                }

                // Backstory field
                FormField {
                    label: "Backstory",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            textarea {
                                value: "{backstory}",
                                oninput: move |e| backstory.set(e.value()),
                                placeholder: "Background, history, key events...",
                                style: "width: 100%; min-height: 100px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                            }
                            div { style: "display: flex; justify-content: flex-end;",
                                SuggestionButton {
                                    suggestion_type: SuggestionType::CharacterBackstory,
                                    context: SuggestionContext {
                                        entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                        hints: Some(archetype.read().clone()),
                                        additional_context: if wants.read().is_empty() { None } else { Some(wants.read().clone()) },
                                        world_setting: if fears.read().is_empty() { None } else { Some(fears.read().clone()) },
                                        ..Default::default()
                                    },
                                    on_select: move |value| backstory.set(value),
                                }
                            }
                        }
                    }
                }

                // Asset Gallery section
                div {
                    class: "assets-section",
                    style: "margin-top: 1rem;",

                    h3 { style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin-bottom: 0.75rem;", "Assets" }

                    AssetGallery {
                        entity_type: "character".to_string(),
                        entity_id: character_id.clone(),
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

