//! Value objects
//!
//! Immutable types that represent concepts in the domain.

pub mod ids;

pub use ids::{
    WorldId, ActId, SceneId, CharacterId, LocationId,
    InteractionId, SkillId, ChallengeId, AssetId, SessionId, UserId,
};
