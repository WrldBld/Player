//! Conversation log component
//!
//! Displays a scrollable log of dialogue turns with speakers and timestamps.

use dioxus::prelude::*;

/// A single turn in the conversation
#[derive(Clone, PartialEq)]
pub struct ConversationTurn {
    /// Who is speaking (character name or "System")
    pub speaker: String,
    /// The spoken text
    pub text: String,
    /// Timestamp of the turn
    pub timestamp: String,
    /// Whether this is a system message
    pub is_system: bool,
}

/// Props for the ConversationLog component
#[derive(Props, Clone, PartialEq)]
pub struct ConversationLogProps {
    /// History of conversation turns
    pub turns: Vec<ConversationTurn>,
    /// Optional CSS class for styling
    #[props(default = "")]
    pub class: &'static str,
}

/// ConversationLog component - Scrollable dialogue history
///
/// Shows a log of all dialogue exchanges with speaker names and timestamps.
/// Useful for reviewing what has been said during the session.
#[component]
pub fn ConversationLog(props: ConversationLogProps) -> Element {
    // Auto-scroll to bottom when new turns are added (CSS scroll-behavior handles this)
    use_effect({
        let _turn_count = props.turns.len();
        move || {
            // Future: trigger scroll to bottom when new turns are added
        }
    });

    rsx! {
        div {
            class: "conversation-log {props.class}",
            style: "display: flex; flex-direction: column; height: 100%; background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

            // Header
            div {
                style: "padding: 0.75rem 1rem; border-bottom: 1px solid #374151;",

                h3 {
                    style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin: 0;",
                    "Conversation Log"
                }
            }

            // Log entries (scrollable)
            div {
                class: "log-entries",
                style: "flex: 1; overflow-y: auto; padding: 1rem; display: flex; flex-direction: column; gap: 0.75rem;",

                // Empty state
                if props.turns.is_empty() {
                    div {
                        style: "display: flex; align-items: center; justify-content: center; height: 100%; color: #6b7280; font-size: 0.875rem;",
                        "Waiting for dialogue..."
                    }
                } else {
                    for turn in props.turns.iter() {
                        ConversationEntry {
                            turn: turn.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Individual conversation entry
#[component]
fn ConversationEntry(turn: ConversationTurn) -> Element {
    let (bg_color, text_color, speaker_color) = if turn.is_system {
        ("rgba(59, 130, 246, 0.1)", "#60a5fa", "#60a5fa")
    } else {
        ("rgba(0, 0, 0, 0.3)", "white", "#3b82f6")
    };

    rsx! {
        div {
            class: "log-entry",
            style: format!(
                "padding: 0.75rem; background: {}; border-radius: 0.375rem; border-left: 2px solid {};",
                bg_color,
                speaker_color
            ),

            // Header with speaker and timestamp
            div {
                style: "display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.25rem;",

                span {
                    style: format!("color: {}; font-weight: 600; font-size: 0.875rem;", speaker_color),
                    "{turn.speaker}"
                }

                span {
                    style: "color: #6b7280; font-size: 0.75rem; margin-left: auto;",
                    "{turn.timestamp}"
                }
            }

            // Message text
            p {
                style: format!("color: {}; font-size: 0.875rem; line-height: 1.4; margin: 0;", text_color),
                "{turn.text}"
            }
        }
    }
}

