//! Application services
//!
//! This module contains application services that implement use cases
//! for the WrldBldr Player. Services depend on port traits, not concrete
//! infrastructure implementations.

pub mod action_service;
pub mod challenge_service;
pub mod character_service;
pub mod location_service;
pub mod session_service;
pub mod skill_service;
pub mod world_service;

// Re-export action service
pub use action_service::ActionService;

// Re-export session service types
pub use session_service::{
    connection_state_to_status, port_connection_state_to_status,
    ConnectionStatePort, ParticipantRolePort,
    SessionService, DEFAULT_ENGINE_URL,
};

#[cfg(not(target_arch = "wasm32"))]
pub use session_service::{handle_session_event, SessionEvent};

// Re-export world service types
pub use world_service::{CreateWorldRequest, CreateWorldResponse, WorldService, WorldSummary};

// Re-export character service types
pub use character_service::{CharacterData, CharacterService, CharacterSummary};

// Re-export location service types
pub use location_service::{ConnectionData, LocationData, LocationService, LocationSummary};

// Re-export skill service types
pub use skill_service::{
    CreateSkillRequest, SkillCategory, SkillData, SkillService, UpdateSkillRequest,
};

// Re-export challenge service types
pub use challenge_service::{
    ChallengeData, ChallengeDifficulty, ChallengeOutcomes, ChallengeService,
};
