//! Session-related DTOs for application layer
//!
//! These types represent session and challenge data in the application layer,
//! independent of presentation layer state structures.

/// Connection status as seen by application layer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppConnectionStatus {
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

/// A pending approval request from the LLM that the DM needs to review
#[derive(Debug, Clone, PartialEq)]
pub struct AppPendingApproval {
    /// Unique request ID for this approval
    pub request_id: String,
    /// Name of the NPC responding
    pub npc_name: String,
    /// The proposed dialogue from the LLM
    pub proposed_dialogue: String,
    /// Internal reasoning from the LLM
    pub internal_reasoning: String,
    /// Proposed tool calls (re-export from dto mod)
    pub proposed_tools: Vec<crate::application::dto::ProposedTool>,
    /// Optional challenge suggestion from the Engine
    pub challenge_suggestion: Option<crate::application::dto::ChallengeSuggestionInfo>,
}

/// Challenge prompt data shown to player
#[derive(Debug, Clone, PartialEq)]
pub struct AppChallengePromptData {
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
pub struct AppChallengeResultData {
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
pub struct AppConversationLogEntry {
    /// Speaker name (or "System" for system messages)
    pub speaker: String,
    /// The message text
    pub text: String,
    /// Whether this is a system message
    pub is_system: bool,
    /// Timestamp (for ordering)
    pub timestamp: u64,
}
