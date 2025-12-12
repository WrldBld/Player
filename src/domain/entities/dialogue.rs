//! Dialogue domain entities
//!
//! Represents dialogue choices in conversations.

/// A dialogue choice presented to the player
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueChoice {
    /// Unique identifier for this choice
    pub id: String,
    /// Display text for the choice
    pub text: String,
    /// Whether this choice allows custom text input
    pub is_custom_input: bool,
}

impl DialogueChoice {
    /// Create a new dialogue choice
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            is_custom_input: false,
        }
    }

    /// Mark this choice as a custom input option
    pub fn as_custom_input(mut self) -> Self {
        self.is_custom_input = true;
        self
    }
}

/// A dialogue response from an NPC
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueResponse {
    /// Speaker ID
    pub speaker_id: String,
    /// Speaker name
    pub speaker_name: String,
    /// The dialogue text
    pub text: String,
    /// Available response choices
    pub choices: Vec<DialogueChoice>,
}

impl DialogueResponse {
    /// Create a new dialogue response
    pub fn new(
        speaker_id: impl Into<String>,
        speaker_name: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            speaker_id: speaker_id.into(),
            speaker_name: speaker_name.into(),
            text: text.into(),
            choices: Vec::new(),
        }
    }

    /// Add choices
    pub fn with_choices(mut self, choices: Vec<DialogueChoice>) -> Self {
        self.choices = choices;
        self
    }
}
