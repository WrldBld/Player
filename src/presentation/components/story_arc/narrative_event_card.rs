//! Narrative Event Card - Display a single narrative event

use dioxus::prelude::*;

use crate::infrastructure::asset_loader::NarrativeEventData;

#[derive(Props, Clone)]
pub struct NarrativeEventCardProps {
    pub event: NarrativeEventData,
    pub on_click: EventHandler<()>,
    pub on_toggle_favorite: EventHandler<()>,
    pub on_toggle_active: EventHandler<()>,
}

impl PartialEq for NarrativeEventCardProps {
    fn eq(&self, other: &Self) -> bool {
        self.event.id == other.event.id
            && self.event.is_favorite == other.event.is_favorite
            && self.event.is_active == other.event.is_active
            && self.event.is_triggered == other.event.is_triggered
    }
}

#[component]
pub fn NarrativeEventCard(props: NarrativeEventCardProps) -> Element {
    let event = &props.event;

    // Determine status color and label
    let (status_label, status_color) = if event.is_triggered {
        ("Triggered", "#22c55e")
    } else if event.is_active {
        ("Active", "#3b82f6")
    } else {
        ("Inactive", "#6b7280")
    };

    // Priority indicator
    let priority_color = match event.priority {
        p if p >= 8 => "#ef4444",
        p if p >= 5 => "#f59e0b",
        p if p >= 3 => "#3b82f6",
        _ => "#6b7280",
    };

    rsx! {
        div {
            class: "narrative-event-card",
            style: format!(
                "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem; cursor: pointer; transition: all 0.2s; border: 1px solid {}; opacity: {};",
                if event.is_active { "#374151" } else { "#1f2937" },
                if event.is_active { "1" } else { "0.7" }
            ),
            onclick: move |_| props.on_click.call(()),

            // Header row
            div {
                style: "display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.75rem;",

                // Title and priority
                div {
                    style: "flex: 1; min-width: 0;",

                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.25rem;",

                        // Priority indicator
                        div {
                            style: format!(
                                "width: 8px; height: 8px; border-radius: 50%; background: {};",
                                priority_color
                            ),
                            title: "Priority: {event.priority}",
                        }

                        h3 {
                            style: "color: white; margin: 0; font-size: 1rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                            "{event.name}"
                        }
                    }

                    // Status badge
                    span {
                        style: format!(
                            "display: inline-block; padding: 0.125rem 0.5rem; border-radius: 0.25rem; font-size: 0.6875rem; background: {}20; color: {};",
                            status_color, status_color
                        ),
                        "{status_label}"
                    }
                }

                // Actions
                div {
                    style: "display: flex; gap: 0.25rem;",

                    // Favorite button
                    button {
                        onclick: move |e| {
                            e.stop_propagation();
                            props.on_toggle_favorite.call(());
                        },
                        style: format!(
                            "background: none; border: none; cursor: pointer; padding: 0.25rem; font-size: 1rem; {}",
                            if event.is_favorite { "color: #f59e0b;" } else { "color: #6b7280; opacity: 0.5;" }
                        ),
                        title: if event.is_favorite { "Remove from favorites" } else { "Add to favorites" },
                        "⭐"
                    }

                    // Active toggle
                    button {
                        onclick: move |e| {
                            e.stop_propagation();
                            props.on_toggle_active.call(());
                        },
                        style: format!(
                            "background: none; border: none; cursor: pointer; padding: 0.25rem; font-size: 0.875rem; {}",
                            if event.is_active { "color: #22c55e;" } else { "color: #6b7280;" }
                        ),
                        title: if event.is_active { "Deactivate" } else { "Activate" },
                        if event.is_active { "●" } else { "○" }
                    }
                }
            }

            // Description
            if !event.description.is_empty() {
                p {
                    style: "color: #9ca3af; font-size: 0.8125rem; margin: 0 0 0.75rem 0; overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;",
                    "{event.description}"
                }
            }

            // Stats row
            div {
                style: "display: flex; gap: 1rem; color: #6b7280; font-size: 0.75rem;",

                // Trigger count
                span {
                    title: "Times triggered",
                    "🎯 {event.trigger_count}"
                }

                // Outcome count
                span {
                    title: "Possible outcomes",
                    "🔀 {event.outcome_count}"
                }

                // Trigger conditions
                span {
                    title: "Trigger conditions",
                    "⚡ {event.trigger_condition_count}"
                }

                // Chain indicator
                if event.chain_id.is_some() {
                    span {
                        style: "color: #8b5cf6;",
                        title: "Part of an event chain",
                        "🔗 Chain"
                    }
                }
            }

            // Tags
            if !event.tags.is_empty() {
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 0.25rem; margin-top: 0.75rem;",

                    for tag in event.tags.iter().take(4) {
                        span {
                            style: "background: #374151; color: #9ca3af; padding: 0.125rem 0.375rem; border-radius: 0.25rem; font-size: 0.6875rem;",
                            "#{tag}"
                        }
                    }
                    if event.tags.len() > 4 {
                        {
                            let extra = event.tags.len() - 4;
                            rsx! {
                                span {
                                    style: "color: #6b7280; font-size: 0.6875rem;",
                                    "+{extra}"
                                }
                            }
                        }
                    }
                }
            }

            // Triggered timestamp
            if event.is_triggered {
                if let Some(ref triggered_at) = event.triggered_at {
                    div {
                        style: "margin-top: 0.75rem; padding-top: 0.5rem; border-top: 1px solid #374151; color: #22c55e; font-size: 0.75rem;",
                        "✓ Triggered: {triggered_at}"
                    }
                }
            }
        }
    }
}
