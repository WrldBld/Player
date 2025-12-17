//! NPC motivation tracking component
//!
//! Displays and allows editing of NPC mood and immediate goals.

use dioxus::prelude::*;

use crate::application::dto::websocket_messages::CharacterData;
use crate::presentation::components::dm_panel::director_generate_modal::DirectorGenerateModal;

/// NPC motivation state
#[derive(Clone, PartialEq)]
pub struct Motivation {
    /// Current mood/emotional state
    pub mood: String,
    /// Immediate goal the NPC is pursuing
    pub goal: String,
}

/// Props for the NPCMotivation component
#[derive(Props, Clone, PartialEq)]
pub struct NPCMotivationProps {
    /// The character being tracked
    pub character: CharacterData,
    /// Current motivation state
    pub motivation: Motivation,
    /// Handler called when motivation is updated
    pub on_update: EventHandler<Motivation>,
    /// Optional character description for pre-populating generation prompts
    #[props(default)]
    pub character_description: Option<String>,
}

/// Mood options available for selection
const MOOD_OPTIONS: &[&str] = &[
    "Friendly",
    "Neutral",
    "Suspicious",
    "Hostile",
    "Curious",
    "Fearful",
    "Greedy",
    "Dutiful",
    "Conflicted",
    "Determined",
];

/// NPCMotivation component - Track NPC mood and goals
///
/// Displays NPC name with editable mood dropdown and goal text input.
/// Helps DMs manage NPC state during gameplay.
#[component]
pub fn NPCMotivation(props: NPCMotivationProps) -> Element {
    let char_name = props.character.name.clone();
    let char_id = props.character.id.clone();
    let motivation_mood = props.motivation.mood.clone();
    let motivation_goal = props.motivation.goal.clone();
    let mut show_generate_modal = use_signal(|| false);
    let mut generate_asset_type = use_signal(|| "portrait".to_string());

    // Clone for each closure to avoid move conflicts
    let motivation_for_mood = props.motivation.clone();
    let motivation_for_goal = props.motivation.clone();

    rsx! {
        div {
            class: "npc-motivation",
            style: "padding: 0.75rem; background: #0f0f23; border-radius: 0.5rem; margin-bottom: 0.5rem; border-left: 3px solid #8b5cf6;",

            // Character name header
            h4 {
                style: "color: #8b5cf6; font-size: 0.875rem; margin: 0 0 0.75rem 0;",
                "{char_name}"
            }

            // Mood selector
            div {
                style: "margin-bottom: 0.75rem;",

                label {
                    style: "display: block; color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                    "Mood"
                }

                select {
                    value: "{motivation_mood}",
                    onchange: move |e| {
                        let mut updated = motivation_for_mood.clone();
                        updated.mood = e.value();
                        props.on_update.call(updated);
                    },
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem; cursor: pointer;",

                    for mood in MOOD_OPTIONS.iter() {
                        option {
                            value: "{mood}",
                            "{mood}"
                        }
                    }
                    option {
                        value: "Custom",
                        "Custom..."
                    }
                }
            }

            // Goal input
            div {
                style: "margin-bottom: 0.75rem;",
                label {
                    style: "display: block; color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem;",
                    "Immediate Goal"
                }

                input {
                    r#type: "text",
                    value: "{motivation_goal}",
                    placeholder: "What does this NPC want right now?",
                    oninput: move |e| {
                        let mut updated = motivation_for_goal.clone();
                        updated.goal = e.value();
                        props.on_update.call(updated);
                    },
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem; box-sizing: border-box; transition: border-color 0.2s;",
                }
            }

            // Generate asset buttons
            div {
                style: "display: flex; gap: 0.5rem; margin-top: 0.75rem;",
                button {
                    onclick: move |_| {
                        generate_asset_type.set("portrait".to_string());
                        show_generate_modal.set(true);
                    },
                    style: "flex: 1; padding: 0.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.75rem; font-weight: 500;",
                    "🎨 Generate Portrait"
                }
                button {
                    onclick: move |_| {
                        generate_asset_type.set("sprite".to_string());
                        show_generate_modal.set(true);
                    },
                    style: "flex: 1; padding: 0.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.75rem; font-weight: 500;",
                    "🖼️ Generate Sprite"
                }
            }
        }

        // Generate modal
        if *show_generate_modal.read() {
            DirectorGenerateModal {
                entity_type: "character".to_string(),
                entity_id: char_id.clone(),
                asset_type: generate_asset_type.read().clone(),
                character_name: char_name.clone(),
                initial_prompt: props.character_description.clone().unwrap_or_default(),
                on_close: move |_| show_generate_modal.set(false),
            }
        }
    }
}

