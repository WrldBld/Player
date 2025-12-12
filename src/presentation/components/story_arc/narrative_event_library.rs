//! Narrative Event Library - Browse and manage future narrative events

use dioxus::prelude::*;

use crate::infrastructure::asset_loader::NarrativeEventData;
use crate::infrastructure::http_client::HttpClient;
use crate::presentation::components::story_arc::NarrativeEventCard;

#[derive(Props, Clone, PartialEq)]
pub struct NarrativeEventLibraryProps {
    pub world_id: String,
}

#[component]
pub fn NarrativeEventLibrary(props: NarrativeEventLibraryProps) -> Element {
    let mut events: Signal<Vec<NarrativeEventData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut search_text = use_signal(|| String::new());
    let mut filter_status = use_signal(|| "all".to_string());
    let mut show_favorites_only = use_signal(|| false);
    let mut selected_event: Signal<Option<NarrativeEventData>> = use_signal(|| None);

    // Load events
    let world_id = props.world_id.clone();
    use_effect(move || {
        let world_id = world_id.clone();
        spawn(async move {
            is_loading.set(true);
            error.set(None);

            match fetch_narrative_events(&world_id).await {
                Ok(loaded) => events.set(loaded),
                Err(e) => error.set(Some(e)),
            }
            is_loading.set(false);
        });
    });

    // Filter events
    let filtered_events = {
        let search = search_text.read().to_lowercase();
        let status = filter_status.read().clone();
        let favorites_only = *show_favorites_only.read();
        let all_events = events.read().clone();

        all_events.into_iter().filter(|event| {
            // Filter by favorites
            if favorites_only && !event.is_favorite {
                return false;
            }

            // Filter by status
            match status.as_str() {
                "active" => {
                    if !event.is_active {
                        return false;
                    }
                }
                "triggered" => {
                    if !event.is_triggered {
                        return false;
                    }
                }
                "pending" => {
                    if event.is_triggered || !event.is_active {
                        return false;
                    }
                }
                _ => {}
            }

            // Filter by search
            if !search.is_empty() {
                let matches_name = event.name.to_lowercase().contains(&search);
                let matches_desc = event.description.to_lowercase().contains(&search);
                let matches_tags = event.tags.iter().any(|t| t.to_lowercase().contains(&search));
                if !matches_name && !matches_desc && !matches_tags {
                    return false;
                }
            }

            true
        }).collect::<Vec<_>>()
    };

    rsx! {
        div {
            class: "narrative-event-library",
            style: "height: 100%; display: flex; flex-direction: column; gap: 1rem; padding: 1rem;",

            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",

                h2 { style: "color: white; margin: 0; font-size: 1.25rem;", "Narrative Events" }

                button {
                    onclick: move |_| {
                        // TODO: Open create event modal
                        tracing::info!("Create new narrative event");
                    },
                    style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; display: flex; align-items: center; gap: 0.5rem;",
                    span { "+" }
                    span { "New Event" }
                }
            }

            // Filters
            div {
                style: "background: #1a1a2e; border-radius: 0.5rem; padding: 0.75rem; display: flex; gap: 0.75rem; align-items: center; flex-wrap: wrap;",

                // Search
                input {
                    r#type: "text",
                    placeholder: "Search events...",
                    value: "{search_text}",
                    oninput: move |e| search_text.set(e.value()),
                    style: "flex: 1; min-width: 200px; padding: 0.5rem 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem;",
                }

                // Status filter
                select {
                    value: "{filter_status}",
                    onchange: move |e| filter_status.set(e.value()),
                    style: "padding: 0.5rem 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem;",

                    option { value: "all", "All Events" }
                    option { value: "active", "Active Only" }
                    option { value: "pending", "Pending" }
                    option { value: "triggered", "Triggered" }
                }

                // Favorites toggle
                {
                    let is_favorites = *show_favorites_only.read();
                    rsx! {
                        label {
                            style: "display: flex; align-items: center; gap: 0.375rem; color: #9ca3af; font-size: 0.875rem; cursor: pointer;",

                            input {
                                r#type: "checkbox",
                                checked: is_favorites,
                                onchange: move |_| show_favorites_only.set(!is_favorites),
                            }
                            "⭐ Favorites"
                        }
                    }
                }
            }

            // Stats bar
            div {
                style: "display: flex; gap: 1rem; color: #6b7280; font-size: 0.875rem;",

                span { "{events.read().len()} total" }
                span { "{events.read().iter().filter(|e| e.is_active).count()} active" }
                span { "{events.read().iter().filter(|e| e.is_triggered).count()} triggered" }
                span { "{events.read().iter().filter(|e| e.is_favorite).count()} favorites" }
            }

            // Event list
            div {
                style: "flex: 1; overflow-y: auto;",

                if *is_loading.read() {
                    div {
                        style: "display: flex; justify-content: center; align-items: center; padding: 3rem; color: #9ca3af;",
                        "Loading narrative events..."
                    }
                } else if let Some(err) = error.read().as_ref() {
                    div {
                        style: "background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; padding: 1rem; color: #ef4444;",
                        "Error: {err}"
                    }
                } else if filtered_events.is_empty() {
                    div {
                        style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 3rem; color: #6b7280;",

                        div { style: "font-size: 3rem; margin-bottom: 1rem;", "⭐" }

                        if events.read().is_empty() {
                            p { "No narrative events yet" }
                            p { style: "font-size: 0.875rem;", "Create events to set up story hooks and branching narratives" }
                        } else {
                            p { "No events match your filters" }
                        }
                    }
                } else {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1rem;",

                        for event in filtered_events.iter() {
                            NarrativeEventCard {
                                key: "{event.id}",
                                event: event.clone(),
                                on_click: {
                                    let event = event.clone();
                                    move |_| selected_event.set(Some(event.clone()))
                                },
                                on_toggle_favorite: {
                                    let event_id = event.id.clone();
                                    let world_id = props.world_id.clone();
                                    move |_| {
                                        let event_id = event_id.clone();
                                        let world_id = world_id.clone();
                                        spawn(async move {
                                            if let Err(e) = toggle_favorite(&event_id).await {
                                                tracing::error!("Failed to toggle favorite: {}", e);
                                            }
                                            if let Ok(reloaded) = fetch_narrative_events(&world_id).await {
                                                events.set(reloaded);
                                            }
                                        });
                                    }
                                },
                                on_toggle_active: {
                                    let event_id = event.id.clone();
                                    let is_active = event.is_active;
                                    let world_id = props.world_id.clone();
                                    move |_| {
                                        let event_id = event_id.clone();
                                        let world_id = world_id.clone();
                                        spawn(async move {
                                            if let Err(e) = set_active(&event_id, !is_active).await {
                                                tracing::error!("Failed to toggle active: {}", e);
                                            }
                                            if let Ok(reloaded) = fetch_narrative_events(&world_id).await {
                                                events.set(reloaded);
                                            }
                                        });
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn fetch_narrative_events(world_id: &str) -> Result<Vec<NarrativeEventData>, String> {
    let path = format!("/api/worlds/{}/narrative-events", world_id);
    HttpClient::get(&path).await.map_err(|e| e.to_string())
}

async fn toggle_favorite(event_id: &str) -> Result<(), String> {
    let path = format!("/api/narrative-events/{}/favorite", event_id);
    HttpClient::post_empty(&path).await.map_err(|e| e.to_string())
}

async fn set_active(event_id: &str, active: bool) -> Result<(), String> {
    let path = format!("/api/narrative-events/{}/active", event_id);
    HttpClient::put_no_response(&path, &active).await.map_err(|e| e.to_string())
}
