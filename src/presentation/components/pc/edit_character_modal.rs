//! Edit Character Modal - Edit player character information

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::application::dto::{FieldValue, SheetTemplate};
use crate::application::services::{PlayerCharacterData, UpdatePlayerCharacterRequest};
use crate::application::services::player_character_service::CharacterSheetDataApi;
use crate::presentation::services::{use_player_character_service, use_world_service};

/// Props for EditCharacterModal
#[derive(Props, Clone, PartialEq)]
pub struct EditCharacterModalProps {
    pub pc: PlayerCharacterData,
    pub on_close: EventHandler<()>,
    pub on_saved: EventHandler<PlayerCharacterData>,
}

/// Edit Character Modal component
#[component]
pub fn EditCharacterModal(props: EditCharacterModalProps) -> Element {
    let pc_service = use_player_character_service();
    let world_service = use_world_service();

    // Form state
    let mut name = use_signal(|| props.pc.name.clone());
    let mut description = use_signal(|| props.pc.description.clone().unwrap_or_default());
    let mut sheet_template: Signal<Option<SheetTemplate>> = use_signal(|| None);
    let mut sheet_values: Signal<HashMap<String, FieldValue>> = use_signal(|| {
        props.pc.sheet_data.as_ref()
            .map(|s| s.values.clone())
            .unwrap_or_default()
    });
    let mut is_saving = use_signal(|| false);
    let mut error_message: Signal<Option<String>> = use_signal(|| None);
    let mut loading = use_signal(|| true);

    // Load sheet template
    {
        let world_id = props.pc.world_id.clone();
        let world_svc = world_service.clone();
        use_effect(move || {
            let svc = world_svc.clone();
            let world_id_clone = world_id.clone();
            spawn(async move {
                match svc.get_sheet_template(&world_id_clone).await {
                    Ok(template_json) => {
                        match serde_json::from_value::<SheetTemplate>(template_json) {
                            Ok(template) => {
                                sheet_template.set(Some(template));
                            }
                            Err(_) => {}
                        }
                    }
                    Err(_) => {}
                }
                loading.set(false);
            });
        });
    }

    let save = move |_| {
        let name_val = name.read().clone();
        let desc_val = description.read().clone();
        let sheet_vals = sheet_values.read().clone();
        let pc_id = props.pc.id.clone();
        let pc_svc = pc_service.clone();
        let on_saved_handler = props.on_saved.clone();
        let on_close_handler = props.on_close.clone();

        if name_val.trim().is_empty() {
            error_message.set(Some("Character name is required".to_string()));
            return;
        }

        is_saving.set(true);
        error_message.set(None);

        spawn(async move {
            let sheet_data = if sheet_vals.is_empty() {
                None
            } else {
                Some(CharacterSheetDataApi { values: sheet_vals })
            };

            let request = UpdatePlayerCharacterRequest {
                name: Some(name_val),
                description: if desc_val.trim().is_empty() {
                    None
                } else {
                    Some(desc_val)
                },
                sheet_data,
                sprite_asset: None,
                portrait_asset: None,
            };

            match pc_svc.update_pc(&pc_id, &request).await {
                Ok(updated_pc) => {
                    on_saved_handler.call(updated_pc);
                    on_close_handler.call(());
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to update character: {}", e)));
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.75); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| {
                props.on_close.call(());
            },
            div {
                style: "background: #1a1a2e; border-radius: 0.5rem; width: 90%; max-width: 800px; max-height: 90vh; overflow-y: auto; display: flex; flex-direction: column;",
                onclick: |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 1.5rem; border-bottom: 1px solid #374151;",
                    h2 {
                        style: "margin: 0; color: white; font-size: 1.25rem;",
                        "Edit Character"
                    }
                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "padding: 0.25rem 0.5rem; background: transparent; color: #9ca3af; border: none; cursor: pointer; font-size: 1.25rem;",
                        "×"
                    }
                }

                // Error message
                if let Some(err) = error_message.read().as_ref() {
                    div {
                        style: "padding: 0.75rem 1.5rem; background: rgba(239, 68, 68, 0.1); border-bottom: 1px solid rgba(239, 68, 68, 0.3); color: #ef4444; font-size: 0.875rem;",
                        "{err}"
                    }
                }

                // Content
                div {
                    style: "padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem;",
                    
                    // Name
                    div {
                        label {
                            style: "display: block; margin-bottom: 0.5rem; color: #9ca3af; font-size: 0.875rem; font-weight: 500;",
                            "Character Name *"
                        }
                        input {
                            r#type: "text",
                            value: "{name.read()}",
                            oninput: move |e| name.set(e.value()),
                            placeholder: "Enter character name",
                            style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 1rem;",
                        }
                    }

                    // Description
                    div {
                        label {
                            style: "display: block; margin-bottom: 0.5rem; color: #9ca3af; font-size: 0.875rem; font-weight: 500;",
                            "Description"
                        }
                        textarea {
                            value: "{description.read()}",
                            oninput: move |e| description.set(e.value()),
                            placeholder: "Describe your character...",
                            rows: 4,
                            style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 1rem; resize: vertical;",
                        }
                    }

                    // Character Sheet
                    if !*loading.read() {
                        if let Some(template) = sheet_template.read().as_ref() {
                            div {
                                h3 {
                                    style: "margin: 0 0 1rem 0; color: white; font-size: 1rem;",
                                    "Character Sheet"
                                }
                                crate::presentation::components::creator::sheet_field_input::CharacterSheetForm {
                                    template: template.clone(),
                                    values: sheet_values.read().clone(),
                                    on_values_change: move |v| sheet_values.set(v),
                                }
                            }
                        }
                    }
                }

                // Footer
                div {
                    style: "padding: 1rem 1.5rem; border-top: 1px solid #374151; display: flex; justify-content: flex-end; gap: 0.75rem;",
                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                        "Cancel"
                    }
                    button {
                        onclick: save,
                        disabled: *is_saving.read(),
                        style: "padding: 0.5rem 1.5rem; background: #22c55e; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 500;",
                        if *is_saving.read() {
                            "Saving..."
                        } else {
                            "Save Changes"
                        }
                    }
                }
            }
        }
    }
}

