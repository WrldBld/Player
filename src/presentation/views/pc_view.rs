//! Player Character View - Visual novel style gameplay
//!
//! Main view for players, displaying the visual novel interface
//! with backdrop, character sprites, dialogue, and choices.

use dioxus::prelude::*;

use crate::domain::entities::PlayerAction;
use crate::infrastructure::websocket::InteractionData;
use crate::presentation::components::action_panel::ActionPanel;
use crate::presentation::components::visual_novel::{Backdrop, CharacterLayer, DialogueBox, EmptyDialogueBox};
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

    // Get interactions from game state
    let interactions = game_state.interactions.read().clone();

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

            // Connection status indicator
            if !is_connected {
                div {
                    style: "position: absolute; top: 1rem; right: 1rem; z-index: 100; padding: 0.5rem 1rem; background: rgba(239,68,68,0.8); color: white; border-radius: 0.5rem; font-size: 0.75rem;",
                    "Disconnected"
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

            // Action panel with scene interactions
            ActionPanel {
                interactions: interactions,
                on_interaction: {
                    let session_state = session_state.clone();
                    move |interaction: InteractionData| {
                        handle_interaction(&session_state, &interaction);
                    }
                },
                on_inventory: Some(EventHandler::new(move |_| {
                    tracing::info!("Open inventory");
                })),
                on_character: Some(EventHandler::new(move |_| {
                    tracing::info!("Open character sheet");
                })),
                on_map: Some(EventHandler::new(move |_| {
                    tracing::info!("Open map");
                })),
                on_log: Some(EventHandler::new(move |_| {
                    tracing::info!("Open log");
                })),
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
        let action_type = action.action_type.as_str();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = client.clone();
            let target = action.target.clone();
            let dialogue = action.dialogue.clone();

            tokio::spawn(async move {
                if let Err(e) = client
                    .send_action(action_type, target.as_deref(), dialogue.as_deref())
                    .await
                {
                    tracing::error!("Failed to send action: {}", e);
                }
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Err(e) = client.send_action(
                action_type,
                action.target.as_deref(),
                action.dialogue.as_deref(),
            ) {
                tracing::error!("Failed to send action: {}", e);
            }
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
