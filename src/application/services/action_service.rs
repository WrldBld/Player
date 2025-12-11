//! Action service for sending player actions to the Engine
//!
//! This service wraps the WebSocket client and provides convenient
//! methods for sending player actions.

use std::sync::Arc;

use anyhow::Result;

use crate::domain::entities::PlayerAction;
use crate::infrastructure::websocket::EngineClient;

/// Service for sending player actions to the Engine via WebSocket
pub struct ActionService {
    client: Arc<EngineClient>,
}

// ============================================================================
// Desktop (async) implementation
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
impl ActionService {
    /// Create a new ActionService with the given WebSocket client
    pub fn new(client: Arc<EngineClient>) -> Self {
        Self { client }
    }

    /// Send a player action to the Engine
    pub async fn send_action(&self, action: PlayerAction) -> Result<()> {
        let action_type = action.action_type.as_str();

        self.client
            .send_action(
                action_type,
                action.target.as_deref(),
                action.dialogue.as_deref(),
            )
            .await
    }

    /// Send a dialogue choice selection
    pub async fn select_choice(&self, choice_id: &str) -> Result<()> {
        let action = PlayerAction::dialogue_choice(choice_id);
        self.send_action(action).await
    }

    /// Send custom dialogue input
    pub async fn send_custom_dialogue(&self, text: &str) -> Result<()> {
        let action = PlayerAction::custom(text);
        self.send_action(action).await
    }

    /// Send a talk action to an NPC
    pub async fn talk_to(&self, npc_id: &str, dialogue: Option<&str>) -> Result<()> {
        let action = PlayerAction::talk(npc_id, dialogue);
        self.send_action(action).await
    }

    /// Send an examine action
    pub async fn examine(&self, target: &str) -> Result<()> {
        let action = PlayerAction::examine(target);
        self.send_action(action).await
    }

    /// Send a travel action
    pub async fn travel_to(&self, location_id: &str) -> Result<()> {
        let action = PlayerAction::travel(location_id);
        self.send_action(action).await
    }

    /// Send a use item action
    pub async fn use_item(&self, item_id: &str, target: Option<&str>) -> Result<()> {
        let action = PlayerAction::use_item(item_id, target);
        self.send_action(action).await
    }

    /// Get a reference to the underlying client
    pub fn client(&self) -> &Arc<EngineClient> {
        &self.client
    }
}

// ============================================================================
// WASM (sync) implementation
// ============================================================================

#[cfg(target_arch = "wasm32")]
impl ActionService {
    /// Create a new ActionService with the given WebSocket client
    pub fn new(client: Arc<EngineClient>) -> Self {
        Self { client }
    }

    /// Send a player action to the Engine
    pub fn send_action(&self, action: PlayerAction) -> Result<()> {
        let action_type = action.action_type.as_str();

        self.client.send_action(
            action_type,
            action.target.as_deref(),
            action.dialogue.as_deref(),
        )
    }

    /// Send a dialogue choice selection
    pub fn select_choice(&self, choice_id: &str) -> Result<()> {
        let action = PlayerAction::dialogue_choice(choice_id);
        self.send_action(action)
    }

    /// Send custom dialogue input
    pub fn send_custom_dialogue(&self, text: &str) -> Result<()> {
        let action = PlayerAction::custom(text);
        self.send_action(action)
    }

    /// Send a talk action to an NPC
    pub fn talk_to(&self, npc_id: &str, dialogue: Option<&str>) -> Result<()> {
        let action = PlayerAction::talk(npc_id, dialogue);
        self.send_action(action)
    }

    /// Send an examine action
    pub fn examine(&self, target: &str) -> Result<()> {
        let action = PlayerAction::examine(target);
        self.send_action(action)
    }

    /// Send a travel action
    pub fn travel_to(&self, location_id: &str) -> Result<()> {
        let action = PlayerAction::travel(location_id);
        self.send_action(action)
    }

    /// Send a use item action
    pub fn use_item(&self, item_id: &str, target: Option<&str>) -> Result<()> {
        let action = PlayerAction::use_item(item_id, target);
        self.send_action(action)
    }

    /// Get a reference to the underlying client
    pub fn client(&self) -> &Arc<EngineClient> {
        &self.client
    }
}
