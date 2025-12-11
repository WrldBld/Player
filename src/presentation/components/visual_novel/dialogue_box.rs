//! Dialogue box component for visual novel scenes
//!
//! Displays dialogue with speaker name, text, and choices.

use dioxus::prelude::*;

use crate::infrastructure::websocket::DialogueChoice;

use super::choice_menu::{ChoiceMenu, ContinuePrompt};

/// Props for the DialogueBox component
#[derive(Props, Clone, PartialEq)]
pub struct DialogueBoxProps {
    /// Speaker name
    pub speaker_name: String,
    /// Dialogue text to display (may be partial during typewriter)
    pub dialogue_text: String,
    /// Whether typewriter is still animating
    #[props(default = false)]
    pub is_typing: bool,
    /// Available dialogue choices
    #[props(default)]
    pub choices: Vec<DialogueChoice>,
    /// Handler for when a choice is selected
    pub on_choice_selected: EventHandler<String>,
    /// Handler for custom text input
    pub on_custom_input: EventHandler<String>,
    /// Handler for advancing dialogue (clicking to continue)
    pub on_advance: EventHandler<()>,
}

/// Dialogue box component - displays dialogue with typewriter effect
///
/// Uses `.vn-dialogue-box`, `.vn-character-name`, `.vn-dialogue-text` Tailwind classes.
#[component]
pub fn DialogueBox(props: DialogueBoxProps) -> Element {
    let has_speaker = !props.speaker_name.is_empty();
    let has_choices = !props.choices.is_empty();
    let show_continue = !props.is_typing && !has_choices;

    rsx! {
        div {
            class: "vn-dialogue-box",

            // Speaker name plate
            if has_speaker {
                div {
                    class: "vn-character-name",
                    "{props.speaker_name}"
                }
            }

            // Dialogue text with typewriter cursor
            div {
                class: "dialogue-text-container",
                style: "min-height: 60px;",
                onclick: move |_| {
                    if props.is_typing {
                        props.on_advance.call(());
                    }
                },

                p {
                    class: "vn-dialogue-text",

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

            // Choice menu or continue prompt
            if !props.is_typing {
                if has_choices {
                    ChoiceMenu {
                        choices: props.choices.clone(),
                        on_select: props.on_choice_selected,
                        on_custom_input: props.on_custom_input,
                    }
                } else if show_continue {
                    ContinuePrompt {
                        on_continue: props.on_advance,
                    }
                }
            }
        }
    }
}

/// Minimal dialogue box for narration (no speaker name)
#[derive(Props, Clone, PartialEq)]
pub struct NarrationBoxProps {
    /// Narration text
    pub text: String,
    /// Whether typewriter is animating
    #[props(default = false)]
    pub is_typing: bool,
    /// Click handler to advance
    pub on_advance: EventHandler<()>,
}

#[component]
pub fn NarrationBox(props: NarrationBoxProps) -> Element {
    rsx! {
        div {
            class: "vn-dialogue-box narration",
            style: "text-align: center;",
            onclick: move |_| props.on_advance.call(()),

            p {
                class: "vn-dialogue-text",
                style: "font-style: italic; color: #d1d5db;",

                "{props.text}"

                if props.is_typing {
                    span {
                        class: "typewriter-cursor",
                        style: "animation: blink 0.7s step-end infinite; margin-left: 2px;",
                        "▌"
                    }
                }
            }

            if !props.is_typing {
                div {
                    style: "color: #6b7280; font-size: 0.75rem; margin-top: 0.5rem;",
                    "Click to continue"
                }
            }
        }
    }
}

/// Empty dialogue box placeholder
#[component]
pub fn EmptyDialogueBox() -> Element {
    rsx! {
        div {
            class: "vn-dialogue-box",
            style: "opacity: 0.5;",

            p {
                class: "vn-dialogue-text",
                style: "color: #6b7280; font-style: italic;",
                "..."
            }
        }
    }
}
