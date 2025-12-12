//! Presentation layer - Dioxus UI components and views

pub mod components;
pub mod services;
pub mod state;
pub mod views;

// Re-export service hooks for convenience
pub use services::{
    Services, use_world_service, use_character_service, use_location_service,
    use_skill_service, use_challenge_service,
};
