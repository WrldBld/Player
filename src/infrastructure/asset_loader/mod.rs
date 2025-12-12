//! Asset loader - Sprites, backdrops, world snapshots

mod world_snapshot;

pub use world_snapshot::{
    WorldSnapshot, WorldSnapshotLoader,
    SnapshotMetadata, WorldData, ActData, SceneData,
    CharacterData, WantData, LocationData, BackdropRegionData,
    RegionBoundsData, RelationshipData, ConnectionData,
    RuleSystemConfig, RuleSystemType, RuleSystemVariant,
    StatDefinition, DiceSystem, SuccessComparison,
    SkillData, SkillCategory,
    // Character Sheet Template types
    SheetTemplate, SheetSection, SheetField, SectionLayout,
    FieldType, SelectOption, ItemListType,
    CharacterSheetData, FieldValue,
    // Challenge types
    ChallengeData, ChallengeType, ChallengeDifficulty,
    ChallengeOutcomes, Outcome, OutcomeTrigger,
    TriggerCondition, TriggerType,
    // Story Arc types (Phase 17)
    StoryEventData, StoryEventTypeData, MarkerImportance, DmMarkerType,
    NarrativeEventData, EventChainData, ChainedEventData,
};
