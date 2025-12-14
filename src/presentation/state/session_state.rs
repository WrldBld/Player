//! Session state management using Dioxus signals
//!
//! Tracks connection status and user session information.

use dioxus::prelude::*;
use std::sync::Arc;

// Use port types and application DTOs instead of infrastructure types
use crate::application::ports::outbound::{GameConnectionPort, ParticipantRole, Platform};
use crate::application::dto::{ProposedTool, ChallengeSuggestionInfo};
use crate::presentation::components::tactical::PlayerSkillData;

/// Connection status to the Engine server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Not connected to any server
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and ready
    Connected,
    /// Connection lost, attempting to reconnect
    Reconnecting,
    /// Connection failed
    Failed,
}

impl ConnectionStatus {
    /// Returns true if currently connected
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected)
    }

    /// Returns the status indicator color
    pub fn indicator_color(&self) -> &'static str {
        match self {
            ConnectionStatus::Connected => "#4ade80",    // green
            ConnectionStatus::Connecting => "#facc15",   // yellow
            ConnectionStatus::Reconnecting => "#facc15", // yellow
            ConnectionStatus::Disconnected => "#f87171", // red
            ConnectionStatus::Failed => "#ef4444",       // dark red
        }
    }

    /// Returns the status display text
    pub fn display_text(&self) -> &'static str {
        match self {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Reconnecting => "Reconnecting...",
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Failed => "Connection Failed",
        }
    }
}

/// A pending approval request from the LLM that the DM needs to review
#[derive(Debug, Clone, PartialEq)]
pub struct PendingApproval {
    /// Unique request ID for this approval
    pub request_id: String,
    /// Name of the NPC responding
    pub npc_name: String,
    /// The proposed dialogue from the LLM
    pub proposed_dialogue: String,
    /// Internal reasoning from the LLM
    pub internal_reasoning: String,
    /// Proposed tool calls
    pub proposed_tools: Vec<ProposedTool>,
    /// Optional challenge suggestion from the Engine
    pub challenge_suggestion: Option<ChallengeSuggestionInfo>,
}

/// Challenge prompt data shown to player
#[derive(Debug, Clone, PartialEq)]
pub struct ChallengePromptData {
    /// Unique challenge ID
    pub challenge_id: String,
    /// Human-readable challenge name
    pub challenge_name: String,
    /// Associated skill name
    pub skill_name: String,
    /// Difficulty display (e.g., "DC 12", "Very Hard")
    pub difficulty_display: String,
    /// Challenge description/flavor text
    pub description: String,
    /// Character's skill modifier for this challenge
    pub character_modifier: i32,
}

/// Challenge result data for display
#[derive(Debug, Clone, PartialEq)]
pub struct ChallengeResultData {
    /// Challenge name
    pub challenge_name: String,
    /// Name of character who attempted the challenge
    pub character_name: String,
    /// The d20 roll result
    pub roll: i32,
    /// Applied modifier
    pub modifier: i32,
    /// Total result (roll + modifier)
    pub total: i32,
    /// Outcome type ("success", "failure", "critical_success", etc.)
    pub outcome: String,
    /// Descriptive outcome text
    pub outcome_description: String,
    /// Timestamp for ordering
    pub timestamp: u64,
}

/// A log entry for the conversation
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationLogEntry {
    /// Speaker name (or "System" for system messages)
    pub speaker: String,
    /// The message text
    pub text: String,
    /// Whether this is a system message
    pub is_system: bool,
    /// Timestamp (for ordering)
    pub timestamp: u64,
}

/// Session state for connection and user information
#[derive(Clone)]
pub struct SessionState {
    /// Current connection status
    pub connection_status: Signal<ConnectionStatus>,
    /// Session ID after joining
    pub session_id: Signal<Option<String>>,
    /// User ID (local identifier)
    pub user_id: Signal<Option<String>>,
    /// User role (DungeonMaster, Player, Spectator)
    pub user_role: Signal<Option<ParticipantRole>>,
    /// Server URL we're connected to
    pub server_url: Signal<Option<String>>,
    /// Game connection handle (if connected)
    pub engine_client: Signal<Option<Arc<dyn GameConnectionPort>>>,
    /// Error message if connection failed
    pub error_message: Signal<Option<String>>,
    /// Pending approval requests (for DM)
    pub pending_approvals: Signal<Vec<PendingApproval>>,
    /// Conversation log (for DM view)
    pub conversation_log: Signal<Vec<ConversationLogEntry>>,
    /// Active challenge prompt (if any)
    pub active_challenge: Signal<Option<ChallengePromptData>>,
    /// Recent challenge results for display
    pub challenge_results: Signal<Vec<ChallengeResultData>>,
    /// Player character skills with modifiers
    pub player_skills: Signal<Vec<PlayerSkillData>>,
}

impl SessionState {
    /// Create a new SessionState with disconnected status
    pub fn new() -> Self {
        Self {
            connection_status: Signal::new(ConnectionStatus::Disconnected),
            session_id: Signal::new(None),
            user_id: Signal::new(None),
            user_role: Signal::new(None),
            server_url: Signal::new(None),
            engine_client: Signal::new(None),
            error_message: Signal::new(None),
            pending_approvals: Signal::new(Vec::new()),
            conversation_log: Signal::new(Vec::new()),
            active_challenge: Signal::new(None),
            challenge_results: Signal::new(Vec::new()),
            player_skills: Signal::new(Vec::new()),
        }
    }

    /// Set the connection to connecting state
    pub fn start_connecting(&mut self, server_url: &str) {
        self.connection_status.set(ConnectionStatus::Connecting);
        self.server_url.set(Some(server_url.to_string()));
        self.error_message.set(None);
    }

    /// Set the connection to connected state
    pub fn set_connected(&mut self, client: Arc<dyn GameConnectionPort>) {
        self.connection_status.set(ConnectionStatus::Connected);
        self.engine_client.set(Some(client));
        self.error_message.set(None);
    }

    /// Set the session as joined
    pub fn set_session_joined(&mut self, session_id: String) {
        self.session_id.set(Some(session_id));
    }

    /// Set user information
    pub fn set_user(&mut self, user_id: String, role: ParticipantRole) {
        self.user_id.set(Some(user_id));
        self.user_role.set(Some(role));
    }

    /// Set the connection to disconnected state
    pub fn set_disconnected(&mut self) {
        self.connection_status.set(ConnectionStatus::Disconnected);
        self.engine_client.set(None);
        self.session_id.set(None);
    }

    /// Set the connection to failed state with error
    pub fn set_failed(&mut self, error: String) {
        self.connection_status.set(ConnectionStatus::Failed);
        self.error_message.set(Some(error));
        self.engine_client.set(None);
    }

    /// Set the connection to reconnecting state
    pub fn set_reconnecting(&mut self) {
        self.connection_status.set(ConnectionStatus::Reconnecting);
    }

    /// Clear all session state
    pub fn clear(&mut self) {
        self.connection_status.set(ConnectionStatus::Disconnected);
        self.session_id.set(None);
        self.user_id.set(None);
        self.user_role.set(None);
        self.server_url.set(None);
        self.engine_client.set(None);
        self.error_message.set(None);
        self.pending_approvals.set(Vec::new());
        self.conversation_log.set(Vec::new());
        self.active_challenge.set(None);
        self.challenge_results.set(Vec::new());
        self.player_skills.set(Vec::new());
    }

    /// Add a pending approval request
    pub fn add_pending_approval(&mut self, approval: PendingApproval) {
        self.pending_approvals.write().push(approval);
    }

    /// Remove a pending approval by request_id
    pub fn remove_pending_approval(&mut self, request_id: &str) {
        self.pending_approvals.write().retain(|a| a.request_id != request_id);
    }

    /// Add a conversation log entry
    pub fn add_log_entry(&mut self, speaker: String, text: String, is_system: bool, platform: &Platform) {
        let timestamp = platform.now_unix_secs();
        self.conversation_log.write().push(ConversationLogEntry {
            speaker,
            text,
            is_system,
            timestamp,
        });
    }

    /// Check if we have an active client
    pub fn has_client(&self) -> bool {
        self.engine_client.read().is_some()
    }

    /// Set active challenge prompt
    pub fn set_active_challenge(&mut self, challenge: ChallengePromptData) {
        self.active_challenge.set(Some(challenge));
    }

    /// Clear active challenge
    pub fn clear_active_challenge(&mut self) {
        self.active_challenge.set(None);
    }

    /// Add a challenge result
    pub fn add_challenge_result(&mut self, result: ChallengeResultData) {
        self.challenge_results.write().push(result);
    }

    /// Set player skills
    pub fn set_player_skills(&mut self, skills: Vec<PlayerSkillData>) {
        self.player_skills.set(skills);
    }

    /// Add a player skill
    pub fn add_player_skill(&mut self, skill: PlayerSkillData) {
        self.player_skills.write().push(skill);
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}
