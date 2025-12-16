//! Character Panel - Display and manage player character information

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::application::dto::{FieldValue, SheetTemplate};
use crate::application::services::PlayerCharacterData;
use crate::presentation::services::use_world_service;

/// Props for CharacterPanel
#[derive(Props, Clone, PartialEq)]
pub struct CharacterPanelProps {
    pub pc: PlayerCharacterData,
    pub on_edit: EventHandler<()>,
}

/// Character Panel component - Shows PC information
#[component]
pub fn CharacterPanel(props: CharacterPanelProps) -> Element {
    let world_service = use_world_service();
    let mut sheet_template: Signal<Option<SheetTemplate>> = use_signal(|| None);
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

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 1rem; padding: 1rem; background: #1a1a2e; border-radius: 0.5rem;",
            
            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                h3 {
                    style: "margin: 0; color: white; font-size: 1.125rem;",
                    "{props.pc.name}"
                }
                button {
                    onclick: move |_| props.on_edit.call(()),
                    style: "padding: 0.5rem 1rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                    "Edit"
                }
            }

            // Description
            if let Some(desc) = props.pc.description.as_ref() {
                p {
                    style: "margin: 0; color: #9ca3af; font-size: 0.875rem; line-height: 1.5;",
                    "{desc}"
                }
            }

            // Location
            div {
                style: "padding: 0.75rem; background: #0f0f23; border-radius: 0.5rem;",
                div {
                    style: "color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem; text-transform: uppercase;",
                    "Current Location"
                }
                div {
                    style: "color: white; font-size: 0.875rem;",
                    "Location ID: {props.pc.current_location_id}"
                }
            }

            // Character Sheet
            if !loading.read() {
                if let Some(template) = sheet_template.read().as_ref() {
                    if let Some(sheet_data) = props.pc.sheet_data.as_ref() {
                        div {
                            style: "margin-top: 1rem;",
                            h4 {
                                style: "margin: 0 0 0.5rem 0; color: white; font-size: 1rem;",
                                "Character Sheet"
                            }
                            crate::presentation::components::character_sheet_viewer::CharacterSheetViewer {
                                template: template.clone(),
                                values: sheet_data.values.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

