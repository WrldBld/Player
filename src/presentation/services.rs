//! Service providers for the presentation layer
//!
//! This module provides Dioxus context providers for application services.
//! Components can use `use_context` to access services without depending
//! on infrastructure implementations.

use dioxus::prelude::*;
use std::sync::Arc;

use crate::application::services::{
    AssetService, CharacterService, ChallengeService, LocationService, NarrativeEventService,
    PlayerCharacterService, SkillService, StoryEventService, SuggestionService, WorkflowService, WorldService,
};
use crate::infrastructure::http_client::ApiAdapter;

/// Type aliases for concrete service types
pub type WorldSvc = WorldService<ApiAdapter>;
pub type CharacterSvc = CharacterService<ApiAdapter>;
pub type LocationSvc = LocationService<ApiAdapter>;
pub type PlayerCharacterSvc = PlayerCharacterService<ApiAdapter>;
pub type SkillSvc = SkillService<ApiAdapter>;
pub type ChallengeSvc = ChallengeService<ApiAdapter>;
pub type StoryEventSvc = StoryEventService<ApiAdapter>;
pub type NarrativeEventSvc = NarrativeEventService<ApiAdapter>;
pub type WorkflowSvc = WorkflowService<ApiAdapter>;
pub type AssetSvc = AssetService<ApiAdapter>;
pub type SuggestionSvc = SuggestionService<ApiAdapter>;

/// All services wrapped for context provision
#[derive(Clone)]
pub struct Services {
    pub world: Arc<WorldSvc>,
    pub character: Arc<CharacterSvc>,
    pub location: Arc<LocationSvc>,
    pub player_character: Arc<PlayerCharacterSvc>,
    pub skill: Arc<SkillSvc>,
    pub challenge: Arc<ChallengeSvc>,
    pub story_event: Arc<StoryEventSvc>,
    pub narrative_event: Arc<NarrativeEventSvc>,
    pub workflow: Arc<WorkflowSvc>,
    pub asset: Arc<AssetSvc>,
    pub suggestion: Arc<SuggestionSvc>,
}

impl Default for Services {
    fn default() -> Self {
        Self::new()
    }
}

impl Services {
    /// Create all services with the default API adapter
    pub fn new() -> Self {
        let api = ApiAdapter::new();
        Self {
            world: Arc::new(WorldService::new(api.clone())),
            character: Arc::new(CharacterService::new(api.clone())),
            location: Arc::new(LocationService::new(api.clone())),
            player_character: Arc::new(PlayerCharacterService::new(api.clone())),
            skill: Arc::new(SkillService::new(api.clone())),
            challenge: Arc::new(ChallengeService::new(api.clone())),
            story_event: Arc::new(StoryEventService::new(api.clone())),
            narrative_event: Arc::new(NarrativeEventService::new(api.clone())),
            workflow: Arc::new(WorkflowService::new(api.clone())),
            asset: Arc::new(AssetService::new(api.clone())),
            suggestion: Arc::new(SuggestionService::new(api)),
        }
    }
}

/// Hook to access the WorldService from context
pub fn use_world_service() -> Arc<WorldSvc> {
    let services = use_context::<Services>();
    services.world.clone()
}

/// Hook to access the CharacterService from context
pub fn use_character_service() -> Arc<CharacterSvc> {
    let services = use_context::<Services>();
    services.character.clone()
}

/// Hook to access the LocationService from context
pub fn use_location_service() -> Arc<LocationSvc> {
    let services = use_context::<Services>();
    services.location.clone()
}

/// Hook to access the PlayerCharacterService from context
pub fn use_player_character_service() -> Arc<PlayerCharacterSvc> {
    let services = use_context::<Services>();
    services.player_character.clone()
}

/// Hook to access the SkillService from context
pub fn use_skill_service() -> Arc<SkillSvc> {
    let services = use_context::<Services>();
    services.skill.clone()
}

/// Hook to access the ChallengeService from context
pub fn use_challenge_service() -> Arc<ChallengeSvc> {
    let services = use_context::<Services>();
    services.challenge.clone()
}

/// Hook to access the StoryEventService from context
pub fn use_story_event_service() -> Arc<StoryEventSvc> {
    let services = use_context::<Services>();
    services.story_event.clone()
}

/// Hook to access the NarrativeEventService from context
pub fn use_narrative_event_service() -> Arc<NarrativeEventSvc> {
    let services = use_context::<Services>();
    services.narrative_event.clone()
}

/// Hook to access the WorkflowService from context
pub fn use_workflow_service() -> Arc<WorkflowSvc> {
    let services = use_context::<Services>();
    services.workflow.clone()
}

/// Hook to access the AssetService from context
pub fn use_asset_service() -> Arc<AssetSvc> {
    let services = use_context::<Services>();
    services.asset.clone()
}

/// Hook to access the SuggestionService from context
pub fn use_suggestion_service() -> Arc<SuggestionSvc> {
    let services = use_context::<Services>();
    services.suggestion.clone()
}

use crate::presentation::state::{BatchStatus, GenerationBatch, GenerationState, SuggestionStatus, SuggestionTask};
use crate::infrastructure::storage;
use crate::infrastructure::http_client::HttpClient;
use crate::application::ports::outbound::Platform;
use anyhow::Result;

/// Hydrate GenerationState from the Engine's unified generation queue endpoint.
pub async fn hydrate_generation_queue(
    _platform: &Platform,
    generation_state: &mut GenerationState,
    user_id: Option<String>,
) -> Result<()> {
    let snapshot: GenerationQueueSnapshotDto =
        if let Some(uid) = user_id {
            crate::infrastructure::http_client::HttpClient::get(&format!(
                "/api/generation/queue?user_id={}",
                uid
            ))
            .await?
        } else {
            crate::infrastructure::http_client::HttpClient::get("/api/generation/queue").await?
        };

    #[derive(serde::Deserialize)]
    struct GenerationQueueSnapshotDto {
        batches: Vec<BatchDto>,
        suggestions: Vec<SuggestionDto>,
    }

    #[derive(serde::Deserialize)]
    struct BatchDto {
        batch_id: String,
        entity_type: String,
        entity_id: String,
        asset_type: String,
        status: String,
        position: Option<u32>,
        progress: Option<u8>,
        asset_count: Option<u32>,
        error: Option<String>,
        #[serde(default)]
        is_read: bool,
    }

    #[derive(serde::Deserialize)]
    struct SuggestionDto {
        request_id: String,
        field_type: String,
        entity_id: Option<String>,
        status: String,
        suggestions: Option<Vec<String>>,
        error: Option<String>,
        #[serde(default)]
        is_read: bool,
    }

    // Clear existing state and repopulate from snapshot
    generation_state.clear();

    for b in snapshot.batches {
        let status = match b.status.as_str() {
            "queued" => BatchStatus::Queued {
                position: b.position.unwrap_or(0),
            },
            "generating" => BatchStatus::Generating {
                progress: b.progress.unwrap_or(0),
            },
            "ready" => BatchStatus::Ready {
                asset_count: b.asset_count.unwrap_or(0),
            },
            "failed" => BatchStatus::Failed {
                error: b.error.unwrap_or_else(|| "Unknown error".to_string()),
            },
            _ => BatchStatus::Queued { position: 0 },
        };

        generation_state.add_batch(crate::presentation::state::GenerationBatch {
            batch_id: b.batch_id,
            entity_type: b.entity_type,
            entity_id: b.entity_id,
            asset_type: b.asset_type,
            status,
            is_read: b.is_read,
        });
    }

    for s in snapshot.suggestions {
        let status = match s.status.as_str() {
            "queued" => SuggestionStatus::Queued,
            "processing" => SuggestionStatus::Processing,
            "ready" => SuggestionStatus::Ready {
                suggestions: s.suggestions.unwrap_or_default(),
            },
            "failed" => SuggestionStatus::Failed {
                error: s.error.unwrap_or_else(|| "Unknown error".to_string()),
            },
            _ => SuggestionStatus::Queued,
        };

        generation_state.add_suggestion_task(
            s.request_id.clone(),
            s.field_type,
            s.entity_id,
        );
        // Override status if needed using the same request_id
        let req_id = s.request_id;
        match status {
            SuggestionStatus::Queued => {}
            SuggestionStatus::Processing => {
                generation_state.suggestion_progress(&req_id, "processing");
            }
            SuggestionStatus::Ready { suggestions } => {
                generation_state.suggestion_complete(&req_id, suggestions);
            }
            SuggestionStatus::Failed { error } => {
                generation_state.suggestion_failed(&req_id, error);
            }
        }
    }

    // Re-apply persisted read/unread state based on local storage (secondary layer)
    apply_generation_read_state(generation_state);

    Ok(())
}

const STORAGE_KEY_GEN_READ_BATCHES: &str = "wrldbldr_gen_read_batches";
const STORAGE_KEY_GEN_READ_SUGGESTIONS: &str = "wrldbldr_gen_read_suggestions";

/// Persist the read/unread state of generation queue items to local storage
pub fn persist_generation_read_state(state: &GenerationState) {
    // Persist read batch IDs
    let read_batch_ids: Vec<String> = state
        .get_batches()
        .into_iter()
        .filter(|b| b.is_read)
        .map(|b| b.batch_id)
        .collect();
    let batch_value = read_batch_ids.join(",");
    storage::save(STORAGE_KEY_GEN_READ_BATCHES, &batch_value);

    // Persist read suggestion IDs
    let read_suggestion_ids: Vec<String> = state
        .get_suggestions()
        .into_iter()
        .filter(|s| s.is_read)
        .map(|s| s.request_id)
        .collect();
    let suggestion_value = read_suggestion_ids.join(",");
    storage::save(STORAGE_KEY_GEN_READ_SUGGESTIONS, &suggestion_value);
}

/// Apply persisted read/unread state from local storage to the current GenerationState
fn apply_generation_read_state(state: &mut GenerationState) {
    if let Some(batch_str) = storage::load(STORAGE_KEY_GEN_READ_BATCHES) {
        for id in batch_str.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            state.mark_batch_read(id);
        }
    }

    if let Some(sugg_str) = storage::load(STORAGE_KEY_GEN_READ_SUGGESTIONS) {
        for id in sugg_str.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            state.mark_suggestion_read(id);
        }
    }
}

/// Sync generation read state to the backend.
///
/// This helper collects all read batches and suggestions from the given state
/// and sends them to the Engine's `/api/generation/read-state` endpoint.
/// The `X-User-Id` header is automatically attached by HttpClient.
///
/// # Arguments
/// * `state` - The GenerationState to sync read markers from
/// * `world_id` - Optional world ID to scope read markers (if None, uses "GLOBAL")
pub async fn sync_generation_read_state(
    state: &GenerationState,
    world_id: Option<&str>,
) -> Result<()> {
    let read_batches: Vec<String> = state
        .get_batches()
        .into_iter()
        .filter(|b| b.is_read)
        .map(|b| b.batch_id)
        .collect();

    let read_suggestions: Vec<String> = state
        .get_suggestions()
        .into_iter()
        .filter(|s| s.is_read)
        .map(|s| s.request_id)
        .collect();

    // Only sync if there are read items
    if read_batches.is_empty() && read_suggestions.is_empty() {
        return Ok(());
    }

    let mut body = serde_json::json!({
        "read_batches": read_batches,
        "read_suggestions": read_suggestions,
    });

    // Add world_id if provided
    if let Some(wid) = world_id {
        body["world_id"] = serde_json::Value::String(wid.to_string());
    }

    HttpClient::post_no_response("/api/generation/read-state", &body)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to sync generation read state: {}", e))?;

    Ok(())
}

/// View-model helpers for generation queue filtering and actions

/// Get visible batches based on show_read filter
pub fn visible_batches(
    state: &GenerationState,
    show_read: bool,
) -> Vec<GenerationBatch> {
    state
        .get_batches()
        .into_iter()
        .filter(|b| show_read || !b.is_read)
        .collect()
}

/// Get visible suggestions based on show_read filter
pub fn visible_suggestions(
    state: &GenerationState,
    show_read: bool,
) -> Vec<SuggestionTask> {
    state
        .get_suggestions()
        .into_iter()
        .filter(|s| show_read || !s.is_read)
        .collect()
}

/// Mark a batch as read and sync to backend
pub async fn mark_batch_read_and_sync(
    state: &mut GenerationState,
    batch_id: &str,
    world_id: Option<&str>,
) -> Result<()> {
    state.mark_batch_read(batch_id);
    persist_generation_read_state(state);
    sync_generation_read_state(state, world_id).await
}

/// Mark a suggestion as read and sync to backend
pub async fn mark_suggestion_read_and_sync(
    state: &mut GenerationState,
    request_id: &str,
    world_id: Option<&str>,
) -> Result<()> {
    state.mark_suggestion_read(request_id);
    persist_generation_read_state(state);
    sync_generation_read_state(state, world_id).await
}
