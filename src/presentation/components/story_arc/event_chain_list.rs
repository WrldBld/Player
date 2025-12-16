//! Event Chain List - Display all event chains with progress indicators

use dioxus::prelude::*;
use tracing::info;
use crate::application::services::EventChainData;
use crate::presentation::services::use_event_chain_service;

#[derive(Props, Clone, PartialEq)]
pub struct EventChainListProps {
    pub world_id: String,
    #[props(default)]
    pub filter: ChainFilter,
    pub on_select_chain: EventHandler<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ChainFilter {
    #[default]
    All,
    Active,
    Favorites,
}

/// Event chain list component showing all chains with progress
#[component]
pub fn EventChainList(props: EventChainListProps) -> Element {
    let event_chain_service = use_event_chain_service();
    let mut chains: Signal<Vec<EventChainData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut selected_chain: Signal<Option<String>> = use_signal(|| None);

    // Load chains based on filter
    let world_id = props.world_id.clone();
    let filter = props.filter;
    let service_for_effect = event_chain_service.clone();
    use_effect(move || {
        let world_id = world_id.clone();
        let service = service_for_effect.clone();
        let filter_val = filter;
        spawn(async move {
            is_loading.set(true);
            error.set(None);
            let result = match filter_val {
                ChainFilter::All => service.list_chains(&world_id).await,
                ChainFilter::Active => service.list_active(&world_id).await,
                ChainFilter::Favorites => service.list_favorites(&world_id).await,
            };
            match result {
                Ok(loaded) => {
                    chains.set(loaded);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            is_loading.set(false);
        });
    });

    rsx! {
        div {
            class: "event-chain-list",
            style: "display: flex; flex-direction: column; gap: 1rem; height: 100%;",

            // Filter tabs
            div {
                style: "display: flex; gap: 0.5rem; border-bottom: 1px solid #374151;",
                FilterTab {
                    label: "All",
                    is_active: filter == ChainFilter::All,
                    onclick: move |_| {
                        // Filter change handled by parent
                    },
                }
                FilterTab {
                    label: "Active",
                    is_active: filter == ChainFilter::Active,
                    onclick: move |_| {
                        // Filter change handled by parent
                    },
                }
                FilterTab {
                    label: "Favorites",
                    is_active: filter == ChainFilter::Favorites,
                    onclick: move |_| {
                        // Filter change handled by parent
                    },
                }
            }

            // Export/Import buttons (simplified - copy to clipboard for export)
            div {
                style: "display: flex; gap: 0.5rem; margin-bottom: 0.5rem;",
                button {
                    onclick: {
                        let chains_signal = chains;
                        move |_| {
                            let chains_data = chains_signal.read().clone();
                            if let Ok(json) = serde_json::to_string_pretty(&chains_data) {
                                // Copy to clipboard (simplified export)
                                #[cfg(target_arch = "wasm32")]
                                {
                                    // In WASM, we'd use web_sys to copy to clipboard
                                    // For now, just log - can be enhanced later
                                    info!("Export JSON:\n{}", json);
                                }
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    info!("Export JSON:\n{}", json);
                                }
                            }
                        }
                    },
                    style: "padding: 0.375rem 0.75rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                    "📥 Export JSON"
                }
            }

            // Content
            if *is_loading.read() {
                div {
                    style: "color: #6b7280; font-size: 0.875rem; text-align: center; padding: 2rem;",
                    "Loading chains..."
                }
            } else if let Some(err) = error.read().as_ref() {
                div {
                    style: "color: #ef4444; font-size: 0.875rem; padding: 1rem; background: #1f2937; border-radius: 0.375rem;",
                    "Error: {err}"
                }
            } else if chains.read().is_empty() {
                div {
                    style: "color: #6b7280; font-size: 0.875rem; text-align: center; padding: 2rem;",
                    "No event chains found"
                }
            } else {
                div {
                    style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0.75rem;",
                    for chain in chains.read().iter() {
                        EventChainCard {
                            key: "{chain.id}",
                            chain: chain.clone(),
                            is_selected: selected_chain.read().as_ref() == Some(&chain.id),
                            on_select: move |chain_id: String| {
                                selected_chain.set(Some(chain_id.clone()));
                                props.on_select_chain.call(chain_id);
                            },
                            on_toggle_favorite: {
                                let chain_id = chain.id.clone();
                                let service = event_chain_service.clone();
                                let mut chains_signal = chains;
                                move |_| {
                                    let cid = chain_id.clone();
                                    let svc = service.clone();
                                    let mut chains_state = chains_signal;
                                    spawn(async move {
                                        if let Ok(new_favorite) = svc.toggle_favorite(&cid).await {
                                            // Update local state
                                            let mut chains_list = chains_state.write();
                                            if let Some(chain) = chains_list.iter_mut().find(|c| c.id == cid) {
                                                chain.is_favorite = new_favorite;
                                            }
                                        }
                                    });
                                }
                            },
                            on_toggle_active: {
                                let chain_id = chain.id.clone();
                                let service = event_chain_service.clone();
                                let mut chains_signal = chains;
                                move |is_active| {
                                    let cid = chain_id.clone();
                                    let svc = service.clone();
                                    let mut chains_state = chains_signal;
                                    spawn(async move {
                                        if svc.set_active(&cid, is_active).await.is_ok() {
                                            // Update local state
                                            let mut chains_list = chains_state.write();
                                            if let Some(chain) = chains_list.iter_mut().find(|c| c.id == cid) {
                                                chain.is_active = is_active;
                                            }
                                        }
                                    });
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Filter tab component
#[component]
fn FilterTab(
    label: &'static str,
    is_active: bool,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            onclick: move |_| onclick.call(()),
            style: format!(
                "padding: 0.5rem 1rem; background: transparent; border: none; border-bottom: 2px solid {}; color: {}; font-size: 0.875rem; cursor: pointer; transition: all 0.2s;",
                if is_active { "#8b5cf6" } else { "transparent" },
                if is_active { "white" } else { "#9ca3af" }
            ),
            "{label}"
        }
    }
}

/// Individual event chain card
#[component]
fn EventChainCard(
    chain: EventChainData,
    is_selected: bool,
    on_select: EventHandler<String>,
    on_toggle_favorite: EventHandler<()>,
    on_toggle_active: EventHandler<bool>,
) -> Element {
    let progress_color = if chain.is_complete {
        "#22c55e"
    } else if chain.is_active {
        "#3b82f6"
    } else {
        "#6b7280"
    };

    rsx! {
        div {
            onclick: move |_| on_select.call(chain.id.clone()),
            style: format!(
                "padding: 1rem; background: {}; border: 2px solid {}; border-radius: 0.5rem; cursor: pointer; transition: all 0.2s;",
                if is_selected { "#1a1a2e" } else { "#0f0f23" },
                if is_selected { "#8b5cf6" } else { "#374151" }
            ),

            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 0.75rem;",
                div {
                    style: "flex: 1;",
                    h3 {
                        style: "color: white; margin: 0 0 0.25rem 0; font-size: 1rem; font-weight: 500;",
                        "{chain.name}"
                        if chain.is_favorite {
                            span { style: "color: #f59e0b; margin-left: 0.5rem;", "⭐" }
                        }
                    }
                    if !chain.description.is_empty() {
                        p {
                            style: "color: #9ca3af; margin: 0; font-size: 0.875rem;",
                            "{chain.description}"
                        }
                    }
                }
                div {
                    style: "display: flex; gap: 0.5rem;",
                    button {
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_toggle_favorite.call(());
                        },
                        style: format!(
                            "padding: 0.25rem 0.5rem; background: transparent; border: 1px solid #374151; border-radius: 0.25rem; color: {}; cursor: pointer; font-size: 0.75rem;",
                            if chain.is_favorite { "#f59e0b" } else { "#6b7280" }
                        ),
                        "⭐"
                    }
                    button {
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_toggle_active.call(!chain.is_active);
                        },
                        style: format!(
                            "padding: 0.25rem 0.5rem; background: {}; border: none; border-radius: 0.25rem; color: white; cursor: pointer; font-size: 0.75rem;",
                            if chain.is_active { "#22c55e" } else { "#6b7280" }
                        ),
                        if chain.is_active { "Active" } else { "Inactive" }
                    }
                }
            }

            // Progress bar
            div {
                style: "margin-bottom: 0.5rem;",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.25rem;",
                    span {
                        style: "color: #9ca3af; font-size: 0.75rem;",
                        "{chain.completed_events.len()} / {chain.events.len()} events"
                    }
                    span {
                        style: format!("color: {}; font-size: 0.75rem; font-weight: 500;", progress_color),
                        "{chain.progress_percent}%"
                    }
                }
                div {
                    style: "width: 100%; height: 8px; background: #374151; border-radius: 4px; overflow: hidden;",
                    div {
                        style: format!(
                            "width: {}%; height: 100%; background: {}; transition: width 0.3s;",
                            chain.progress_percent,
                            progress_color
                        ),
                    }
                }
            }

            // Tags
            if !chain.tags.is_empty() {
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 0.25rem;",
                    for tag in chain.tags.iter() {
                        span {
                            style: "padding: 0.125rem 0.375rem; background: #374151; color: #9ca3af; border-radius: 0.25rem; font-size: 0.625rem;",
                            "{tag}"
                        }
                    }
                }
            }
        }
    }
}

