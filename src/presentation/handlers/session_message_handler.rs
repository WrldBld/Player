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
            narrative_event_suggestion,
        } => {
            session_state.add_pending_approval(PendingApproval {
                request_id,
                npc_name,
                proposed_dialogue,
                internal_reasoning,
                proposed_tools,
                challenge_suggestion,
                narrative_event_suggestion,
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

        ServerMessage::SuggestionQueued {
            request_id,
            field_type,
            entity_id,
        } => {
            tracing::info!("Suggestion queued: {} ({})", request_id, field_type);
            generation_state.suggestion_queued(request_id, field_type, entity_id);
        }

        ServerMessage::SuggestionProgress { request_id, status } => {
            tracing::info!("Suggestion progress: {} - {}", request_id, status);
            generation_state.suggestion_progress(&request_id, &status);
        }

        ServerMessage::SuggestionComplete {
            request_id,
            suggestions,
        } => {
            tracing::info!("Suggestion complete: {} ({} suggestions)", request_id, suggestions.len());
            generation_state.suggestion_complete(&request_id, suggestions);
        }

        ServerMessage::SuggestionFailed { request_id, error } => {
            tracing::error!("Suggestion failed: {} - {}", request_id, error);
            generation_state.suggestion_failed(&request_id, error);
        }

        ServerMessage::ComfyUIStateChanged {
            state,
            message,
            retry_in_seconds,
        } => {
            tracing::info!("ComfyUI state changed: {} - {:?}", state, message);
            session_state.comfyui_state.set(state);
            session_state.comfyui_message.set(message);
            session_state.comfyui_retry_in_seconds.set(retry_in_seconds);
        }

        ServerMessage::ChallengePrompt {
            challenge_id,
            challenge_name,
            skill_name,
            difficulty_display,
            description,
            character_modifier,
            suggested_dice,
            rule_system_hint,
        } => {
            let challenge = ChallengePromptData {
                challenge_id,
                challenge_name,
                skill_name,
                difficulty_display,
                description,
                character_modifier,
                suggested_dice,
                rule_system_hint,
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
            roll_breakdown,
            individual_rolls,
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
                roll_breakdown,
                individual_rolls,
            };
            session_state.add_challenge_result(result);
        }

        ServerMessage::NarrativeEventTriggered {
            event_id: _,
            event_name,
            outcome_description,
            scene_direction,
        } => {
            // Log the narrative event trigger for DMs
            tracing::info!(
                "Narrative event '{}' triggered: {} ({})",
                event_name,
                outcome_description,
                scene_direction
            );
            // TODO (Phase 17 Story Arc UI): Update Story Arc timeline when the tab is implemented
            // For now, this is logged to console for DM awareness
        }

        ServerMessage::SplitPartyNotification {
            location_count,
            locations,
        } => {
            tracing::info!(
                "Party is split across {} locations",
                location_count
            );
            // TODO: Update UI to show split party warning with location information
            // For now, this is logged to console for DM awareness
            for loc in locations {
                tracing::debug!(
                    "Location: {} - {} PC(s)",
                    loc.location_name,
                    loc.pc_count
                );
            }
        }

        ServerMessage::OutcomeRegenerated {
            request_id,
            outcome_type,
            new_outcome,
        } => {
            tracing::info!(
                "Outcome '{}' regenerated for request {}: {}",
                outcome_type,
                request_id,
                new_outcome.flavor_text
            );
            // TODO (Phase 22G): Update approval popup with regenerated outcome
        }

        ServerMessage::ChallengeDiscarded { request_id } => {
            tracing::info!("Challenge discarded for request {}", request_id);
            // TODO (Phase 22G): Remove challenge from approval UI
        }

        ServerMessage::AdHocChallengeCreated {
            challenge_id,
            challenge_name,
            target_pc_id,
        } => {
            tracing::info!(
                "Ad-hoc challenge '{}' (ID: {}) created for PC {}",
                challenge_name,
                challenge_id,
                target_pc_id
            );
            // TODO (Phase 22H): Show ad-hoc challenge creation confirmation
        }
    }
}

