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

/// Rule system configuration (matches Engine's RuleSystemConfig exactly)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleSystemConfig {
    pub name: String,
    pub description: String,
    pub system_type: RuleSystemType,
    pub variant: RuleSystemVariant,
    #[serde(default)]
    pub stat_definitions: Vec<StatDefinition>,
    pub dice_system: DiceSystem,
    pub success_comparison: SuccessComparison,
    pub skill_check_formula: String,
}

impl Default for RuleSystemConfig {
    fn default() -> Self {
        Self {
            name: "Generic D20".to_string(),
            description: "A generic d20-based rule system".to_string(),
            system_type: RuleSystemType::D20,
            variant: RuleSystemVariant::GenericD20,
            stat_definitions: vec![],
            dice_system: DiceSystem::D20,
            success_comparison: SuccessComparison::GreaterOrEqual,
            skill_check_formula: "1d20 + modifier vs DC".to_string(),
        }
    }
}

/// The type of rule system (how dice mechanics work)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSystemType {
    /// Roll d20 + modifier vs Difficulty Class (D&D, Pathfinder)
    D20,
    /// Roll d100 under skill value (Call of Cthulhu, RuneQuest)
    D100,
    /// Fiction-first with descriptive outcomes (Kids on Bikes, FATE, PbtA)
    Narrative,
    /// Custom dice mechanics
    Custom,
}

impl RuleSystemType {
    pub fn all() -> Vec<Self> {
        vec![Self::D20, Self::D100, Self::Narrative, Self::Custom]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::D20 => "D20 System",
            Self::D100 => "D100 System",
            Self::Narrative => "Narrative System",
            Self::Custom => "Custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::D20 => "Roll d20 + modifier vs DC (D&D, Pathfinder)",
            Self::D100 => "Roll percentile under skill (Call of Cthulhu)",
            Self::Narrative => "Fiction-first, story-driven (Kids on Bikes, FATE)",
            Self::Custom => "Define your own mechanics",
        }
    }
}

/// Specific rule system variants/presets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSystemVariant {
    // D20 variants
    Dnd5e,
    Pathfinder2e,
    GenericD20,
    // D100 variants
    CallOfCthulhu7e,
    RuneQuest,
    GenericD100,
    // Narrative variants
    KidsOnBikes,
    FateCore,
    PoweredByApocalypse,
    // Custom
    Custom(String),
}

impl RuleSystemVariant {
    pub fn variants_for_type(system_type: RuleSystemType) -> Vec<Self> {
        match system_type {
            RuleSystemType::D20 => vec![Self::Dnd5e, Self::Pathfinder2e, Self::GenericD20],
            RuleSystemType::D100 => vec![Self::CallOfCthulhu7e, Self::RuneQuest, Self::GenericD100],
            RuleSystemType::Narrative => vec![Self::KidsOnBikes, Self::FateCore, Self::PoweredByApocalypse],
            RuleSystemType::Custom => vec![],
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dnd5e => "D&D 5th Edition",
            Self::Pathfinder2e => "Pathfinder 2e",
            Self::GenericD20 => "Generic D20",
            Self::CallOfCthulhu7e => "Call of Cthulhu 7e",
            Self::RuneQuest => "RuneQuest",
            Self::GenericD100 => "Generic D100",
            Self::KidsOnBikes => "Kids on Bikes",
            Self::FateCore => "FATE Core",
            Self::PoweredByApocalypse => "Powered by the Apocalypse",
            Self::Custom(_) => "Custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Dnd5e => "Six classic stats (STR, DEX, CON, INT, WIS, CHA)",
            Self::Pathfinder2e => "Four degrees of success with proficiency",
            Self::GenericD20 => "Simple d20 + modifier vs DC",
            Self::CallOfCthulhu7e => "Skill-based percentile with sanity",
            Self::RuneQuest => "Percentile with hit locations",
            Self::GenericD100 => "Roll under skill value",
            Self::KidsOnBikes => "Six stats representing tropes",
            Self::FateCore => "Aspects, skills, and stunts",
            Self::PoweredByApocalypse => "2d6 with 3 outcome tiers",
            Self::Custom(_) => "Custom configuration",
        }
    }
}

/// How success is determined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuccessComparison {
    /// Roll must be >= target (D20 systems)
    GreaterOrEqual,
    /// Roll must be <= target (D100 systems)
    LessOrEqual,
    /// Success tiers based on roll (narrative systems)
    Narrative,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatDefinition {
    pub name: String,
    pub abbreviation: String,
    pub min_value: i32,
    pub max_value: i32,
    pub default_value: i32,
}

/// The dice system used for resolution (mirrors Engine's enum exactly)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiceSystem {
    /// Classic d20 system (D&D, Pathfinder)
    D20,
    /// Percentile system (Call of Cthulhu)
    D100,
    /// Dice pool system (World of Darkness)
    DicePool { die_type: u8, success_threshold: u8 },
    /// FATE/Fudge dice
    Fate,
    /// Custom dice expression
    Custom(String),
}

/// A skill for character challenges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillData {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub description: String,
    pub category: SkillCategory,
    pub base_attribute: Option<String>,
    pub is_custom: bool,
    pub is_hidden: bool,
    pub order: u32,
}

/// Skill categories for UI organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    // D20 style categories
    Physical,
    Mental,
    Social,
    // D100/CoC style categories
    Interpersonal,
    Investigation,
    Academic,
    Practical,
    Combat,
    // Narrative style
    Approach,
    Aspect,
    // General
    Other,
    Custom,
}

impl SkillCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Physical => "Physical",
            Self::Mental => "Mental",
            Self::Social => "Social",
            Self::Interpersonal => "Interpersonal",
            Self::Investigation => "Investigation",
            Self::Academic => "Academic",
            Self::Practical => "Practical",
            Self::Combat => "Combat",
            Self::Approach => "Approach",
            Self::Aspect => "Aspect",
            Self::Other => "Other",
            Self::Custom => "Custom",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Physical,
            Self::Mental,
            Self::Social,
            Self::Interpersonal,
            Self::Investigation,
            Self::Academic,
            Self::Practical,
            Self::Combat,
            Self::Approach,
            Self::Aspect,
            Self::Other,
            Self::Custom,
        ]
    }
}

// ============================================================================
// Challenge Types
// ============================================================================

/// Challenge data from API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChallengeData {
    pub id: String,
    pub world_id: String,
    pub scene_id: Option<String>,
    pub name: String,
    pub description: String,
    pub challenge_type: ChallengeType,
    pub skill_id: String,
    pub difficulty: ChallengeDifficulty,
    pub outcomes: ChallengeOutcomes,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub prerequisite_challenges: Vec<String>,
    pub active: bool,
    pub order: u32,
    pub is_favorite: bool,
    pub tags: Vec<String>,
}

/// Types of challenges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeType {
    #[default]
    SkillCheck,
    AbilityCheck,
    SavingThrow,
    OpposedCheck,
    ComplexChallenge,
}

impl ChallengeType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SkillCheck => "Skill Check",
            Self::AbilityCheck => "Ability Check",
            Self::SavingThrow => "Saving Throw",
            Self::OpposedCheck => "Opposed Check",
            Self::ComplexChallenge => "Complex Challenge",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::SkillCheck,
            Self::AbilityCheck,
            Self::SavingThrow,
            Self::OpposedCheck,
            Self::ComplexChallenge,
        ]
    }
}

/// Challenge difficulty representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChallengeDifficulty {
    Dc { value: u32 },
    Percentage { value: u32 },
    Descriptor { value: String },
    Opposed,
    Custom { value: String },
}

impl Default for ChallengeDifficulty {
    fn default() -> Self {
        Self::Dc { value: 10 }
    }
}

impl ChallengeDifficulty {
    pub fn display(&self) -> String {
        match self {
            Self::Dc { value } => format!("DC {}", value),
            Self::Percentage { value } => format!("{}%", value),
            Self::Descriptor { value } => value.clone(),
            Self::Opposed => "Opposed".to_string(),
            Self::Custom { value } => value.clone(),
        }
    }
}

/// Outcomes for a challenge
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChallengeOutcomes {
    pub success: Outcome,
    pub failure: Outcome,
    #[serde(default)]
    pub partial: Option<Outcome>,
    #[serde(default)]
    pub critical_success: Option<Outcome>,
    #[serde(default)]
    pub critical_failure: Option<Outcome>,
}

/// A single outcome with narrative text and triggered effects
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub triggers: Vec<OutcomeTrigger>,
}

/// Effects triggered by challenge outcomes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutcomeTrigger {
    RevealInformation { info: String, persist: bool },
    EnableChallenge { challenge_id: String },
    DisableChallenge { challenge_id: String },
    ModifyCharacterStat { stat: String, modifier: i32 },
    TriggerScene { scene_id: String },
    GiveItem { item_name: String, item_description: Option<String> },
    Custom { description: String },
}

/// Condition that triggers LLM to suggest a challenge
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub condition_type: TriggerType,
    pub description: String,
    #[serde(default)]
    pub required: bool,
}

/// Types of trigger conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerType {
    ObjectInteraction { keywords: Vec<String> },
    EnterArea { keywords: Vec<String> },
    DialogueTopic { keywords: Vec<String> },
    ChallengeComplete { challenge_id: String, requires_success: Option<bool> },
    TimeBased { turns: u32 },
    NpcPresent { keywords: Vec<String> },
    Custom { description: String },
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

// ============================================================================
// Character Sheet Template Types
// ============================================================================

/// A character sheet template
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetTemplate {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub description: String,
    pub variant: String,
    pub sections: Vec<SheetSection>,
    pub is_default: bool,
}

/// A section in the character sheet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSection {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub fields: Vec<SheetField>,
    pub layout: SectionLayout,
    #[serde(default)]
    pub collapsible: bool,
    #[serde(default)]
    pub collapsed_by_default: bool,
    #[serde(default)]
    pub order: u32,
}

/// Layout for a section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SectionLayout {
    Vertical,
    Grid { columns: u8 },
    Flow,
    TwoColumn,
}

impl Default for SectionLayout {
    fn default() -> Self {
        Self::Vertical
    }
}

/// A field in the character sheet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetField {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub field_type: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub order: u32,
}

/// Field type with configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldType {
    Number {
        min: Option<i32>,
        max: Option<i32>,
        default: Option<i32>,
    },
    Text {
        multiline: bool,
        max_length: Option<usize>,
    },
    Checkbox {
        default: bool,
    },
    Select {
        options: Vec<SelectOption>,
    },
    SkillReference {
        categories: Option<Vec<String>>,
        show_attribute: bool,
    },
    Derived {
        formula: String,
        depends_on: Vec<String>,
    },
    Resource {
        max_field: Option<String>,
        default_max: Option<i32>,
    },
    ItemList {
        item_type: ItemListType,
        max_items: Option<usize>,
    },
    SkillList {
        show_modifier: bool,
        show_proficiency: bool,
    },
}

/// Option for select fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Type of items in an item list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemListType {
    Inventory,
    Features,
    Spells,
    Notes,
}

/// Character sheet data (actual values)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CharacterSheetData {
    pub values: std::collections::HashMap<String, FieldValue>,
}

/// A value stored for a field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum FieldValue {
    Number(i32),
    Text(String),
    Boolean(bool),
    Resource { current: i32, max: i32 },
    List(Vec<String>),
    SkillEntry {
        skill_id: String,
        proficient: bool,
        bonus: i32,
    },
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

// =============================================================================
// Story Event Types (Phase 17)
// =============================================================================

/// A story event - an immutable record of something that happened during gameplay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryEventData {
    pub id: String,
    pub world_id: String,
    pub session_id: String,
    pub scene_id: Option<String>,
    pub location_id: Option<String>,
    pub event_type: StoryEventTypeData,
    pub timestamp: String,
    pub game_time: Option<String>,
    pub summary: String,
    pub involved_characters: Vec<String>,
    pub is_hidden: bool,
    pub tags: Vec<String>,
    pub triggered_by: Option<String>,
}

/// Categories of story events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StoryEventTypeData {
    LocationChange {
        from_location: Option<String>,
        to_location: String,
        character_id: String,
        travel_method: Option<String>,
    },
    DialogueExchange {
        npc_id: String,
        npc_name: String,
        player_dialogue: String,
        npc_response: String,
        topics_discussed: Vec<String>,
        tone: Option<String>,
    },
    CombatEvent {
        combat_type: String,
        participants: Vec<String>,
        enemies: Vec<String>,
        outcome: Option<String>,
        location_id: String,
        rounds: Option<u32>,
    },
    ChallengeAttempted {
        challenge_id: Option<String>,
        challenge_name: String,
        character_id: String,
        skill_used: Option<String>,
        difficulty: Option<String>,
        roll_result: Option<i32>,
        modifier: Option<i32>,
        outcome: String,
    },
    ItemAcquired {
        item_name: String,
        item_description: Option<String>,
        character_id: String,
        source: String,
        quantity: u32,
    },
    RelationshipChanged {
        from_character: String,
        to_character: String,
        previous_sentiment: Option<f32>,
        new_sentiment: f32,
        sentiment_change: f32,
        reason: String,
    },
    SceneTransition {
        from_scene: Option<String>,
        to_scene: String,
        from_scene_name: Option<String>,
        to_scene_name: String,
        trigger_reason: String,
    },
    InformationRevealed {
        info_type: String,
        title: String,
        content: String,
        source: Option<String>,
        importance: String,
        persist_to_journal: bool,
    },
    DmMarker {
        title: String,
        note: String,
        importance: String,
        marker_type: String,
    },
    NarrativeEventTriggered {
        narrative_event_id: String,
        narrative_event_name: String,
        outcome_branch: Option<String>,
        effects_applied: Vec<String>,
    },
    SessionStarted {
        session_number: u32,
        session_name: Option<String>,
        players_present: Vec<String>,
    },
    SessionEnded {
        duration_minutes: u32,
        summary: String,
    },
    Custom {
        event_subtype: String,
        title: String,
        description: String,
    },
}

/// DM marker importance levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarkerImportance {
    Minor,
    Normal,
    Major,
    Critical,
}

/// DM marker types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DmMarkerType {
    Note,
    PlotPoint,
    Foreshadowing,
    SessionBreak,
    ChapterBreak,
    Recap,
    Custom,
}

// =============================================================================
// Narrative Event Types (Phase 17)
// =============================================================================

/// A narrative event - a DM-designed future event with triggers and outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEventData {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub description: String,
    pub scene_direction: String,
    pub suggested_opening: Option<String>,
    pub trigger_count: u32,
    pub is_active: bool,
    pub is_triggered: bool,
    pub triggered_at: Option<String>,
    pub selected_outcome: Option<String>,
    pub is_repeatable: bool,
    pub delay_turns: u32,
    pub expires_after_turns: Option<u32>,
    pub priority: i32,
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub scene_id: Option<String>,
    pub location_id: Option<String>,
    pub act_id: Option<String>,
    pub chain_id: Option<String>,
    pub chain_position: Option<u32>,
    pub outcome_count: usize,
    pub trigger_condition_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

/// Event chain data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChainData {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub events: Vec<ChainedEventData>,
    pub is_favorite: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A chained event within an event chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedEventData {
    pub event_id: String,
    pub position: u32,
    pub is_completed: bool,
    pub completed_at: Option<String>,
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
