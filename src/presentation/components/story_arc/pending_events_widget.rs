//! Pending Events Widget - Small widget for Director view showing relevant narrative events

use dioxus::prelude::*;

use crate::infrastructure::asset_loader::NarrativeEventData;

#[derive(Props, Clone, PartialEq)]
pub struct PendingEventsWidgetProps {
    pub world_id: String,
    #[props(default = 5)]
    pub max_events: usize,
    pub on_view_story_arc: EventHandler<()>,
}

#[component]
pub fn PendingEventsWidget(props: PendingEventsWidgetProps) -> Element {
    let mut events: Signal<Vec<NarrativeEventData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);

    // Load active/pending events
    let world_id = props.world_id.clone();
    use_effect(move || {
        let world_id = world_id.clone();
        spawn(async move {
            is_loading.set(true);
            if let Ok(loaded) = fetch_pending_events(&world_id).await {
                events.set(loaded);
            }
            is_loading.set(false);
        });
    });

    // Take only the top N events by priority
    let display_events: Vec<NarrativeEventData> = {
        let mut all = events.read().clone();
        all.sort_by(|a, b| b.priority.cmp(&a.priority));
        all.into_iter()
            .filter(|e| e.is_active && !e.is_triggered)
            .take(props.max_events)
            .collect()
    };

    rsx! {
        div {
            class: "pending-events-widget",
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",

                h3 {
                    style: "color: #9ca3af; margin: 0; font-size: 0.875rem; text-transform: uppercase;",
                    "⭐ Pending Events"
                }

                button {
                    onclick: move |_| props.on_view_story_arc.call(()),
                    style: "background: none; border: none; color: #60a5fa; cursor: pointer; font-size: 0.75rem;",
                    "View All →"
                }
            }

            // Content
            if *is_loading.read() {
                div {
                    style: "color: #6b7280; font-size: 0.875rem; text-align: center; padding: 1rem;",
                    "Loading..."
                }
            } else if display_events.is_empty() {
                div {
                    style: "color: #6b7280; font-size: 0.875rem; text-align: center; padding: 1rem;",
                    "No pending events"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem;",

                    for event in display_events.iter() {
                        PendingEventItem {
                            key: "{event.id}",
                            event: event.clone(),
                        }
                    }

                    // Show count if more events exist
                    {
                        let total_active = events.read().iter().filter(|e| e.is_active && !e.is_triggered).count();
                        let max = props.max_events;
                        if total_active > max {
                            let extra = total_active - max;
                            rsx! {
                                div {
                                    style: "color: #6b7280; font-size: 0.75rem; text-align: center; margin-top: 0.5rem;",
                                    "+{extra} more"
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone)]
struct PendingEventItemProps {
    event: NarrativeEventData,
}

impl PartialEq for PendingEventItemProps {
    fn eq(&self, other: &Self) -> bool {
        self.event.id == other.event.id
    }
}

#[component]
fn PendingEventItem(props: PendingEventItemProps) -> Element {
    let event = &props.event;

    let priority_color = match event.priority {
        p if p >= 8 => "#ef4444",
        p if p >= 5 => "#f59e0b",
        p if p >= 3 => "#3b82f6",
        _ => "#6b7280",
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: #0f0f23; border-radius: 0.375rem;",

            // Priority indicator
            div {
                style: format!(
                    "width: 6px; height: 6px; border-radius: 50%; background: {}; flex-shrink: 0;",
                    priority_color
                ),
            }

            // Event info
            div {
                style: "flex: 1; min-width: 0;",

                p {
                    style: "color: white; margin: 0; font-size: 0.8125rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{event.name}"
                }

                div {
                    style: "display: flex; gap: 0.5rem; color: #6b7280; font-size: 0.6875rem;",

                    span { "⚡ {event.trigger_condition_count} triggers" }

                    if event.is_favorite {
                        span { style: "color: #f59e0b;", "⭐" }
                    }
                }
            }
        }
    }
}

async fn fetch_pending_events(world_id: &str) -> Result<Vec<NarrativeEventData>, String> {
    let url = format!("/api/worlds/{}/narrative-events/pending", world_id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<NarrativeEventData>>()
                .await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            // Fall back to fetching all and filtering client-side
            let all_url = format!("/api/worlds/{}/narrative-events", world_id);
            let response = Request::get(&all_url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            if response.ok() {
                let all: Vec<NarrativeEventData> = response
                    .json()
                    .await
                    .map_err(|e| format!("Parse error: {}", e))?;
                Ok(all.into_iter().filter(|e| e.is_active && !e.is_triggered).collect())
            } else {
                Err(format!("HTTP error: {}", response.status()))
            }
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
            response
                .json::<Vec<NarrativeEventData>>()
                .await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            // Fall back to fetching all
            let all_url = format!("http://localhost:3000/api/worlds/{}/narrative-events", world_id);
            let response = client.get(&all_url).send().await.map_err(|e| format!("Request failed: {}", e))?;
            if response.status().is_success() {
                let all: Vec<NarrativeEventData> = response.json().await.map_err(|e| format!("Parse error: {}", e))?;
                Ok(all.into_iter().filter(|e| e.is_active && !e.is_triggered).collect())
            } else {
                Err(format!("HTTP error: {}", response.status()))
            }
        }
    }
}
