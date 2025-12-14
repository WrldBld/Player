//! Presentation-layer handler for Engine WebSocket `ServerMessage`.
//!
//! This is the canonical place to translate incoming server messages into
//! presentation state mutations. Keeping this here avoids application→presentation
//! dependencies and keeps the WebSocket transport parsing separate from UI state.

use crate::application::ports::outbound::Platform;
use crate::application::dto::ServerMessage;
use crate::application::dto::SessionWorldSnapshot;
use dioxus::prelude::{ReadableExt, WritableExt};
use crate::presentation::state::{
    DialogueState, GameState, GenerationState, PendingApproval, SessionState,
    session_state::{ChallengePromptData, ChallengeResultData},
};

/// Handle an incoming `ServerMessage` and update presentation state.
pub fn handle_server_message(
    message: ServerMessage,
    session_state: &mut SessionState,
    game_state: &mut GameState,
    dialogue_state: &mut DialogueState,
    generation_state: &mut GenerationState,
    platform: &Platform,
) {
    match message {
        ServerMessage::SessionJoined {
            session_id,
            role: _,
            participants: _,
            world_snapshot,
        } => {
            tracing::info!("SessionJoined received");

            session_state.set_session_joined(session_id.clone());
            session_state.add_log_entry(
                "System".to_string(),
                format!("Joined session: {}", session_id),
                true,
                platform,
            );

            match serde_json::from_value::<SessionWorldSnapshot>(world_snapshot) {
                Ok(snapshot) => {
                    game_state.load_world(snapshot);
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

        ServerMessage::PlayerJoined {
            user_id,
            role,
            character_name,
        } => {
            tracing::info!("Player joined: {} as {:?}", user_id, role);
            session_state.add_log_entry(
                "System".to_string(),
                format!(
                    "Player {} joined as {:?}{}",
                    user_id,
                    role,
                    character_name
                        .map(|n| format!(" ({})", n))
                        .unwrap_or_default()
                ),
                true,
                platform,
            );
        }

        ServerMessage::PlayerLeft { user_id } => {
            tracing::info!("Player left: {}", user_id);
            session_state.add_log_entry(
                "System".to_string(),
                format!("Player {} left", user_id),
                true,
                platform,
            );
        }

        ServerMessage::ActionReceived {
            action_id,
            player_id,
            action_type,
        } => {
            tracing::info!("Action received: {} -> {}", action_type, player_id);
            session_state.add_log_entry(
                "System".to_string(),
                format!("Action {} received: {}", action_id, action_type),
                true,
                platform,
            );
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
            // Add to conversation log for DM view
            session_state.add_log_entry(speaker_name.clone(), text.clone(), false, platform);
            dialogue_state.apply_dialogue(speaker_id, speaker_name, text, choices);
        }

        ServerMessage::LLMProcessing { action_id } => {
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
            narrative_event_suggestion: _, // TODO: Surface narrative event suggestion in UI
        } => {
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
            npc_dialogue: _,
            executed_tools,
        } => {
            tracing::info!("ResponseApproved: executed {} tools", executed_tools.len());
        }

        ServerMessage::Error { code, message } => {
            let error_msg = format!("Server error [{}]: {}", code, message);
            tracing::error!("{}", error_msg);
            session_state.error_message.set(Some(error_msg));
        }

        ServerMessage::Pong => {}

        // Generation events (Creator Mode)
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
            generation_state.batch_queued(
                batch_id,
                entity_type,
                entity_id,
                asset_type,
                position,
            );
        }

        ServerMessage::GenerationProgress { batch_id, progress } => {
            tracing::info!("Generation progress: {} at {}%", batch_id, progress);
            generation_state.batch_progress(&batch_id, progress);
        }

        ServerMessage::GenerationComplete { batch_id, asset_count } => {
            tracing::info!("Generation complete: {} ({} assets)", batch_id, asset_count);
            generation_state.batch_complete(&batch_id, asset_count);
        }

        ServerMessage::GenerationFailed { batch_id, error } => {
            tracing::error!("Generation failed: {} - {}", batch_id, error);
            generation_state.batch_failed(&batch_id, error);
        }

        ServerMessage::ChallengePrompt {
            challenge_id,
            challenge_name,
            skill_name,
            difficulty_display,
            description,
            character_modifier,
        } => {
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
            // Clear active challenge if it matches
            let active = { session_state.active_challenge.read().clone() };
            if let Some(active_challenge) = active {
                if active_challenge.challenge_id == challenge_id {
                    session_state.clear_active_challenge();
                }
            }

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

