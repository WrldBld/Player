//! Asset Service - Application service for asset management
//!
//! This service provides use case implementations for fetching, generating,
//! activating, and deleting entity assets. It abstracts away the HTTP client
//! details from the presentation layer.

use serde::{Deserialize, Serialize};

use crate::application::ports::outbound::{ApiError, ApiPort};

/// Asset data from API
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Asset {
    pub id: String,
    pub asset_type: String,
    pub label: Option<String>,
    pub is_active: bool,
}

/// Gallery response containing assets
#[derive(Clone, Debug, Deserialize)]
pub struct GalleryResponse {
    pub assets: Vec<Asset>,
}

/// Request to generate new assets
#[derive(Clone, Debug, Serialize)]
pub struct GenerateRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub asset_type: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    pub count: u8,
}

/// Asset service for managing entity assets
///
/// This service provides methods for asset-related operations
/// while depending only on the `ApiPort` trait, not concrete
/// infrastructure implementations.
pub struct AssetService<A: ApiPort> {
    api: A,
}

impl<A: ApiPort> AssetService<A> {
    /// Create a new AssetService with the given API port
    pub fn new(api: A) -> Self {
        Self { api }
    }

    /// Fetch all assets for an entity
    pub async fn get_assets(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<Asset>, ApiError> {
        let path = format!("/api/{}/{}/gallery", entity_type, entity_id);
        let response: GalleryResponse = self.api.get(&path).await?;
        Ok(response.assets)
    }

    /// Activate a specific asset
    pub async fn activate_asset(
        &self,
        entity_type: &str,
        entity_id: &str,
        asset_id: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/api/{}/{}/gallery/{}/activate",
            entity_type, entity_id, asset_id
        );
        self.api.put_empty(&path).await
    }

    /// Delete an asset
    pub async fn delete_asset(
        &self,
        entity_type: &str,
        entity_id: &str,
        asset_id: &str,
    ) -> Result<(), ApiError> {
        let path = format!("/api/{}/{}/gallery/{}", entity_type, entity_id, asset_id);
        self.api.delete(&path).await
    }

    /// Queue asset generation
    pub async fn generate_assets(&self, request: &GenerateRequest) -> Result<(), ApiError> {
        self.api
            .post_no_response("/api/assets/generate", request)
            .await
    }
}

impl<A: ApiPort + Clone> Clone for AssetService<A> {
    fn clone(&self) -> Self {
        Self {
            api: self.api.clone(),
        }
    }
}
