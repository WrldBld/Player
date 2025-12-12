//! Data transfer objects
//!
//! DTOs are used to transfer data between layers. The application layer
//! provides these types so that presentation doesn't need to import
//! directly from infrastructure.
//!
//! Phase 8 will move many of these to proper domain entities.

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
};

// Re-export websocket message types through the application layer
pub use crate::infrastructure::websocket::{
    // Server message types (for message handling)
    ServerMessage,
    // Scene display types
    SceneData, CharacterData, InteractionData,
    CharacterPosition, DialogueChoice,
    // DM types
    DirectorialContext, NpcMotivationData,
    ApprovalDecision, ProposedTool, ChallengeSuggestionInfo,
    // Client messages
    ClientMessage,
};
