//! Domain entities

pub mod approval;
pub mod character;
pub mod dialogue;
pub mod interaction;
pub mod location;
pub mod player_action;
pub mod scene;
pub mod world;

// Only re-export what is currently used outside the domain module.
pub use player_action::PlayerAction;
