//! DM panel components - Directorial controls for gameplay
//!
//! Provides reusable components for the DM view including scene preview,
//! directorial notes, NPC motivation tracking, and LLM response approval.

pub mod scene_preview;
pub mod directorial_notes;
pub mod npc_motivation;
pub mod approval_popup;
pub mod conversation_log;
pub mod tone_selector;

pub use scene_preview::ScenePreview;
pub use directorial_notes::DirectorialNotes;
pub use npc_motivation::NPCMotivation;
pub use approval_popup::ApprovalPopup;
pub use conversation_log::ConversationLog;
pub use tone_selector::ToneSelector;
