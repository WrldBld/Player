//! Asset loader - Sprites, backdrops, world snapshots

mod world_snapshot;

pub use world_snapshot::{
    WorldSnapshot, WorldSnapshotLoader,
    SnapshotMetadata, WorldData, ActData, SceneData,
    CharacterData, WantData, LocationData, BackdropRegionData,
    RegionBoundsData, RelationshipData, ConnectionData,
    RuleSystemConfig, StatDefinition, DiceSystem,
};
