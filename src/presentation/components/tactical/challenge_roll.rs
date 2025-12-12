//! Challenge Roll Modal Component
//!
//! Displays a modal for performing challenge rolls with d20 dice mechanics.
//! Allows players to roll dice and add character modifiers.

use dioxus::prelude::*;

/// Props for the ChallengeRollModal component
#[derive(Props, Clone, PartialEq)]
pub struct ChallengeRollModalProps {
    /// Unique challenge ID
    pub challenge_id: String,
    /// Human-readable challenge name
    pub challenge_name: String,
    /// Associated skill name
    pub skill_name: String,
    /// Difficulty display (e.g., "DC 12", "Very Hard")
    pub difficulty_display: String,
    /// Challenge description/flavor text
    pub description: String,
    /// Character's skill modifier for this challenge
    pub character_modifier: i32,
    /// Called with the d20 roll result when roll is submitted
    pub on_roll: EventHandler<i32>,
    /// Called when modal should close
    pub on_close: EventHandler<()>,
}

/// ChallengeRollModal component
///
/// Displays a modal with:
/// - Challenge name, skill, and difficulty
/// - Animated d20 roll button
/// - Roll result display with modifier calculation
/// - Submit button to send result back to engine
#[component]
pub fn ChallengeRollModal(props: ChallengeRollModalProps) -> Element {
    let mut roll_result = use_signal(|| None::<i32>);
    let mut is_rolling = use_signal(|| false);

    // Perform the d20 roll
    let do_roll = move |_| {
        is_rolling.set(true);
        // Generate random d20 roll (1-20)
        #[cfg(target_arch = "wasm32")]
        {
            let roll = ((js_sys::Math::random() * 20.0).floor() as i32) + 1;
            roll_result.set(Some(roll));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(42);
            let roll = ((nanos % 20) + 1) as i32;
            roll_result.set(Some(roll));
        }
        is_rolling.set(false);
    };

    rsx! {
        // Modal overlay
        div {
            id: "challenge-overlay",
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| {
                props.on_close.call(());
            },

            // Modal content
            div {
                id: "challenge-modal",
                style: "background: linear-gradient(135deg, #1a1a2e 0%, #0f0f23 100%); padding: 2rem; border-radius: 1rem; max-width: 450px; width: 90%; border: 2px solid #f59e0b; box-shadow: 0 20px 60px rgba(245, 158, 11, 0.2);",

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;",

                    h2 {
                        style: "color: #f59e0b; margin: 0; font-size: 1.5rem;",
                        "Challenge"
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #9ca3af; cursor: pointer; font-size: 1.5rem; padding: 0;",
                        "×"
                    }
                }

                // Challenge details
                div {
                    style: "margin-bottom: 1.5rem;",

                    h3 {
                        style: "color: white; margin: 0 0 0.5rem 0; font-size: 1.25rem;",
                        "{props.challenge_name}"
                    }

                    p {
                        style: "color: #9ca3af; margin: 0 0 1rem 0; line-height: 1.5;",
                        "{props.description}"
                    }
                }

                // Skill and difficulty info
                div {
                    style: "display: flex; justify-content: space-between; margin-bottom: 1.5rem; padding: 1rem; background: rgba(0, 0, 0, 0.3); border-radius: 0.5rem;",

                    div {
                        span {
                            style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; display: block; margin-bottom: 0.25rem;",
                            "Skill"
                        }
                        span {
                            style: "color: white; font-weight: bold;",
                            "{props.skill_name}"
                        }
                    }

                    div {
                        span {
                            style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; display: block; margin-bottom: 0.25rem;",
                            "Difficulty"
                        }
                        span {
                            style: "color: #f59e0b; font-weight: bold;",
                            "{props.difficulty_display}"
                        }
                    }
                }

                // Modifier display
                div {
                    style: "text-align: center; margin-bottom: 1.5rem; padding: 1rem; background: rgba(59, 130, 246, 0.1); border-radius: 0.5rem; border: 1px solid rgba(59, 130, 246, 0.3);",

                    p {
                        style: "color: #9ca3af; margin: 0 0 0.5rem 0; font-size: 0.875rem;",
                        "Character Modifier"
                    }

                    div {
                        style: "font-size: 2rem; font-weight: bold; color: #3b82f6;",
                        if props.character_modifier >= 0 {
                            "+{props.character_modifier}"
                        } else {
                            "{props.character_modifier}"
                        }
                    }
                }

                // Roll section
                if let Some(roll) = *roll_result.read() {
                    // Show roll result
                    div {
                        style: "margin-bottom: 1.5rem;",

                        div {
                            style: "text-align: center; margin-bottom: 1rem;",

                            div {
                                style: "font-size: 3.5rem; font-weight: bold; color: #f59e0b; margin-bottom: 0.5rem; text-shadow: 0 0 10px rgba(245, 158, 11, 0.5);",
                                "🎲 {roll}"
                            }

                            div {
                                style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;",
                                "d20 Roll"
                            }
                        }

                        // Calculation breakdown
                        div {
                            style: "background: rgba(0, 0, 0, 0.3); padding: 1rem; border-radius: 0.5rem; margin-bottom: 1rem;",

                            div {
                                style: "display: flex; justify-content: space-between; margin-bottom: 0.5rem;",

                                span { style: "color: #9ca3af;", "Roll:" }
                                span { style: "color: white; font-weight: bold;", "{roll}" }
                            }

                            div {
                                style: "display: flex; justify-content: space-between; margin-bottom: 0.5rem;",

                                span { style: "color: #9ca3af;", "Modifier:" }
                                span {
                                    style: "color: #3b82f6; font-weight: bold;",
                                    if props.character_modifier >= 0 {
                                        "+{props.character_modifier}"
                                    } else {
                                        "{props.character_modifier}"
                                    }
                                }
                            }

                            div {
                                style: "border-top: 1px solid rgba(255, 255, 255, 0.1); padding-top: 0.5rem; display: flex; justify-content: space-between;",

                                span { style: "color: #9ca3af; font-weight: bold;", "Total:" }
                                span {
                                    style: "color: #22c55e; font-weight: bold; font-size: 1.125rem;",
                                    "{roll + props.character_modifier}"
                                }
                            }
                        }

                        // Submit button
                        button {
                            onclick: move |_| {
                                props.on_roll.call(roll);
                            },
                            style: "width: 100%; padding: 1rem; background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%); color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 1rem; font-weight: 600; transition: all 0.2s;",
                            "Submit Roll"
                        }
                    }
                } else {
                    // Roll button
                    button {
                        onclick: do_roll,
                        disabled: *is_rolling.read(),
                        style: "width: 100%; padding: 2rem; background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 1.5rem; font-weight: bold; transition: all 0.2s;",
                        class: if *is_rolling.read() { "rolling" } else { "" },

                        if *is_rolling.read() {
                            "Rolling..."
                        } else {
                            "🎲 Roll d20"
                        }
                    }

                    // Info text
                    p {
                        style: "color: #9ca3af; font-size: 0.875rem; text-align: center; margin: 1rem 0 0 0;",
                        "Click to roll the dice and test your character's skill"
                    }
                }
            }
        }
    }
}
