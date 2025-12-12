//! Approval domain entities
//!
//! Represents the DM approval workflow for LLM responses.

/// A proposed tool call from the LLM
#[derive(Debug, Clone, PartialEq)]
pub struct ProposedTool {
    /// Unique identifier
    pub id: String,
    /// Tool name
    pub name: String,
    /// Description of what the tool will do
    pub description: String,
    /// Tool arguments as JSON
    pub arguments: serde_json::Value,
}

impl ProposedTool {
    /// Create a new proposed tool
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            arguments: serde_json::Value::Null,
        }
    }

    /// Set the arguments
    pub fn with_arguments(mut self, args: serde_json::Value) -> Self {
        self.arguments = args;
        self
    }
}

/// Challenge suggestion from the Engine
#[derive(Debug, Clone, PartialEq)]
pub struct ChallengeSuggestion {
    /// Challenge ID
    pub challenge_id: String,
    /// Challenge name
    pub challenge_name: String,
    /// Associated skill name
    pub skill_name: String,
    /// Difficulty display string (e.g., "DC 15")
    pub difficulty_display: String,
    /// Confidence level
    pub confidence: String,
    /// Reasoning for the suggestion
    pub reasoning: String,
}

impl ChallengeSuggestion {
    /// Create a new challenge suggestion
    pub fn new(
        challenge_id: impl Into<String>,
        challenge_name: impl Into<String>,
        skill_name: impl Into<String>,
    ) -> Self {
        Self {
            challenge_id: challenge_id.into(),
            challenge_name: challenge_name.into(),
            skill_name: skill_name.into(),
            difficulty_display: String::new(),
            confidence: String::new(),
            reasoning: String::new(),
        }
    }

    /// Set difficulty display
    pub fn with_difficulty(mut self, difficulty: impl Into<String>) -> Self {
        self.difficulty_display = difficulty.into();
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: impl Into<String>) -> Self {
        self.confidence = confidence.into();
        self
    }

    /// Set reasoning
    pub fn with_reasoning(mut self, reasoning: impl Into<String>) -> Self {
        self.reasoning = reasoning.into();
        self
    }
}
