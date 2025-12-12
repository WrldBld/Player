//! DM panel components - Directorial controls for gameplay
//!
//! Provides reusable components for the DM view including scene preview,
//! directorial notes, NPC motivation tracking, LLM response approval,
//! and challenge management.

pub mod approval_popup;
pub mod challenge_library;
pub mod conversation_log;
pub mod directorial_notes;
pub mod npc_motivation;
pub mod scene_preview;
pub mod tone_selector;
pub mod trigger_challenge_modal;

pub use approval_popup::ApprovalPopup;
pub use challenge_library::ChallengeLibrary;
pub use conversation_log::ConversationLog;
pub use directorial_notes::DirectorialNotes;
pub use npc_motivation::NPCMotivation;
pub use scene_preview::ScenePreview;
pub use tone_selector::ToneSelector;
pub use trigger_challenge_modal::TriggerChallengeModal;
