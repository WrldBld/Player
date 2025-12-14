//! Spectator View - Watch the game without participating
//!
//! Displays the same scene as PC view but without interaction capabilities.
//! Spectators can see the backdrop, characters, and dialogue but cannot make choices.

use dioxus::prelude::*;

use crate::presentation::components::visual_novel::{Backdrop, CharacterLayer, EmptyDialogueBox};
use crate::presentation::state::{use_dialogue_state, use_game_state, use_typewriter_effect};

/// Props for SpectatorView
#[derive(Props, Clone, PartialEq)]
pub struct SpectatorViewProps {
    /// Handler for back button
    pub on_back: EventHandler<()>,
}

/// Spectator View - read-only view of the game
#[component]
pub fn SpectatorView(props: SpectatorViewProps) -> Element {
    // Get global state from context
    let game_state = use_game_state();
    let mut dialogue_state = use_dialogue_state();

    // Run typewriter effect for read-only dialogue display
    use_typewriter_effect(&mut dialogue_state);

    // Read scene characters from game state (reactive)
    let scene_characters = game_state.scene_characters.read().clone();

    // Get conversation history for the log
    let mut conversation_log = use_signal(|| Vec::<ConversationEntry>::new());

    // Track dialogue updates to add to log
    {
        let dialogue_state_clone = dialogue_state.clone();
        use_effect(move || {
            let is_typing = *dialogue_state_clone.is_typing.read();
            let has_dialogue = !dialogue_state_clone.full_text.read().is_empty();
            let current_speaker = dialogue_state_clone.speaker_name.read().clone();
            let current_text = dialogue_state_clone.displayed_text.read().clone();

            // Only add to log once typing is complete
            if !is_typing && has_dialogue && !current_text.is_empty() {
                let mut log = conversation_log.write();

                // Check if we should add a new entry (different speaker or new dialogue)
                let should_add = log.is_empty() ||
                   log.last().map(|e| &e.speaker) != Some(&current_speaker) ||
                   log.last().map(|e| &e.text) != Some(&current_text);

                if should_add {
                    log.push(ConversationEntry {
                        speaker: current_speaker,
                        text: current_text,
                    });
                }
            }
        });
    }

    // Read current state for rendering
    let speaker_name = dialogue_state.speaker_name.read().clone();
    let displayed_text = dialogue_state.displayed_text.read().clone();
    let is_typing = *dialogue_state.is_typing.read();
    let has_dialogue = dialogue_state.has_dialogue();
    let is_llm_processing = *dialogue_state.is_llm_processing.read();

    rsx! {
        div {
            class: "spectator-view",
            style: "height: 100%; display: flex; flex-direction: column; position: relative; background: linear-gradient(to bottom, #1a1a2e, #2d1b3d);",

            // Back button
            button {
                onclick: move |_| props.on_back.call(()),
                style: "position: absolute; top: 1rem; left: 1rem; z-index: 100; padding: 0.5rem 1rem; background: rgba(0,0,0,0.5); color: white; border: 1px solid #374151; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                "← Back"
            }

            // Spectator badge
            div {
                style: "position: absolute; top: 1rem; right: 1rem; z-index: 100; padding: 0.5rem 1rem; background: rgba(139, 92, 246, 0.2); color: #a78bfa; border: 1px solid #8b5cf6; border-radius: 0.5rem; font-size: 0.875rem;",
                "Spectating"
            }

            // Visual novel stage (2.3.1 - Scene display)
            Backdrop {
                image_url: game_state.backdrop_url(),

                // Character layer with real scene characters
                CharacterLayer {
                    characters: scene_characters,
                    on_character_click: None, // Spectators cannot interact
                }
            }

            // Dialogue box (fixed at bottom) - 2.3.2 Read-only dialogue display
            div {
                class: "dialogue-container",
                style: "position: absolute; bottom: 0; left: 0; right: 0; z-index: 10;",

                if has_dialogue {
                    SpectatorDialogueBox {
                        speaker_name: speaker_name.clone(),
                        dialogue_text: displayed_text.clone(),
                        is_typing: is_typing,
                        is_llm_processing: is_llm_processing,
                    }
                } else {
                    EmptyDialogueBox {}
                }
            }

            // Conversation log (2.3.3 - Scrollable history) - only show if log has entries
            if !conversation_log.read().is_empty() {
                ConversationLog {
                    entries: conversation_log.read().clone(),
                }
            }
        }
    }
}

/// A read-only dialogue box for spectators (no choices)
///
/// Displays the speaker name and dialogue text with typewriter animation,
/// but does not show choice buttons or custom input fields.
#[derive(Props, Clone, PartialEq)]
pub struct SpectatorDialogueBoxProps {
    /// Speaker name
    pub speaker_name: String,
    /// Dialogue text to display
    pub dialogue_text: String,
    /// Whether typewriter is still animating
    #[props(default = false)]
    pub is_typing: bool,
    /// Whether NPC is currently thinking
    #[props(default = false)]
    pub is_llm_processing: bool,
}

/// Spectator-specific dialogue box (no interaction)
#[component]
fn SpectatorDialogueBox(props: SpectatorDialogueBoxProps) -> Element {
    let has_speaker = !props.speaker_name.is_empty();

    rsx! {
        div {
            class: "spectator-dialogue-box",
            style: "background: rgba(0, 0, 0, 0.85); border-top: 2px solid #8b5cf6; padding: 1rem; max-height: 200px;",

            // Speaker name plate
            if has_speaker {
                div {
                    class: "spectator-character-name",
                    style: "color: #a78bfa; font-weight: 600; font-size: 0.875rem; margin-bottom: 0.5rem; text-transform: uppercase; letter-spacing: 0.05em;",
                    "{props.speaker_name}"
                }
            }

            // Dialogue text with typewriter cursor
            div {
                class: "spectator-dialogue-text-container",
                style: "min-height: 60px; overflow: hidden;",

                if props.is_llm_processing {
                    p {
                        class: "spectator-dialogue-text",
                        style: "color: #9ca3af; font-style: italic; font-size: 0.95rem; line-height: 1.5; margin: 0;",

                        "NPC is thinking"

                        // Animated ellipsis
                        span {
                            style: "animation: ellipsis 1.5s steps(4, end) infinite;",
                            "..."
                        }
                    }
                } else {
                    p {
                        class: "spectator-dialogue-text",
                        style: "color: #e5e7eb; font-size: 0.95rem; line-height: 1.5; margin: 0;",

                        "{props.dialogue_text}"

                        // Blinking cursor during typing
                        if props.is_typing {
                            span {
                                class: "typewriter-cursor",
                                style: "animation: blink 0.7s step-end infinite; margin-left: 2px;",
                                "▌"
                            }
                        }
                    }
                }
            }

            // Spectator indicator (instead of choices)
            div {
                style: "margin-top: 0.75rem; padding-top: 0.75rem; border-top: 1px solid #374151; color: #8b5cf6; font-size: 0.75rem; text-align: center; font-style: italic;",
                "Spectating - No choices available"
            }
        }
    }
}

/// A conversation log entry
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConversationEntry {
    /// Speaker name
    speaker: String,
    /// Dialogue text
    text: String,
}

/// Conversation log component - scrollable history
///
/// Shows a history of all dialogue exchanges in chronological order.
#[derive(Props, Clone, PartialEq)]
pub struct ConversationLogProps {
    /// Log entries
    pub entries: Vec<ConversationEntry>,
}

#[component]
fn ConversationLog(props: ConversationLogProps) -> Element {
    rsx! {
        div {
            class: "conversation-log",
            style: "position: absolute; bottom: 220px; left: 0; right: 0; height: 180px; background: rgba(0, 0, 0, 0.7); border-top: 1px solid #374151; border-bottom: 1px solid #374151; overflow-y: auto; padding: 1rem; font-size: 0.85rem; line-height: 1.4;",

            for (idx, entry) in props.entries.iter().enumerate() {
                div {
                    key: "{idx}",
                    style: "margin-bottom: 0.5rem; padding-bottom: 0.5rem; border-bottom: 1px solid #1f2937;",

                    div {
                        style: "color: #a78bfa; font-weight: 600; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em;",
                        "{entry.speaker}"
                    }

                    div {
                        style: "color: #d1d5db; margin-top: 0.25rem; word-wrap: break-word;",
                        "{entry.text}"
                    }
                }
            }
        }
    }
}
