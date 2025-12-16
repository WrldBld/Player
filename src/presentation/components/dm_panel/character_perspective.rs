//! Character Perspective Viewer - DM tool to see any character's perspective

use dioxus::prelude::*;

use crate::application::services::{CharacterService, PlayerCharacterService};
use crate::presentation::services::{use_character_service, use_player_character_service};

/// Props for CharacterPerspectiveViewer
#[derive(Props, Clone, PartialEq)]
pub struct CharacterPerspectiveViewerProps {
    pub session_id: String,
    pub world_id: String,
    pub on_view_as: EventHandler<String>,
}

/// Character Perspective Viewer component
#[component]
pub fn CharacterPerspectiveViewer(props: CharacterPerspectiveViewerProps) -> Element {
    let pc_service = use_player_character_service();
    let character_service = use_character_service();
    let mut pcs: Signal<Vec<crate::application::services::PlayerCharacterData>> = use_signal(Vec::new);
    let mut npcs: Signal<Vec<crate::application::services::CharacterSummary>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Load PCs and NPCs on mount
    {
        let session_id = props.session_id.clone();
        let world_id = props.world_id.clone();
        let pc_svc = pc_service.clone();
        let char_svc = character_service.clone();
        use_effect(move || {
            let sid = session_id.clone();
            let wid = world_id.clone();
            let pc_svc_clone = pc_svc.clone();
            let char_svc_clone = char_svc.clone();
            loading.set(true);
            spawn(async move {
                // Load PCs
                let pc_result = pc_svc_clone.list_pcs(&sid).await;
                
                // Load NPCs
                let npc_result = char_svc_clone.list_characters(&wid).await;

                match (pc_result, npc_result) {
                    (Ok(pc_list), Ok(npc_list)) => {
                        pcs.set(pc_list);
                        npcs.set(npc_list);
                        loading.set(false);
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error.set(Some(format!("Failed to load characters: {}", e)));
                        loading.set(false);
                    }
                }
            });
        });
    }

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 1rem; padding: 1rem; background: #1a1a2e; border-radius: 0.5rem;",
            
            h3 {
                style: "margin: 0; color: white; font-size: 1.125rem;",
                "Character Perspective"
            }

            if let Some(err) = error.read().as_ref() {
                div {
                    style: "padding: 0.75rem; background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; color: #ef4444; font-size: 0.875rem;",
                    "{err}"
                }
            }

            if *loading.read() {
                div {
                    style: "padding: 2rem; text-align: center; color: #9ca3af;",
                    "Loading characters..."
                }
            } else {
                // Player Characters section
                if !pcs.read().is_empty() {
                    div {
                        h4 {
                            style: "margin: 0 0 0.75rem 0; color: #9ca3af; font-size: 0.875rem; text-transform: uppercase;",
                            "Player Characters"
                        }
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            for pc in pcs.read().iter() {
                                CharacterCard {
                                    name: pc.name.clone(),
                                    id: pc.id.clone(),
                                    location_id: pc.current_location_id.clone(),
                                    on_view_as: move |_| props.on_view_as.call(pc.id.clone()),
                                }
                            }
                        }
                    }
                }

                // NPCs section
                if !npcs.read().is_empty() {
                    div {
                        h4 {
                            style: "margin: 1rem 0 0.75rem 0; color: #9ca3af; font-size: 0.875rem; text-transform: uppercase;",
                            "NPCs"
                        }
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            for npc in npcs.read().iter() {
                                CharacterCard {
                                    name: npc.name.clone(),
                                    id: npc.id.clone(),
                                    location_id: "unknown".to_string(),
                                    on_view_as: move |_| props.on_view_as.call(npc.id.clone()),
                                }
                            }
                        }
                    }
                }

                if pcs.read().is_empty() && npcs.read().is_empty() {
                    div {
                        style: "padding: 2rem; text-align: center; color: #9ca3af;",
                        "No characters available"
                    }
                }
            }
        }
    }
}

/// Character Card component
#[derive(Props, Clone, PartialEq)]
struct CharacterCardProps {
    name: String,
    id: String,
    location_id: String,
    on_view_as: EventHandler<()>,
}

#[component]
fn CharacterCard(props: CharacterCardProps) -> Element {
    rsx! {
        div {
            style: "padding: 0.75rem; background: #0f0f23; border-radius: 0.5rem; border: 1px solid #374151; display: flex; justify-content: space-between; align-items: center;",
            
            div {
                div {
                    style: "color: white; font-size: 0.875rem; font-weight: 500;",
                    "{props.name}"
                }
                div {
                    style: "color: #9ca3af; font-size: 0.75rem;",
                    "Location: {props.location_id}"
                }
            }
            button {
                onclick: move |_| props.on_view_as.call(()),
                style: "padding: 0.5rem 1rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                "View as"
            }
        }
    }
}

