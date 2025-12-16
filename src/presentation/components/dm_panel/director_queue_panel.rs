//! Director Queue Panel - Minimal generation queue view for Director Mode
//!
//! Shows active generation batches and suggestions in a compact format
//! suitable for Director Mode overlay.

use dioxus::prelude::*;

use crate::presentation::state::{use_generation_state, BatchStatus, SuggestionStatus};

/// Props for DirectorQueuePanel
#[derive(Props, Clone, PartialEq)]
pub struct DirectorQueuePanelProps {
    /// Handler called when panel should close
    pub on_close: EventHandler<()>,
}

/// Minimal queue panel for Director Mode
#[component]
pub fn DirectorQueuePanel(props: DirectorQueuePanelProps) -> Element {
    let generation_state = use_generation_state();
    let batches = generation_state.get_batches();
    let suggestions = generation_state.get_suggestions();

    // Filter to only active items
    let active_batches: Vec<_> = batches
        .iter()
        .filter(|b| matches!(b.status, BatchStatus::Queued { .. } | BatchStatus::Generating { .. }))
        .collect();
    let active_suggestions: Vec<_> = suggestions
        .iter()
        .filter(|s| matches!(s.status, SuggestionStatus::Queued | SuggestionStatus::Processing))
        .collect();

    rsx! {
        div {
            class: "director-queue-panel",
            style: "position: fixed; top: 0; right: 0; bottom: 0; width: 400px; background: #1a1a2e; border-left: 1px solid #374151; z-index: 1000; display: flex; flex-direction: column; box-shadow: -4px 0 6px rgba(0, 0, 0, 0.3);",

            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-bottom: 1px solid #374151;",
                h3 { style: "color: white; margin: 0; font-size: 1rem;", "Generation Queue" }
                button {
                    onclick: move |_| props.on_close.call(()),
                    style: "padding: 0.25rem 0.5rem; background: transparent; color: #9ca3af; border: none; cursor: pointer; font-size: 1.25rem;",
                    "×"
                }
            }

            // Content
            div {
                style: "flex: 1; overflow-y: auto; padding: 1rem;",
                if active_batches.is_empty() && active_suggestions.is_empty() {
                    div {
                        style: "text-align: center; color: #6b7280; padding: 2rem;",
                        "No active generations"
                    }
                } else {
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.75rem;",
                        
                        // Active batches
                        for batch in active_batches.iter() {
                            div {
                                key: "{batch.batch_id}",
                                style: "padding: 0.75rem; background: #0f0f23; border-radius: 0.5rem; border-left: 3px solid #8b5cf6;",
                                div {
                                    style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 0.5rem;",
                                    div {
                                        style: "flex: 1;",
                                        div {
                                            style: "color: white; font-size: 0.875rem; font-weight: 500;",
                                            "{batch.entity_type} - {batch.asset_type}"
                                        }
                                        div {
                                            style: "color: #9ca3af; font-size: 0.75rem; margin-top: 0.25rem;",
                                            match &batch.status {
                                                BatchStatus::Queued { position } => format!("#{} in queue", position),
                                                BatchStatus::Generating { progress } => format!("Generating... {}%", progress),
                                                _ => String::new(),
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Active suggestions
                        for suggestion in active_suggestions.iter() {
                            div {
                                key: "{suggestion.request_id}",
                                style: "padding: 0.75rem; background: #0f0f23; border-radius: 0.5rem; border-left: 3px solid #3b82f6;",
                                div {
                                    style: "display: flex; justify-content: space-between; align-items: start;",
                                    div {
                                        style: "flex: 1;",
                                        div {
                                            style: "color: white; font-size: 0.875rem; font-weight: 500;",
                                            "{suggestion.field_type} suggestion"
                                        }
                                        div {
                                            style: "color: #9ca3af; font-size: 0.75rem; margin-top: 0.25rem;",
                                            match &suggestion.status {
                                                SuggestionStatus::Queued => "Queued",
                                                SuggestionStatus::Processing => "Processing...",
                                                _ => "",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

