//! Location Service - Application service for location management
//!
//! This service provides use case implementations for listing, creating,
//! updating, and fetching locations. It abstracts away the HTTP client
//! details from the presentation layer.

use serde::{Deserialize, Serialize};

use crate::application::ports::outbound::{ApiError, ApiPort};

/// Location summary for list views
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocationSummary {
    pub id: String,
    pub name: String,
    pub location_type: Option<String>,
}

/// Full location data for editing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocationData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub location_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop_asset: Option<String>,
    #[serde(default)]
    pub backdrop_regions: Vec<serde_json::Value>,
}

/// Location connection data
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConnectionData {
    pub from_location_id: String,
    pub to_location_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_bidirectional")]
    pub bidirectional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub travel_time: Option<u32>,
}

fn default_bidirectional() -> bool {
    true
}

/// Location service for managing locations
///
/// This service provides methods for location-related operations
/// while depending only on the `ApiPort` trait, not concrete
/// infrastructure implementations.
pub struct LocationService<A: ApiPort> {
    api: A,
}

impl<A: ApiPort> LocationService<A> {
    /// Create a new LocationService with the given API port
    pub fn new(api: A) -> Self {
        Self { api }
    }

    /// List all locations in a world
    pub async fn list_locations(&self, world_id: &str) -> Result<Vec<LocationSummary>, ApiError> {
        let path = format!("/api/worlds/{}/locations", world_id);
        self.api.get(&path).await
    }

    /// Get a single location by ID
    pub async fn get_location(
        &self,
        world_id: &str,
        location_id: &str,
    ) -> Result<LocationData, ApiError> {
        let path = format!("/api/worlds/{}/locations/{}", world_id, location_id);
        self.api.get(&path).await
    }

    /// Create a new location
    pub async fn create_location(
        &self,
        world_id: &str,
        location: &LocationData,
    ) -> Result<LocationData, ApiError> {
        let path = format!("/api/worlds/{}/locations", world_id);
        self.api.post(&path, location).await
    }

    /// Update an existing location
    pub async fn update_location(
        &self,
        location_id: &str,
        location: &LocationData,
    ) -> Result<LocationData, ApiError> {
        let path = format!("/api/locations/{}", location_id);
        self.api.put(&path, location).await
    }

    /// Delete a location
    pub async fn delete_location(&self, location_id: &str) -> Result<(), ApiError> {
        let path = format!("/api/locations/{}", location_id);
        self.api.delete(&path).await
    }

    /// Get connections from a location
    pub async fn get_connections(
        &self,
        location_id: &str,
    ) -> Result<Vec<ConnectionData>, ApiError> {
        let path = format!("/api/locations/{}/connections", location_id);
        self.api.get(&path).await
    }

    /// Create a connection between locations
    pub async fn create_connection(&self, connection: &ConnectionData) -> Result<(), ApiError> {
        self.api
            .post_no_response("/api/connections", connection)
            .await
    }
}

impl<A: ApiPort + Clone> Clone for LocationService<A> {
    fn clone(&self) -> Self {
        Self {
            api: self.api.clone(),
        }
    }
}
