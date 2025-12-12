//! Interaction domain entity
//!
//! Represents available interactions in a scene.

/// An available interaction in a scene
#[derive(Debug, Clone, PartialEq)]
pub struct Interaction {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Type of interaction (e.g., "talk", "examine", "use")
    pub interaction_type: String,
    /// Target name (if any)
    pub target_name: Option<String>,
    /// Whether this interaction is currently available
    pub is_available: bool,
}

impl Interaction {
    /// Create a new interaction
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        interaction_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            interaction_type: interaction_type.into(),
            target_name: None,
            is_available: true,
        }
    }

    /// Set the target name
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target_name = Some(target.into());
        self
    }

    /// Set availability
    pub fn available(mut self, available: bool) -> Self {
        self.is_available = available;
        self
    }
}
