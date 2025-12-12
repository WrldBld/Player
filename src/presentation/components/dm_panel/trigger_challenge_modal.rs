//! Trigger Challenge Modal Component
//!
//! Allows DM to select and trigger a challenge for a specific character.

use dioxus::prelude::*;
use crate::application::dto::{ChallengeData, CharacterData};

/// Props for TriggerChallengeModal
#[derive(Props, Clone, PartialEq)]
pub struct TriggerChallengeModalProps {
    /// List of available challenges
    pub challenges: Vec<ChallengeData>,
    /// List of characters in the current scene to target
    pub scene_characters: Vec<CharacterData>,
    /// Called when a challenge is triggered
    pub on_trigger: EventHandler<(String, String)>, // (challenge_id, character_id)
    /// Called when modal should close
    pub on_close: EventHandler<()>,
}

/// TriggerChallengeModal component
///
/// Allows DM to:
/// - Select a challenge from available challenges
/// - Select a target character
/// - Trigger the challenge
#[component]
pub fn TriggerChallengeModal(props: TriggerChallengeModalProps) -> Element {
    let mut selected_challenge = use_signal(|| String::new());
    let mut selected_character = use_signal(|| String::new());

    let challenges = props.challenges.clone();
    let scene_characters = props.scene_characters.clone();

    rsx! {
        // Modal overlay
        div {
            id: "trigger-overlay",
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| {
                props.on_close.call(());
            },

            // Modal content
            div {
                id: "trigger-modal",
                style: "background: linear-gradient(135deg, #1a1a2e 0%, #0f0f23 100%); padding: 2rem; border-radius: 1rem; max-width: 500px; width: 90%; border: 2px solid #f59e0b;",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;",

                    h2 {
                        style: "color: #f59e0b; margin: 0; font-size: 1.5rem;",
                        "Trigger Challenge"
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #9ca3af; cursor: pointer; font-size: 1.5rem; padding: 0;",
                        "×"
                    }
                }

                // Challenge selection
                div {
                    style: "margin-bottom: 1.5rem;",

                    label {
                        style: "display: block; color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin-bottom: 0.5rem;",
                        "Select Challenge"
                    }

                    select {
                        value: "{selected_challenge}",
                        onchange: move |e| selected_challenge.set(e.value()),
                        style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; cursor: pointer; font-size: 0.875rem;",

                        option {
                            value: "",
                            disabled: true,
                            selected: true,
                            "Choose a challenge..."
                        }

                        for challenge in challenges.iter() {
                            option {
                                key: "{challenge.id}",
                                value: "{challenge.id}",
                                "{challenge.name}"
                            }
                        }
                    }
                }

                // Challenge preview
                if !selected_challenge.read().is_empty() {
                    {
                        let selected_id = selected_challenge.read().clone();
                        if let Some(challenge) = challenges.iter().find(|c| c.id == selected_id) {
                            rsx! {
                                div {
                                    style: "margin-bottom: 1.5rem; padding: 1rem; background: rgba(0, 0, 0, 0.3); border-radius: 0.5rem; border-left: 3px solid #f59e0b;",

                                    p {
                                        style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0 0 0.5rem 0;",
                                        "Challenge Preview"
                                    }

                                    p {
                                        style: "color: white; margin: 0 0 0.5rem 0; line-height: 1.4;",
                                        "{challenge.description}"
                                    }

                                    div {
                                        style: "display: flex; gap: 1rem; font-size: 0.875rem;",

                                        span { style: "color: #9ca3af;",
                                            "Type: "
                                            span { style: "color: #3b82f6;", "{challenge.challenge_type:?}" }
                                        }

                                        span { style: "color: #9ca3af;",
                                            "Difficulty: "
                                            span { style: "color: #f59e0b;", "{challenge.difficulty:?}" }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! { div {} }
                        }
                    }
                }

                // Character selection
                div {
                    style: "margin-bottom: 1.5rem;",

                    label {
                        style: "display: block; color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin-bottom: 0.5rem;",
                        "Target Character"
                    }

                    select {
                        value: "{selected_character}",
                        onchange: move |e| selected_character.set(e.value()),
                        style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; cursor: pointer; font-size: 0.875rem;",

                        option {
                            value: "",
                            disabled: true,
                            selected: true,
                            "Choose a character..."
                        }

                        for character in scene_characters.iter() {
                            option {
                                key: "{character.id}",
                                value: "{character.id}",
                                "{character.name}"
                            }
                        }
                    }
                }

                // Character preview
                if !selected_character.read().is_empty() {
                    {
                        let selected_char_id = selected_character.read().clone();
                        if let Some(character) = scene_characters.iter().find(|c| c.id == selected_char_id) {
                            rsx! {
                                div {
                                    style: "margin-bottom: 1.5rem; padding: 1rem; background: rgba(59, 130, 246, 0.1); border-radius: 0.5rem; border-left: 3px solid #3b82f6;",

                                    p {
                                        style: "color: white; margin: 0; font-weight: bold;",
                                        "Target: {character.name}"
                                    }
                                }
                            }
                        } else {
                            rsx! { div {} }
                        }
                    }
                }

                // Action buttons
                div {
                    style: "display: flex; gap: 0.75rem;",

                    button {
                        onclick: move |_| {
                            let challenge_id = selected_challenge.read().clone();
                            let character_id = selected_character.read().clone();
                            if !challenge_id.is_empty() && !character_id.is_empty() {
                                props.on_trigger.call((challenge_id, character_id));
                            }
                        },
                        disabled: selected_challenge.read().is_empty() || selected_character.read().is_empty(),
                        style: format!(
                            "flex: 1; padding: 0.75rem; background: {}; color: white; border: none; border-radius: 0.5rem; cursor: {}; font-weight: 600;",
                            if selected_challenge.read().is_empty() || selected_character.read().is_empty() {
                                "#6b7280"
                            } else {
                                "#22c55e"
                            },
                            if selected_challenge.read().is_empty() || selected_character.read().is_empty() {
                                "not-allowed"
                            } else {
                                "pointer"
                            }
                        ),

                        "Trigger Challenge"
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "flex: 1; padding: 0.75rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600;",
                        "Cancel"
                    }
                }
            }
        }
    }
}
