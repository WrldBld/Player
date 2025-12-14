//! Application services
//!
//! This module contains application services that implement use cases
//! for the WrldBldr Player. Services depend on port traits, not concrete
//! infrastructure implementations.

pub mod action_service;
pub mod asset_service;
pub mod challenge_service;
pub mod character_service;
pub mod location_service;
pub mod narrative_event_service;
pub mod session_service;
pub mod session_command_service;
pub mod skill_service;
pub mod story_event_service;
pub mod suggestion_service;
pub mod workflow_service;
pub mod world_service;

// Re-export action service
pub use action_service::ActionService;

// Re-export session command service
pub use session_command_service::SessionCommandService;

// Re-export session service types
pub use session_service::{
    connection_state_to_status, port_connection_state_to_status,
    ConnectionStatePort, ParticipantRolePort,
    SessionService, DEFAULT_ENGINE_URL,
};

#[cfg(not(target_arch = "wasm32"))]
pub use session_service::SessionEvent;

// Re-export world service types
pub use world_service::{CreateWorldRequest, CreateWorldResponse, WorldService, WorldSummary};

// Re-export character service types
pub use character_service::{CharacterData, CharacterService, CharacterSheetDataApi, CharacterSummary};

// Re-export location service types
pub use location_service::{ConnectionData, LocationData, LocationService, LocationSummary};

// Re-export skill service types
pub use skill_service::{CreateSkillRequest, SkillService, UpdateSkillRequest};
// Re-export SkillData and SkillCategory from dto (not skill_service)
pub use crate::application::dto::{SkillCategory, SkillData};

// Re-export challenge service types
pub use challenge_service::ChallengeService;
// Re-export ChallengeData, ChallengeDifficulty, ChallengeOutcomes from dto (not challenge_service)
pub use crate::application::dto::{ChallengeData, ChallengeDifficulty, ChallengeOutcomes};

// Re-export story event service types
pub use story_event_service::{
    CreateDmMarkerRequest, PaginatedStoryEventsResponse, StoryEventService,
};

// Re-export narrative event service types
pub use narrative_event_service::NarrativeEventService;

// Re-export workflow service types
pub use workflow_service::{
    AnalyzeWorkflowResponse, InputDefault, PromptMapping, WorkflowAnalysis, WorkflowConfig,
    WorkflowConfigBrief, WorkflowInput, WorkflowService, WorkflowSlotCategory,
    WorkflowSlotStatus, WorkflowSlotsResponse, TestWorkflowResponse,
};

// Re-export asset service types
pub use asset_service::{Asset, AssetService, GalleryResponse, GenerateRequest};

// Re-export suggestion service types
pub use suggestion_service::{SuggestionContext, SuggestionResponse, SuggestionService};
