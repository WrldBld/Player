//! Session service for managing Engine WebSocket connection
//!
//! This service handles:
//! - Connecting to the Engine WebSocket server
//! - Sending JoinSession messages
//! - Processing server messages and updating application state
//!
//! Platform-specific implementations for desktop (async) and WASM (sync callbacks).
//!
//! NOTE: This service currently has some architecture violations that will be
//! addressed in Phase 7. Specifically:
//! - Uses infrastructure types (WorldSnapshot, ServerMessage)
//! - Directly mutates presentation state (should return events instead)

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::application::ports::outbound::{
    ConnectionState as PortConnectionState, GameConnectionPort, ParticipantRole as PortParticipantRole,
};
// TODO Phase 7: These infrastructure imports should be abstracted
use crate::infrastructure::asset_loader::WorldSnapshot;
use crate::infrastructure::websocket::{EngineClient, ServerMessage};
// TODO Phase 6: Application layer should not import presentation state
// This should return domain events instead of mutating state directly
use crate::presentation::state::{
    ConnectionStatus, DialogueState, GameState, PendingApproval, SessionState,
    session_state::{ChallengePromptData, ChallengeResultData},
};

/// Default WebSocket URL for the Engine server
pub const DEFAULT_ENGINE_URL: &str = "ws://localhost:3000/ws";

// Re-export port types for external use
pub use crate::application::ports::outbound::{
    ConnectionState as ConnectionStatePort,
    ParticipantRole as ParticipantRolePort,
};

// Import infrastructure types for internal mapping
use crate::infrastructure::websocket::ConnectionState as InfraConnectionState;

/// Convert port ConnectionState to presentation ConnectionStatus
pub fn port_connection_state_to_status(state: PortConnectionState) -> ConnectionStatus {
    match state {
        PortConnectionState::Disconnected => ConnectionStatus::Disconnected,
        PortConnectionState::Connecting => ConnectionStatus::Connecting,
        PortConnectionState::Connected => ConnectionStatus::Connected,
        PortConnectionState::Reconnecting => ConnectionStatus::Reconnecting,
        PortConnectionState::Failed => ConnectionStatus::Failed,
    }
}

/// Convert infrastructure ConnectionState to port ConnectionState
pub fn infra_to_port_connection_state(state: InfraConnectionState) -> PortConnectionState {
    match state {
        InfraConnectionState::Disconnected => PortConnectionState::Disconnected,
        InfraConnectionState::Connecting => PortConnectionState::Connecting,
        InfraConnectionState::Connected => PortConnectionState::Connected,
        InfraConnectionState::Reconnecting => PortConnectionState::Reconnecting,
        InfraConnectionState::Failed => PortConnectionState::Failed,
    }
}

/// Convert infrastructure ConnectionState to presentation ConnectionStatus
/// (Convenience function that combines the two conversions)
pub fn connection_state_to_status(state: InfraConnectionState) -> ConnectionStatus {
    port_connection_state_to_status(infra_to_port_connection_state(state))
}

// ============================================================================
// Desktop (Tokio) Implementation
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use super::*;
    use crate::infrastructure::websocket::ParticipantRole as InfraParticipantRole;
    use dioxus::prelude::WritableExt;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::sync::mpsc;

    /// Events sent from WebSocket thread to UI
    ///
    /// Uses port ConnectionState type for proper abstraction.
    /// ServerMessage still uses infrastructure type (to be addressed in Phase 7).
    #[derive(Debug, Clone)]
    pub enum SessionEvent {
        /// Connection state changed (uses port type)
        StateChanged(PortConnectionState),
        /// Server message received (TODO Phase 7: abstract this)
        MessageReceived(ServerMessage),
    }

    /// Session service for managing Engine connection (Desktop)
    ///
    /// Uses channels to communicate between the WebSocket thread and UI.
    pub struct SessionService {
        client: Arc<EngineClient>,
        connected: Arc<AtomicBool>,
    }

    impl SessionService {
        /// Create a new SessionService with the given WebSocket URL
        pub fn new(url: impl Into<String>) -> Self {
            Self {
                client: Arc::new(EngineClient::new(url)),
                connected: Arc::new(AtomicBool::new(false)),
            }
        }

        /// Create a new SessionService with the default URL
        pub fn with_default_url() -> Self {
            Self::new(DEFAULT_ENGINE_URL)
        }

        /// Get a reference to the underlying client
        pub fn client(&self) -> &Arc<EngineClient> {
            &self.client
        }

        /// Check if currently connected
        pub fn is_connected(&self) -> bool {
            self.connected.load(Ordering::SeqCst)
        }

        /// Connect to the Engine and return a channel for receiving events
        ///
        /// This method:
        /// 1. Creates a channel for events
        /// 2. Sets up callbacks to send events through the channel
        /// 3. Initiates the WebSocket connection
        /// 4. Sends a JoinSession message once connected
        ///
        /// The caller should poll the returned receiver to handle events.
        ///
        /// # Arguments
        /// * `user_id` - Unique identifier for this user
        /// * `role` - The participant role (uses port type)
        pub async fn connect(
            &self,
            user_id: String,
            role: PortParticipantRole,
        ) -> Result<mpsc::Receiver<SessionEvent>> {
            let (tx, rx) = mpsc::channel::<SessionEvent>(64);

            // Convert port role to infrastructure role
            let infra_role = match role {
                PortParticipantRole::DungeonMaster => InfraParticipantRole::DungeonMaster,
                PortParticipantRole::Player => InfraParticipantRole::Player,
                PortParticipantRole::Spectator => InfraParticipantRole::Spectator,
            };

            // Set up state change callback
            {
                let tx = tx.clone();
                let connected = Arc::clone(&self.connected);
                self.client
                    .set_on_state_change(move |state| {
                        connected.store(
                            matches!(state, InfraConnectionState::Connected),
                            Ordering::SeqCst,
                        );
                        // Convert infrastructure state to port state
                        let port_state = infra_to_port_connection_state(state);
                        let _ = tx.try_send(SessionEvent::StateChanged(port_state));
                    })
                    .await;
            }

            // Set up message handler
            {
                let tx = tx.clone();
                self.client
                    .set_on_message(move |message| {
                        let _ = tx.try_send(SessionEvent::MessageReceived(message));
                    })
                    .await;
            }

            // Spawn connection task
            let client = Arc::clone(&self.client);
            let user_id_clone = user_id.clone();
            tokio::spawn(async move {
                if let Err(e) = client.connect().await {
                    tracing::error!("Connection error: {}", e);
                }
            });

            // Wait briefly for connection to establish
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            // Send JoinSession message if connected
            if self.client.state().await == InfraConnectionState::Connected {
                self.client.join_session(&user_id_clone, infra_role).await?;
                tracing::info!("Sent JoinSession for user: {}", user_id_clone);
            }

            Ok(rx)
        }

        /// Disconnect from the Engine
        pub async fn disconnect(&self) {
            self.client.disconnect().await;
            self.connected.store(false, Ordering::SeqCst);
        }

        /// Send a heartbeat to keep the connection alive
        pub async fn heartbeat(&self) -> Result<()> {
            self.client.heartbeat().await
        }
    }

    /// Process a session event and update application state
    ///
    /// NOTE: This function mutates presentation state directly, which is an
    /// architecture violation. In Phase 7, this should be refactored to
    /// return domain events instead.
    pub fn handle_session_event(
        event: SessionEvent,
        session_state: &mut SessionState,
        game_state: &mut GameState,
        dialogue_state: &mut DialogueState,
    ) {
        match event {
            SessionEvent::StateChanged(state) => {
                // Convert port state to presentation status
                let status = port_connection_state_to_status(state);
                session_state.connection_status.set(status);

                if matches!(state, PortConnectionState::Disconnected | PortConnectionState::Failed) {
                    session_state.engine_client.set(None);
                }
            }
            SessionEvent::MessageReceived(message) => {
                handle_server_message(message, session_state, game_state, dialogue_state);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::{handle_session_event, SessionEvent, SessionService};

// ============================================================================
// WASM (Web-sys) Implementation
// ============================================================================

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use crate::infrastructure::websocket::ParticipantRole as InfraParticipantRole;
    use dioxus::prelude::WritableExt;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Session service for managing Engine connection (WASM)
    pub struct SessionService {
        client: Rc<RefCell<Option<EngineClient>>>,
        url: String,
    }

    impl SessionService {
        /// Create a new SessionService with the given WebSocket URL
        pub fn new(url: impl Into<String>) -> Self {
            Self {
                client: Rc::new(RefCell::new(None)),
                url: url.into(),
            }
        }

        /// Create a new SessionService with the default URL
        pub fn with_default_url() -> Self {
            Self::new(DEFAULT_ENGINE_URL)
        }

        /// Get the configured URL
        pub fn url(&self) -> &str {
            &self.url
        }

        /// Connect to the Engine and set up message handlers
        ///
        /// This method:
        /// 1. Creates the WebSocket client
        /// 2. Sets up callbacks for connection state changes
        /// 3. Sets up callbacks for incoming messages
        /// 4. Initiates the WebSocket connection
        ///
        /// The JoinSession message is sent automatically when the connection opens.
        ///
        /// # Arguments
        /// * `user_id` - Unique identifier for this user
        /// * `role` - The participant role (uses port type)
        pub fn connect_and_join(
            &self,
            user_id: String,
            role: PortParticipantRole,
            mut session_state: SessionState,
            mut game_state: GameState,
            mut dialogue_state: DialogueState,
        ) -> Result<()> {
            // Convert port role to infrastructure role
            let infra_role = match role {
                PortParticipantRole::DungeonMaster => InfraParticipantRole::DungeonMaster,
                PortParticipantRole::Player => InfraParticipantRole::Player,
                PortParticipantRole::Spectator => InfraParticipantRole::Spectator,
            };

            // Update session state to connecting
            session_state.start_connecting(&self.url);
            session_state.set_user(user_id.clone(), role);

            // Create the client
            let client = EngineClient::new(&self.url);

            // Set up state change callback
            {
                let mut session_state = session_state.clone();
                let client_ref = Rc::clone(&self.client);
                let user_id = user_id.clone();

                client.set_on_state_change(move |state| {
                    let status = connection_state_to_status(state);
                    session_state.connection_status.set(status);

                    match state {
                        InfraConnectionState::Connected => {
                            // Send JoinSession when connected
                            if let Some(ref client) = *client_ref.borrow() {
                                if let Err(e) = client.join_session(&user_id, infra_role) {
                                    web_sys::console::error_1(
                                        &format!("Failed to send JoinSession: {}", e).into(),
                                    );
                                } else {
                                    web_sys::console::log_1(
                                        &format!("Sent JoinSession for user: {}", user_id).into(),
                                    );
                                }
                            }
                        }
                        InfraConnectionState::Disconnected | InfraConnectionState::Failed => {
                            session_state.engine_client.set(None);
                        }
                        _ => {}
                    }
                });
            }

            // Set up message handler
            {
                let mut session_state = session_state.clone();
                let mut game_state = game_state.clone();
                let mut dialogue_state = dialogue_state.clone();

                client.set_on_message(move |message| {
                    handle_server_message(
                        message,
                        &mut session_state,
                        &mut game_state,
                        &mut dialogue_state,
                    );
                });
            }

            // Store the client
            *self.client.borrow_mut() = Some(client.clone());

            // Store client reference in session state (wrap in Arc for compatibility)
            session_state
                .engine_client
                .set(Some(Arc::new(client.clone())));

            // Initiate connection
            client.connect()?;

            Ok(())
        }

        /// Disconnect from the Engine
        pub fn disconnect(&self, mut session_state: SessionState) {
            if let Some(ref client) = *self.client.borrow() {
                client.disconnect();
            }
            *self.client.borrow_mut() = None;
            session_state.set_disconnected();
        }

        /// Send a heartbeat to keep the connection alive
        pub fn heartbeat(&self) -> Result<()> {
            if let Some(ref client) = *self.client.borrow() {
                client.heartbeat()
            } else {
                Err(anyhow::anyhow!("Not connected"))
            }
        }

        /// Check if currently connected
        pub fn is_connected(&self) -> bool {
            if let Some(ref client) = *self.client.borrow() {
                client.state() == InfraConnectionState::Connected
            } else {
                false
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::SessionService;

// ============================================================================
// Shared Message Handler
// ============================================================================

use dioxus::prelude::{WritableExt, ReadableExt};

/// Handle incoming server messages and update application state
fn handle_server_message(
    message: ServerMessage,
    session_state: &mut SessionState,
    game_state: &mut GameState,
    dialogue_state: &mut DialogueState,
) {
    match message {
        ServerMessage::SessionJoined {
            session_id,
            world_snapshot,
        } => {
            log_message("SessionJoined received");

            // Update session state
            session_state.set_session_joined(session_id.clone());

            // Add log entry
            session_state.add_log_entry(
                "System".to_string(),
                format!("Joined session: {}", session_id),
                true,
            );

            // Parse and load world snapshot
            match serde_json::from_value::<WorldSnapshot>(world_snapshot) {
                Ok(snapshot) => {
                    game_state.load_world(snapshot);
                    log_message(&format!("World loaded for session: {}", session_id));
                    session_state.add_log_entry(
                        "System".to_string(),
                        "World data loaded".to_string(),
                        true,
                    );
                }
                Err(e) => {
                    log_error(&format!("Failed to parse world snapshot: {}", e));
                }
            }
        }

        ServerMessage::SceneUpdate {
            scene,
            characters,
            interactions,
        } => {
            log_message(&format!("SceneUpdate: {}", scene.name));
            game_state.apply_scene_update(scene, characters, interactions);
        }

        ServerMessage::DialogueResponse {
            speaker_id,
            speaker_name,
            text,
            choices,
        } => {
            log_message(&format!("DialogueResponse from: {}", speaker_name));
            // Add to conversation log for DM view
            session_state.add_log_entry(speaker_name.clone(), text.clone(), false);
            dialogue_state.apply_dialogue(speaker_id, speaker_name, text, choices);
        }

        ServerMessage::LLMProcessing { action_id } => {
            log_message(&format!("LLM processing action: {}", action_id));
            // Show "NPC is thinking..." indicator
            dialogue_state.is_llm_processing.set(true);
            session_state.add_log_entry(
                "System".to_string(),
                format!("Processing action: {}", action_id),
                true,
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
            log_message(&format!(
                "Approval required for {} (request: {})",
                npc_name, request_id
            ));
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
            log_message(&format!(
                "Response approved, executed {} tools",
                executed_tools.len()
            ));
            let _ = npc_dialogue;
        }

        ServerMessage::Error { message, code } => {
            let error_msg = if let Some(c) = code {
                format!("Server error [{}]: {}", c, message)
            } else {
                format!("Server error: {}", message)
            };
            log_error(&error_msg);
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
            log_message(&format!(
                "Generation queued: {} {} ({}) at position {}",
                entity_type, entity_id, asset_type, position
            ));
            // TODO: Update GenerationState when passed to this function
            let _ = batch_id;
        }

        ServerMessage::GenerationProgress { batch_id, progress } => {
            log_message(&format!(
                "Generation progress: {} at {}%",
                batch_id, progress
            ));
        }

        ServerMessage::GenerationComplete {
            batch_id,
            asset_count,
        } => {
            log_message(&format!(
                "Generation complete: {} with {} assets",
                batch_id, asset_count
            ));
        }

        ServerMessage::GenerationFailed { batch_id, error } => {
            log_error(&format!("Generation failed: {} - {}", batch_id, error));
        }

        ServerMessage::ChallengePrompt {
            challenge_id,
            challenge_name,
            skill_name,
            difficulty_display,
            description,
            character_modifier,
        } => {
            log_message(&format!("Challenge prompt received: {}", challenge_name));
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
            log_message(&format!(
                "Challenge resolved: {} - {} (result: {})",
                challenge_name, character_name, outcome
            ));
            // Clear active challenge if it matches
            let active = session_state.active_challenge.read().clone();
            if let Some(active_challenge) = active {
                if active_challenge.challenge_id == challenge_id {
                    session_state.clear_active_challenge();
                }
            }
            // Add to challenge results
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
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

// Platform-specific logging helpers

#[cfg(not(target_arch = "wasm32"))]
fn log_message(msg: &str) {
    tracing::info!("{}", msg);
}

#[cfg(not(target_arch = "wasm32"))]
fn log_error(msg: &str) {
    tracing::error!("{}", msg);
}

#[cfg(target_arch = "wasm32")]
fn log_message(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

#[cfg(target_arch = "wasm32")]
fn log_error(msg: &str) {
    web_sys::console::error_1(&msg.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_conversion() {
        assert_eq!(
            connection_state_to_status(ConnectionState::Disconnected),
            ConnectionStatus::Disconnected
        );
        assert_eq!(
            connection_state_to_status(ConnectionState::Connecting),
            ConnectionStatus::Connecting
        );
        assert_eq!(
            connection_state_to_status(ConnectionState::Connected),
            ConnectionStatus::Connected
        );
        assert_eq!(
            connection_state_to_status(ConnectionState::Reconnecting),
            ConnectionStatus::Reconnecting
        );
        assert_eq!(
            connection_state_to_status(ConnectionState::Failed),
            ConnectionStatus::Failed
        );
    }
}
