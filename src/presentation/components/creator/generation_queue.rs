//! Generation Queue Panel - Shows active and completed generation batches

use dioxus::prelude::*;

use crate::presentation::state::{use_generation_state, use_session_state, BatchStatus, GenerationBatch, SuggestionStatus, SuggestionTask};
use crate::presentation::services::persist_generation_read_state;
use crate::infrastructure::http_client::HttpClient;

/// Panel showing generation queue status (images and suggestions)
#[component]
pub fn GenerationQueuePanel() -> Element {
    let generation_state = use_generation_state();
    let batches = generation_state.get_batches();
    let suggestions = generation_state.get_suggestions();
    let mut selected_suggestion: Signal<Option<SuggestionTask>> = use_signal(|| None);
    let mut show_read: Signal<bool> = use_signal(|| false);

    let show_read_val = *show_read.read();
    let visible_batches: Vec<GenerationBatch> = batches
        .into_iter()
        .filter(|b| show_read_val || !b.is_read)
        .collect();
    let visible_suggestions: Vec<SuggestionTask> = suggestions
        .into_iter()
        .filter(|s| show_read_val || !s.is_read)
        .collect();
    let total_items = visible_batches.len() + visible_suggestions.len();

    rsx! {
        div {
            class: "generation-queue",
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 0.75rem;",

            // Header with toggle for read items
            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem;",
                h3 {
                    style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0;",
                    "Generation Queue"
                }
                label {
                    style: "display: inline-flex; align-items: center; gap: 0.25rem; color: #9ca3af; font-size: 0.75rem;",
                    input {
                        r#type: "checkbox",
                        checked: *show_read.read(),
                        onchange: move |_| {
                            let current = *show_read.read();
                            show_read.set(!current);
                        },
                    }
                    span { "Show already read" }
                }
            }

            if total_items == 0 {
                div {
                    style: "color: #6b7280; font-size: 0.875rem; text-align: center; padding: 1rem;",
                    "No generations in progress"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem;",

                    // Show image batches
                    for batch in visible_batches.iter() {
                        QueueItemRow { batch: batch.clone(), show_read: show_read_val }
                    }

                    // Show suggestion tasks
                    for suggestion in visible_suggestions.iter() {
                        SuggestionQueueRow {
                            suggestion: suggestion.clone(),
                            selected_suggestion,
                            show_read: show_read_val,
                        }
                    }
                }
            }

            // Modal for viewing suggestion details
            if let Some(active) = selected_suggestion.read().as_ref() {
                SuggestionViewModal {
                    suggestion: active.clone(),
                    on_close: {
                        move |_| {
                            selected_suggestion.set(None);
                        }
                    },
                }
            }
        }
    }
}

/// Individual queue item row for image batches
#[component]
fn QueueItemRow(batch: GenerationBatch, #[props(default = false)] show_read: bool) -> Element {
    let session_state = use_session_state();
    let (status_icon, status_color, status_text) = match &batch.status {
        BatchStatus::Queued { position } => ("🖼️", "#9ca3af", format!("#{} in queue", position)),
        BatchStatus::Generating { progress } => ("⚙️", "#f59e0b", format!("{}%", progress)),
        BatchStatus::Ready { asset_count } => ("✅", "#22c55e", format!("{} ready", asset_count)),
        BatchStatus::Failed { error: _ } => ("❌", "#ef4444", "Failed".into()),
    };

    let display_name = format!("{} ({})", batch.entity_id, batch.entity_type);

    // Dim read items when history is shown
    let opacity_style = if batch.is_read && show_read {
        "opacity: 0.6;"
    } else {
        ""
    };

    rsx! {
        div {
            class: "queue-item",
            style: format!(
                "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: #0f0f23; border-radius: 0.25rem; {}",
                opacity_style
            ),

            span { style: format!("color: {};", status_color), "{status_icon}" }

            div { style: "flex: 1; min-width: 0;",
                div { style: "color: white; font-size: 0.875rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{display_name}"
                }
                div { style: "color: #6b7280; font-size: 0.75rem;",
                    "{batch.asset_type}"
                }
            }

            match &batch.status {
                BatchStatus::Generating { progress } => rsx! {
                    div {
                        style: "width: 50px; height: 4px; background: #374151; border-radius: 2px; overflow: hidden;",
                        div {
                            style: format!("width: {}%; height: 100%; background: #f59e0b;", progress),
                        }
                    }
                },
                BatchStatus::Ready { .. } => rsx! {
                    button {
                        onclick: move |_| {
                            let mut state = use_generation_state();
                            state.mark_batch_read(&batch.batch_id);
                            persist_generation_read_state(&state);

                            // Best-effort backend sync
                            let user_id = session_state.user_id.read().clone();
                            if let Some(uid) = user_id {
                                let batch_id = batch.batch_id.clone();
                                spawn(async move {
                                    let body = serde_json::json!({
                                        "user_id": uid,
                                        "read_batches": [batch_id],
                                        "read_suggestions": [],
                                    });
                                    if let Err(e) = HttpClient::post::<serde_json::Value, _>("/api/generation/read-state", &body).await {
                                        tracing::error!("Failed to sync batch read-state: {}", e);
                                    }
                                });
                            }

                            // TODO: Also navigate/select in the relevant Creator UI in the future
                        },
                        style: "padding: 0.25rem 0.5rem; background: #22c55e; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                        "Select"
                    }
                },
                _ => rsx! {
                    span { style: format!("color: {}; font-size: 0.75rem;", status_color), "{status_text}" }
                },
            }
        }
    }
}

/// Queue row for suggestion tasks (text generation)
#[component]
fn SuggestionQueueRow(
    suggestion: SuggestionTask,
    selected_suggestion: Signal<Option<SuggestionTask>>,
    #[props(default = false)]
    show_read: bool,
) -> Element {
    let session_state = use_session_state();
    let (status_icon, status_color, status_text) = match &suggestion.status {
        SuggestionStatus::Queued => ("💭", "#9ca3af", "Queued".to_string()),
        SuggestionStatus::Processing => ("⚙️", "#f59e0b", "Processing".to_string()),
        SuggestionStatus::Ready { suggestions: results } => {
            ("✅", "#22c55e", format!("{} ready", results.len()))
        }
        SuggestionStatus::Failed { error: _ } => ("❌", "#ef4444", "Failed".to_string()),
    };

    let display_name = format!("{} suggestion", suggestion.field_type.replace("_", " "));
    let suggestion_clone = suggestion.clone();
    let request_id = suggestion.request_id.clone();

    let opacity_style = if suggestion.is_read && show_read {
        "opacity: 0.6;"
    } else {
        ""
    };

    rsx! {
        div {
            class: "queue-item",
            style: format!(
                "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: #0f0f23; border-radius: 0.25rem; {}",
                opacity_style
            ),

            span { style: format!("color: {};", status_color), "{status_icon}" }

            div { style: "flex: 1; min-width: 0;",
                div { style: "color: white; font-size: 0.875rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{display_name}"
                }
                if let Some(entity_id) = &suggestion.entity_id {
                    div { style: "color: #6b7280; font-size: 0.75rem;",
                        "{entity_id}"
                    }
                }
            }

            match &suggestion.status {
                SuggestionStatus::Ready { .. } => rsx! {
                    button {
                        onclick: move |_| {
                            selected_suggestion.set(Some(suggestion_clone.clone()));
                            let mut state = use_generation_state();
                            state.mark_suggestion_read(&request_id);
                            persist_generation_read_state(&state);

                            // Best-effort backend sync
                            let user_id = session_state.user_id.read().clone();
                            if let Some(uid) = user_id {
                                let req_id = request_id.clone();
                                spawn(async move {
                                    let body = serde_json::json!({
                                        "user_id": uid,
                                        "read_batches": [],
                                        "read_suggestions": [req_id],
                                    });
                                    if let Err(e) = HttpClient::post::<serde_json::Value, _>("/api/generation/read-state", &body).await {
                                        tracing::error!("Failed to sync suggestion read-state: {}", e);
                                    }
                                });
                            }
                        },
                        style: "padding: 0.25rem 0.5rem; background: #22c55e; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                        "View"
                    }
                },
                _ => rsx! {
                    span { style: format!("color: {}; font-size: 0.75rem;", status_color), "{status_text}" }
                },
            }
        }
    }
}

/// Modal displaying full suggestion options for a selected task
#[component]
fn SuggestionViewModal(suggestion: SuggestionTask, on_close: EventHandler<()>) -> Element {
    // Extract suggestions if ready
    let suggestions = match &suggestion.status {
        SuggestionStatus::Ready { suggestions } => suggestions.clone(),
        _ => Vec::new(),
    };

    let title = format!("Suggestions for {}", suggestion.field_type.replace("_", " "));

    rsx! {
        // Backdrop
        div {
            onclick: move |_| on_close.call(()),
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 200;",

            // Modal content
            div {
                onclick: move |evt| evt.stop_propagation(),
                style: "background: #111827; border-radius: 0.5rem; padding: 1rem 1.25rem; max-width: 480px; width: 100%; max-height: 70vh; overflow-y: auto; box-shadow: 0 20px 25px -5px rgba(0,0,0,0.4);",

                h3 {
                    style: "color: white; font-size: 0.95rem; margin-bottom: 0.5rem;",
                    "{title}"
                }

                if let Some(entity_id) = &suggestion.entity_id {
                    div {
                        style: "color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.75rem;",
                        "Entity: {entity_id}"
                    }
                }

                if suggestions.is_empty() {
                    div {
                        style: "color: #9ca3af; font-size: 0.85rem;",
                        "No suggestion options available (still processing or failed)."
                    }
                } else {
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem;",
                        for (idx, text) in suggestions.iter().enumerate() {
                            div {
                                key: "{idx}",
                                style: "padding: 0.5rem 0.75rem; background: #1f2937; border-radius: 0.375rem; color: #e5e7eb; font-size: 0.875rem;",
                                "{text}"
                            }
                        }
                    }
                }

                div {
                    style: "display: flex; justify-content: flex-end; margin-top: 0.75rem;",
                    button {
                        onclick: move |_| on_close.call(()),
                        style: "padding: 0.25rem 0.75rem; background: #4b5563; color: white; border: none; border-radius: 0.375rem; font-size: 0.8rem; cursor: pointer;",
                        "Close"
                    }
                }
            }
        }
    }
}
