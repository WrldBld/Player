//! Generation Queue Panel - Shows active and completed generation batches

use dioxus::prelude::*;

use crate::presentation::state::{use_generation_state, use_game_state, BatchStatus, GenerationBatch, SuggestionStatus, SuggestionTask};
use crate::presentation::services::{
    visible_batches,
    visible_suggestions,
    mark_batch_read_and_sync,
    mark_suggestion_read_and_sync,
};

/// Filter type for the generation queue
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum QueueFilter {
    #[default]
    All,
    Images,
    Suggestions,
}

/// Panel showing generation queue status (images and suggestions)
#[component]
pub fn GenerationQueuePanel() -> Element {
    let generation_state = use_generation_state();
    let game_state = use_game_state();
    let mut selected_suggestion: Signal<Option<SuggestionTask>> = use_signal(|| None);
    let mut show_read: Signal<bool> = use_signal(|| false);
    let mut active_filter: Signal<QueueFilter> = use_signal(|| QueueFilter::All);

    let show_read_val = *show_read.read();
    let filter_val = *active_filter.read();
    let all_batches = visible_batches(&generation_state, show_read_val);
    let all_suggestions = visible_suggestions(&generation_state, show_read_val);
    
    // Compute counts before filtering
    let batch_count = all_batches.len();
    let suggestion_count = all_suggestions.len();
    let total_count = batch_count + suggestion_count;
    
    // Filter by active filter
    let visible_batches = match filter_val {
        QueueFilter::All | QueueFilter::Images => all_batches.clone(),
        QueueFilter::Suggestions => Vec::new(),
    };
    let visible_suggestions = match filter_val {
        QueueFilter::All | QueueFilter::Suggestions => all_suggestions.clone(),
        QueueFilter::Images => Vec::new(),
    };
    let total_items = visible_batches.len() + visible_suggestions.len();
    
    // Counts for badge
    let active_batch_count = generation_state.active_count();
    let active_suggestion_count = generation_state.active_suggestion_count();
    let total_active = active_batch_count + active_suggestion_count;

    // Derive world_id from game state if available (for scoping read markers)
    let world_id = game_state
        .world
        .read()
        .as_ref()
        .map(|w| w.world.id.clone());

    rsx! {
        div {
            class: "generation-queue",
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 0.75rem;",

            // Header with filter tabs and toggle for read items
            div {
                style: "margin-bottom: 0.5rem;",
                
                // Title and badge
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem;",
                    h3 {
                        style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0; display: flex; align-items: center; gap: 0.5rem;",
                        "Generation Queue"
                        if total_active > 0 {
                            span {
                                style: "background: #f59e0b; color: white; border-radius: 0.75rem; padding: 0.125rem 0.375rem; font-size: 0.625rem; font-weight: bold;",
                                "{total_active}"
                            }
                        }
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
                        span { "Show read" }
                    }
                }
                
                // Filter tabs
                div {
                    style: "display: flex; gap: 0.25rem; border-bottom: 1px solid #374151;",
                    FilterTab {
                        label: "All",
                        count: total_count,
                        is_active: filter_val == QueueFilter::All,
                        onclick: move |_| active_filter.set(QueueFilter::All),
                    }
                    FilterTab {
                        label: "Images",
                        count: batch_count,
                        is_active: filter_val == QueueFilter::Images,
                        onclick: move |_| active_filter.set(QueueFilter::Images),
                    }
                    FilterTab {
                        label: "Suggestions",
                        count: suggestion_count,
                        is_active: filter_val == QueueFilter::Suggestions,
                        onclick: move |_| active_filter.set(QueueFilter::Suggestions),
                    }
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
                        QueueItemRow {
                            batch: batch.clone(),
                            show_read: show_read_val,
                            world_id: world_id.clone(),
                        }
                    }

                    // Show suggestion tasks
                    for suggestion in visible_suggestions.iter() {
                        SuggestionQueueRow {
                            suggestion: suggestion.clone(),
                            selected_suggestion,
                            show_read: show_read_val,
                            world_id: world_id.clone(),
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

/// Filter tab component
#[component]
fn FilterTab(
    label: &'static str,
    count: usize,
    is_active: bool,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            onclick: move |_| onclick.call(()),
            style: format!(
                "flex: 1; padding: 0.375rem 0.5rem; background: transparent; border: none; border-bottom: 2px solid {}; color: {}; font-size: 0.75rem; cursor: pointer; transition: all 0.2s;",
                if is_active { "#8b5cf6" } else { "transparent" },
                if is_active { "white" } else { "#9ca3af" }
            ),
            "{label}"
            if count > 0 {
                span {
                    style: "margin-left: 0.25rem; color: #6b7280;",
                    "({count})"
                }
            }
        }
    }
}

/// Individual queue item row for image batches
#[component]
fn QueueItemRow(
    batch: GenerationBatch,
    #[props(default = false)] show_read: bool,
    world_id: Option<String>,
) -> Element {
    let mut expanded_error: Signal<bool> = use_signal(|| false);
    let batch_id = batch.batch_id.clone();
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

            div {
                style: "display: flex; align-items: center; gap: 0.25rem;",
                match &batch.status {
                    BatchStatus::Generating { progress } => rsx! {
                        div {
                            style: "width: 50px; height: 4px; background: #374151; border-radius: 2px; overflow: hidden;",
                            div {
                                style: format!("width: {}%; height: 100%; background: #f59e0b;", progress),
                            }
                        }
                        button {
                            onclick: move |_| {
                                // TODO: Cancel batch (requires Engine endpoint)
                                tracing::warn!("Cancel batch not yet implemented");
                            },
                            style: "padding: 0.125rem 0.375rem; background: #ef4444; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.625rem;",
                            "Cancel"
                        }
                    },
                    BatchStatus::Ready { .. } => rsx! {
                        button {
                            onclick: move |_| {
                                let mut state = use_generation_state();
                                let batch_id = batch.batch_id.clone();
                                let world_id_clone = world_id.clone();
                                spawn(async move {
                                    if let Err(e) = mark_batch_read_and_sync(&mut state, &batch_id, world_id_clone.as_deref()).await {
                                        tracing::error!("Failed to mark batch read and sync: {}", e);
                                    }
                                });
                                // TODO: Also navigate/select in the relevant Creator UI in the future
                            },
                            style: "padding: 0.25rem 0.5rem; background: #22c55e; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "Select"
                        }
                        button {
                            onclick: {
                                let batch_id = batch_id.clone();
                                move |_| {
                                    let mut state = use_generation_state();
                                    state.remove_batch(&batch_id);
                                }
                            },
                            style: "padding: 0.25rem 0.5rem; background: #6b7280; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "Clear"
                        }
                    },
                    BatchStatus::Failed { error: _ } => rsx! {
                        button {
                            onclick: move |_| {
                                let current = *expanded_error.read();
                                expanded_error.set(!current);
                            },
                            style: "padding: 0.25rem 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            if *expanded_error.read() { "Hide Error" } else { "Show Error" }
                        }
                        button {
                            onclick: move |_| {
                                // TODO: Retry batch (requires Engine endpoint)
                                tracing::warn!("Retry batch not yet implemented");
                            },
                            style: "padding: 0.25rem 0.5rem; background: #f59e0b; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "Retry"
                        }
                        button {
                            onclick: move |_| {
                                let mut state = use_generation_state();
                                state.remove_batch(&batch.batch_id);
                            },
                            style: "padding: 0.25rem 0.5rem; background: #6b7280; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "Clear"
                        }
                    },
                    _ => rsx! {
                        span { style: format!("color: {}; font-size: 0.75rem;", status_color), "{status_text}" }
                    },
                }
            }
            
            // Expanded error details for failed batches
            if let BatchStatus::Failed { error } = &batch.status {
                if *expanded_error.read() {
                    div {
                        style: "margin-top: 0.5rem; padding: 0.5rem; background: #1f2937; border-radius: 0.25rem; border-left: 3px solid #ef4444;",
                        div {
                            style: "color: #ef4444; font-size: 0.75rem; font-weight: bold; margin-bottom: 0.25rem;",
                            "Error Details:"
                        }
                        div {
                            style: "color: #e5e7eb; font-size: 0.75rem; white-space: pre-wrap; word-break: break-word;",
                            "{error}"
                        }
                    }
                }
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
    world_id: Option<String>,
) -> Element {
    let mut expanded_error: Signal<bool> = use_signal(|| false);
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
    let request_id_for_view = suggestion.request_id.clone();
    let request_id_for_clear = suggestion.request_id.clone();
    let request_id_for_failed_clear = suggestion.request_id.clone();

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

            div {
                style: "display: flex; align-items: center; gap: 0.25rem;",
                match &suggestion.status {
                    SuggestionStatus::Ready { .. } => rsx! {
                        button {
                            onclick: move |_| {
                                selected_suggestion.set(Some(suggestion_clone.clone()));
                                let mut state = use_generation_state();
                                let req_id = request_id_for_view.clone();
                                let world_id_clone = world_id.clone();
                                spawn(async move {
                                    if let Err(e) = mark_suggestion_read_and_sync(&mut state, &req_id, world_id_clone.as_deref()).await {
                                        tracing::error!("Failed to mark suggestion read and sync: {}", e);
                                    }
                                });
                            },
                            style: "padding: 0.25rem 0.5rem; background: #22c55e; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "View"
                        }
                        button {
                            onclick: move |_| {
                                let mut state = use_generation_state();
                                state.remove_suggestion(&request_id_for_clear);
                            },
                            style: "padding: 0.25rem 0.5rem; background: #6b7280; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "Clear"
                        }
                    },
                    SuggestionStatus::Queued | SuggestionStatus::Processing => rsx! {
                        span { style: format!("color: {}; font-size: 0.75rem;", status_color), "{status_text}" }
                        button {
                            onclick: move |_| {
                                // TODO: Cancel suggestion (requires Engine endpoint)
                                tracing::warn!("Cancel suggestion not yet implemented");
                            },
                            style: "padding: 0.125rem 0.375rem; background: #ef4444; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.625rem;",
                            "Cancel"
                        }
                    },
                    SuggestionStatus::Failed { error: _ } => rsx! {
                        button {
                            onclick: move |_| {
                                let current = *expanded_error.read();
                                expanded_error.set(!current);
                            },
                            style: "padding: 0.25rem 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            if *expanded_error.read() { "Hide Error" } else { "Show Error" }
                        }
                        button {
                            onclick: move |_| {
                                // TODO: Retry suggestion (requires Engine endpoint)
                                tracing::warn!("Retry suggestion not yet implemented");
                            },
                            style: "padding: 0.25rem 0.5rem; background: #f59e0b; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "Retry"
                        }
                        button {
                            onclick: move |_| {
                                let mut state = use_generation_state();
                                state.remove_suggestion(&request_id_for_failed_clear);
                            },
                            style: "padding: 0.25rem 0.5rem; background: #6b7280; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "Clear"
                        }
                    },
                }
            }
            
            // Expanded error details for failed suggestions
            if let SuggestionStatus::Failed { error } = &suggestion.status {
                if *expanded_error.read() {
                    div {
                        style: "margin-top: 0.5rem; padding: 0.5rem; background: #1f2937; border-radius: 0.25rem; border-left: 3px solid #ef4444;",
                        div {
                            style: "color: #ef4444; font-size: 0.75rem; font-weight: bold; margin-bottom: 0.25rem;",
                            "Error Details:"
                        }
                        div {
                            style: "color: #e5e7eb; font-size: 0.75rem; white-space: pre-wrap; word-break: break-word;",
                            "{error}"
                        }
                    }
                }
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
