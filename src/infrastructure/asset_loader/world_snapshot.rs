//! WorldSnapshot loader
//!
//! Loads world data exported from the Engine.

use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete snapshot of a world from the Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Metadata about this snapshot
    pub metadata: SnapshotMetadata,
    /// The world itself
    pub world: WorldData,
    /// All acts in the world
    pub acts: Vec<ActData>,
    /// All scenes in the world
    pub scenes: Vec<SceneData>,
    /// All characters in the world
    pub characters: Vec<CharacterData>,
    /// All locations in the world
    pub locations: Vec<LocationData>,
    /// All relationships between characters
    pub relationships: Vec<RelationshipData>,
    /// Location connections (graph edges)
    pub connections: Vec<ConnectionData>,
}

impl WorldSnapshot {
    /// Load a world snapshot from a JSON file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Load a world snapshot from a JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Get a location by ID
    pub fn get_location(&self, id: &str) -> Option<&LocationData> {
        self.locations.iter().find(|l| l.id == id)
    }

    /// Get a character by ID
    pub fn get_character(&self, id: &str) -> Option<&CharacterData> {
        self.characters.iter().find(|c| c.id == id)
    }

    /// Get a scene by ID
    pub fn get_scene(&self, id: &str) -> Option<&SceneData> {
        self.scenes.iter().find(|s| s.id == id)
    }

    /// Get all child locations of a parent location
    pub fn get_child_locations(&self, parent_id: &str) -> Vec<&LocationData> {
        self.locations
            .iter()
            .filter(|l| l.parent_id.as_ref().map(|p| p == parent_id).unwrap_or(false))
            .collect()
    }

    /// Get connections from a location
    pub fn get_connections_from(&self, location_id: &str) -> Vec<&ConnectionData> {
        self.connections
            .iter()
            .filter(|c| c.from_location_id == location_id)
            .collect()
    }

    /// Get scenes at a location
    pub fn get_scenes_at_location(&self, location_id: &str) -> Vec<&SceneData> {
        self.scenes
            .iter()
            .filter(|s| s.location_id == location_id)
            .collect()
    }

    /// Build a lookup map of locations by ID for efficient access
    pub fn location_map(&self) -> HashMap<&str, &LocationData> {
        self.locations.iter().map(|l| (l.id.as_str(), l)).collect()
    }

    /// Build a lookup map of characters by ID for efficient access
    pub fn character_map(&self) -> HashMap<&str, &CharacterData> {
        self.characters.iter().map(|c| (c.id.as_str(), c)).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub version: String,
    pub exported_at: String,
    pub engine_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_system: RuleSystemConfig,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSystemConfig {
    pub name: String,
    pub stat_definitions: Vec<StatDefinition>,
    #[serde(default)]
    pub dice_system: Option<DiceSystem>,
    #[serde(default)]
    pub skill_check_formula: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatDefinition {
    pub name: String,
    pub abbreviation: String,
    pub min_value: i32,
    pub max_value: i32,
    pub default_value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceSystem {
    pub dice_notation: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActData {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub stage: String,
    pub description: String,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    pub id: String,
    pub act_id: String,
    pub name: String,
    pub location_id: String,
    pub time_context: String,
    pub backdrop_override: Option<String>,
    pub featured_characters: Vec<String>,
    pub directorial_notes: String,
    pub entry_conditions: Vec<String>,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub description: String,
    pub base_archetype: String,
    pub current_archetype: String,
    pub sprite_asset: Option<String>,
    pub portrait_asset: Option<String>,
    pub is_alive: bool,
    pub is_active: bool,
    pub stats: serde_json::Value,
    pub wants: Vec<WantData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantData {
    pub description: String,
    pub target: Option<String>,
    pub intensity: f32,
    pub known_to_player: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationData {
    pub id: String,
    pub world_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub description: String,
    pub location_type: String,
    pub backdrop_asset: Option<String>,
    pub grid_map_id: Option<String>,
    pub backdrop_regions: Vec<BackdropRegionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackdropRegionData {
    pub id: String,
    pub name: String,
    pub bounds: RegionBoundsData,
    pub backdrop_asset: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RegionBoundsData {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl BackdropRegionData {
    /// Check if a grid position is within this region
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.bounds.x
            && x < self.bounds.x + self.bounds.width
            && y >= self.bounds.y
            && y < self.bounds.y + self.bounds.height
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipData {
    pub id: String,
    pub from_character_id: String,
    pub to_character_id: String,
    pub relationship_type: String,
    pub sentiment: f32,
    pub known_to_player: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionData {
    pub from_location_id: String,
    pub to_location_id: String,
    pub connection_type: String,
    pub description: String,
    pub bidirectional: bool,
    pub travel_time: Option<u32>,
}

/// Loader for world snapshots
pub struct WorldSnapshotLoader;

impl WorldSnapshotLoader {
    /// Load a world snapshot from a file path
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<WorldSnapshot> {
        WorldSnapshot::from_file(path)
    }

    /// Load a world snapshot from JSON string
    pub fn load_from_json(json: &str) -> Result<WorldSnapshot> {
        WorldSnapshot::from_json(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_snapshot() {
        let json = r#"{
            "metadata": {
                "version": "1.0",
                "exported_at": "2024-01-01T00:00:00Z",
                "engine_version": "0.1.0"
            },
            "world": {
                "id": "test-world",
                "name": "Test World",
                "description": "A test world",
                "rule_system": {
                    "name": "Test Rules",
                    "stat_definitions": []
                },
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            "acts": [],
            "scenes": [],
            "characters": [],
            "locations": [
                {
                    "id": "town-1",
                    "world_id": "test-world",
                    "parent_id": null,
                    "name": "Town",
                    "description": "A small town",
                    "location_type": "Exterior",
                    "backdrop_asset": "town.png",
                    "grid_map_id": null,
                    "backdrop_regions": [
                        {
                            "id": "church-region",
                            "name": "Church Area",
                            "bounds": {"x": 0, "y": 0, "width": 10, "height": 10},
                            "backdrop_asset": "church.png",
                            "description": "The church district"
                        }
                    ]
                },
                {
                    "id": "bar-1",
                    "world_id": "test-world",
                    "parent_id": "town-1",
                    "name": "The Rusty Mug",
                    "description": "A tavern",
                    "location_type": "Interior",
                    "backdrop_asset": "bar.png",
                    "grid_map_id": null,
                    "backdrop_regions": []
                }
            ],
            "relationships": [],
            "connections": [
                {
                    "from_location_id": "town-1",
                    "to_location_id": "bar-1",
                    "connection_type": "Enters",
                    "description": "Enter the tavern",
                    "bidirectional": false,
                    "travel_time": null
                }
            ]
        }"#;

        let snapshot = WorldSnapshot::from_json(json).unwrap();
        assert_eq!(snapshot.world.name, "Test World");
        assert_eq!(snapshot.locations.len(), 2);

        // Test hierarchy
        let bar = snapshot.get_location("bar-1").unwrap();
        assert_eq!(bar.parent_id.as_deref(), Some("town-1"));

        // Test child locations
        let children = snapshot.get_child_locations("town-1");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "The Rusty Mug");

        // Test backdrop regions
        let town = snapshot.get_location("town-1").unwrap();
        assert_eq!(town.backdrop_regions.len(), 1);
        assert!(town.backdrop_regions[0].contains(5, 5));
        assert!(!town.backdrop_regions[0].contains(15, 15));
    }
}
