//! Character Form - Create and edit characters

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::asset_gallery::AssetGallery;
use super::sheet_field_input::CharacterSheetForm;
use super::suggestion_button::{SuggestionButton, SuggestionContext, SuggestionType};
use crate::application::dto::{FieldValue, SheetTemplate};
// TODO Phase 7.4: Replace HttpClient with service calls
use crate::infrastructure::http_client::HttpClient;
use crate::presentation::state::GameState;

/// Character data structure for API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterData {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub archetype: Option<String>,
    pub wants: Option<String>,
    pub fears: Option<String>,
    pub backstory: Option<String>,
    #[serde(default)]
    pub sheet_data: Option<CharacterSheetDataApi>,
}

/// Character sheet data from API
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CharacterSheetDataApi {
    #[serde(default)]
    pub values: HashMap<String, FieldValue>,
}

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
    let game_state = use_context::<GameState>();

    // Form state
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut archetype = use_signal(|| "Hero".to_string());
    let mut wants = use_signal(|| String::new());
    let mut fears = use_signal(|| String::new());
    let mut backstory = use_signal(|| String::new());
    let mut is_loading = use_signal(|| !is_new);
    let mut is_saving = use_signal(|| false);
    let mut success_message: Signal<Option<String>> = use_signal(|| None);
    let mut error_message: Signal<Option<String>> = use_signal(|| None);

    // Sheet template state
    let mut sheet_template: Signal<Option<SheetTemplate>> = use_signal(|| None);
    let mut sheet_values: Signal<HashMap<String, FieldValue>> = use_signal(HashMap::new);
    let mut show_sheet_section = use_signal(|| true);

    // Load sheet template on mount
    {
        use_effect(move || {
            spawn(async move {
                let world_id = game_state.world.read().as_ref().map(|w| w.world.id.clone());

                if let Some(world_id) = world_id {
                    match fetch_sheet_template(&world_id).await {
                        Ok(template) => {
                            sheet_template.set(Some(template));
                        }
                        Err(_e) => {
                            // Template fetch failure is not critical - sheet section just won't appear
                            #[cfg(target_arch = "wasm32")]
                            web_sys::console::log_1(&format!("Failed to load sheet template: {}", _e).into());
                        }
                    }
                }
            });
        });
    }

    // Load character data if editing existing character
    {
        let char_id_for_effect = character_id.clone();
        use_effect(move || {
            let char_id = char_id_for_effect.clone();
            if !char_id.is_empty() {
                spawn(async move {
                    let world_id = game_state.world.read().as_ref().map(|w| w.world.id.clone());

                    if let Some(world_id) = world_id {
                        match fetch_character(&world_id, &char_id).await {
                            Ok(char_data) => {
                                name.set(char_data.name);
                                description.set(char_data.description.unwrap_or_default());
                                archetype.set(char_data.archetype.unwrap_or_else(|| "Hero".to_string()));
                                wants.set(char_data.wants.unwrap_or_default());
                                fears.set(char_data.fears.unwrap_or_default());
                                backstory.set(char_data.backstory.unwrap_or_default());
                                // Load sheet values if present
                                if let Some(data) = char_data.sheet_data {
                                    sheet_values.set(data.values);
                                }
                                is_loading.set(false);
                            }
                            Err(e) => {
                                error_message.set(Some(format!("Failed to load character: {}", e)));
                                is_loading.set(false);
                            }
                        }
                    } else {
                        error_message.set(Some("No world loaded".to_string()));
                        is_loading.set(false);
                    }
                });
            }
        });
    }

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

            // Error/Success messages
            if let Some(msg) = error_message.read().as_ref() {
                div {
                    style: "padding: 0.75rem 1rem; background: rgba(239, 68, 68, 0.1); border-bottom: 1px solid rgba(239, 68, 68, 0.3); color: #ef4444; font-size: 0.875rem;",
                    "{msg}"
                }
            }
            if let Some(msg) = success_message.read().as_ref() {
                div {
                    style: "padding: 0.75rem 1rem; background: rgba(34, 197, 94, 0.1); border-bottom: 1px solid rgba(34, 197, 94, 0.3); color: #22c55e; font-size: 0.875rem;",
                    "{msg}"
                }
            }

            // Form content (scrollable)
            div {
                class: "form-content",
                style: "flex: 1; overflow-y: auto; padding: 1rem; display: flex; flex-direction: column; gap: 1rem;",

                if *is_loading.read() {
                    div {
                        style: "display: flex; align-items: center; justify-content: center; padding: 2rem; color: #6b7280;",
                        "Loading character data..."
                    }
                } else {

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

                    // Character Sheet section (if template available)
                    if let Some(template) = sheet_template.read().as_ref() {
                        div {
                            class: "sheet-section",
                            style: "margin-top: 1.5rem; border-top: 1px solid #374151; padding-top: 1rem;",

                            // Section header with collapse toggle
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; cursor: pointer;",
                                onclick: move |_| {
                                    let current = *show_sheet_section.read();
                                    show_sheet_section.set(!current);
                                },

                                h3 {
                                    style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin: 0;",
                                    "Character Sheet ({template.name})"
                                }

                                span {
                                    style: "color: #6b7280; font-size: 0.875rem;",
                                    if *show_sheet_section.read() { "[-]" } else { "[+]" }
                                }
                            }

                            if *show_sheet_section.read() {
                                CharacterSheetForm {
                                    template: template.clone(),
                                    values: sheet_values.read().clone(),
                                    on_change: move |(field_id, value)| {
                                        sheet_values.write().insert(field_id, value);
                                    },
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
            }

            // Footer with action buttons
            div {
                class: "form-footer",
                style: "display: flex; justify-content: flex-end; gap: 0.5rem; padding: 1rem; border-top: 1px solid #374151;",

                button {
                    onclick: move |_| on_close.call(()),
                    style: "padding: 0.5rem 1rem; background: transparent; color: #9ca3af; border: 1px solid #374151; border-radius: 0.25rem; cursor: pointer;",
                    disabled: *is_saving.read(),
                    "Cancel"
                }

                button {
                    style: format!(
                        "padding: 0.5rem 1rem; background: #22c55e; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-weight: 500; opacity: {};",
                        if *is_saving.read() { "0.6" } else { "1" }
                    ),
                    disabled: *is_saving.read(),
                    onclick: move |_| {
                        let char_name = name.read().clone();
                        if char_name.is_empty() {
                            error_message.set(Some("Character name is required".to_string()));
                            return;
                        }

                        error_message.set(None);
                        success_message.set(None);
                        is_saving.set(true);

                        let char_id = character_id.clone();
                        let on_close = on_close.clone();

                        spawn(async move {
                            let world_id = game_state.world.read().as_ref().map(|w| w.world.id.clone());

                            if let Some(world_id) = world_id {
                                // Get sheet values
                                let sheet_data_to_save = {
                                    let values = sheet_values.read().clone();
                                    if values.is_empty() {
                                        None
                                    } else {
                                        Some(CharacterSheetDataApi { values })
                                    }
                                };

                                let char_data = CharacterData {
                                    id: if is_new { None } else { Some(char_id.clone()) },
                                    name: name.read().clone(),
                                    description: {
                                        let desc = description.read().clone();
                                        if desc.is_empty() { None } else { Some(desc) }
                                    },
                                    archetype: {
                                        let arch = archetype.read().clone();
                                        if arch.is_empty() { None } else { Some(arch) }
                                    },
                                    wants: {
                                        let w = wants.read().clone();
                                        if w.is_empty() { None } else { Some(w) }
                                    },
                                    fears: {
                                        let f = fears.read().clone();
                                        if f.is_empty() { None } else { Some(f) }
                                    },
                                    backstory: {
                                        let b = backstory.read().clone();
                                        if b.is_empty() { None } else { Some(b) }
                                    },
                                    sheet_data: sheet_data_to_save,
                                };

                                match if is_new {
                                    save_character(&world_id, char_data).await
                                } else {
                                    update_character(&world_id, &char_id, char_data).await
                                } {
                                    Ok(_) => {
                                        success_message.set(Some(if is_new {
                                            "Character created successfully".to_string()
                                        } else {
                                            "Character saved successfully".to_string()
                                        }));
                                        is_saving.set(false);
                                        // Close form - let the user see the success message
                                        on_close.call(());
                                    }
                                    Err(e) => {
                                        error_message.set(Some(format!("Save failed: {}", e)));
                                        is_saving.set(false);
                                    }
                                }
                            } else {
                                error_message.set(Some("No world loaded".to_string()));
                                is_saving.set(false);
                            }
                        });
                    },
                    if *is_saving.read() { "Saving..." } else { if is_new { "Create" } else { "Save" } }
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

/// Fetch a single character from the API
async fn fetch_character(world_id: &str, character_id: &str) -> Result<CharacterData, String> {
    let path = format!("/api/worlds/{}/characters/{}", world_id, character_id);
    HttpClient::get(&path).await.map_err(|e| e.to_string())
}

/// Save a new character via the API
async fn save_character(world_id: &str, character: CharacterData) -> Result<CharacterData, String> {
    let path = format!("/api/worlds/{}/characters", world_id);
    HttpClient::post(&path, &character).await.map_err(|e| e.to_string())
}

/// Update an existing character via the API
async fn update_character(_world_id: &str, character_id: &str, character: CharacterData) -> Result<CharacterData, String> {
    let path = format!("/api/characters/{}", character_id);
    HttpClient::put(&path, &character).await.map_err(|e| e.to_string())
}

/// Fetch the sheet template for a world
async fn fetch_sheet_template(world_id: &str) -> Result<SheetTemplate, String> {
    let path = format!("/api/worlds/{}/sheet-template", world_id);
    HttpClient::get(&path).await.map_err(|e| e.to_string())
}
