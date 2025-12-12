//! Game state management using Dioxus signals
//!
//! Central game state for the Player application.

use dioxus::prelude::*;
use std::sync::Arc;

use crate::application::dto::{WorldSnapshot, CharacterData, InteractionData, SceneData};

/// Central game state stored as Dioxus signals
#[derive(Clone)]
pub struct GameState {
    /// Loaded world data (from WorldSnapshot)
    pub world: Signal<Option<Arc<WorldSnapshot>>>,
    /// Current scene data (from server SceneUpdate)
    pub current_scene: Signal<Option<SceneData>>,
    /// Characters in the current scene
    pub scene_characters: Signal<Vec<CharacterData>>,
    /// Available interactions in the scene
    pub interactions: Signal<Vec<InteractionData>>,
}

impl GameState {
    /// Create a new GameState with empty signals
    pub fn new() -> Self {
        Self {
            world: Signal::new(None),
            current_scene: Signal::new(None),
            scene_characters: Signal::new(Vec::new()),
            interactions: Signal::new(Vec::new()),
        }
    }

    /// Load a world snapshot
    pub fn load_world(&mut self, snapshot: WorldSnapshot) {
        self.world.set(Some(Arc::new(snapshot)));
    }

    /// Update from ServerMessage::SceneUpdate
    pub fn apply_scene_update(
        &mut self,
        scene: SceneData,
        characters: Vec<CharacterData>,
        interactions: Vec<InteractionData>,
    ) {
        self.current_scene.set(Some(scene));
        self.scene_characters.set(characters);
        self.interactions.set(interactions);
    }

    /// Get the backdrop URL for the current scene
    pub fn backdrop_url(&self) -> Option<String> {
        // First check scene override, then location backdrop
        let scene_binding = self.current_scene.read();
        if let Some(scene) = scene_binding.as_ref() {
            if let Some(ref backdrop) = scene.backdrop_asset {
                return Some(backdrop.clone());
            }
        }

        // Fall back to location backdrop from world data
        let world_binding = self.world.read();
        if let (Some(scene), Some(world)) = (scene_binding.as_ref(), world_binding.as_ref()) {
            if let Some(location) = world.get_location(&scene.location_id) {
                return location.backdrop_asset.clone();
            }
        }

        None
    }

    /// Clear all scene data (e.g., when disconnecting)
    pub fn clear_scene(&mut self) {
        self.current_scene.set(None);
        self.scene_characters.set(Vec::new());
        self.interactions.set(Vec::new());
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.world.set(None);
        self.clear_scene();
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
