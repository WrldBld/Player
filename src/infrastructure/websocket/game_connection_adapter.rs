//! Adapter implementing the application `GameConnectionPort` for `EngineClient`.
//!
//! This allows higher layers (presentation/application) to depend on the port
//! rather than the concrete WebSocket client type.

use anyhow::Result;

use crate::application::ports::outbound::{
    ApprovalDecision as PortApprovalDecision, ConnectionState as PortConnectionState,
    DirectorialContext as PortDirectorialContext, GameConnectionPort, NpcMotivation as PortNpcMotivation,
    ParticipantRole as PortParticipantRole,
};

use super::{ClientMessage, EngineClient};
use super::{ConnectionState as InfraConnectionState, ParticipantRole as InfraParticipantRole};
use super::{ApprovalDecision as InfraApprovalDecision, DirectorialContext as InfraDirectorialContext, NpcMotivationData as InfraNpcMotivationData};

fn map_state(state: InfraConnectionState) -> PortConnectionState {
    match state {
        InfraConnectionState::Disconnected => PortConnectionState::Disconnected,
        InfraConnectionState::Connecting => PortConnectionState::Connecting,
        InfraConnectionState::Connected => PortConnectionState::Connected,
        InfraConnectionState::Reconnecting => PortConnectionState::Reconnecting,
        InfraConnectionState::Failed => PortConnectionState::Failed,
    }
}

fn map_role(role: PortParticipantRole) -> InfraParticipantRole {
    match role {
        PortParticipantRole::DungeonMaster => InfraParticipantRole::DungeonMaster,
        PortParticipantRole::Player => InfraParticipantRole::Player,
        PortParticipantRole::Spectator => InfraParticipantRole::Spectator,
    }
}

fn map_npc_motivation(m: PortNpcMotivation) -> InfraNpcMotivationData {
    InfraNpcMotivationData {
        character_id: m.character_id,
        mood: m.mood,
        immediate_goal: m.immediate_goal,
        secret_agenda: m.secret_agenda,
    }
}

fn map_directorial_context(ctx: PortDirectorialContext) -> InfraDirectorialContext {
    InfraDirectorialContext {
        scene_notes: ctx.scene_notes,
        tone: ctx.tone,
        npc_motivations: ctx.npc_motivations.into_iter().map(map_npc_motivation).collect(),
        forbidden_topics: ctx.forbidden_topics,
    }
}

fn map_approval_decision(decision: PortApprovalDecision) -> InfraApprovalDecision {
    match decision {
        PortApprovalDecision::Accept => InfraApprovalDecision::Accept,
        PortApprovalDecision::AcceptWithModification {
            modified_dialogue,
            approved_tools,
            rejected_tools,
        } => InfraApprovalDecision::AcceptWithModification {
            modified_dialogue,
            approved_tools,
            rejected_tools,
        },
        PortApprovalDecision::Reject { feedback } => InfraApprovalDecision::Reject { feedback },
        PortApprovalDecision::TakeOver { dm_response } => InfraApprovalDecision::TakeOver { dm_response },
    }
}

/// Concrete adapter wrapping an `EngineClient`.
#[derive(Clone)]
pub struct EngineGameConnection {
    client: EngineClient,
}

impl EngineGameConnection {
    pub fn new(client: EngineClient) -> Self {
        Self { client }
    }
}

impl GameConnectionPort for EngineGameConnection {
    fn state(&self) -> PortConnectionState {
        #[cfg(target_arch = "wasm32")]
        {
            map_state(self.client.state())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Desktop EngineClient state() is async; for now, treat it as "best-effort"
            // and return Disconnected. Connection state changes should be observed via
            // SessionService events.
            PortConnectionState::Disconnected
        }
    }

    fn url(&self) -> &str {
        self.client.url()
    }

    fn connect(&self) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            self.client.connect()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.connect().await {
                    tracing::error!("Failed to connect to Engine: {}", e);
                }
            });
            Ok(())
        }
    }

    fn disconnect(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            self.client.disconnect();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            tokio::spawn(async move {
                client.disconnect().await;
            });
        }
    }

    fn join_session(&self, user_id: &str, role: PortParticipantRole) -> Result<()> {
        let role = map_role(role);
        #[cfg(target_arch = "wasm32")]
        {
            self.client.join_session(user_id, role, None)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            let user_id = user_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = client.join_session(&user_id, role, None).await {
                    tracing::error!("Failed to join session: {}", e);
                }
            });
            Ok(())
        }
    }

    fn send_action(&self, action_type: &str, target: Option<&str>, dialogue: Option<&str>) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            self.client.send_action(action_type, target, dialogue)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            let action_type = action_type.to_string();
            let target = target.map(|s| s.to_string());
            let dialogue = dialogue.map(|s| s.to_string());
            tokio::spawn(async move {
                if let Err(e) = client.send_action(&action_type, target.as_deref(), dialogue.as_deref()).await {
                    tracing::error!("Failed to send action: {}", e);
                }
            });
            Ok(())
        }
    }

    fn request_scene_change(&self, scene_id: &str) -> Result<()> {
        let msg = ClientMessage::RequestSceneChange { scene_id: scene_id.to_string() };
        #[cfg(target_arch = "wasm32")]
        {
            self.client.send(msg)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.send(msg).await {
                    tracing::error!("Failed to request scene change: {}", e);
                }
            });
            Ok(())
        }
    }

    fn send_directorial_update(&self, context: PortDirectorialContext) -> Result<()> {
        let msg = ClientMessage::DirectorialUpdate { context: map_directorial_context(context) };
        #[cfg(target_arch = "wasm32")]
        {
            self.client.send(msg)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.send(msg).await {
                    tracing::error!("Failed to send directorial update: {}", e);
                }
            });
            Ok(())
        }
    }

    fn send_approval_decision(&self, request_id: &str, decision: PortApprovalDecision) -> Result<()> {
        let msg = ClientMessage::ApprovalDecision {
            request_id: request_id.to_string(),
            decision: map_approval_decision(decision),
        };
        #[cfg(target_arch = "wasm32")]
        {
            self.client.send(msg)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.send(msg).await {
                    tracing::error!("Failed to send approval decision: {}", e);
                }
            });
            Ok(())
        }
    }

    fn trigger_challenge(&self, challenge_id: &str, target_character_id: &str) -> Result<()> {
        let msg = ClientMessage::TriggerChallenge {
            challenge_id: challenge_id.to_string(),
            target_character_id: target_character_id.to_string(),
        };
        #[cfg(target_arch = "wasm32")]
        {
            self.client.send(msg)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.send(msg).await {
                    tracing::error!("Failed to trigger challenge: {}", e);
                }
            });
            Ok(())
        }
    }

    fn submit_challenge_roll(&self, challenge_id: &str, roll: i32) -> Result<()> {
        let msg = ClientMessage::ChallengeRoll {
            challenge_id: challenge_id.to_string(),
            roll,
        };
        #[cfg(target_arch = "wasm32")]
        {
            self.client.send(msg)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.send(msg).await {
                    tracing::error!("Failed to submit challenge roll: {}", e);
                }
            });
            Ok(())
        }
    }

    fn heartbeat(&self) -> Result<()> {
        let msg = ClientMessage::Heartbeat;
        #[cfg(target_arch = "wasm32")]
        {
            self.client.send(msg)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = self.client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.send(msg).await {
                    tracing::error!("Failed to send heartbeat: {}", e);
                }
            });
            Ok(())
        }
    }

    fn on_state_change(&self, _callback: Box<dyn FnMut(PortConnectionState) + 'static>) {
        // NOTE:
        // - Desktop `EngineClient` requires `Send + Sync` callback and is typically wired
        //   via `SessionService` → `SessionEvent` channel.
        // - WASM callback wiring is possible but unused in the current refactor step.
        //
        // This adapter is currently used as a *send-only* abstraction to remove
        // `EngineClient` from presentation state and eliminate duplicated cfg-branches.
    }

    fn on_message(&self, _callback: Box<dyn FnMut(serde_json::Value) + 'static>) {
        // See note in `on_state_change`.
    }
}

