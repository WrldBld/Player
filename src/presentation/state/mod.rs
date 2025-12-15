//! Application state management
//!
//! Central state management using Dioxus signals and context.

pub mod dialogue_state;
pub mod game_state;
pub mod generation_state;
pub mod session_state;

pub use dialogue_state::{use_typewriter_effect, DialogueState};
pub use game_state::GameState;
pub use generation_state::{BatchStatus, GenerationBatch, GenerationState, SuggestionStatus, SuggestionTask};
pub use session_state::{ConnectionStatus, PendingApproval, SessionState};

use dioxus::prelude::*;

/// Get the game state from context
///
/// # Panics
/// Panics if GameState has not been provided via use_context_provider
pub fn use_game_state() -> GameState {
    use_context::<GameState>()
}

/// Get the session state from context
///
/// # Panics
/// Panics if SessionState has not been provided via use_context_provider
pub fn use_session_state() -> SessionState {
    use_context::<SessionState>()
}

/// Get the dialogue state from context
///
/// # Panics
/// Panics if DialogueState has not been provided via use_context_provider
pub fn use_dialogue_state() -> DialogueState {
    use_context::<DialogueState>()
}

/// Get the generation state from context
///
/// # Panics
/// Panics if GenerationState has not been provided via use_context_provider
pub fn use_generation_state() -> GenerationState {
    use_context::<GenerationState>()
}
