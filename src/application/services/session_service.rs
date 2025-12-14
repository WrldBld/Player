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
//! addressed in Phase 16. Specifically:
//! - Uses infrastructure types (WorldSnapshot, ServerMessage)
//! - WASM variant previously mutated presentation state directly (now removed)

use std::sync::Arc;

use anyhow::Result;

use crate::application::ports::outbound::{
    ConnectionState as PortConnectionState, GameConnectionPort, ParticipantRole as PortParticipantRole,
};

use crate::application::dto::AppConnectionStatus;

/// Default WebSocket URL for the Engine server
pub const DEFAULT_ENGINE_URL: &str = "ws://localhost:3000/ws";

// Re-export port types for external use
pub use crate::application::ports::outbound::{
    ConnectionState as ConnectionStatePort,
    ParticipantRole as ParticipantRolePort,
};

/// Convert port ConnectionState to application ConnectionStatus
pub fn port_connection_state_to_status(state: PortConnectionState) -> AppConnectionStatus {
    match state {
        PortConnectionState::Disconnected => AppConnectionStatus::Disconnected,
        PortConnectionState::Connecting => AppConnectionStatus::Connecting,
        PortConnectionState::Connected => AppConnectionStatus::Connected,
        PortConnectionState::Reconnecting => AppConnectionStatus::Reconnecting,
        PortConnectionState::Failed => AppConnectionStatus::Failed,
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use super::*;
    use tokio::sync::mpsc;

    /// Events sent from the connection callbacks to the UI task.
    #[derive(Debug, Clone)]
    pub enum SessionEvent {
        /// Connection state changed (uses port type)
        StateChanged(PortConnectionState),
        /// Raw server message payload (JSON)
        MessageReceived(serde_json::Value),
    }

    /// Session service for managing Engine connection (Desktop).
    ///
    /// This service depends only on the `GameConnectionPort` abstraction.
    pub struct SessionService {
        connection: Arc<dyn GameConnectionPort>,
    }

    impl SessionService {
        pub fn new(connection: Arc<dyn GameConnectionPort>) -> Self {
            Self { connection }
        }

        pub fn connection(&self) -> &Arc<dyn GameConnectionPort> {
            &self.connection
        }

        pub async fn connect(
            &self,
            user_id: String,
            role: PortParticipantRole,
        ) -> Result<mpsc::Receiver<SessionEvent>> {
            let (tx, rx) = mpsc::channel::<SessionEvent>(64);

            // On connect, join when Connected is observed.
            {
                let tx = tx.clone();
                let connection = Arc::clone(&self.connection);
                let user_id_for_join = user_id.clone();

                self.connection.on_state_change(Box::new(move |state| {
                    let _ = tx.try_send(SessionEvent::StateChanged(state));
                    if matches!(state, PortConnectionState::Connected) {
                        let _ = connection.join_session(&user_id_for_join, role);
                    }
                }));
            }

            // Forward raw messages
            {
                let tx = tx.clone();
                self.connection.on_message(Box::new(move |value| {
                    let _ = tx.try_send(SessionEvent::MessageReceived(value));
                }));
            }

            // Initiate connection (adapter handles async details)
            self.connection.connect()?;

            Ok(rx)
        }

        pub fn disconnect(&self) {
            self.connection.disconnect();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::{SessionEvent, SessionService};
