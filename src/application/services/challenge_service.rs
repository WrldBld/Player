//! Challenge Service - Application service for challenge management
//!
//! This service provides use case implementations for listing, creating,
//! updating, and managing challenges. It abstracts away the HTTP client
//! details from the presentation layer.

use serde::{Deserialize, Serialize};

use crate::application::ports::outbound::{ApiError, ApiPort};

/// Challenge difficulty
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChallengeDifficulty {
    Trivial,
    Easy,
    Medium,
    Hard,
    Extreme,
    Custom(i32),
}

/// Challenge outcome description
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChallengeOutcomes {
    pub critical_success: String,
    pub success: String,
    pub failure: String,
    pub critical_failure: String,
}

/// Full challenge data
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChallengeData {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub description: String,
    pub skill_id: String,
    pub difficulty: ChallengeDifficulty,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcomes: Option<ChallengeOutcomes>,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Challenge service for managing challenges
///
/// This service provides methods for challenge-related operations
/// while depending only on the `ApiPort` trait, not concrete
/// infrastructure implementations.
pub struct ChallengeService<A: ApiPort> {
    api: A,
}

impl<A: ApiPort> ChallengeService<A> {
    /// Create a new ChallengeService with the given API port
    pub fn new(api: A) -> Self {
        Self { api }
    }

    /// List all challenges in a world
    pub async fn list_challenges(&self, world_id: &str) -> Result<Vec<ChallengeData>, ApiError> {
        let path = format!("/api/worlds/{}/challenges", world_id);
        self.api.get(&path).await
    }

    /// Get a single challenge by ID
    pub async fn get_challenge(&self, challenge_id: &str) -> Result<ChallengeData, ApiError> {
        let path = format!("/api/challenges/{}", challenge_id);
        self.api.get(&path).await
    }

    /// Create a new challenge
    pub async fn create_challenge(
        &self,
        world_id: &str,
        challenge: &ChallengeData,
    ) -> Result<ChallengeData, ApiError> {
        let path = format!("/api/worlds/{}/challenges", world_id);
        self.api.post(&path, challenge).await
    }

    /// Update an existing challenge
    pub async fn update_challenge(
        &self,
        challenge: &ChallengeData,
    ) -> Result<ChallengeData, ApiError> {
        let path = format!("/api/challenges/{}", challenge.id);
        self.api.put(&path, challenge).await
    }

    /// Delete a challenge
    pub async fn delete_challenge(&self, challenge_id: &str) -> Result<(), ApiError> {
        let path = format!("/api/challenges/{}", challenge_id);
        self.api.delete(&path).await
    }

    /// Toggle challenge favorite status
    pub async fn toggle_favorite(&self, challenge_id: &str) -> Result<bool, ApiError> {
        let path = format!("/api/challenges/{}/favorite", challenge_id);
        self.api.put_empty_with_response(&path).await
    }

    /// Set challenge active status
    pub async fn set_active(&self, challenge_id: &str, active: bool) -> Result<(), ApiError> {
        let path = format!("/api/challenges/{}/active", challenge_id);
        self.api.put_no_response(&path, &active).await
    }
}

impl<A: ApiPort + Clone> Clone for ChallengeService<A> {
    fn clone(&self) -> Self {
        Self {
            api: self.api.clone(),
        }
    }
}
