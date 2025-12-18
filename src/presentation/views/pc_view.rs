//! Player Character View - Visual novel style gameplay
//!
//! Main view for players, displaying the visual novel interface
//! with backdrop, character sprites, dialogue, and choices.

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::domain::entities::PlayerAction;
use crate::application::dto::{FieldValue, SheetTemplate, InteractionData, DiceInputType};
use crate::presentation::components::action_panel::ActionPanel;
use crate::presentation::components::character_sheet_viewer::CharacterSheetViewer;
use crate::presentation::components::navigation_panel::NavigationPanel;
use crate::presentation::components::tactical::ChallengeRollModal;
use crate::presentation::components::visual_novel::{Backdrop, CharacterLayer, DialogueBox, EmptyDialogueBox};
use crate::presentation::services::{use_character_service, use_world_service};
use crate::presentation::state::{use_dialogue_state, use_game_state, use_session_state, use_typewriter_effect, RollSubmissionStatus};

/// Player Character View - visual novel gameplay interface
///
/// Connection handling and back navigation are provided by WorldSessionLayout wrapper.
#[component]
pub fn PCView() -> Element {
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

    // Navigation panel state
    let mut show_navigation_panel = use_signal(|| false);

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
    let active_challenge = session_state.active_challenge().read().clone();

    // Get roll status for result popup (Phase D)
    let roll_status = session_state.roll_status().read().clone();

    // Check if connected
    let is_connected = session_state.connection_status().read().is_connected();

    // Get navigation data from game state
    let current_region = game_state.current_region.read().clone();
    let navigation = game_state.navigation.read().clone();
    let selected_pc_id = game_state.selected_pc_id.read().clone();

    rsx! {
        div {
            class: "pc-view h-full flex flex-col relative",

            // Location and status indicator (top right)
            div {
                class: "absolute top-4 right-4 z-[100] flex flex-col gap-2 items-end",

                // Location/Region name - prefer region data if available
                if let Some(ref region) = current_region {
                    div {
                        class: "px-4 py-2 bg-black/70 text-white rounded-lg text-sm font-medium",
                        "📍 {region.name}"
                    }
                    div {
                        class: "px-3 py-1 bg-black/50 text-gray-300 rounded-lg text-xs",
                        "{region.location_name}"
                    }
                } else if let Some(scene) = game_state.current_scene.read().as_ref() {
                    div {
                        class: "px-4 py-2 bg-black/70 text-white rounded-lg text-sm font-medium",
                        "📍 {scene.location_name}"
                    }
                }

                // Connection status
            if !is_connected {
                div {
                        class: "px-4 py-2 bg-red-500/80 text-white rounded-lg text-xs",
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
                class: "dialogue-container absolute bottom-0 left-0 right-0 z-10",

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
                    tracing::info!("Open navigation panel");
                    show_navigation_panel.set(true);
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
                        class: "character-sheet-overlay fixed inset-0 bg-black/85 z-[1000] flex items-center justify-center p-8",
                        onclick: move |_| show_character_sheet.set(false),

                        div {
                            class: "bg-dark-surface rounded-xl p-8 max-w-md text-center",
                            onclick: move |e| e.stop_propagation(),

                            div {
                                class: "text-gray-400 text-xl",
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
                                class: "character-sheet-overlay fixed inset-0 bg-black/85 z-[1000] flex items-center justify-center p-8",
                                onclick: move |_| show_character_sheet.set(false),

                                div {
                                    class: "bg-dark-surface rounded-xl p-8 max-w-md text-center",
                                    onclick: move |e| e.stop_propagation(),

                                    h2 {
                                        class: "text-gray-100 m-0 mb-4",
                                        "Character Sheet"
                                    }

                                    if characters.is_empty() {
                                        p {
                                            class: "text-gray-400 m-0 mb-6",
                                            "No characters available in this world."
                                        }
                                    } else {
                                        p {
                                            class: "text-gray-400 m-0 mb-6",
                                            "No character sheet template available for this world. The DM may need to configure the rule system."
                                        }
                                    }

                                    button {
                                        onclick: move |_| show_character_sheet.set(false),
                                        class: "py-2 px-6 bg-gray-700 text-white border-0 rounded-lg cursor-pointer",
                                        "Close"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Challenge roll modal (for active challenges you're rolling)
            if let Some(ref challenge) = active_challenge {
                ChallengeRollModal {
                    challenge_id: challenge.challenge_id.clone(),
                    challenge_name: challenge.challenge_name.clone(),
                    skill_name: challenge.skill_name.clone(),
                    difficulty_display: challenge.difficulty_display.clone(),
                    description: challenge.description.clone(),
                    character_modifier: challenge.character_modifier,
                    suggested_dice: challenge.suggested_dice.clone(),
                    rule_system_hint: challenge.rule_system_hint.clone(),
                    on_roll: {
                        let session_state = session_state.clone();
                        let challenge_id = challenge.challenge_id.clone();
                        move |input: DiceInputType| {
                            send_challenge_roll_input(&session_state, &challenge_id, input);
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

            // Challenge result popup (for received results without active challenge - Phase D)
            if let RollSubmissionStatus::ResultReady(result) = roll_status {
                if active_challenge.is_none() {
                    ChallengeResultPopup {
                        result: result.clone(),
                        on_dismiss: {
                            let mut session_state = session_state.clone();
                            move |_| {
                                session_state.dismiss_result();
                                session_state.clear_roll_status();
                            }
                        },
                    }
                }
            }

            // Navigation panel modal
            if *show_navigation_panel.read() {
                if let Some(ref nav) = navigation {
                    NavigationPanel {
                        navigation: nav.clone(),
                        current_region_name: current_region.as_ref().map(|r| r.name.clone()).unwrap_or_else(|| "Unknown".to_string()),
                        current_location_name: current_region.as_ref().map(|r| r.location_name.clone()).unwrap_or_else(|| "Unknown".to_string()),
                        disabled: is_llm_processing,
                        on_move_to_region: {
                            let session_state = session_state.clone();
                            let pc_id = selected_pc_id.clone();
                            move |region_id: String| {
                                if let Some(ref pc) = pc_id {
                                    send_move_to_region(&session_state, pc, &region_id);
                                    show_navigation_panel.set(false);
                                } else {
                                    tracing::warn!("Cannot move: no PC selected");
                                }
                            }
                        },
                        on_exit_to_location: {
                            let session_state = session_state.clone();
                            let pc_id = selected_pc_id.clone();
                            move |(location_id, arrival_region_id): (String, String)| {
                                if let Some(ref pc) = pc_id {
                                    send_exit_to_location(&session_state, pc, &location_id, Some(&arrival_region_id));
                                    show_navigation_panel.set(false);
                                } else {
                                    tracing::warn!("Cannot exit: no PC selected");
                                }
                            }
                        },
                        on_close: move |_| {
                            show_navigation_panel.set(false);
                        },
                    }
                }
            }
        }
    }
}

/// Standalone challenge result popup (Phase D)
/// Shown when a ChallengeResolved message is received without an active challenge modal.
#[component]
fn ChallengeResultPopup(
    result: crate::presentation::state::challenge_state::ChallengeResultData,
    on_dismiss: EventHandler<()>,
) -> Element {
    // Determine display colors and text based on outcome
    let (outcome_text, outcome_class, border_class) = match result.outcome.as_str() {
        "critical_success" => ("CRITICAL SUCCESS", "text-yellow-400", "border-yellow-400"),
        "success" => ("SUCCESS", "text-green-500", "border-green-500"),
        "failure" => ("FAILURE", "text-red-500", "border-red-500"),
        "critical_failure" => ("CRITICAL FAILURE", "text-red-700", "border-red-700"),
        _ => ("RESULT", "text-amber-500", "border-amber-500"),
    };

    rsx! {
        // Modal overlay
        div {
            class: "fixed inset-0 bg-black/80 flex items-center justify-center z-[1000]",
            onclick: move |_| on_dismiss.call(()),

            // Modal content
            div {
                class: "bg-gradient-to-br from-dark-surface to-dark-bg p-8 rounded-2xl max-w-[450px] w-[90%] border-2 {border_class}",
                onclick: |e| e.stop_propagation(),

                // Header
                div {
                    class: "text-center mb-6",

                    h2 {
                        class: "text-2xl font-bold {outcome_class} mb-2",
                        "{outcome_text}"
                    }

                    p {
                        class: "text-gray-400 text-sm",
                        "{result.challenge_name}"
                    }

                    p {
                        class: "text-gray-500 text-xs",
                        "by {result.character_name}"
                    }
                }

                // Roll breakdown
                div {
                    class: "bg-black/30 rounded-lg p-4 mb-4",

                    div {
                        class: "flex justify-between mb-2",
                        span { class: "text-gray-400", "Roll" }
                        span { class: "text-white font-bold", "{result.roll}" }
                    }

                    div {
                        class: "flex justify-between mb-2",
                        span { class: "text-gray-400", "Modifier" }
                        span {
                            class: "text-blue-500 font-bold",
                            if result.modifier >= 0 { "+{result.modifier}" } else { "{result.modifier}" }
                        }
                    }

                    div {
                        class: "border-t border-white/10 pt-2 flex justify-between",
                        span { class: "text-gray-400 font-bold", "Total" }
                        span { class: "{outcome_class} font-bold text-xl", "{result.total}" }
                    }
                }

                // Optional roll breakdown string
                if let Some(breakdown) = &result.roll_breakdown {
                    p {
                        class: "text-gray-500 text-xs text-center mb-4 font-mono",
                        "{breakdown}"
                    }
                }

                // Outcome description
                if !result.outcome_description.is_empty() {
                    div {
                        class: "bg-black/20 rounded-lg p-4 mb-4",
                        p {
                            class: "text-gray-300 text-sm leading-relaxed italic",
                            "{result.outcome_description}"
                        }
                    }
                }

                // Dismiss button
                button {
                    onclick: move |_| on_dismiss.call(()),
                    class: "w-full p-3 bg-gradient-to-br from-amber-500 to-amber-600 text-white border-none rounded-lg cursor-pointer font-semibold",
                    "Continue"
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
    let engine_client_signal = session_state.engine_client();
    let client_binding = engine_client_signal.read();
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

/// Send a challenge roll with dice input via WebSocket
fn send_challenge_roll_input(
    session_state: &crate::presentation::state::SessionState,
    challenge_id: &str,
    input: DiceInputType,
) {
    let engine_client_signal = session_state.engine_client();
    let client_binding = engine_client_signal.read();
    if let Some(ref client) = *client_binding {
        let svc = crate::application::services::SessionCommandService::new(std::sync::Arc::clone(client));
        if let Err(e) = svc.submit_challenge_roll_input(challenge_id, input) {
            tracing::error!("Failed to send challenge roll input: {}", e);
        }
    } else {
        tracing::warn!("Cannot send challenge roll: not connected to server");
    }
}

/// Send a move to region command via WebSocket
fn send_move_to_region(
    session_state: &crate::presentation::state::SessionState,
    pc_id: &str,
    region_id: &str,
) {
    let engine_client_signal = session_state.engine_client();
    let client_binding = engine_client_signal.read();
    if let Some(ref client) = *client_binding {
        if let Err(e) = client.move_to_region(pc_id, region_id) {
            tracing::error!("Failed to send move to region: {}", e);
        }
    } else {
        tracing::warn!("Cannot move: not connected to server");
    }
}

/// Send an exit to location command via WebSocket
fn send_exit_to_location(
    session_state: &crate::presentation::state::SessionState,
    pc_id: &str,
    location_id: &str,
    arrival_region_id: Option<&str>,
) {
    let engine_client_signal = session_state.engine_client();
    let client_binding = engine_client_signal.read();
    if let Some(ref client) = *client_binding {
        if let Err(e) = client.exit_to_location(pc_id, location_id, arrival_region_id) {
            tracing::error!("Failed to send exit to location: {}", e);
        }
    } else {
        tracing::warn!("Cannot exit: not connected to server");
    }
}
