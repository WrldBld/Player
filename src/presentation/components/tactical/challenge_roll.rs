//! Challenge Roll Modal Component
//!
//! Displays a modal for performing challenge rolls with configurable dice systems.
//! Supports both formula-based rolls (e.g., "1d20+5") and manual result entry
//! for physical dice rolls.

use dioxus::prelude::*;
use crate::application::dto::websocket_messages::DiceInputType;
use crate::application::ports::outbound::Platform;

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
    /// Suggested dice formula based on rule system (e.g., "1d20", "1d100", "2d6")
    #[props(default)]
    pub suggested_dice: Option<String>,
    /// Human-readable hint about the rule system
    #[props(default)]
    pub rule_system_hint: Option<String>,
    /// Called with the dice input when roll is submitted
    pub on_roll: EventHandler<DiceInputType>,
    /// Called when modal should close
    pub on_close: EventHandler<()>,
}

/// ChallengeRollModal component
///
/// Displays a modal with:
/// - Challenge name, skill, and difficulty
/// - Mode toggle (formula vs manual)
/// - Dice formula input or manual result input
/// - Roll result display with breakdown
/// - Submit button to send result back to engine
#[component]
pub fn ChallengeRollModal(props: ChallengeRollModalProps) -> Element {
    // Input mode: true = use formula roll, false = manual input
    let mut use_formula_mode = use_signal(|| true);
    // Formula input text (e.g., "1d20+5")
    let default_formula = props.suggested_dice.clone().unwrap_or_else(|| "1d20".to_string());
    let mut formula_input = use_signal(move || default_formula.clone());
    // Manual result input
    let mut manual_input = use_signal(|| "".to_string());
    // Roll result state
    let mut roll_result = use_signal(|| None::<RollDisplayState>);
    let mut is_rolling = use_signal(|| false);
    // Error message
    let mut error_message = use_signal(|| None::<String>);

    let platform = use_context::<Platform>();

    // Parse dice formula (simple XdY+Z pattern)
    let parse_formula = |formula: &str| -> Result<(u8, u8, i32), String> {
        let formula = formula.trim().to_lowercase();

        // Match patterns like "1d20", "2d6+3", "1d20-2"
        let re_pattern = regex_lite::Regex::new(r"^(\d+)d(\d+)([+-]\d+)?$")
            .map_err(|_| "Invalid regex".to_string())?;

        if let Some(caps) = re_pattern.captures(&formula) {
            let count: u8 = caps.get(1)
                .and_then(|m| m.as_str().parse().ok())
                .ok_or("Invalid dice count")?;
            let sides: u8 = caps.get(2)
                .and_then(|m| m.as_str().parse().ok())
                .ok_or("Invalid die size")?;
            let modifier: i32 = caps.get(3)
                .map(|m| m.as_str().parse().unwrap_or(0))
                .unwrap_or(0);

            if count == 0 || count > 20 {
                return Err("Dice count must be 1-20".to_string());
            }
            if sides == 0 || sides > 100 {
                return Err("Die size must be 1-100".to_string());
            }

            Ok((count, sides, modifier))
        } else {
            Err(format!("Invalid formula format. Use XdY or XdY+Z (e.g., 1d20, 2d6+3)"))
        }
    };

    // Perform dice roll with formula
    let do_formula_roll = {
        let formula_input = formula_input.clone();
        let platform = platform.clone();
        move |_| {
            let formula = formula_input.read().clone();
            match parse_formula(&formula) {
                Ok((count, sides, modifier)) => {
                    is_rolling.set(true);
                    error_message.set(None);

                    // Roll each die
                    let mut rolls = Vec::new();
                    for _ in 0..count {
                        rolls.push(platform.random_range(1, sides as i32));
                    }
                    let dice_total: i32 = rolls.iter().sum();
                    let total = dice_total + modifier + props.character_modifier;

                    roll_result.set(Some(RollDisplayState {
                        formula: formula.clone(),
                        individual_rolls: rolls,
                        dice_total,
                        formula_modifier: modifier,
                        character_modifier: props.character_modifier,
                        total,
                        is_manual: false,
                    }));

                    is_rolling.set(false);
                }
                Err(e) => {
                    error_message.set(Some(e));
                }
            }
        }
    };

    // Get suggested dice display
    let suggested_dice_display = props.suggested_dice.clone().unwrap_or_else(|| "1d20".to_string());
    let rule_hint = props.rule_system_hint.clone();

    rsx! {
        // Modal overlay
        div {
            id: "challenge-overlay",
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| {
                props.on_close.call(());
            },

            // Modal content - stop propagation to prevent closing when clicking inside
            div {
                id: "challenge-modal",
                style: "background: linear-gradient(135deg, #1a1a2e 0%, #0f0f23 100%); padding: 2rem; border-radius: 1rem; max-width: 500px; width: 90%; border: 2px solid #f59e0b; box-shadow: 0 20px 60px rgba(245, 158, 11, 0.2);",
                onclick: |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;",

                    h2 {
                        style: "color: #f59e0b; margin: 0; font-size: 1.5rem;",
                        "Skill Challenge"
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
                    style: "display: flex; justify-content: space-between; margin-bottom: 1rem; padding: 1rem; background: rgba(0, 0, 0, 0.3); border-radius: 0.5rem;",

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

                    div {
                        span {
                            style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; display: block; margin-bottom: 0.25rem;",
                            "Modifier"
                        }
                        span {
                            style: "color: #3b82f6; font-weight: bold;",
                            if props.character_modifier >= 0 {
                                "+{props.character_modifier}"
                            } else {
                                "{props.character_modifier}"
                            }
                        }
                    }
                }

                // Rule system hint
                if let Some(hint) = &rule_hint {
                    p {
                        style: "color: #9ca3af; font-size: 0.75rem; text-align: center; margin: 0 0 1rem 0; font-style: italic;",
                        "{hint}"
                    }
                }

                // Mode toggle
                div {
                    style: "display: flex; gap: 0.5rem; margin-bottom: 1rem;",

                    button {
                        onclick: move |_| use_formula_mode.set(true),
                        style: if *use_formula_mode.read() {
                            "flex: 1; padding: 0.75rem; background: #f59e0b; color: white; border: none; border-radius: 0.5rem 0 0 0.5rem; cursor: pointer; font-weight: 600;"
                        } else {
                            "flex: 1; padding: 0.75rem; background: rgba(255, 255, 255, 0.1); color: #9ca3af; border: 1px solid rgba(255, 255, 255, 0.2); border-radius: 0.5rem 0 0 0.5rem; cursor: pointer;"
                        },
                        "Digital Roll"
                    }

                    button {
                        onclick: move |_| use_formula_mode.set(false),
                        style: if !*use_formula_mode.read() {
                            "flex: 1; padding: 0.75rem; background: #f59e0b; color: white; border: none; border-radius: 0 0.5rem 0.5rem 0; cursor: pointer; font-weight: 600;"
                        } else {
                            "flex: 1; padding: 0.75rem; background: rgba(255, 255, 255, 0.1); color: #9ca3af; border: 1px solid rgba(255, 255, 255, 0.2); border-radius: 0 0.5rem 0.5rem 0; cursor: pointer;"
                        },
                        "Physical Dice"
                    }
                }

                // Roll/Input section
                if let Some(result) = roll_result.read().clone() {
                    // Show roll result
                    RollResultDisplay {
                        result: result.clone(),
                        on_submit: move |_| {
                            let result = result.clone();
                            if result.is_manual {
                                props.on_roll.call(DiceInputType::Manual(result.total - props.character_modifier));
                            } else {
                                props.on_roll.call(DiceInputType::Formula(result.formula.clone()));
                            }
                        },
                        on_reroll: move |_| {
                            roll_result.set(None);
                            error_message.set(None);
                        },
                    }
                } else if *use_formula_mode.read() {
                    // Formula input mode
                    div {
                        // Formula input
                        div {
                            style: "margin-bottom: 1rem;",

                            label {
                                style: "color: #9ca3af; font-size: 0.75rem; display: block; margin-bottom: 0.5rem;",
                                "Dice Formula (e.g., {suggested_dice_display})"
                            }

                            input {
                                r#type: "text",
                                value: "{formula_input}",
                                oninput: move |e| formula_input.set(e.value().to_string()),
                                placeholder: "{suggested_dice_display}",
                                style: "width: 100%; padding: 1rem; background: rgba(0, 0, 0, 0.3); border: 1px solid rgba(255, 255, 255, 0.2); border-radius: 0.5rem; color: white; font-size: 1.25rem; text-align: center; font-family: monospace; box-sizing: border-box;",
                            }
                        }

                        // Error message
                        if let Some(err) = error_message.read().as_ref() {
                            p {
                                style: "color: #ef4444; font-size: 0.875rem; text-align: center; margin: 0 0 1rem 0;",
                                "{err}"
                            }
                        }

                        // Roll button
                        button {
                            onclick: do_formula_roll,
                            disabled: *is_rolling.read(),
                            style: "width: 100%; padding: 1.5rem; background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 1.25rem; font-weight: bold; transition: all 0.2s;",

                            if *is_rolling.read() {
                                "Rolling..."
                            } else {
                                "Roll"
                            }
                        }
                    }
                } else {
                    // Manual input mode
                    div {
                        p {
                            style: "color: #9ca3af; font-size: 0.875rem; text-align: center; margin: 0 0 1rem 0;",
                            "Enter the result from your physical dice roll"
                        }

                        div {
                            style: "margin-bottom: 1rem;",

                            label {
                                style: "color: #9ca3af; font-size: 0.75rem; display: block; margin-bottom: 0.5rem;",
                                "Dice Result (before modifiers)"
                            }

                            input {
                                r#type: "number",
                                value: "{manual_input}",
                                oninput: move |e| manual_input.set(e.value().to_string()),
                                placeholder: "Enter roll result",
                                style: "width: 100%; padding: 1rem; background: rgba(0, 0, 0, 0.3); border: 1px solid rgba(255, 255, 255, 0.2); border-radius: 0.5rem; color: white; font-size: 1.5rem; text-align: center; box-sizing: border-box;",
                            }
                        }

                        // Error message
                        if let Some(err) = error_message.read().as_ref() {
                            p {
                                style: "color: #ef4444; font-size: 0.875rem; text-align: center; margin: 0 0 1rem 0;",
                                "{err}"
                            }
                        }

                        // Submit button
                        button {
                            onclick: move |_| {
                                let input = manual_input.read().clone();
                                match input.trim().parse::<i32>() {
                                    Ok(value) if value >= 1 => {
                                        error_message.set(None);
                                        let total = value + props.character_modifier;
                                        roll_result.set(Some(RollDisplayState {
                                            formula: "Manual".to_string(),
                                            individual_rolls: vec![value],
                                            dice_total: value,
                                            formula_modifier: 0,
                                            character_modifier: props.character_modifier,
                                            total,
                                            is_manual: true,
                                        }));
                                    }
                                    Ok(_) => {
                                        error_message.set(Some("Value must be at least 1".to_string()));
                                    }
                                    Err(_) => {
                                        error_message.set(Some("Please enter a valid number".to_string()));
                                    }
                                }
                            },
                            disabled: manual_input.read().is_empty(),
                            style: "width: 100%; padding: 1.5rem; background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 1.25rem; font-weight: bold; transition: all 0.2s;",
                            "Submit Result"
                        }
                    }
                }
            }
        }
    }
}

/// State for displaying roll results
#[derive(Clone, PartialEq)]
struct RollDisplayState {
    formula: String,
    individual_rolls: Vec<i32>,
    dice_total: i32,
    formula_modifier: i32,
    character_modifier: i32,
    total: i32,
    is_manual: bool,
}

/// Component for displaying roll results
#[component]
fn RollResultDisplay(
    result: RollDisplayState,
    on_submit: EventHandler<()>,
    on_reroll: EventHandler<()>,
) -> Element {
    // Format individual rolls for display
    let rolls_display = result.individual_rolls.iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    // Determine if this is a critical (natural 20 or 1 on d20)
    let is_nat_20 = result.individual_rolls.len() == 1 && result.individual_rolls[0] == 20;
    let is_nat_1 = result.individual_rolls.len() == 1 && result.individual_rolls[0] == 1;

    rsx! {
        div {
            style: "margin-bottom: 1rem;",

            // Roll result display
            div {
                style: "text-align: center; margin-bottom: 1rem;",

                div {
                    style: if is_nat_20 {
                        "font-size: 3rem; font-weight: bold; color: #22c55e; margin-bottom: 0.5rem; text-shadow: 0 0 20px rgba(34, 197, 94, 0.7);"
                    } else if is_nat_1 {
                        "font-size: 3rem; font-weight: bold; color: #ef4444; margin-bottom: 0.5rem; text-shadow: 0 0 20px rgba(239, 68, 68, 0.7);"
                    } else {
                        "font-size: 3rem; font-weight: bold; color: #f59e0b; margin-bottom: 0.5rem; text-shadow: 0 0 10px rgba(245, 158, 11, 0.5);"
                    },

                    if is_nat_20 {
                        "Natural 20!"
                    } else if is_nat_1 {
                        "Natural 1!"
                    } else if result.is_manual {
                        "Manual: {result.dice_total}"
                    } else {
                        "{result.formula}({rolls_display})"
                    }
                }
            }

            // Calculation breakdown
            div {
                style: "background: rgba(0, 0, 0, 0.3); padding: 1rem; border-radius: 0.5rem; margin-bottom: 1rem;",

                div {
                    style: "display: flex; justify-content: space-between; margin-bottom: 0.5rem;",

                    span { style: "color: #9ca3af;", "Dice:" }
                    span { style: "color: white; font-weight: bold;", "{result.dice_total}" }
                }

                if result.formula_modifier != 0 {
                    div {
                        style: "display: flex; justify-content: space-between; margin-bottom: 0.5rem;",

                        span { style: "color: #9ca3af;", "Formula Mod:" }
                        span {
                            style: "color: #a855f7; font-weight: bold;",
                            if result.formula_modifier >= 0 {
                                "+{result.formula_modifier}"
                            } else {
                                "{result.formula_modifier}"
                            }
                        }
                    }
                }

                div {
                    style: "display: flex; justify-content: space-between; margin-bottom: 0.5rem;",

                    span { style: "color: #9ca3af;", "Skill Mod:" }
                    span {
                        style: "color: #3b82f6; font-weight: bold;",
                        if result.character_modifier >= 0 {
                            "+{result.character_modifier}"
                        } else {
                            "{result.character_modifier}"
                        }
                    }
                }

                div {
                    style: "border-top: 1px solid rgba(255, 255, 255, 0.1); padding-top: 0.5rem; display: flex; justify-content: space-between;",

                    span { style: "color: #9ca3af; font-weight: bold;", "Total:" }
                    span {
                        style: "color: #22c55e; font-weight: bold; font-size: 1.25rem;",
                        "{result.total}"
                    }
                }
            }

            // Action buttons
            div {
                style: "display: flex; gap: 0.5rem;",

                button {
                    onclick: move |_| on_reroll.call(()),
                    style: "flex: 1; padding: 1rem; background: rgba(255, 255, 255, 0.1); color: #9ca3af; border: 1px solid rgba(255, 255, 255, 0.2); border-radius: 0.5rem; cursor: pointer; font-size: 1rem;",
                    "Re-roll"
                }

                button {
                    onclick: move |_| on_submit.call(()),
                    style: "flex: 2; padding: 1rem; background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%); color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 1rem; font-weight: 600;",
                    "Submit Roll"
                }
            }
        }
    }
}
