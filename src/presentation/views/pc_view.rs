//! Player Character View - Visual novel style gameplay
//!
//! Main view for players, displaying the visual novel interface
//! with backdrop, character sprites, dialogue, and choices.

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::domain::entities::PlayerAction;
use crate::application::dto::{FieldValue, SheetTemplate, InteractionData};
use crate::presentation::components::action_panel::ActionPanel;
use crate::presentation::components::character_sheet_viewer::CharacterSheetViewer;
use crate::presentation::components::tactical::ChallengeRollModal;
use crate::presentation::components::visual_novel::{Backdrop, CharacterLayer, DialogueBox, EmptyDialogueBox};
use crate::presentation::services::{use_character_service, use_world_service};
use crate::presentation::state::{use_dialogue_state, use_game_state, use_session_state, use_typewriter_effect};

/// Props for PCView
#[derive(Props, Clone, PartialEq)]
pub struct PCViewProps {
    /// Handler for back button
    pub on_back: EventHandler<()>,
}

/// Player Character View - visual novel gameplay interface
#[component]
pub fn PCView(props: PCViewProps) -> Element {
    // Get global state from context
    let game_state = use_game_state();
    let mut dialogue_state = use_dialogue_state();
    let session_state = use_session_state();

    // Get services
    let world_service = use_world_service();
    let character_service = use_character_service();

    // Character sheet viewer state
    let mut show_character_sheet = use_signal(|| false);
    let mut character_sheet_template: Signal<Option<SheetTemplate>> = use_signal(|| None);
    let mut character_sheet_values: Signal<HashMap<String, FieldValue>> = use_signal(HashMap::new);
    let mut player_character_name = use_signal(|| "Your Character".to_string());
    let mut selected_character_id: Signal<Option<String>> = use_signal(|| None);
    let mut is_loading_sheet = use_signal(|| false);

    // Run typewriter effect
    use_typewriter_effect(&mut dialogue_state);

    // Read scene characters from game state (reactive)
    let scene_characters = game_state.scene_characters.read().clone();

    // Get current dialogue state
    let speaker_name = dialogue_state.speaker_name.read().clone();
    let displayed_text = dialogue_state.displayed_text.read().clone();
    let is_typing = *dialogue_state.is_typing.read();
    let choices = dialogue_state.choices.read().clone();
    let has_dialogue = dialogue_state.has_dialogue();
    let is_llm_processing = *dialogue_state.is_llm_processing.read();

    // Get interactions from game state
    let interactions = game_state.interactions.read().clone();

    // Get active challenge if any
    let active_challenge = session_state.active_challenge.read().clone();

    // Check if connected
    let is_connected = session_state.connection_status.read().is_connected();

    rsx! {
        div {
            class: "pc-view",
            style: "height: 100%; display: flex; flex-direction: column; position: relative;",

            // Back button
            button {
                onclick: move |_| props.on_back.call(()),
                style: "position: absolute; top: 1rem; left: 1rem; z-index: 100; padding: 0.5rem 1rem; background: rgba(0,0,0,0.5); color: white; border: 1px solid #374151; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                "< Back"
            }

            // Connection status and location indicator
            div {
                style: "position: absolute; top: 1rem; right: 1rem; z-index: 100; display: flex; flex-direction: column; gap: 0.5rem; align-items: flex-end;",
                
                // Location name
                if let Some(scene) = game_state.current_scene.read().as_ref() {
                    div {
                        style: "padding: 0.5rem 1rem; background: rgba(0,0,0,0.7); color: white; border-radius: 0.5rem; font-size: 0.875rem; font-weight: 500;",
                        "📍 {scene.location_name}"
                    }
                }
                
                // Connection status
            if !is_connected {
                div {
                        style: "padding: 0.5rem 1rem; background: rgba(239,68,68,0.8); color: white; border-radius: 0.5rem; font-size: 0.75rem;",
                    "Disconnected"
                    }
                }
            }

            // Visual novel stage
            Backdrop {
                image_url: game_state.backdrop_url(),

                // Character layer with real scene characters
                CharacterLayer {
                    characters: scene_characters,
                    on_character_click: {
                        let session_state = session_state.clone();
                        move |character_id: String| {
                            tracing::info!("Clicked character: {}", character_id);
                            // Send a talk action when clicking a character
                            send_player_action(
                                &session_state,
                                PlayerAction::talk(&character_id, None),
                            );
                        }
                    }
                }
            }

            // Dialogue box (fixed at bottom)
            div {
                class: "dialogue-container",
                style: "position: absolute; bottom: 0; left: 0; right: 0; z-index: 10;",

                if has_dialogue {
                    DialogueBox {
                        speaker_name: speaker_name,
                        dialogue_text: displayed_text,
                        is_typing: is_typing,
                        is_llm_processing: is_llm_processing,
                        choices: choices,
                        on_choice_selected: {
                            let session_state = session_state.clone();
                            let mut dialogue_state = dialogue_state.clone();
                            move |choice_id: String| {
                                handle_choice_selected(&session_state, &mut dialogue_state, &choice_id);
                            }
                        },
                        on_custom_input: {
                            let session_state = session_state.clone();
                            let mut dialogue_state = dialogue_state.clone();
                            move |text: String| {
                                handle_custom_input(&session_state, &mut dialogue_state, &text);
                            }
                        },
                        on_advance: {
                            let mut dialogue_state = dialogue_state.clone();
                            move |_| {
                                handle_advance(&mut dialogue_state);
                            }
                        },
                    }
                } else {
                    EmptyDialogueBox {}
                }
            }

            // Action panel with scene interactions (disabled while LLM is processing)
            ActionPanel {
                interactions: interactions,
                disabled: is_llm_processing,
                on_interaction: {
                    let session_state = session_state.clone();
                    move |interaction: InteractionData| {
                        handle_interaction(&session_state, &interaction);
                    }
                },
                on_inventory: Some(EventHandler::new(move |_| {
                    tracing::info!("Open inventory");
                })),
                on_character: Some(EventHandler::new({
                    let game_state = game_state.clone();
                    let world_service = world_service.clone();
                    let character_service = character_service.clone();
                    move |_| {
                        tracing::info!("Open character sheet");
                        // Show the modal first (loading state)
                        show_character_sheet.set(true);
                        is_loading_sheet.set(true);

                        // Get world ID and first available character
                        let world_id = game_state.world.read().as_ref()
                            .map(|w| w.world.id.clone());
                        let characters = game_state.world.read().as_ref()
                            .map(|w| w.characters.clone())
                            .unwrap_or_default();

                        // Auto-select first character if none selected
                        let char_id = selected_character_id.read().clone()
                            .or_else(|| characters.first().map(|c| c.id.clone()));

                        if let (Some(wid), Some(cid)) = (world_id, char_id.clone()) {
                            selected_character_id.set(Some(cid.clone()));
                            let world_svc = world_service.clone();
                            let char_svc = character_service.clone();
                            spawn(async move {
                                // Load template
                                match world_svc.get_sheet_template(&wid).await {
                                    Ok(template_json) => {
                                        if let Ok(template) = serde_json::from_value::<SheetTemplate>(template_json) {
                                            character_sheet_template.set(Some(template));
                                        }
                                    }
                                    Err(e) => tracing::warn!("Failed to load sheet template: {}", e),
                                }
                                // Load character data
                                match char_svc.get_character(&cid).await {
                                    Ok(char_data) => {
                                        player_character_name.set(char_data.name);
                                        if let Some(sheet_data) = char_data.sheet_data {
                                            character_sheet_values.set(sheet_data.values);
                                        }
                                    }
                                    Err(e) => tracing::warn!("Failed to load character: {}", e),
                                }
                                is_loading_sheet.set(false);
                            });
                        } else {
                            is_loading_sheet.set(false);
                        }
                    }
                })),
                on_map: Some(EventHandler::new(move |_| {
                    tracing::info!("Open map");
                })),
                on_log: Some(EventHandler::new(move |_| {
                    tracing::info!("Open log");
                })),
            }

            // Character sheet viewer modal
            if *show_character_sheet.read() {
                if *is_loading_sheet.read() {
                    // Loading state
                    div {
                        class: "character-sheet-overlay",
                        style: "position: fixed; inset: 0; background: rgba(0,0,0,0.85); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 2rem;",
                        onclick: move |_| show_character_sheet.set(false),

                        div {
                            style: "background: #1a1a2e; border-radius: 1rem; padding: 2rem; max-width: 400px; text-align: center;",
                            onclick: move |e| e.stop_propagation(),

                            div {
                                style: "color: #9ca3af; font-size: 1.25rem;",
                                "Loading character sheet..."
                            }
                        }
                    }
                } else if let Some(template) = character_sheet_template.read().as_ref() {
                    CharacterSheetViewer {
                        character_name: player_character_name.read().clone(),
                        template: template.clone(),
                        values: character_sheet_values.read().clone(),
                        on_close: move |_| show_character_sheet.set(false),
                    }
                } else {
                    // No template loaded - show placeholder with character selection
                    {
                        let characters = game_state.world.read().as_ref()
                            .map(|w| w.characters.clone())
                            .unwrap_or_default();
                        rsx! {
                            div {
                                class: "character-sheet-overlay",
                                style: "position: fixed; inset: 0; background: rgba(0,0,0,0.85); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 2rem;",
                                onclick: move |_| show_character_sheet.set(false),

                                div {
                                    style: "background: #1a1a2e; border-radius: 1rem; padding: 2rem; max-width: 400px; text-align: center;",
                                    onclick: move |e| e.stop_propagation(),

                                    h2 {
                                        style: "color: #f3f4f6; margin: 0 0 1rem 0;",
                                        "Character Sheet"
                                    }

                                    if characters.is_empty() {
                                        p {
                                            style: "color: #9ca3af; margin: 0 0 1.5rem 0;",
                                            "No characters available in this world."
                                        }
                                    } else {
                                        p {
                                            style: "color: #9ca3af; margin: 0 0 1.5rem 0;",
                                            "No character sheet template available for this world. The DM may need to configure the rule system."
                                        }
                                    }

                                    button {
                                        onclick: move |_| show_character_sheet.set(false),
                                        style: "padding: 0.5rem 1.5rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                                        "Close"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Challenge roll modal
            if let Some(challenge) = active_challenge {
                ChallengeRollModal {
                    challenge_id: challenge.challenge_id.clone(),
                    challenge_name: challenge.challenge_name.clone(),
                    skill_name: challenge.skill_name.clone(),
                    difficulty_display: challenge.difficulty_display.clone(),
                    description: challenge.description.clone(),
                    character_modifier: challenge.character_modifier,
                    on_roll: {
                        let session_state = session_state.clone();
                        let challenge_id = challenge.challenge_id.clone();
                        move |roll: i32| {
                            send_challenge_roll(&session_state, &challenge_id, roll);
                        }
                    },
                    on_close: {
                        let mut session_state = session_state.clone();
                        move |_| {
                            session_state.clear_active_challenge();
                        }
                    },
                }
            }
        }
    }
}

/// Send a player action via WebSocket
fn send_player_action(
    session_state: &crate::presentation::state::SessionState,
    action: PlayerAction,
) {
    let client_binding = session_state.engine_client.read();
    if let Some(ref client) = *client_binding {
        let svc = crate::application::services::ActionService::new(std::sync::Arc::clone(client));
        if let Err(e) = svc.send_action(action) {
            tracing::error!("Failed to send action: {}", e);
        }
    } else {
        tracing::warn!("Cannot send action: not connected to server");
    }
}

/// Handle a dialogue choice being selected
fn handle_choice_selected(
    session_state: &crate::presentation::state::SessionState,
    dialogue_state: &mut crate::presentation::state::DialogueState,
    choice_id: &str,
) {
    tracing::info!("Choice selected: {}", choice_id);

    // Clear awaiting state since we're making a choice
    dialogue_state.awaiting_input.set(false);

    // Send dialogue choice action to the server
    send_player_action(session_state, PlayerAction::dialogue_choice(choice_id));
}

/// Handle custom text input
fn handle_custom_input(
    session_state: &crate::presentation::state::SessionState,
    dialogue_state: &mut crate::presentation::state::DialogueState,
    text: &str,
) {
    tracing::info!("Custom input: {}", text);

    // Clear awaiting state
    dialogue_state.awaiting_input.set(false);

    // Send custom action to the server
    send_player_action(session_state, PlayerAction::custom(text));
}

/// Handle advancing dialogue (clicking to continue or skipping typewriter)
fn handle_advance(dialogue_state: &mut crate::presentation::state::DialogueState) {
    if *dialogue_state.is_typing.read() {
        // Skip typewriter animation
        dialogue_state.skip_typewriter();
    } else {
        // If no choices and dialogue is done, the server will send next content
        if !dialogue_state.has_choices() {
            tracing::info!("Dialogue complete, awaiting server response");
        }
    }
}

/// Handle an interaction being selected from the action panel
fn handle_interaction(
    session_state: &crate::presentation::state::SessionState,
    interaction: &InteractionData,
) {
    tracing::info!("Selected interaction: {} ({})", interaction.name, interaction.interaction_type);

    // Convert interaction type to player action
    let action = match interaction.interaction_type.to_lowercase().as_str() {
        "talk" | "dialogue" | "speak" => {
            PlayerAction::talk(&interaction.id, None)
        }
        "examine" | "look" | "inspect" => {
            PlayerAction::examine(&interaction.id)
        }
        "travel" | "go" | "move" => {
            PlayerAction::travel(&interaction.id)
        }
        "use" | "interact" => {
            // Use the interaction ID as both item and target for generic "use"
            PlayerAction::use_item(&interaction.id, interaction.target_name.as_deref())
        }
        _ => {
            // For unknown interaction types, send as custom action
            PlayerAction::custom_targeted(&interaction.id, &interaction.name)
        }
    };

    send_player_action(session_state, action);
}

/// Send a challenge roll via WebSocket
fn send_challenge_roll(
    session_state: &crate::presentation::state::SessionState,
    challenge_id: &str,
    roll: i32,
) {
    let client_binding = session_state.engine_client.read();
    if let Some(ref client) = *client_binding {
        let svc = crate::application::services::SessionCommandService::new(std::sync::Arc::clone(client));
        if let Err(e) = svc.submit_challenge_roll(challenge_id, roll) {
            tracing::error!("Failed to send challenge roll: {}", e);
        }
    } else {
        tracing::warn!("Cannot send challenge roll: not connected to server");
    }
}
