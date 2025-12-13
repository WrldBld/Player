//! Session event handler for desktop platform
//!
//! This module handles SessionEvent from the application layer and updates
//! presentation state accordingly. This is where the application-to-presentation
//! boundary is properly maintained.

#[cfg(not(target_arch = "wasm32"))]
use crate::application::services::SessionEvent;
#[cfg(not(target_arch = "wasm32"))]
use crate::application::ports::outbound::{ConnectionState as PortConnectionState, Platform};
#[cfg(not(target_arch = "wasm32"))]
use crate::application::services::port_connection_state_to_status;
#[cfg(not(target_arch = "wasm32"))]
use crate::presentation::state::{ConnectionStatus, DialogueState, GameState, SessionState};
#[cfg(not(target_arch = "wasm32"))]
use crate::infrastructure::websocket::ServerMessage;
#[cfg(not(target_arch = "wasm32"))]
use crate::infrastructure::asset_loader::WorldSnapshot;
#[cfg(not(target_arch = "wasm32"))]
use crate::presentation::state::{PendingApproval, session_state::{ChallengePromptData, ChallengeResultData}};
#[cfg(not(target_arch = "wasm32"))]
use dioxus::prelude::{WritableExt, ReadableExt};

/// Process a session event and update presentation state
///
/// This function receives events from the application layer's SessionService
/// and updates the presentation layer's state signals accordingly.
#[cfg(not(target_arch = "wasm32"))]
pub fn handle_session_event(
    event: SessionEvent,
    session_state: &mut SessionState,
    game_state: &mut GameState,
    dialogue_state: &mut DialogueState,
    platform: &Platform,
) {
    match event {
        SessionEvent::StateChanged(state) => {
            // Convert application connection state to presentation status
            let status = port_connection_state_to_status(state);

            // Map application status to presentation status
            let presentation_status = match status {
                crate::application::dto::AppConnectionStatus::Disconnected => ConnectionStatus::Disconnected,
                crate::application::dto::AppConnectionStatus::Connecting => ConnectionStatus::Connecting,
                crate::application::dto::AppConnectionStatus::Connected => ConnectionStatus::Connected,
                crate::application::dto::AppConnectionStatus::Reconnecting => ConnectionStatus::Reconnecting,
                crate::application::dto::AppConnectionStatus::Failed => ConnectionStatus::Failed,
            };

            session_state.connection_status.set(presentation_status);

            if matches!(state, PortConnectionState::Disconnected | PortConnectionState::Failed) {
                session_state.engine_client.set(None);
            }
        }
        SessionEvent::MessageReceived(message) => {
            handle_server_message(message, session_state, game_state, dialogue_state, platform);
        }
    }
}

/// Handle incoming server messages and update presentation state
///
/// This function is part of the presentation layer and directly mutates
/// presentation state in response to infrastructure-layer messages.
#[cfg(not(target_arch = "wasm32"))]
fn handle_server_message(
    message: ServerMessage,
    session_state: &mut SessionState,
    game_state: &mut GameState,
    dialogue_state: &mut DialogueState,
    platform: &Platform,
) {
    match message {
        ServerMessage::SessionJoined {
            session_id,
            world_snapshot,
        } => {
            tracing::info!("SessionJoined received");

            // Update session state
            session_state.set_session_joined(session_id.clone());

            // Add log entry
            session_state.add_log_entry(
                "System".to_string(),
                format!("Joined session: {}", session_id),
                true,
                platform,
            );

            // Parse and load world snapshot
            match serde_json::from_value::<WorldSnapshot>(world_snapshot) {
                Ok(snapshot) => {
                    game_state.load_world(snapshot);
                    tracing::info!("World loaded for session: {}", session_id);
                    session_state.add_log_entry(
                        "System".to_string(),
                        "World data loaded".to_string(),
                        true,
                        platform,
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to parse world snapshot: {}", e);
                }
            }
        }

        ServerMessage::SceneUpdate {
            scene,
            characters,
            interactions,
        } => {
            tracing::info!("SceneUpdate: {}", scene.name);
            game_state.apply_scene_update(scene, characters, interactions);
        }

        ServerMessage::DialogueResponse {
            speaker_id,
            speaker_name,
            text,
            choices,
        } => {
            tracing::info!("DialogueResponse from: {}", speaker_name);
            // Add to conversation log for DM view
            session_state.add_log_entry(speaker_name.clone(), text.clone(), false, platform);
            dialogue_state.apply_dialogue(speaker_id, speaker_name, text, choices);
        }

        ServerMessage::LLMProcessing { action_id } => {
            tracing::info!("LLM processing action: {}", action_id);
            // Show "NPC is thinking..." indicator
            dialogue_state.is_llm_processing.set(true);
            session_state.add_log_entry(
                "System".to_string(),
                format!("Processing action: {}", action_id),
                true,
                platform,
            );
        }

        ServerMessage::ApprovalRequired {
            request_id,
            npc_name,
            proposed_dialogue,
            internal_reasoning,
            proposed_tools,
            challenge_suggestion,
        } => {
            tracing::info!(
                "Approval required for {} (request: {})",
                npc_name,
                request_id
            );
            // Add to pending approvals for DM view
            session_state.add_pending_approval(PendingApproval {
                request_id,
                npc_name,
                proposed_dialogue,
                internal_reasoning,
                proposed_tools,
                challenge_suggestion,
            });
        }

        ServerMessage::ResponseApproved {
            npc_dialogue,
            executed_tools,
        } => {
            tracing::info!(
                "Response approved, executed {} tools",
                executed_tools.len()
            );
            let _ = npc_dialogue;
        }

        ServerMessage::Error { message, code } => {
            let error_msg = if let Some(c) = code {
                format!("Server error [{}]: {}", c, message)
            } else {
                format!("Server error: {}", message)
            };
            tracing::error!("{}", error_msg);
            session_state.error_message.set(Some(error_msg));
        }

        ServerMessage::Pong => {
            // Heartbeat response - no action needed
        }

        // Generation events - these will be handled by GenerationState when integrated
        ServerMessage::GenerationQueued {
            batch_id,
            entity_type,
            entity_id,
            asset_type,
            position,
        } => {
            tracing::info!(
                "Generation queued: {} {} ({}) at position {}",
                entity_type,
                entity_id,
                asset_type,
                position
            );
            // TODO: Update GenerationState when passed to this function
            let _ = batch_id;
        }

        ServerMessage::GenerationProgress { batch_id, progress } => {
            tracing::info!("Generation progress: {} at {}%", batch_id, progress);
        }

        ServerMessage::GenerationComplete {
            batch_id,
            asset_count,
        } => {
            tracing::info!(
                "Generation complete: {} with {} assets",
                batch_id,
                asset_count
            );
        }

        ServerMessage::GenerationFailed { batch_id, error } => {
            tracing::error!("Generation failed: {} - {}", batch_id, error);
        }

        ServerMessage::ChallengePrompt {
            challenge_id,
            challenge_name,
            skill_name,
            difficulty_display,
            description,
            character_modifier,
        } => {
            tracing::info!("Challenge prompt received: {}", challenge_name);
            // Set the active challenge for the player to respond to
            let challenge = ChallengePromptData {
                challenge_id,
                challenge_name,
                skill_name,
                difficulty_display,
                description,
                character_modifier,
            };
            session_state.set_active_challenge(challenge);
        }

        ServerMessage::ChallengeResolved {
            challenge_id,
            challenge_name,
            character_name,
            roll,
            modifier,
            total,
            outcome,
            outcome_description,
        } => {
            tracing::info!(
                "Challenge resolved: {} - {} (result: {})",
                challenge_name,
                character_name,
                outcome
            );
            // Clear active challenge if it matches
            let active = session_state.active_challenge.read().clone();
            if let Some(active_challenge) = active {
                if active_challenge.challenge_id == challenge_id {
                    session_state.clear_active_challenge();
                }
            }
            // Add to challenge results
            let timestamp = platform.now_unix_secs();
            let result = ChallengeResultData {
                challenge_name,
                character_name,
                roll,
                modifier,
                total,
                outcome,
                outcome_description,
                timestamp,
            };
            session_state.add_challenge_result(result);
        }
    }
}
