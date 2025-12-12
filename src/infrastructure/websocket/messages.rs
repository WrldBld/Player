//! WebSocket message types for Engine communication

use serde::{Deserialize, Serialize};

/// Messages sent from Player to Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Join a game session
    JoinSession {
        user_id: String,
        role: ParticipantRole,
    },
    /// Player performs an action
    PlayerAction {
        action_type: String,
        target: Option<String>,
        dialogue: Option<String>,
    },
    /// Request to change scene
    RequestSceneChange {
        scene_id: String,
    },
    /// DM updates directorial context
    DirectorialUpdate {
        context: DirectorialContext,
    },
    /// DM approves/rejects LLM response
    ApprovalDecision {
        request_id: String,
        decision: ApprovalDecision,
    },
    /// DM triggers a challenge for a character
    TriggerChallenge {
        challenge_id: String,
        target_character_id: String,
    },
    /// Player submits a challenge roll
    ChallengeRoll {
        challenge_id: String,
        roll: i32,
    },
    /// Heartbeat ping
    Heartbeat,
}

/// Messages received from Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Session successfully joined
    SessionJoined {
        session_id: String,
        world_snapshot: serde_json::Value, // WorldSnapshot as JSON
    },
    /// Scene update
    SceneUpdate {
        scene: SceneData,
        characters: Vec<CharacterData>,
        interactions: Vec<InteractionData>,
    },
    /// NPC dialogue response
    DialogueResponse {
        speaker_id: String,
        speaker_name: String,
        text: String,
        choices: Vec<DialogueChoice>,
    },
    /// LLM is processing (shown to DM)
    LLMProcessing {
        action_id: String,
    },
    /// Approval required (sent to DM)
    ApprovalRequired {
        request_id: String,
        npc_name: String,
        proposed_dialogue: String,
        internal_reasoning: String,
        proposed_tools: Vec<ProposedTool>,
        challenge_suggestion: Option<ChallengeSuggestionInfo>,
    },
    /// Response was approved and executed
    ResponseApproved {
        npc_dialogue: String,
        executed_tools: Vec<String>,
    },
    /// Error message
    Error {
        message: String,
        code: Option<String>,
    },
    /// Heartbeat response
    Pong,

    /// Challenge prompt sent to player
    ChallengePrompt {
        challenge_id: String,
        challenge_name: String,
        skill_name: String,
        difficulty_display: String,
        description: String,
        character_modifier: i32,
    },

    /// Challenge result broadcast to all
    ChallengeResolved {
        challenge_id: String,
        challenge_name: String,
        character_name: String,
        roll: i32,
        modifier: i32,
        total: i32,
        outcome: String,  // "success", "failure", "critical_success", etc.
        outcome_description: String,
    },

    // Generation events (for Creator Mode)
    /// A generation batch has been queued
    GenerationQueued {
        batch_id: String,
        entity_type: String,
        entity_id: String,
        asset_type: String,
        position: u32,
    },
    /// Generation progress update
    GenerationProgress {
        batch_id: String,
        progress: u8,
    },
    /// Generation batch completed
    GenerationComplete {
        batch_id: String,
        asset_count: u32,
    },
    /// Generation batch failed
    GenerationFailed {
        batch_id: String,
        error: String,
    },
}

/// Participant role in the session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantRole {
    DungeonMaster,
    Player,
    Spectator,
}

/// Scene data from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    pub id: String,
    pub name: String,
    pub location_id: String,
    pub location_name: String,
    pub backdrop_asset: Option<String>,
    pub time_context: String,
    pub directorial_notes: String,
}

/// Character data for display
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterData {
    pub id: String,
    pub name: String,
    pub sprite_asset: Option<String>,
    pub portrait_asset: Option<String>,
    pub position: CharacterPosition,
    pub is_speaking: bool,
}

/// Character position on screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterPosition {
    Left,
    Center,
    Right,
    OffScreen,
}

/// Available interaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionData {
    pub id: String,
    pub name: String,
    pub interaction_type: String,
    pub target_name: Option<String>,
    pub is_available: bool,
}

/// Dialogue choice for player
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub id: String,
    pub text: String,
    pub is_custom_input: bool,
}

/// Directorial context from DM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorialContext {
    pub scene_notes: String,
    pub tone: String,
    pub npc_motivations: Vec<NpcMotivationData>,
    pub forbidden_topics: Vec<String>,
}

/// NPC motivation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcMotivationData {
    pub character_id: String,
    pub mood: String,
    pub immediate_goal: String,
    pub secret_agenda: Option<String>,
}

/// DM's approval decision
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "decision")]
pub enum ApprovalDecision {
    Accept,
    AcceptWithModification {
        modified_dialogue: String,
        approved_tools: Vec<String>,
        rejected_tools: Vec<String>,
    },
    Reject {
        feedback: String,
    },
    TakeOver {
        dm_response: String,
    },
}

/// Proposed tool call from LLM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposedTool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub arguments: serde_json::Value,
}

/// Challenge suggestion from Engine based on action context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChallengeSuggestionInfo {
    pub challenge_id: String,
    pub challenge_name: String,
    pub skill_name: String,
    pub difficulty_display: String,
    pub confidence: String,
    pub reasoning: String,
}
