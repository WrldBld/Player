//! PC Management Panel - DM view of all player characters

use dioxus::prelude::*;

use crate::application::services::PlayerCharacterData;
use crate::presentation::services::use_player_character_service;

/// Props for PCManagementPanel
#[derive(Props, Clone, PartialEq)]
pub struct PCManagementPanelProps {
    pub session_id: String,
    pub on_view_as_character: EventHandler<String>,
}

/// PC Management Panel component
#[component]
pub fn PCManagementPanel(props: PCManagementPanelProps) -> Element {
    let pc_service = use_player_character_service();
    let mut pcs: Signal<Vec<PlayerCharacterData>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Load PCs on mount
    {
        let session_id = props.session_id.clone();
        let pc_svc = pc_service.clone();
        use_effect(move || {
            let sid = session_id.clone();
            let svc = pc_svc.clone();
            loading.set(true);
            spawn(async move {
                match svc.list_pcs(&sid).await {
                    Ok(pc_list) => {
                        pcs.set(pc_list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load PCs: {}", e)));
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
                "Player Characters"
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
                    "Loading player characters..."
                }
            } else if pcs.read().is_empty() {
                div {
                    style: "padding: 2rem; text-align: center; color: #9ca3af;",
                    "No player characters in this session"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.75rem;",
                    for pc in pcs.read().iter() {
                        PCManagementCard {
                            pc: pc.clone(),
                            on_view_as: move |_| props.on_view_as_character.call(pc.id.clone()),
                        }
                    }
                }
            }
        }
    }
}

/// PC Management Card component
#[derive(Props, Clone, PartialEq)]
struct PCManagementCardProps {
    pc: PlayerCharacterData,
    on_view_as: EventHandler<()>,
}

#[component]
fn PCManagementCard(props: PCManagementCardProps) -> Element {
    rsx! {
        div {
            style: "padding: 1rem; background: #0f0f23; border-radius: 0.5rem; border: 1px solid #374151;",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 0.75rem;",
                div {
                    h4 {
                        style: "margin: 0 0 0.25rem 0; color: white; font-size: 1rem;",
                        "{props.pc.name}"
                    }
                    div {
                        style: "color: #9ca3af; font-size: 0.75rem;",
                        "User: {props.pc.user_id}"
                    }
                }
                button {
                    onclick: move |_| props.on_view_as.call(()),
                    style: "padding: 0.5rem 1rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                    "View as"
                }
            }

            div {
                style: "display: flex; flex-direction: column; gap: 0.5rem;",
                div {
                    style: "display: flex; gap: 1rem;",
                    div {
                        style: "flex: 1;",
                        div {
                            style: "color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;",
                            "Current Location"
                        }
                        div {
                            style: "color: white; font-size: 0.875rem;",
                            "Location ID: {props.pc.current_location_id}"
                        }
                    }
                    div {
                        style: "flex: 1;",
                        div {
                            style: "color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;",
                            "Last Active"
                        }
                        div {
                            style: "color: white; font-size: 0.875rem;",
                            "{props.pc.last_active_at}"
                        }
                    }
                }
            }
        }
    }
}

/// Props for PCLocationsWidget
#[derive(Props, Clone, PartialEq)]
pub struct PCLocationsWidgetProps {
    pub session_id: String,
    pub on_manage: EventHandler<()>,
}

/// PC Locations Widget - Shows PC locations summary in Director View
#[component]
pub fn PCLocationsWidget(props: PCLocationsWidgetProps) -> Element {
    let pc_service = use_player_character_service();
    let mut pcs: Signal<Vec<PlayerCharacterData>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);

    // Load PCs on mount
    {
        let session_id = props.session_id.clone();
        let pc_svc = pc_service.clone();
        use_effect(move || {
            let sid = session_id.clone();
            let svc = pc_svc.clone();
            loading.set(true);
            spawn(async move {
                match svc.list_pcs(&sid).await {
                    Ok(pc_list) => {
                        pcs.set(pc_list);
                        loading.set(false);
                    }
                    Err(_) => {
                        loading.set(false);
                    }
                }
            });
        });
    }

    // Group PCs by location
    let location_counts: Vec<(String, usize)> = {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for pc in pcs.read().iter() {
            *counts.entry(pc.current_location_id.clone()).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    };

    let is_split = location_counts.len() > 1;

    rsx! {
        div {
            class: "panel-section",
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",
            
            h3 {
                style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;",
                "PC Locations"
            }

            if *loading.read() {
                div {
                    style: "color: #9ca3af; font-size: 0.875rem;",
                    "Loading..."
                }
            } else if pcs.read().is_empty() {
                div {
                    style: "color: #9ca3af; font-size: 0.875rem;",
                    "No player characters"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem;",
                    
                    if is_split {
                        div {
                            style: "padding: 0.5rem; background: rgba(245, 158, 11, 0.1); border: 1px solid #f59e0b; border-radius: 0.5rem; color: #f59e0b; font-size: 0.875rem; font-weight: 500;",
                            "⚠️ Party split across {location_counts.len()} locations"
                        }
                    }

                    for (location_id, count) in location_counts.iter() {
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; padding: 0.5rem; background: #0f0f23; border-radius: 0.5rem;",
                            div {
                                style: "color: white; font-size: 0.875rem;",
                                "Location: {location_id}"
                            }
                            div {
                                style: "color: #9ca3af; font-size: 0.875rem;",
                                "{count} PC{if *count > 1 { "s" } else { "" }}"
                            }
                        }
                    }

                    button {
                        onclick: move |_| props.on_manage.call(()),
                        style: "margin-top: 0.5rem; padding: 0.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                        "Manage PCs"
                    }
                }
            }
        }
    }
}
