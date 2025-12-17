//! Settings Service - Application service for managing Engine settings
//!
//! This service provides use case implementations for fetching and
//! updating the Engine's application settings. It abstracts away
//! the HTTP client details from the presentation layer.

use crate::application::dto::AppSettings;
use crate::application::ports::outbound::{ApiError, ApiPort};

/// Settings service for managing Engine application settings
///
/// This service provides methods for settings-related operations
/// while depending only on the `ApiPort` trait, not concrete
/// infrastructure implementations.
pub struct SettingsService<A: ApiPort> {
    api: A,
}

impl<A: ApiPort> SettingsService<A> {
    /// Create a new SettingsService with the given API port
    pub fn new(api: A) -> Self {
        Self { api }
    }

    /// Get current application settings from the Engine
    pub async fn get(&self) -> Result<AppSettings, ApiError> {
        self.api.get("/api/settings").await
    }

    /// Update application settings on the Engine
    ///
    /// # Arguments
    /// * `settings` - The updated settings to save
    ///
    /// # Returns
    /// The updated settings as confirmed by the Engine
    pub async fn update(&self, settings: &AppSettings) -> Result<AppSettings, ApiError> {
        self.api.put("/api/settings", settings).await
    }

    /// Reset application settings to defaults
    ///
    /// # Returns
    /// The default settings as confirmed by the Engine
    pub async fn reset(&self) -> Result<AppSettings, ApiError> {
        self.api.post("/api/settings/reset", &()).await
    }
}
