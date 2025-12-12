//! Timeline View - Display past story events

use dioxus::prelude::*;

use crate::infrastructure::asset_loader::{StoryEventData, StoryEventTypeData};
use crate::presentation::components::story_arc::{
    TimelineEventCard, TimelineFilters, AddDmMarkerModal,
};
use crate::presentation::state::use_game_state;

/// Filter options for the timeline
#[derive(Debug, Clone, Default)]
pub struct TimelineFilterState {
    pub event_type: Option<String>,
    pub character_id: Option<String>,
    pub location_id: Option<String>,
    pub search_text: String,
    pub show_hidden: bool,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Props, Clone, PartialEq)]
pub struct TimelineViewProps {
    pub world_id: String,
    #[props(default)]
    pub session_id: Option<String>,
}

#[component]
pub fn TimelineView(props: TimelineViewProps) -> Element {
    let game_state = use_game_state();

    let mut events: Signal<Vec<StoryEventData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut filters = use_signal(TimelineFilterState::default);
    let mut show_add_marker = use_signal(|| false);
    let mut selected_event: Signal<Option<StoryEventData>> = use_signal(|| None);

    // Load events when component mounts or world changes
    let world_id = props.world_id.clone();
    use_effect(move || {
        let world_id = world_id.clone();
        spawn(async move {
            is_loading.set(true);
            error.set(None);

            match fetch_story_events(&world_id, None).await {
                Ok(loaded_events) => {
                    events.set(loaded_events);
                }
                Err(e) => {
                    error.set(Some(e));
                }
            }
            is_loading.set(false);
        });
    });

    // Filter events based on current filter state
    let filtered_events = {
        let filter_state = filters.read().clone();
        let all_events = events.read().clone();

        all_events.into_iter().filter(|event| {
            // Hide hidden events unless show_hidden is true
            if event.is_hidden && !filter_state.show_hidden {
                return false;
            }

            // Filter by event type
            if let Some(ref type_filter) = filter_state.event_type {
                let event_type_name = get_event_type_name(&event.event_type);
                if &event_type_name != type_filter {
                    return false;
                }
            }

            // Filter by character
            if let Some(ref char_id) = filter_state.character_id {
                if !event.involved_characters.contains(char_id) {
                    return false;
                }
            }

            // Filter by location
            if let Some(ref loc_id) = filter_state.location_id {
                if event.location_id.as_ref() != Some(loc_id) {
                    return false;
                }
            }

            // Filter by search text
            if !filter_state.search_text.is_empty() {
                let search = filter_state.search_text.to_lowercase();
                let matches_summary = event.summary.to_lowercase().contains(&search);
                let matches_tags = event.tags.iter().any(|t| t.to_lowercase().contains(&search));
                if !matches_summary && !matches_tags {
                    return false;
                }
            }

            true
        }).collect::<Vec<_>>()
    };

    rsx! {
        div {
            class: "timeline-view",
            style: "height: 100%; display: flex; flex-direction: column; gap: 1rem; padding: 1rem;",

            // Header with title and add marker button
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",

                h2 { style: "color: white; margin: 0; font-size: 1.25rem;", "Timeline" }

                button {
                    onclick: move |_| show_add_marker.set(true),
                    style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; display: flex; align-items: center; gap: 0.5rem;",
                    span { "+" }
                    span { "Add DM Marker" }
                }
            }

            // Filters
            TimelineFilters {
                filters: filters.clone(),
                on_filter_change: move |new_filters: TimelineFilterState| filters.set(new_filters),
            }

            // Event list
            div {
                style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0.75rem;",

                if *is_loading.read() {
                    div {
                        style: "display: flex; justify-content: center; align-items: center; padding: 3rem; color: #9ca3af;",
                        "Loading timeline..."
                    }
                } else if let Some(err) = error.read().as_ref() {
                    div {
                        style: "background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; padding: 1rem; color: #ef4444;",
                        "Error loading timeline: {err}"
                    }
                } else if filtered_events.is_empty() {
                    div {
                        style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 3rem; color: #6b7280;",

                        div { style: "font-size: 3rem; margin-bottom: 1rem;", "📜" }

                        if events.read().is_empty() {
                            p { "No events recorded yet" }
                            p { style: "font-size: 0.875rem;", "Events will appear here as gameplay progresses" }
                        } else {
                            p { "No events match your filters" }
                            button {
                                onclick: move |_| filters.set(TimelineFilterState::default()),
                                style: "margin-top: 0.5rem; padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                                "Clear Filters"
                            }
                        }
                    }
                } else {
                    // Event count
                    {
                        let count = filtered_events.len();
                        let suffix = if count == 1 { "" } else { "s" };
                        rsx! {
                            div {
                                style: "color: #6b7280; font-size: 0.875rem; margin-bottom: 0.5rem;",
                                "{count} event{suffix}"
                            }
                        }
                    }

                    for event in filtered_events.iter() {
                        TimelineEventCard {
                            key: "{event.id}",
                            event: event.clone(),
                            on_click: {
                                let event = event.clone();
                                move |_| selected_event.set(Some(event.clone()))
                            },
                            on_toggle_visibility: {
                                let event_id = event.id.clone();
                                let world_id = props.world_id.clone();
                                move |_| {
                                    let event_id = event_id.clone();
                                    let world_id = world_id.clone();
                                    spawn(async move {
                                        if let Err(e) = toggle_event_visibility(&world_id, &event_id).await {
                                            tracing::error!("Failed to toggle visibility: {}", e);
                                        }
                                        // Reload events
                                        if let Ok(reloaded) = fetch_story_events(&world_id, None).await {
                                            events.set(reloaded);
                                        }
                                    });
                                }
                            },
                        }
                    }
                }
            }

            // Add DM Marker modal
            if *show_add_marker.read() {
                AddDmMarkerModal {
                    world_id: props.world_id.clone(),
                    session_id: props.session_id.clone(),
                    on_close: move |_| show_add_marker.set(false),
                    on_created: {
                        let world_id = props.world_id.clone();
                        move |_| {
                            show_add_marker.set(false);
                            // Reload events
                            let world_id = world_id.clone();
                            spawn(async move {
                                if let Ok(reloaded) = fetch_story_events(&world_id, None).await {
                                    events.set(reloaded);
                                }
                            });
                        }
                    },
                }
            }

            // Event detail modal
            if let Some(event) = selected_event.read().as_ref() {
                EventDetailModal {
                    event: event.clone(),
                    on_close: move |_| selected_event.set(None),
                }
            }
        }
    }
}

/// Get a human-readable name for an event type
fn get_event_type_name(event_type: &StoryEventTypeData) -> String {
    match event_type {
        StoryEventTypeData::LocationChange { .. } => "Location Change".to_string(),
        StoryEventTypeData::DialogueExchange { .. } => "Dialogue".to_string(),
        StoryEventTypeData::CombatEvent { .. } => "Combat".to_string(),
        StoryEventTypeData::ChallengeAttempted { .. } => "Challenge".to_string(),
        StoryEventTypeData::ItemAcquired { .. } => "Item Acquired".to_string(),
        StoryEventTypeData::RelationshipChanged { .. } => "Relationship".to_string(),
        StoryEventTypeData::SceneTransition { .. } => "Scene Transition".to_string(),
        StoryEventTypeData::InformationRevealed { .. } => "Information".to_string(),
        StoryEventTypeData::DmMarker { .. } => "DM Marker".to_string(),
        StoryEventTypeData::NarrativeEventTriggered { .. } => "Narrative Event".to_string(),
        StoryEventTypeData::SessionStarted { .. } => "Session Start".to_string(),
        StoryEventTypeData::SessionEnded { .. } => "Session End".to_string(),
        StoryEventTypeData::Custom { .. } => "Custom".to_string(),
    }
}

/// Event detail modal
#[derive(Props, Clone)]
struct EventDetailModalProps {
    event: StoryEventData,
    on_close: EventHandler<()>,
}

impl PartialEq for EventDetailModalProps {
    fn eq(&self, other: &Self) -> bool {
        self.event.id == other.event.id
    }
}

#[component]
fn EventDetailModal(props: EventDetailModalProps) -> Element {
    let event = &props.event;
    let type_name = get_event_type_name(&event.event_type);
    let icon = get_event_type_icon(&event.event_type);

    rsx! {
        div {
            class: "modal-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_close.call(()),

            div {
                class: "modal-content",
                style: "background: #1a1a2e; border-radius: 0.75rem; padding: 1.5rem; max-width: 600px; width: 90%; max-height: 80vh; overflow-y: auto;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 1rem;",

                    div {
                        style: "display: flex; align-items: center; gap: 0.75rem;",
                        span { style: "font-size: 1.5rem;", "{icon}" }
                        div {
                            h3 { style: "color: white; margin: 0; font-size: 1.125rem;", "{type_name}" }
                            p { style: "color: #6b7280; margin: 0; font-size: 0.75rem;", "{event.timestamp}" }
                        }
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #9ca3af; font-size: 1.5rem; cursor: pointer;",
                        "×"
                    }
                }

                // Summary
                div {
                    style: "background: #0f0f23; border-radius: 0.5rem; padding: 1rem; margin-bottom: 1rem;",
                    p { style: "color: white; margin: 0;", "{event.summary}" }
                }

                // Event-specific details
                div {
                    style: "display: flex; flex-direction: column; gap: 0.75rem;",

                    // Show event type specific details
                    match &event.event_type {
                        StoryEventTypeData::DialogueExchange { npc_name, player_dialogue, npc_response, topics_discussed, .. } => rsx! {
                            DetailRow { label: "NPC", value: npc_name.clone() }
                            DetailRow { label: "Player said", value: player_dialogue.clone() }
                            DetailRow { label: "NPC responded", value: npc_response.clone() }
                            if !topics_discussed.is_empty() {
                                DetailRow { label: "Topics", value: topics_discussed.join(", ") }
                            }
                        },
                        StoryEventTypeData::ChallengeAttempted { challenge_name, skill_used, roll_result, outcome, .. } => rsx! {
                            DetailRow { label: "Challenge", value: challenge_name.clone() }
                            if let Some(skill) = skill_used {
                                DetailRow { label: "Skill", value: skill.clone() }
                            }
                            if let Some(roll) = roll_result {
                                DetailRow { label: "Roll", value: roll.to_string() }
                            }
                            DetailRow { label: "Outcome", value: outcome.clone() }
                        },
                        StoryEventTypeData::DmMarker { title, note, importance, marker_type } => rsx! {
                            DetailRow { label: "Title", value: title.clone() }
                            DetailRow { label: "Note", value: note.clone() }
                            DetailRow { label: "Importance", value: importance.clone() }
                            DetailRow { label: "Type", value: marker_type.clone() }
                        },
                        StoryEventTypeData::SceneTransition { from_scene_name, to_scene_name, trigger_reason, .. } => rsx! {
                            if let Some(from) = from_scene_name {
                                DetailRow { label: "From", value: from.clone() }
                            }
                            DetailRow { label: "To", value: to_scene_name.clone() }
                            DetailRow { label: "Reason", value: trigger_reason.clone() }
                        },
                        StoryEventTypeData::RelationshipChanged { reason, previous_sentiment, new_sentiment, .. } => rsx! {
                            DetailRow { label: "Reason", value: reason.clone() }
                            if let Some(prev) = previous_sentiment {
                                DetailRow { label: "Previous", value: format!("{:.1}", prev) }
                            }
                            DetailRow { label: "New", value: format!("{:.1}", new_sentiment) }
                        },
                        _ => rsx! {}
                    }

                    // Tags
                    if !event.tags.is_empty() {
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 0.25rem; margin-top: 0.5rem;",
                            for tag in event.tags.iter() {
                                span {
                                    style: "background: #374151; color: #9ca3af; padding: 0.25rem 0.5rem; border-radius: 0.25rem; font-size: 0.75rem;",
                                    "#{tag}"
                                }
                            }
                        }
                    }

                    // Visibility status
                    if event.is_hidden {
                        div {
                            style: "color: #f59e0b; font-size: 0.875rem; margin-top: 0.5rem;",
                            "👁 Hidden from timeline"
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DetailRowProps {
    label: String,
    value: String,
}

#[component]
fn DetailRow(props: DetailRowProps) -> Element {
    rsx! {
        div {
            style: "display: flex; gap: 0.5rem;",
            span { style: "color: #6b7280; min-width: 80px;", "{props.label}:" }
            span { style: "color: white;", "{props.value}" }
        }
    }
}

/// Get an icon for an event type
pub fn get_event_type_icon(event_type: &StoryEventTypeData) -> &'static str {
    match event_type {
        StoryEventTypeData::LocationChange { .. } => "🚶",
        StoryEventTypeData::DialogueExchange { .. } => "💬",
        StoryEventTypeData::CombatEvent { .. } => "⚔️",
        StoryEventTypeData::ChallengeAttempted { .. } => "🎲",
        StoryEventTypeData::ItemAcquired { .. } => "📦",
        StoryEventTypeData::RelationshipChanged { .. } => "❤️",
        StoryEventTypeData::SceneTransition { .. } => "🎬",
        StoryEventTypeData::InformationRevealed { .. } => "💡",
        StoryEventTypeData::DmMarker { .. } => "📝",
        StoryEventTypeData::NarrativeEventTriggered { .. } => "⭐",
        StoryEventTypeData::SessionStarted { .. } => "▶️",
        StoryEventTypeData::SessionEnded { .. } => "⏹️",
        StoryEventTypeData::Custom { .. } => "📌",
    }
}

/// Paginated response wrapper from Engine API
#[derive(Debug, Clone, serde::Deserialize)]
struct PaginatedStoryEventsResponse {
    events: Vec<StoryEventData>,
    #[allow(dead_code)]
    total: u64,
    #[allow(dead_code)]
    limit: u32,
    #[allow(dead_code)]
    offset: u32,
}

/// Fetch story events from the Engine API
async fn fetch_story_events(world_id: &str, session_id: Option<&str>) -> Result<Vec<StoryEventData>, String> {
    let url = if let Some(sid) = session_id {
        format!("/api/worlds/{}/story-events?session_id={}", world_id, sid)
    } else {
        format!("/api/worlds/{}/story-events", world_id)
    };

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let paginated: PaginatedStoryEventsResponse = response
                .json()
                .await
                .map_err(|e| format!("Parse error: {}", e))?;
            Ok(paginated.events)
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let full_url = format!("http://localhost:3000{}", url);
        let response = client
            .get(&full_url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            let paginated: PaginatedStoryEventsResponse = response
                .json()
                .await
                .map_err(|e| format!("Parse error: {}", e))?;
            Ok(paginated.events)
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }
}

/// Toggle event visibility
async fn toggle_event_visibility(world_id: &str, event_id: &str) -> Result<(), String> {
    let url = format!("/api/story-events/{}/visibility", event_id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let response = Request::put(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            Ok(())
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world_id; // Unused but kept for API consistency
        let client = reqwest::Client::new();
        let full_url = format!("http://localhost:3000{}", url);
        let response = client
            .put(&full_url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }
}
