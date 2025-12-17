//! Dungeon Master View - Directorial control panel and Creator mode

use dioxus::prelude::*;

use crate::application::dto::{ChallengeData, SkillData};
use crate::application::ports::outbound::{ApprovalDecision, Platform};
use crate::application::services::SessionCommandService;
use crate::presentation::components::creator::CreatorMode;
use crate::presentation::services::{use_challenge_service, use_skill_service};
use crate::presentation::components::dm_panel::challenge_library::ChallengeLibrary;
use crate::presentation::components::dm_panel::adhoc_challenge_modal::{
    AdHocChallengeModal, AdHocChallengeData,
};
use crate::presentation::components::dm_panel::decision_queue::DecisionQueuePanel;
use crate::presentation::components::dm_panel::trigger_challenge_modal::TriggerChallengeModal;
use crate::presentation::components::settings::SettingsView;
use crate::presentation::components::story_arc::timeline_view::TimelineView;
use crate::presentation::components::story_arc::narrative_event_library::NarrativeEventLibrary;
use crate::presentation::state::{use_game_state, use_session_state, use_generation_state, PendingApproval};
use crate::routes::Route;

/// The active tab/mode in the DM View
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DMMode {
    #[default]
    Director,
    Creator,
    StoryArc,
    Settings,
}

/// Props for DMView - receives active mode from parent
#[derive(Props, Clone, PartialEq)]
pub struct DMViewProps {
    /// World ID from the route
    pub world_id: String,
    /// Currently active DM mode/tab
    pub active_mode: DMMode,
    /// Optional Creator sub-tab (characters, locations, items, maps)
    #[props(default)]
    pub creator_subtab: Option<String>,
    /// Optional Settings sub-tab (workflows, skills)
    #[props(default)]
    pub settings_subtab: Option<String>,
    /// Optional Story Arc sub-tab (timeline, events, chains)
    #[props(default)]
    pub story_arc_subtab: Option<String>,
}

#[component]
pub fn DMView(props: DMViewProps) -> Element {
    // Local UI state for ad-hoc challenge modal visibility
    let mut show_adhoc_modal = use_signal(|| false);

    rsx! {
        div {
            class: "dm-view",
            style: "height: 100%; display: flex; flex-direction: column; background: #0f0f23;",

            // Content area - no header, tabs are in main AppHeader
            div {
                class: "dm-content",
                style: "flex: 1; overflow: hidden;",

                match props.active_mode {
                    DMMode::Director => rsx! {
                        DirectorModeContent {}
                    },
                    DMMode::Creator => rsx! {
                        CreatorMode {
                            world_id: props.world_id.clone(),
                            selected_tab: props.creator_subtab.clone(),
                        }
                    },
                    DMMode::StoryArc => rsx! {
                        StoryArcContent {
                            world_id: props.world_id.clone(),
                            selected_tab: props.story_arc_subtab.clone(),
                        }
                    },
                    DMMode::Settings => rsx! {
                        SettingsView {
                            world_id: props.world_id.clone(),
                            selected_tab: props.settings_subtab.clone(),
                        }
                    },
                }
            }
            // Global ad-hoc challenge modal overlay
            if *show_adhoc_modal.read() {
                AdHocChallengeEntryPoint {
                    on_close: move || show_adhoc_modal.set(false),
                }
            }
        }
    }
}

/// Thin wrapper that wires the AdHocChallengeModal to the SessionCommandService
/// and current session state.
#[component]
fn AdHocChallengeEntryPoint(on_close: EventHandler<()>) -> Element {
    let mut session_state = crate::presentation::state::use_session_state();
    let game_state = use_context::<crate::presentation::state::GameState>();
    let platform = use_context::<crate::application::ports::outbound::Platform>();

    let player_characters = game_state.scene_characters.read().clone();

    // Build a command service if we have a live client
    let client = session_state.engine_client().read().clone();
    let command_svc = client.map(|c| SessionCommandService::new(c));

    rsx! {
        AdHocChallengeModal {
            player_characters: player_characters,
            on_create: move |data: AdHocChallengeData| {
                if let Some(_svc) = command_svc.as_ref() {
                    // TODO: Implement create_adhoc_challenge in GameConnectionPort
                    tracing::warn!(
                        "Ad-hoc challenge creation not yet implemented: {} for {}",
                        data.challenge_name,
                        data.target_pc_id
                    );
                    let _ = (data.skill_name, data.difficulty, data.outcomes); // suppress unused warnings
                    if false {
                        tracing::error!("Placeholder error");
                    }
                } else {
                    tracing::warn!("No Engine client available for ad-hoc challenge");
                }

                // Add a quick log entry for instant feedback
                session_state.add_log_entry(
                    "System".to_string(),
                    format!(
                        "Ad-hoc challenge '{}' created for PC {}",
                        data.challenge_name, data.target_pc_id
                    ),
                    true,
                    &platform,
                );

                on_close.call(());
            },
            on_close: move |_| on_close.call(()),
        }
    }
}

/// Story Arc sub-tab within Story Arc mode
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum StoryArcSubTab {
    #[default]
    Timeline,
    NarrativeEvents,
    EventChains,
}

impl StoryArcSubTab {
    fn from_str(s: &str) -> Self {
        match s {
            "timeline" => Self::Timeline,
            "events" => Self::NarrativeEvents,
            "chains" => Self::EventChains,
            _ => Self::Timeline,
        }
    }

    fn to_route_str(&self) -> &'static str {
        match self {
            Self::Timeline => "timeline",
            Self::NarrativeEvents => "events",
            Self::EventChains => "chains",
        }
    }
}

/// Story Arc mode content - Timeline, Narrative Events, Event Chains
#[derive(Props, Clone, PartialEq)]
struct StoryArcContentProps {
    world_id: String,
    #[props(default)]
    selected_tab: Option<String>,
}

#[component]
fn StoryArcContent(props: StoryArcContentProps) -> Element {
    // Parse selected tab from URL, default to Timeline
    let active_tab = props.selected_tab
        .as_ref()
        .map(|s| StoryArcSubTab::from_str(s))
        .unwrap_or(StoryArcSubTab::Timeline);

    rsx! {
        div {
            style: "height: 100%; display: flex; flex-direction: column;",

            // Sub-tab navigation using router Links
            div {
                style: "display: flex; gap: 0; background: #0f0f23; border-bottom: 1px solid #374151;",

                StoryArcTabLink {
                    label: "Timeline",
                    icon: "📜",
                    subtab: "timeline",
                    world_id: props.world_id.clone(),
                    is_active: active_tab == StoryArcSubTab::Timeline,
                }
                StoryArcTabLink {
                    label: "Narrative Events",
                    icon: "⭐",
                    subtab: "events",
                    world_id: props.world_id.clone(),
                    is_active: active_tab == StoryArcSubTab::NarrativeEvents,
                }
                StoryArcTabLink {
                    label: "Event Chains",
                    icon: "🔗",
                    subtab: "chains",
                    world_id: props.world_id.clone(),
                    is_active: active_tab == StoryArcSubTab::EventChains,
                }
            }

            // Content area
            div {
                style: "flex: 1; overflow: hidden;",

                match active_tab {
                    StoryArcSubTab::Timeline => rsx! {
                        TimelineView { world_id: props.world_id.clone() }
                    },
                    StoryArcSubTab::NarrativeEvents => rsx! {
                        NarrativeEventLibrary { world_id: props.world_id.clone() }
                    },
                    StoryArcSubTab::EventChains => rsx! {
                        EventChainsView {
                            world_id: props.world_id.clone(),
                        }
                    },
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct StoryArcTabLinkProps {
    label: &'static str,
    icon: &'static str,
    subtab: &'static str,
    world_id: String,
    is_active: bool,
}

#[component]
fn StoryArcTabLink(props: StoryArcTabLinkProps) -> Element {
    rsx! {
        Link {
            to: Route::DMStoryArcSubTabRoute {
                world_id: props.world_id.clone(),
                subtab: props.subtab.to_string(),
            },
            style: format!(
                "padding: 0.75rem 1.25rem; cursor: pointer; display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; transition: all 0.2s; text-decoration: none; {}",
                if props.is_active {
                    "background: #1a1a2e; color: white; border-bottom: 2px solid #8b5cf6;"
                } else {
                    "background: transparent; color: #9ca3af; border-bottom: 2px solid transparent;"
                }
            ),
            span { "{props.icon}" }
            span { "{props.label}" }
        }
    }
}

/// Event Chains view - main component for managing event chains
#[component]
fn EventChainsView(world_id: String) -> Element {
    use crate::presentation::components::story_arc::event_chain_list::{EventChainList, ChainFilter};
    use crate::presentation::components::story_arc::event_chain_visualizer::EventChainVisualizer;
    use crate::presentation::components::story_arc::event_chain_editor::EventChainEditor;
    use crate::application::services::EventChainData;
    use crate::presentation::services::use_event_chain_service;

    let event_chain_service = use_event_chain_service();
    let mut selected_chain: Signal<Option<String>> = use_signal(|| None);
    let mut selected_chain_data: Signal<Option<EventChainData>> = use_signal(|| None);
    let mut show_editor: Signal<bool> = use_signal(|| false);
    let mut editing_chain: Signal<Option<EventChainData>> = use_signal(|| None);
    let mut filter: Signal<ChainFilter> = use_signal(|| ChainFilter::All);

    // Load chain data when selected
    let chain_id = selected_chain.read().clone();
    let service = event_chain_service.clone();
    use_effect(move || {
        if let Some(id) = chain_id.as_ref() {
            let svc = service.clone();
            let id_clone = id.clone();
            spawn(async move {
                if let Ok(chain) = svc.get_chain(&id_clone).await {
                    selected_chain_data.set(Some(chain));
                }
            });
        } else {
            selected_chain_data.set(None);
        }
    });

    let chain_data = selected_chain_data.read().clone();

    rsx! {
        div {
            style: "height: 100%; display: flex; flex-direction: column; gap: 1rem;",

            // Header with create button
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                h2 {
                    style: "color: white; margin: 0; font-size: 1.5rem;",
                    "Event Chains"
                }
                button {
                    onclick: move |_| {
                        editing_chain.set(None);
                        show_editor.set(true);
                    },
                    style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.875rem;",
                    "+ Create Chain"
                }
            }

            // Main content area
            if *show_editor.read() {
                EventChainEditor {
                    chain: editing_chain.read().clone(),
                    world_id: world_id.clone(),
                    on_save: move |chain: EventChainData| {
                        selected_chain.set(Some(chain.id.clone()));
                        selected_chain_data.set(Some(chain.clone()));
                        show_editor.set(false);
                        editing_chain.set(None);
                    },
                    on_cancel: move |_| {
                        show_editor.set(false);
                        editing_chain.set(None);
                    },
                }
            } else if let Some(chain) = chain_data.as_ref() {
                // Show visualizer for selected chain
                div {
                    style: "display: flex; gap: 1rem; height: 100%;",
                    // Sidebar with chain list
                    div {
                        style: "width: 300px; flex-shrink: 0; overflow-y: auto;",
                        EventChainList {
                            world_id: world_id.clone(),
                            filter: *filter.read(),
                            on_select_chain: move |id| selected_chain.set(Some(id)),
                        }
                    }
                    // Visualizer
                    div {
                        style: "flex: 1; min-height: 0;",
                        EventChainVisualizer {
                            chain: chain.clone(),
                            world_id: world_id.clone(),
                            on_select_event: move |event_id| {
                                // TODO: Navigate to event details
                            },
                        }
                    }
                }
            } else {
                EventChainList {
                    world_id: world_id.clone(),
                    filter: *filter.read(),
                    on_select_chain: move |id| selected_chain.set(Some(id)),
                }
            }
        }
    }
}

/// The original Director mode content (directing gameplay)
#[component]
fn DirectorModeContent() -> Element {
    let session_state = use_session_state();
    let game_state = use_game_state();
    let skill_service = use_skill_service();
    let challenge_service = use_challenge_service();
    let generation_state = use_generation_state();
    let mut show_queue_panel = use_signal(|| false);

    // Local state for directorial inputs
    let mut scene_notes = use_signal(|| String::new());
    let mut current_tone = use_signal(|| "Serious".to_string());
    let mut show_challenge_library = use_signal(|| false);
    let mut show_trigger_challenge = use_signal(|| false);
    let mut show_pc_management = use_signal(|| false);
    let mut show_location_navigator = use_signal(|| false);
    let mut show_character_perspective = use_signal(|| false);
    let mut skills: Signal<Vec<SkillData>> = use_signal(Vec::new);
    let mut challenges: Signal<Vec<ChallengeData>> = use_signal(Vec::new);

    // Load skills and challenges when world is available
    let world_id_for_skills = game_state.world.read().as_ref().map(|w| w.world.id.clone());
    let world_id_for_challenges = game_state.world.read().as_ref().map(|w| w.world.id.clone());
    use_effect(move || {
        if let Some(world_id) = world_id_for_skills.clone() {
            let svc = skill_service.clone();
            spawn(async move {
                if let Ok(skill_list) = svc.list_skills(&world_id).await {
                    // Convert service types to DTO types via JSON
                    if let Ok(json) = serde_json::to_value(&skill_list) {
                        if let Ok(dto_skills) = serde_json::from_value::<Vec<SkillData>>(json) {
                            skills.set(dto_skills);
                        }
                    }
                }
            });
        }
    });
    use_effect(move || {
        if let Some(world_id) = world_id_for_challenges.clone() {
            let svc = challenge_service.clone();
            spawn(async move {
                if let Ok(challenge_list) = svc.list_challenges(&world_id).await {
                    // Convert service types to DTO types via JSON
                    if let Ok(json) = serde_json::to_value(&challenge_list) {
                        if let Ok(dto_challenges) = serde_json::from_value::<Vec<ChallengeData>>(json) {
                            challenges.set(dto_challenges);
                        }
                    }
                }
            });
        }
    });

    // Get pending approvals from state
    let pending_approvals = session_state.pending_approvals().read().clone();
    let conversation_log = session_state.conversation_log().read().clone();

    // Get scene characters from game state
    let scene_characters = game_state.scene_characters.read().clone();

    rsx! {
        div {
            style: "height: 100%; display: grid; grid-template-columns: 1fr 350px; gap: 1rem; padding: 1rem;",

            // Left panel - Scene preview and conversation
            div {
                class: "main-panel",
                style: "display: flex; flex-direction: column; gap: 1rem;",

                // Scene preview (smaller version of what players see)
                div {
                    class: "scene-preview",
                    style: "height: 200px; background: linear-gradient(to bottom, #1a1a2e, #2d1b3d); border-radius: 0.5rem; position: relative; overflow: hidden;",

                    // Show actual characters in scene
                    div {
                        style: "position: absolute; bottom: 20%; left: 50%; transform: translateX(-50%); display: flex; gap: 2rem;",
                        for character in scene_characters.iter() {
                            div {
                                key: "{character.id}",
                                style: "display: flex; flex-direction: column; align-items: center;",
                                div {
                                    style: "width: 80px; height: 120px; background: rgba(59,130,246,0.2); border-radius: 0.25rem; display: flex; align-items: center; justify-content: center;",
                                    if character.sprite_asset.is_some() {
                                        // Would show actual sprite here
                                        span { style: "color: #60a5fa; font-size: 2rem;", "🧑" }
                                    } else {
                                        span { style: "color: #60a5fa; font-size: 2rem;", "🧑" }
                                    }
                                }
                                span { style: "color: #9ca3af; font-size: 0.75rem; margin-top: 0.25rem;", "{character.name}" }
                            }
                        }
                        if scene_characters.is_empty() {
                            div { style: "color: #6b7280; font-style: italic;", "No characters in scene" }
                        }
                    }
                }

                // Conversation log
                div {
                    class: "conversation-log",
                    style: "flex: 1; background: #1a1a2e; border-radius: 0.5rem; padding: 1rem; overflow-y: auto;",

                    h3 { style: "color: #9ca3af; margin-bottom: 1rem; font-size: 0.875rem; text-transform: uppercase;", "Conversation Log" }

                    div {
                        style: "display: flex; flex-direction: column; gap: 0.75rem;",

                        if conversation_log.is_empty() {
                            div { style: "color: #6b7280; font-style: italic; text-align: center; padding: 2rem;",
                                "Waiting for session activity..."
                            }
                        }

                        for (idx, entry) in conversation_log.iter().enumerate() {
                            DynamicLogEntry {
                                key: "{idx}",
                                speaker: entry.speaker.clone(),
                                text: entry.text.clone(),
                                is_system: entry.is_system,
                            }
                        }
                    }
                }

                // Approval popup(s)
                for approval in pending_approvals.iter() {
                    ApprovalPopup {
                        key: "{approval.request_id}",
                        approval: approval.clone(),
                    }
                }

                if pending_approvals.is_empty() && !conversation_log.is_empty() {
                    div {
                        style: "background: #1f2937; border: 1px solid #374151; border-radius: 0.5rem; padding: 1rem; text-align: center; color: #9ca3af;",
                        "No pending approvals"
                    }
                }
            }

            // Right panel - Directorial controls
            div {
                class: "control-panel",
                style: "display: flex; flex-direction: column; gap: 1rem; overflow-y: auto;",

                // Connection status
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Session Info" }

                    div { style: "color: white; font-size: 0.875rem;",
                        if let Some(session_id) = session_state.session_id().read().as_ref() {
                            p { style: "margin: 0.25rem 0;", "Session: {session_id}" }
                        } else {
                            p { style: "margin: 0.25rem 0; color: #f59e0b;", "Not connected to session" }
                        }
                    }
                }

                // Decision queue (pending approvals + recent decisions)
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    DecisionQueuePanel {}
                }

                // Scene notes
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Scene Notes" }
                    textarea {
                        value: "{scene_notes}",
                        oninput: move |e| scene_notes.set(e.value()),
                        placeholder: "Add notes for the current scene...",
                        style: "width: 100%; height: 100px; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; resize: vertical; box-sizing: border-box;",
                    }
                }

                // Tone selection
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Tone" }
                    select {
                        value: "{current_tone}",
                        onchange: move |e| current_tone.set(e.value()),
                        style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white;",
                        option { value: "Serious", "Serious" }
                        option { value: "Lighthearted", "Lighthearted" }
                        option { value: "Tense", "Tense" }
                        option { value: "Mysterious", "Mysterious" }
                        option { value: "Comedic", "Comedic" }
                    }
                }

                // Scene NPCs (from real data)
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Scene Characters" }

                    div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                        if scene_characters.is_empty() {
                            div { style: "color: #6b7280; font-style: italic;", "No characters loaded" }
                        }
                        for character in scene_characters.iter() {
                            div {
                                key: "{character.id}",
                                style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: #0f0f23; border-radius: 0.25rem;",
                                span { style: "color: #60a5fa;", "🧑" }
                                span { style: "color: white;", "{character.name}" }
                                if character.is_speaking {
                                    span { style: "color: #4ade80; font-size: 0.75rem; margin-left: auto;", "(speaking)" }
                                }
                            }
                        }
                    }
                }

                // Quick actions
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Quick Actions" }

                    div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                        button {
                            onclick: move |_| show_challenge_library.set(true),
                            style: "padding: 0.5rem; background: #f59e0b; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                            "Manage Challenges"
                        }
                        button {
                            onclick: move |_| show_trigger_challenge.set(true),
                            style: "padding: 0.5rem; background: #ec4899; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                            "⚔️ Trigger Challenge"
                        }
                        button { style: "padding: 0.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer;", "View Social Graph" }
                        button { style: "padding: 0.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer;", "View Timeline" }
                        button { style: "padding: 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.5rem; cursor: pointer;", "Start Combat" }
                    }
                }
            }

            // Challenge Library Modal
            if *show_challenge_library.read() {
                {
                    let world_id = game_state.world.read().as_ref().map(|w| w.world.id.clone());
                    if let Some(world_id) = world_id {
                        rsx! {
                            ChallengeLibrary {
                                world_id: world_id,
                                skills: skills.read().clone(),
                                on_close: move |_| show_challenge_library.set(false),
                                on_trigger_challenge: None,
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                                onclick: move |_| show_challenge_library.set(false),
                                div {
                                    style: "background: #1a1a2e; padding: 2rem; border-radius: 0.5rem; text-align: center;",
                                    onclick: move |e| e.stop_propagation(),
                                    p { style: "color: #ef4444;", "No world loaded. Start a session first." }
                                    button {
                                        onclick: move |_| show_challenge_library.set(false),
                                        style: "margin-top: 1rem; padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                                        "Close"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // PC Management Modal
            if *show_pc_management.read() {
                if let Some(session_id) = session_state.session_id().read().as_ref() {
                    div {
                        style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                        onclick: move |_| show_pc_management.set(false),
                        div {
                            style: "background: #1a1a2e; border-radius: 0.5rem; width: 90%; max-width: 800px; max-height: 90vh; overflow-y: auto; padding: 1.5rem;",
                            onclick: move |e| e.stop_propagation(),
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;",
                                h2 {
                                    style: "margin: 0; color: white; font-size: 1.25rem;",
                                    "Player Character Management"
                                }
                                button {
                                    onclick: move |_| show_pc_management.set(false),
                                    style: "padding: 0.25rem 0.5rem; background: transparent; color: #9ca3af; border: none; cursor: pointer; font-size: 1.25rem;",
                                    "×"
                                }
                            }
                            crate::presentation::components::dm_panel::pc_management::PCManagementPanel {
                                session_id: session_id.clone(),
                                on_view_as_character: move |character_id| {
                                    // TODO: Implement view as character
                                    tracing::info!("View as character: {}", character_id);
                                    show_pc_management.set(false);
                                },
                            }
                        }
                    }
                }
            }

            // Director Queue Panel
            if *show_queue_panel.read() {
                crate::presentation::components::dm_panel::director_queue_panel::DirectorQueuePanel {
                    on_close: move |_| show_queue_panel.set(false),
                }
            }

            // Location Navigator Modal
            if *show_location_navigator.read() {
                if let Some(world_id) = game_state.world.read().as_ref().map(|w| w.world.id.clone()) {
                    div {
                        style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                        onclick: move |_| show_location_navigator.set(false),
                        div {
                            style: "background: #1a1a2e; border-radius: 0.5rem; width: 90%; max-width: 800px; max-height: 90vh; overflow-y: auto; padding: 1.5rem;",
                            onclick: move |e| e.stop_propagation(),
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;",
                                h2 {
                                    style: "margin: 0; color: white; font-size: 1.25rem;",
                                    "Location Navigator"
                                }
                                button {
                                    onclick: move |_| show_location_navigator.set(false),
                                    style: "padding: 0.25rem 0.5rem; background: transparent; color: #9ca3af; border: none; cursor: pointer; font-size: 1.25rem;",
                                    "×"
                                }
                            }
                            crate::presentation::components::dm_panel::location_navigator::LocationNavigator {
                                world_id: world_id.clone(),
                                on_preview: move |location_id| {
                                    // TODO: Implement location preview
                                    tracing::info!("Preview location: {}", location_id);
                                    show_location_navigator.set(false);
                                },
                            }
                        }
                    }
                }
            }

            // Character Perspective Viewer Modal
            if *show_character_perspective.read() {
                if let (Some(session_id), Some(world_id)) = (
                    session_state.session_id().read().as_ref().map(|s| s.clone()),
                    game_state.world.read().as_ref().map(|w| w.world.id.clone())
                ) {
                    div {
                        style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                        onclick: move |_| show_character_perspective.set(false),
                        div {
                            style: "background: #1a1a2e; border-radius: 0.5rem; width: 90%; max-width: 800px; max-height: 90vh; overflow-y: auto; padding: 1.5rem;",
                            onclick: move |e| e.stop_propagation(),
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;",
                                h2 {
                                    style: "margin: 0; color: white; font-size: 1.25rem;",
                                    "Character Perspective Viewer"
                                }
                                button {
                                    onclick: move |_| show_character_perspective.set(false),
                                    style: "padding: 0.25rem 0.5rem; background: transparent; color: #9ca3af; border: none; cursor: pointer; font-size: 1.25rem;",
                                    "×"
                                }
                            }
                            crate::presentation::components::dm_panel::character_perspective::CharacterPerspectiveViewer {
                                session_id: session_id.clone(),
                                world_id: world_id.clone(),
                                on_view_as: move |character_id| {
                                    // TODO: Implement view as character
                                    tracing::info!("View as character: {}", character_id);
                                    show_character_perspective.set(false);
                                },
                            }
                        }
                    }
                }
            }

            // Trigger Challenge Modal
            if *show_trigger_challenge.read() {
                {
                    let active_challenges: Vec<ChallengeData> = challenges.read().iter()
                        .filter(|c| c.active)
                        .cloned()
                        .collect();
                    let chars = scene_characters.clone();

                    if active_challenges.is_empty() {
                        rsx! {
                            div {
                                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                                onclick: move |_| show_trigger_challenge.set(false),
                                div {
                                    style: "background: #1a1a2e; padding: 2rem; border-radius: 0.5rem; text-align: center; max-width: 400px;",
                                    onclick: move |e| e.stop_propagation(),
                                    h3 { style: "color: #f59e0b; margin-bottom: 1rem;", "⚔️ No Active Challenges" }
                                    p { style: "color: #9ca3af; margin-bottom: 1rem;", "Create and activate challenges in the Challenge Library first." }
                                    button {
                                        onclick: move |_| {
                                            show_trigger_challenge.set(false);
                                            show_challenge_library.set(true);
                                        },
                                        style: "padding: 0.5rem 1rem; background: #f59e0b; color: white; border: none; border-radius: 0.25rem; cursor: pointer; margin-right: 0.5rem;",
                                        "Open Challenge Library"
                                    }
                                    button {
                                        onclick: move |_| show_trigger_challenge.set(false),
                                        style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                                        "Close"
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            TriggerChallengeModal {
                                challenges: active_challenges,
                                scene_characters: chars,
                                on_trigger: move |(challenge_id, character_id): (String, String)| {
                                    tracing::info!("Triggering challenge {} for character {}", challenge_id, character_id);
                                    if let Some(client) = session_state.engine_client().read().as_ref() {
                                        let svc = SessionCommandService::new(std::sync::Arc::clone(client));
                                        if let Err(e) = svc.trigger_challenge(&challenge_id, &character_id) {
                                            tracing::error!("Failed to trigger challenge: {}", e);
                                        }
                                    } else {
                                        tracing::warn!("No engine client available to trigger challenge");
                                    }
                                    show_trigger_challenge.set(false);
                                },
                                on_close: move |_| show_trigger_challenge.set(false),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Approval popup for DM to approve/reject LLM responses
#[derive(Props, Clone, PartialEq)]
struct ApprovalPopupProps {
    approval: PendingApproval,
}

#[component]
fn ApprovalPopup(props: ApprovalPopupProps) -> Element {
    let session_state = use_session_state();
    let platform = use_context::<Platform>();
    let mut modified_dialogue = use_signal(|| props.approval.proposed_dialogue.clone());
    let mut show_reasoning = use_signal(|| false);
    let mut rejection_feedback = use_signal(|| String::new());
    let mut show_reject_input = use_signal(|| false);

    // Track which tools are approved
    let mut approved_tools = use_signal(|| {
        props.approval.proposed_tools.iter().map(|t| (t.id.clone(), true)).collect::<std::collections::HashMap<_, _>>()
    });

    let request_id = props.approval.request_id.clone();
    let npc_name = props.approval.npc_name.clone();

    rsx! {
        div {
            class: "approval-popup",
            style: "background: #1f2937; border: 2px solid #f59e0b; border-radius: 0.75rem; padding: 1.25rem; margin-bottom: 1rem;",

            h4 { style: "color: #f59e0b; margin-bottom: 1rem; display: flex; justify-content: space-between; align-items: center;",
                span { "Approval Required" }
                span { style: "font-size: 0.75rem; color: #9ca3af; font-weight: normal;", "{props.approval.request_id}" }
            }

            div { style: "margin-bottom: 1rem;",
                p { style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "{npc_name} will say:" }
                textarea {
                    value: "{modified_dialogue}",
                    oninput: move |e| modified_dialogue.set(e.value()),
                    style: "width: 100%; min-height: 80px; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; resize: vertical; box-sizing: border-box; font-style: italic;",
                }
            }

            // Show/hide reasoning
            {
                let current_showing = *show_reasoning.read();
                rsx! {
                    button {
                        onclick: move |_| show_reasoning.set(!current_showing),
                        style: "background: none; border: none; color: #60a5fa; cursor: pointer; font-size: 0.875rem; margin-bottom: 0.5rem;",
                        if current_showing { "Hide reasoning ▲" } else { "Show reasoning ▼" }
                    }
                }
            }

            if *show_reasoning.read() {
                div { style: "margin-bottom: 1rem; padding: 0.75rem; background: rgba(0,0,0,0.3); border-radius: 0.5rem;",
                    p { style: "color: #9ca3af; font-size: 0.75rem; margin: 0;", "{props.approval.internal_reasoning}" }
                }
            }

            // Proposed tools
            if !props.approval.proposed_tools.is_empty() {
                div { style: "margin-bottom: 1rem;",
                    p { style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;", "Proposed Actions:" }
                    div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                        for tool in props.approval.proposed_tools.iter() {
                            {
                                let tool_id = tool.id.clone();
                                let tool_id_for_change = tool.id.clone();
                                let is_approved = *approved_tools.read().get(&tool_id).unwrap_or(&true);
                                rsx! {
                                    div {
                                        key: "{tool_id}",
                                        style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: rgba(0,0,0,0.2); border-radius: 0.25rem;",
                                        input {
                                            r#type: "checkbox",
                                            checked: is_approved,
                                            onchange: move |_| {
                                                let mut tools = approved_tools.write();
                                                if let Some(val) = tools.get_mut(&tool_id_for_change) {
                                                    *val = !*val;
                                                }
                                            },
                                        }
                                        div {
                                            span { style: "color: white; font-size: 0.875rem;", "{tool.name}" }
                                            span { style: "color: #9ca3af; font-size: 0.75rem; margin-left: 0.5rem;", "- {tool.description}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Narrative event suggestion section
            if let Some(suggestion) = &props.approval.narrative_event_suggestion {
                div {
                    style: "margin-bottom: 1rem; padding: 1rem; background: rgba(139, 92, 246, 0.1); border: 1px solid #8b5cf6; border-radius: 0.5rem;",

                    h4 {
                        style: "color: #8b5cf6; margin: 0 0 0.75rem 0; font-size: 0.875rem; display: flex; gap: 0.5rem; align-items: center;",
                        "📖 Narrative Event Suggested"
                    }

                    div {
                        style: "margin-bottom: 0.5rem;",
                        span {
                            style: "color: white; font-weight: bold; font-size: 0.875rem;",
                            "{suggestion.event_name}"
                        }
                    }

                    p {
                        style: "color: #9ca3af; font-size: 0.75rem; margin: 0 0 0.5rem 0;",
                        "Confidence: {suggestion.confidence}"
                    }

                    p {
                        style: "color: #9ca3af; font-size: 0.75rem; font-style: italic; margin: 0 0 0.75rem 0; line-height: 1.4;",
                        "\"{suggestion.reasoning}\""
                    }

                    if let Some(outcome) = &suggestion.suggested_outcome {
                        p {
                            style: "color: #a78bfa; font-size: 0.75rem; margin: 0 0 0.75rem 0; line-height: 1.4;",
                            "Suggested Outcome: {outcome}"
                        }
                    }

                    p {
                        style: "color: #9ca3af; font-size: 0.65rem; margin: 0;",
                        "Note: Narrative event triggers are handled separately via the NarrativeEventSuggestionDecision message"
                    }
                }
            }

            // Rejection feedback input
            if *show_reject_input.read() {
                div { style: "margin-bottom: 1rem;",
                    p { style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Feedback for LLM:" }
                    textarea {
                        value: "{rejection_feedback}",
                        oninput: move |e| rejection_feedback.set(e.value()),
                        placeholder: "Tell the LLM what to change...",
                        style: "width: 100%; min-height: 60px; padding: 0.5rem; background: #0f0f23; border: 1px solid #ef4444; border-radius: 0.5rem; color: white; resize: vertical; box-sizing: border-box;",
                    }
                    div { style: "display: flex; gap: 0.5rem; margin-top: 0.5rem;",
                        {
                            let feedback = rejection_feedback.read().clone();
                            let request_id = request_id.clone();
                            let mut session_state = session_state.clone();
                            let platform_reject = platform.clone();
                            rsx! {
                                button {
                                    onclick: move |_| {
                                        session_state.record_approval_decision(
                                            request_id.clone(),
                                            &ApprovalDecision::Reject {
                                                feedback: feedback.clone(),
                                            },
                                            &platform_reject,
                                        );
                                    },
                                    style: "flex: 1; padding: 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                                    "Send Rejection"
                                }
                            }
                        }
                        button {
                            onclick: move |_| show_reject_input.set(false),
                            style: "padding: 0.5rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                            "Cancel"
                        }
                    }
                }
            }

            // Action buttons
            if !*show_reject_input.read() {
                {
                    let request_id_accept = request_id.clone();
                    let mut session_state_accept = session_state.clone();
                    let request_id_modify = request_id.clone();
                    let session_state_modify = session_state.clone();
                    let platform_accept = platform.clone();
                    let platform_modify = platform.clone();
                    let dialogue = modified_dialogue.read().clone();
                    let original = props.approval.proposed_dialogue.clone();
                    let approved = approved_tools.read().clone();
                    let tools = props.approval.proposed_tools.clone();

                    rsx! {
                        div { style: "display: flex; gap: 0.5rem;",
                            button {
                                onclick: move |_| {
                                    session_state_accept.record_approval_decision(
                                        request_id_accept.clone(),
                                        &ApprovalDecision::Accept,
                                        &platform_accept,
                                    );
                                },
                                style: "flex: 1; padding: 0.75rem; background: #22c55e; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600;",
                                "Accept"
                            }
                            button {
                                onclick: {
                                    let dialogue = dialogue.clone();
                                    let original = original.clone();
                                    let approved = approved.clone();
                                    let tools = tools.clone();
                                    let request_id = request_id_modify.clone();
                                    let mut session_state = session_state_modify.clone();
                                    let platform = platform_modify.clone();
                                    move |_| {
                                        // Only send modification if something changed
                                        if dialogue != original || approved.values().any(|&v| !v) {
                                            let approved_list: Vec<String> = tools.iter()
                                                .filter(|t| *approved.get(&t.id).unwrap_or(&true))
                                                .map(|t| t.id.clone())
                                                .collect();
                                            let rejected_list: Vec<String> = tools.iter()
                                                .filter(|t| !*approved.get(&t.id).unwrap_or(&true))
                                                .map(|t| t.id.clone())
                                                .collect();
                                            session_state.record_approval_decision(
                                                request_id.clone(),
                                                &ApprovalDecision::AcceptWithModification {
                                                    modified_dialogue: dialogue.clone(),
                                                    approved_tools: approved_list,
                                                    rejected_tools: rejected_list,
                                                },
                                                &platform,
                                            );
                                        } else {
                                            session_state.record_approval_decision(
                                                request_id.clone(),
                                                &ApprovalDecision::Accept,
                                                &platform,
                                            );
                                        }
                                    }
                                },
                                style: "flex: 1; padding: 0.75rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600;",
                                "Accept Modified"
                            }
                            button {
                                onclick: move |_| show_reject_input.set(true),
                                style: "flex: 1; padding: 0.75rem; background: #ef4444; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600;",
                                "Reject"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Dynamic log entry that accepts String values
#[derive(Props, Clone, PartialEq)]
struct DynamicLogEntryProps {
    speaker: String,
    text: String,
    is_system: bool,
}

#[component]
fn DynamicLogEntry(props: DynamicLogEntryProps) -> Element {
    rsx! {
        div {
            style: format!(
                "padding: 0.5rem; border-radius: 0.25rem; {}",
                if props.is_system { "background: rgba(59, 130, 246, 0.1); color: #60a5fa; font-size: 0.875rem;" }
                else { "color: white;" }
            ),
            if !props.is_system {
                span { style: "color: #3b82f6; font-weight: bold;", "{props.speaker}: " }
            }
            span { "{props.text}" }
        }
    }
}

#[component]
fn LogEntry(speaker: &'static str, text: &'static str, is_system: bool) -> Element {
    rsx! {
        div {
            style: format!(
                "padding: 0.5rem; border-radius: 0.25rem; {}",
                if is_system { "background: rgba(59, 130, 246, 0.1); color: #60a5fa; font-size: 0.875rem;" }
                else { "color: white;" }
            ),
            if !is_system {
                span { style: "color: #3b82f6; font-weight: bold;", "{speaker}: " }
            }
            span { "{text}" }
        }
    }
}

#[component]
fn ProposedAction(name: &'static str, description: &'static str, approved: bool) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: rgba(0,0,0,0.2); border-radius: 0.25rem;",
            input {
                r#type: "checkbox",
                checked: approved,
            }
            div {
                span { style: "color: white; font-size: 0.875rem;", "{name}" }
                span { style: "color: #9ca3af; font-size: 0.75rem; margin-left: 0.5rem;", "- {description}" }
            }
        }
    }
}

#[component]
fn NPCMotivationCard(name: &'static str, mood: &'static str, goal: &'static str) -> Element {
    rsx! {
        div {
            style: "padding: 0.75rem; background: #0f0f23; border-radius: 0.5rem; margin-bottom: 0.5rem;",
            h4 { style: "color: #3b82f6; font-size: 0.875rem; margin-bottom: 0.25rem;", "{name}" }
            p { style: "color: #9ca3af; font-size: 0.75rem;", "Mood: {mood}" }
            p { style: "color: #9ca3af; font-size: 0.75rem;", "Goal: {goal}" }
        }
    }
}

#[component]
fn NPCToggle(name: &'static str, active: bool) -> Element {
    rsx! {
        label {
            style: "display: flex; align-items: center; gap: 0.5rem; color: white; cursor: pointer;",
            input {
                r#type: "checkbox",
                checked: active,
            }
            span { "{name}" }
        }
    }
}
