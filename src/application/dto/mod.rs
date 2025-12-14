//! Data transfer objects
//!
//! DTOs are used to transfer data between layers. The application layer
//! provides these types so that presentation doesn't need to import
//! directly from infrastructure.
//!
//! TODO (Phase 16.3): replace infra re-exports with real application DTOs + conversions.

pub mod session_dto;
pub mod websocket_messages;

// Re-export session DTOs
pub use session_dto::{
    AppConnectionStatus, AppPendingApproval, AppChallengePromptData,
    AppChallengeResultData, AppConversationLogEntry,
};

// Re-export WebSocket protocol DTOs (application-owned).
pub use websocket_messages::*;

// Re-export infrastructure types through the application layer
// This allows presentation to import from application instead of infrastructure
pub use crate::infrastructure::asset_loader::{
    // World snapshot types
    WorldSnapshot, SnapshotMetadata, WorldData, ActData,
    SceneData as SnapshotSceneData, CharacterData as SnapshotCharacterData,
    LocationData, BackdropRegionData, RegionBoundsData,
    RelationshipData, ConnectionData, WantData,
    // Rule system types
    RuleSystemConfig, RuleSystemType, RuleSystemVariant,
    StatDefinition, DiceSystem, SuccessComparison,
    // Skill types
    SkillData, SkillCategory,
    // Character sheet types
    SheetTemplate, SheetSection, SheetField, SectionLayout,
    FieldType, SelectOption, ItemListType,
    CharacterSheetData, FieldValue,
    // Challenge types
    ChallengeData, ChallengeType, ChallengeDifficulty,
    ChallengeOutcomes, Outcome, OutcomeTrigger,
    TriggerCondition, TriggerType,
    // Story arc types
    StoryEventData, StoryEventTypeData, MarkerImportance, DmMarkerType,
    NarrativeEventData, EventChainData, ChainedEventData,
    // Session snapshot types (simplified format from Engine)
    SessionWorldSnapshot, SessionWorldData, SessionLocationData,
    SessionCharacterData, SessionSceneData,
};

// NOTE (Phase 16.3): asset_loader types are still infrastructure-owned for now.
// Next step is to introduce application-owned snapshot DTOs and make the loader depend inward.
