//! Ad-hoc Challenge Modal Component
//!
//! Allows DM to create custom challenges on-the-fly without LLM involvement.
//! The DM specifies all challenge details including outcomes.

use dioxus::prelude::*;
use crate::application::dto::{CharacterData, AdHocOutcomes};

/// Data for an ad-hoc challenge creation
#[derive(Debug, Clone, PartialEq)]
pub struct AdHocChallengeData {
    pub challenge_name: String,
    pub skill_name: String,
    pub difficulty: String,
    pub target_pc_id: String,
    pub outcomes: AdHocOutcomes,
}

/// Props for AdHocChallengeModal
#[derive(Props, Clone, PartialEq)]
pub struct AdHocChallengeModalProps {
    /// List of PCs that can be targeted
    pub player_characters: Vec<CharacterData>,
    /// Called when the challenge is created
    pub on_create: EventHandler<AdHocChallengeData>,
    /// Called when modal should close
    pub on_close: EventHandler<()>,
}

/// AdHocChallengeModal component
///
/// Allows DM to create custom challenges with:
/// - Challenge name
/// - Skill being tested
/// - Difficulty (e.g., "DC 15", "Hard")
/// - Target PC selection
/// - Custom outcomes (success, failure, optional criticals)
#[component]
pub fn AdHocChallengeModal(props: AdHocChallengeModalProps) -> Element {
    let mut challenge_name = use_signal(|| String::new());
    let mut skill_name = use_signal(|| String::new());
    let mut difficulty = use_signal(|| String::new());
    let mut selected_pc = use_signal(|| String::new());
    let mut success_outcome = use_signal(|| String::new());
    let mut failure_outcome = use_signal(|| String::new());
    let mut critical_success = use_signal(|| String::new());
    let mut critical_failure = use_signal(|| String::new());
    let mut show_criticals = use_signal(|| false);

    let player_characters = props.player_characters.clone();

    // Validation: all required fields must be filled
    let is_valid = !challenge_name.read().is_empty()
        && !skill_name.read().is_empty()
        && !difficulty.read().is_empty()
        && !selected_pc.read().is_empty()
        && !success_outcome.read().is_empty()
        && !failure_outcome.read().is_empty();

    rsx! {
        // Modal overlay
        div {
            id: "adhoc-overlay",
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 1000; overflow-y: auto; padding: 1rem;",
            onclick: move |_| {
                props.on_close.call(());
            },

            // Modal content
            div {
                id: "adhoc-modal",
                style: "background: linear-gradient(135deg, #1a1a2e 0%, #0f0f23 100%); padding: 2rem; border-radius: 1rem; max-width: 600px; width: 90%; border: 2px solid #a855f7; max-height: 90vh; overflow-y: auto;",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;",

                    h2 {
                        style: "color: #a855f7; margin: 0; font-size: 1.5rem;",
                        "Create Ad-hoc Challenge"
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #9ca3af; cursor: pointer; font-size: 1.5rem; padding: 0;",
                        "x"
                    }
                }

                // Challenge details section
                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 1.5rem;",

                    // Challenge name
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                            "Challenge Name *"
                        }
                        input {
                            r#type: "text",
                            value: "{challenge_name}",
                            placeholder: "e.g., Negotiate Price",
                            oninput: move |e| challenge_name.set(e.value()),
                            style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 0.875rem; box-sizing: border-box;",
                        }
                    }

                    // Skill name
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                            "Skill Being Tested *"
                        }
                        input {
                            r#type: "text",
                            value: "{skill_name}",
                            placeholder: "e.g., Persuasion",
                            oninput: move |e| skill_name.set(e.value()),
                            style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 0.875rem; box-sizing: border-box;",
                        }
                    }

                    // Difficulty
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                            "Difficulty *"
                        }
                        input {
                            r#type: "text",
                            value: "{difficulty}",
                            placeholder: "e.g., DC 15, Hard",
                            oninput: move |e| difficulty.set(e.value()),
                            style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 0.875rem; box-sizing: border-box;",
                        }
                    }

                    // Target PC
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                            "Target PC *"
                        }
                        select {
                            value: "{selected_pc}",
                            onchange: move |e| selected_pc.set(e.value()),
                            style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; cursor: pointer; font-size: 0.875rem; box-sizing: border-box;",

                            option {
                                value: "",
                                disabled: true,
                                selected: true,
                                "Select PC..."
                            }

                            for character in player_characters.iter() {
                                option {
                                    key: "{character.id}",
                                    value: "{character.id}",
                                    "{character.name}"
                                }
                            }
                        }
                    }
                }

                // Outcomes section header
                div {
                    style: "border-top: 1px solid #374151; padding-top: 1.5rem; margin-bottom: 1rem;",

                    h3 {
                        style: "color: #f59e0b; margin: 0 0 0.5rem 0; font-size: 1.125rem;",
                        "Outcomes"
                    }
                    p {
                        style: "color: #6b7280; font-size: 0.75rem; margin: 0;",
                        "Define what happens when the player succeeds or fails the challenge."
                    }
                }

                // Success outcome
                div {
                    style: "margin-bottom: 1rem;",

                    label {
                        style: "display: block; color: #22c55e; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                        "Success Outcome *"
                    }
                    textarea {
                        value: "{success_outcome}",
                        placeholder: "What happens when the player succeeds...",
                        oninput: move |e| success_outcome.set(e.value()),
                        style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 0.875rem; min-height: 80px; resize: vertical; box-sizing: border-box;",
                    }
                }

                // Failure outcome
                div {
                    style: "margin-bottom: 1rem;",

                    label {
                        style: "display: block; color: #ef4444; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                        "Failure Outcome *"
                    }
                    textarea {
                        value: "{failure_outcome}",
                        placeholder: "What happens when the player fails...",
                        oninput: move |e| failure_outcome.set(e.value()),
                        style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 0.875rem; min-height: 80px; resize: vertical; box-sizing: border-box;",
                    }
                }

                // Toggle for critical outcomes
                div {
                    style: "margin-bottom: 1rem;",

                    {
                        let is_expanded = *show_criticals.read();
                        let rotation = if is_expanded { "90deg" } else { "0deg" };
                        rsx! {
                            button {
                                onclick: move |_| {
                                    let current = *show_criticals.read();
                                    show_criticals.set(!current);
                                },
                                style: "background: none; border: none; color: #a855f7; cursor: pointer; font-size: 0.875rem; padding: 0; display: flex; align-items: center; gap: 0.5rem;",

                                span {
                                    style: "transform: rotate({rotation}); transition: transform 0.2s;",
                                    ">"
                                }
                                "Add Critical Outcomes (optional)"
                            }
                        }
                    }
                }

                // Critical outcomes (collapsible)
                if *show_criticals.read() {
                    div {
                        style: "padding-left: 1rem; border-left: 2px solid #a855f7;",

                        // Critical success
                        div {
                            style: "margin-bottom: 1rem;",

                            label {
                                style: "display: block; color: #fbbf24; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                                "Critical Success (optional)"
                            }
                            textarea {
                                value: "{critical_success}",
                                placeholder: "What happens on a critical success (e.g., nat 20)...",
                                oninput: move |e| critical_success.set(e.value()),
                                style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 0.875rem; min-height: 60px; resize: vertical; box-sizing: border-box;",
                            }
                        }

                        // Critical failure
                        div {
                            style: "margin-bottom: 1rem;",

                            label {
                                style: "display: block; color: #dc2626; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                                "Critical Failure (optional)"
                            }
                            textarea {
                                value: "{critical_failure}",
                                placeholder: "What happens on a critical failure (e.g., nat 1)...",
                                oninput: move |e| critical_failure.set(e.value()),
                                style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 0.875rem; min-height: 60px; resize: vertical; box-sizing: border-box;",
                            }
                        }
                    }
                }

                // Action buttons
                div {
                    style: "display: flex; gap: 0.75rem; margin-top: 1.5rem;",

                    button {
                        onclick: move |_| {
                            if is_valid {
                                let data = AdHocChallengeData {
                                    challenge_name: challenge_name.read().clone(),
                                    skill_name: skill_name.read().clone(),
                                    difficulty: difficulty.read().clone(),
                                    target_pc_id: selected_pc.read().clone(),
                                    outcomes: AdHocOutcomes {
                                        success: success_outcome.read().clone(),
                                        failure: failure_outcome.read().clone(),
                                        critical_success: if critical_success.read().is_empty() {
                                            None
                                        } else {
                                            Some(critical_success.read().clone())
                                        },
                                        critical_failure: if critical_failure.read().is_empty() {
                                            None
                                        } else {
                                            Some(critical_failure.read().clone())
                                        },
                                    },
                                };
                                props.on_create.call(data);
                            }
                        },
                        disabled: !is_valid,
                        style: format!(
                            "flex: 1; padding: 0.75rem; background: {}; color: white; border: none; border-radius: 0.5rem; cursor: {}; font-weight: 600;",
                            if is_valid { "#a855f7" } else { "#6b7280" },
                            if is_valid { "pointer" } else { "not-allowed" }
                        ),

                        "Create Challenge"
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
