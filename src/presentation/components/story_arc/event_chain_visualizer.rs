//! Event Chain Visualizer - Flowchart view of event chains

use dioxus::prelude::*;
use crate::application::services::EventChainData;
use crate::application::dto::NarrativeEventData;
use crate::presentation::services::use_event_chain_service;

#[derive(Props, Clone, PartialEq)]
pub struct EventChainVisualizerProps {
    pub chain: EventChainData,
    pub world_id: String,
    pub on_select_event: EventHandler<String>,
}

/// Event chain visualizer component showing flowchart of events
#[component]
pub fn EventChainVisualizer(props: EventChainVisualizerProps) -> Element {
    let mut events: Signal<Vec<NarrativeEventData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| false);
    let mut zoom_level: Signal<f32> = use_signal(|| 1.0);
    let mut pan_x: Signal<f32> = use_signal(|| 0.0);
    let mut pan_y: Signal<f32> = use_signal(|| 0.0);

    // Initialize with event placeholders based on the chain's event IDs
    {
        let chain_events = props.chain.events.clone();
        use_effect(move || {
            let placeholder_events: Vec<NarrativeEventData> = chain_events.iter()
                .map(|id| NarrativeEventData {
                    id: id.clone(),
                    world_id: String::new(),
                    name: format!("Event: {}", id),
                    description: String::new(),
                    scene_direction: String::new(),
                    suggested_opening: None,
                    trigger_count: 0,
                    is_active: false,
                    is_triggered: false,
                    triggered_at: None,
                    selected_outcome: None,
                    is_repeatable: false,
                    delay_turns: 0,
                    expires_after_turns: None,
                    priority: 0,
                    is_favorite: false,
                    tags: Vec::new(),
                    scene_id: None,
                    location_id: None,
                    act_id: None,
                    chain_id: Some(props.chain.id.clone()),
                    chain_position: None,
                    outcome_count: 0,
                    trigger_condition_count: 0,
                    created_at: String::new(),
                    updated_at: String::new(),
                })
                .collect();
            events.set(placeholder_events);
        });
    }

    let events_list = events.read().clone();

    rsx! {
        div {
            class: "event-chain-visualizer",
            style: "position: relative; width: 100%; height: 100%; overflow: hidden; background: #0f0f23; border-radius: 0.5rem;",

            // Controls
            div {
                style: "position: absolute; top: 1rem; right: 1rem; z-index: 10; display: flex; gap: 0.5rem;",
                button {
                    onclick: move |_| {
                        let current = *zoom_level.read();
                        zoom_level.set((current * 1.2).min(2.0));
                    },
                    style: "padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; cursor: pointer;",
                    "+"
                }
                button {
                    onclick: move |_| {
                        let current = *zoom_level.read();
                        zoom_level.set((current / 1.2).max(0.5));
                    },
                    style: "padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; cursor: pointer;",
                    "-"
                }
                button {
                    onclick: move |_| {
                        zoom_level.set(1.0);
                        pan_x.set(0.0);
                        pan_y.set(0.0);
                    },
                    style: "padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; cursor: pointer;",
                    "Reset"
                }
            }

            // Flowchart canvas
            div {
                style: format!(
                    "position: relative; width: 100%; height: 100%; transform: translate({}px, {}px) scale({}); transform-origin: 0 0;",
                    *pan_x.read(),
                    *pan_y.read(),
                    *zoom_level.read()
                ),
                if *is_loading.read() {
                    div {
                        style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); color: #6b7280;",
                        "Loading events..."
                    }
                } else {
                    div {
                        style: "display: flex; flex-direction: column; align-items: center; gap: 2rem; padding: 2rem; min-width: fit-content;",
                        for (index, event) in events_list.iter().enumerate() {
                            div {
                                key: "{event.id}",
                                style: "display: flex; flex-direction: column; align-items: center; gap: 1rem;",
                                // Connection line (except for first event)
                                if index > 0 {
                                    div {
                                        style: "width: 2px; height: 2rem; background: #374151;",
                                    }
                                }
                                // Event node
                                EventNode {
                                    event: event.clone(),
                                    is_completed: props.chain.completed_events.contains(&event.id),
                                    is_current: props.chain.current_position as usize == index,
                                    on_click: move |event_id| props.on_select_event.call(event_id),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Individual event node in the flowchart
#[component]
fn EventNode(
    event: NarrativeEventData,
    is_completed: bool,
    is_current: bool,
    on_click: EventHandler<String>,
) -> Element {
    let bg_color = if is_completed {
        "#22c55e"
    } else if is_current {
        "#3b82f6"
    } else {
        "#374151"
    };

    let border_color = if is_current {
        "#8b5cf6"
    } else {
        "#6b7280"
    };

    rsx! {
        div {
            onclick: move |_| on_click.call(event.id.clone()),
            style: format!(
                "width: 200px; padding: 1rem; background: {}; border: 2px solid {}; border-radius: 0.5rem; cursor: pointer; transition: all 0.2s;",
                bg_color,
                border_color
            ),
            onmouseenter: move |_| {
                // Could add hover effect
            },
            h4 {
                style: "color: white; margin: 0 0 0.5rem 0; font-size: 0.875rem; font-weight: 500; text-align: center;",
                "{event.name}"
            }
            if !event.description.is_empty() {
                p {
                    style: "color: rgba(255, 255, 255, 0.7); margin: 0; font-size: 0.75rem; text-align: center; overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;",
                    "{event.description}"
                }
            }
            div {
                style: "display: flex; justify-content: center; gap: 0.5rem; margin-top: 0.5rem;",
                if is_completed {
                    span { style: "color: white; font-size: 0.75rem;", "✅" }
                }
                if is_current {
                    span { style: "color: white; font-size: 0.75rem;", "📍" }
                }
            }
        }
    }
}

