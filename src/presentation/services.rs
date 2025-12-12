//! Service providers for the presentation layer
//!
//! This module provides Dioxus context providers for application services.
//! Components can use `use_context` to access services without depending
//! on infrastructure implementations.

use dioxus::prelude::*;
use std::sync::Arc;

use crate::application::services::{
    WorldService, CharacterService, LocationService, SkillService, ChallengeService,
};
use crate::infrastructure::http_client::ApiAdapter;

/// Type aliases for concrete service types
pub type WorldSvc = WorldService<ApiAdapter>;
pub type CharacterSvc = CharacterService<ApiAdapter>;
pub type LocationSvc = LocationService<ApiAdapter>;
pub type SkillSvc = SkillService<ApiAdapter>;
pub type ChallengeSvc = ChallengeService<ApiAdapter>;

/// All services wrapped for context provision
#[derive(Clone)]
pub struct Services {
    pub world: Arc<WorldSvc>,
    pub character: Arc<CharacterSvc>,
    pub location: Arc<LocationSvc>,
    pub skill: Arc<SkillSvc>,
    pub challenge: Arc<ChallengeSvc>,
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
            skill: Arc::new(SkillService::new(api.clone())),
            challenge: Arc::new(ChallengeService::new(api)),
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
