//! Timeline Event Card - Display a single story event

use dioxus::prelude::*;

use crate::application::dto::{StoryEventData, StoryEventTypeData};
use crate::presentation::components::story_arc::timeline_view::get_event_type_icon;

#[derive(Props, Clone)]
pub struct TimelineEventCardProps {
    pub event: StoryEventData,
    pub on_click: EventHandler<()>,
    pub on_toggle_visibility: EventHandler<()>,
}

impl PartialEq for TimelineEventCardProps {
    fn eq(&self, other: &Self) -> bool {
        self.event.id == other.event.id
    }
}

#[component]
pub fn TimelineEventCard(props: TimelineEventCardProps) -> Element {
    let event = &props.event;
    let icon = get_event_type_icon(&event.event_type);
    let type_color = get_event_type_color(&event.event_type);

    // Parse and format timestamp
    let formatted_time = format_timestamp(&event.timestamp);

    rsx! {
        div {
            class: "timeline-event-card",
            style: format!(
                "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem; cursor: pointer; transition: all 0.2s; border-left: 3px solid {}; opacity: {};",
                type_color,
                if event.is_hidden { "0.5" } else { "1" }
            ),
            onclick: move |_| props.on_click.call(()),

            // Top row: Icon, summary, timestamp
            div {
                style: "display: flex; align-items: flex-start; gap: 0.75rem;",

                // Icon
                div {
                    style: format!("font-size: 1.25rem; padding: 0.5rem; background: {}20; border-radius: 0.5rem;", type_color),
                    "{icon}"
                }

                // Content
                div {
                    style: "flex: 1; min-width: 0;",

                    // Summary
                    p {
                        style: "color: white; margin: 0 0 0.25rem 0; font-size: 0.9375rem; line-height: 1.4;",
                        "{event.summary}"
                    }

                    // Metadata row
                    div {
                        style: "display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center;",

                        // Timestamp
                        span {
                            style: "color: #6b7280; font-size: 0.75rem;",
                            "{formatted_time}"
                        }

                        // Game time if available
                        if let Some(ref game_time) = event.game_time {
                            span {
                                style: "color: #8b5cf6; font-size: 0.75rem;",
                                "🕐 {game_time}"
                            }
                        }

                        // Tags
                        for tag in event.tags.iter().take(3) {
                            span {
                                style: "background: #374151; color: #9ca3af; padding: 0.125rem 0.375rem; border-radius: 0.25rem; font-size: 0.6875rem;",
                                "#{tag}"
                            }
                        }
                        if event.tags.len() > 3 {
                            {
                                let extra = event.tags.len() - 3;
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

                // Actions
                div {
                    style: "display: flex; gap: 0.25rem;",

                    // Visibility toggle
                    button {
                        onclick: move |e| {
                            e.stop_propagation();
                            props.on_toggle_visibility.call(());
                        },
                        style: "background: none; border: none; cursor: pointer; padding: 0.25rem; color: #6b7280;",
                        title: if event.is_hidden { "Show in timeline" } else { "Hide from timeline" },
                        if event.is_hidden { "👁️‍🗨️" } else { "👁️" }
                    }
                }
            }

            // Event-specific preview content
            match &event.event_type {
                StoryEventTypeData::DialogueExchange { npc_name, npc_response, .. } => rsx! {
                    div {
                        style: "margin-top: 0.75rem; padding: 0.75rem; background: rgba(0,0,0,0.2); border-radius: 0.375rem;",
                        p {
                            style: "color: #60a5fa; font-size: 0.8125rem; margin: 0 0 0.25rem 0;",
                            "{npc_name}:"
                        }
                        p {
                            style: "color: #9ca3af; font-size: 0.8125rem; margin: 0; font-style: italic; overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;",
                            "\"{npc_response}\""
                        }
                    }
                },
                StoryEventTypeData::ChallengeAttempted { challenge_name, outcome, roll_result, .. } => rsx! {
                    div {
                        style: "margin-top: 0.5rem; display: flex; gap: 0.75rem; align-items: center;",
                        span {
                            style: "color: #9ca3af; font-size: 0.8125rem;",
                            "{challenge_name}"
                        }
                        if let Some(roll) = roll_result {
                            span {
                                style: "background: #374151; color: white; padding: 0.25rem 0.5rem; border-radius: 0.25rem; font-size: 0.75rem; font-family: monospace;",
                                "🎲 {roll}"
                            }
                        }
                        span {
                            style: format!("padding: 0.25rem 0.5rem; border-radius: 0.25rem; font-size: 0.75rem; background: {}; color: white;",
                                get_outcome_color(outcome)),
                            "{outcome}"
                        }
                    }
                },
                StoryEventTypeData::DmMarker { title, importance, .. } => rsx! {
                    div {
                        style: "margin-top: 0.5rem; display: flex; align-items: center; gap: 0.5rem;",
                        span {
                            style: format!("padding: 0.25rem 0.5rem; border-radius: 0.25rem; font-size: 0.75rem; background: {}; color: white;",
                                get_importance_color(importance)),
                            "{importance}"
                        }
                        span {
                            style: "color: #9ca3af; font-size: 0.8125rem;",
                            "{title}"
                        }
                    }
                },
                StoryEventTypeData::RelationshipChanged { previous_sentiment, new_sentiment, .. } => {
                    let change = new_sentiment - previous_sentiment.unwrap_or(0.0);
                    let change_icon = if change > 0.0 { "📈" } else { "📉" };
                    let change_color = if change > 0.0 { "#22c55e" } else { "#ef4444" };
                    let change_str = format!("{:+.1}", change);
                    let sentiment_str = format!("{:.1}", new_sentiment);
                    rsx! {
                        div {
                            style: "margin-top: 0.5rem; display: flex; align-items: center; gap: 0.5rem;",
                            span { "{change_icon}" }
                            span {
                                style: format!("color: {}; font-size: 0.8125rem;", change_color),
                                "{change_str}"
                            }
                            span {
                                style: "color: #6b7280; font-size: 0.8125rem;",
                                "→ {sentiment_str}"
                            }
                        }
                    }
                },
                _ => rsx! {}
            }
        }
    }
}

/// Get a color for each event type
fn get_event_type_color(event_type: &StoryEventTypeData) -> &'static str {
    match event_type {
        StoryEventTypeData::LocationChange { .. } => "#22c55e",
        StoryEventTypeData::DialogueExchange { .. } => "#3b82f6",
        StoryEventTypeData::CombatEvent { .. } => "#ef4444",
        StoryEventTypeData::ChallengeAttempted { .. } => "#f59e0b",
        StoryEventTypeData::ItemAcquired { .. } => "#a855f7",
        StoryEventTypeData::RelationshipChanged { .. } => "#ec4899",
        StoryEventTypeData::SceneTransition { .. } => "#06b6d4",
        StoryEventTypeData::InformationRevealed { .. } => "#eab308",
        StoryEventTypeData::DmMarker { .. } => "#8b5cf6",
        StoryEventTypeData::NarrativeEventTriggered { .. } => "#14b8a6",
        StoryEventTypeData::SessionStarted { .. } => "#22c55e",
        StoryEventTypeData::SessionEnded { .. } => "#6b7280",
        StoryEventTypeData::Custom { .. } => "#9ca3af",
    }
}

/// Get color for challenge outcome
fn get_outcome_color(outcome: &str) -> &'static str {
    match outcome.to_lowercase().as_str() {
        "critical_success" | "criticalsuccess" => "#22c55e",
        "success" => "#3b82f6",
        "partial_success" | "partialsuccess" => "#f59e0b",
        "failure" => "#ef4444",
        "critical_failure" | "criticalfailure" => "#dc2626",
        _ => "#6b7280",
    }
}

/// Get color for DM marker importance
fn get_importance_color(importance: &str) -> &'static str {
    match importance.to_lowercase().as_str() {
        "critical" => "#ef4444",
        "major" => "#f59e0b",
        "normal" => "#3b82f6",
        "minor" => "#6b7280",
        _ => "#6b7280",
    }
}

/// Format a timestamp string for display
fn format_timestamp(timestamp: &str) -> String {
    // Try to parse as RFC3339 and format nicely
    // If that fails, just return the original string
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%b %d, %H:%M").to_string()
    } else {
        timestamp.to_string()
    }
}
