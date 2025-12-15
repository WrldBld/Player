//! Generation Queue Panel - Shows active and completed generation batches

use dioxus::prelude::*;

use crate::presentation::state::{use_generation_state, BatchStatus, GenerationBatch, SuggestionStatus, SuggestionTask};

/// Panel showing generation queue status (images and suggestions)
#[component]
pub fn GenerationQueuePanel() -> Element {
    let generation_state = use_generation_state();
    let batches = generation_state.get_batches();
    let suggestions = generation_state.get_suggestions();
    let total_items = batches.len() + suggestions.len();

    rsx! {
        div {
            class: "generation-queue",
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 0.75rem;",

            h3 {
                style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0 0 0.75rem 0;",
                "Generation Queue"
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
                    for batch in batches.iter() {
                        QueueItemRow { batch: Some(batch.clone()), suggestion: None }
                    }

                    // Show suggestion tasks
                    for suggestion in suggestions.iter() {
                        QueueItemRow { batch: None, suggestion: Some(suggestion.clone()) }
                    }
                }
            }
        }
    }
}

/// Individual queue item row (can be batch or suggestion)
#[component]
fn QueueItemRow(
    #[props(default)]
    batch: Option<GenerationBatch>,
    #[props(default)]
    suggestion: Option<SuggestionTask>,
) -> Element {
    // Handle batch (image generation)
    if let Some(batch) = batch {
    let (status_icon, status_color, status_text) = match &batch.status {
            BatchStatus::Queued { position } => ("🖼️", "#9ca3af", format!("#{} in queue", position)),
            BatchStatus::Generating { progress } => ("⚙️", "#f59e0b", format!("{}%", progress)),
            BatchStatus::Ready { asset_count } => ("✅", "#22c55e", format!("{} ready", asset_count)),
            BatchStatus::Failed { error: _ } => ("❌", "#ef4444", "Failed".into()),
    };

    let display_name = format!("{} ({})", batch.entity_id, batch.entity_type);

    rsx! {
        div {
            class: "queue-item",
            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: #0f0f23; border-radius: 0.25rem;",

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
    // Handle suggestion (text generation)
    else if let Some(suggestion) = suggestion {
        let (status_icon, status_color, status_text) = match &suggestion.status {
            SuggestionStatus::Queued => ("💭", "#9ca3af", "Queued".to_string()),
            SuggestionStatus::Processing => ("⚙️", "#f59e0b", "Processing".to_string()),
            SuggestionStatus::Ready { suggestions: results } => ("✅", "#22c55e", format!("{} ready", results.len())),
            SuggestionStatus::Failed { error: _ } => ("❌", "#ef4444", "Failed".to_string()),
        };

        let display_name = format!("{} suggestion", suggestion.field_type.replace("_", " "));

        rsx! {
            div {
                class: "queue-item",
                style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: #0f0f23; border-radius: 0.25rem;",

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
    // Neither provided (shouldn't happen)
    else {
        rsx! { div {} }
    }
}
