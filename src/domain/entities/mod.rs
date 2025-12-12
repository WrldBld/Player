//! Domain entities

pub mod approval;
pub mod character;
pub mod dialogue;
pub mod interaction;
pub mod location;
pub mod player_action;
pub mod scene;
pub mod world;

pub use approval::{ChallengeSuggestion, ProposedTool};
pub use character::{Character, CharacterPosition};
pub use dialogue::{DialogueChoice, DialogueResponse};
pub use interaction::Interaction;
pub use location::{Location, LocationSummary};
pub use player_action::{PlayerAction, PlayerActionType};
pub use scene::Scene;
pub use world::{World, WorldSummary};
