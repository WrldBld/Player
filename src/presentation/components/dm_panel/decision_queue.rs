//! Decision Queue Panel - Shows pending approvals and recent decisions for the DM

use dioxus::prelude::*;

use crate::presentation::state::use_session_state;

/// Compact decision queue view for Director mode
#[component]
pub fn DecisionQueuePanel() -> Element {
    let session_state = use_session_state();

    let pending = session_state.pending_approvals.read().clone();
    let history = session_state.get_approval_history();

    let mut show_history_only: Signal<bool> = use_signal(|| false);

    let has_pending = !pending.is_empty();
    let has_history = !history.is_empty();

    rsx! {
        div {
            class: "decision-queue-panel",
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 0.75rem; display: flex; flex-direction: column; gap: 0.5rem;",

            // Header with toggle
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",

                h3 {
                    style: "color: #9ca3af; margin: 0; font-size: 0.75rem; text-transform: uppercase;",
                    "Decision Queue"
                }

                if has_history {
                    label {
                        style: "display: inline-flex; align-items: center; gap: 0.25rem; color: #9ca3af; font-size: 0.75rem;",
                        input {
                            r#type: "checkbox",
                            checked: *show_history_only.read(),
                            onchange: move |_| {
                                let current = *show_history_only.read();
                                show_history_only.set(!current);
                            },
                        }
                        span { "Show history only" }
                    }
                }
            }

            // Content
            if !has_pending && !has_history {
                div {
                    style: "color: #6b7280; font-size: 0.8125rem; text-align: center; padding: 0.5rem;",
                    "No decisions yet"
                }
            } else {
                // Pending approvals list
                if has_pending && !*show_history_only.read() {
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.4rem; margin-bottom: 0.25rem;",
                        for approval in pending.iter() {
                            div {
                                key: "{approval.request_id}",
                                style: "display: flex; flex-direction: column; gap: 0.15rem; padding: 0.4rem 0.5rem; background: #0f0f23; border-radius: 0.375rem;",

                                div {
                                    style: "display: flex; justify-content: space-between; align-items: center;",
                                    span { style: "color: white; font-size: 0.8rem;", "{approval.npc_name}" }
                                    span { style: "color: #fbbf24; font-size: 0.7rem;", "Pending" }
                                }

                                if let Some(challenge) = &approval.challenge_suggestion {
                                    div {
                                        style: "color: #9ca3af; font-size: 0.7rem;",
                                        "Challenge: {challenge.challenge_name}"
                                    }
                                } else if let Some(narrative) = &approval.narrative_event_suggestion {
                                    div {
                                        style: "color: #9ca3af; font-size: 0.7rem;",
                                        "Narrative: {narrative.event_name}"
                                    }
                                } else {
                                    div {
                                        style: "color: #9ca3af; font-size: 0.7rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                        "{approval.proposed_dialogue}"
                                    }
                                }
                            }
                        }
                    }
                }

                // History list
                if has_history {
                    div {
                        style: "border-top: 1px solid #374151; margin-top: 0.25rem; padding-top: 0.25rem; display: flex; flex-direction: column; gap: 0.35rem;",

                        // Caption
                        div {
                            style: "color: #6b7280; font-size: 0.7rem;",
                            "Recent decisions ({history.len()})"
                        }

                        for entry in history.iter().rev().take(10) {
                            div {
                                key: "{entry.request_id}-{entry.timestamp}",
                                style: "display: flex; justify-content: space-between; align-items: center; padding: 0.3rem 0.4rem; background: #0b1120; border-radius: 0.25rem;",

                                div {
                                    style: "display: flex; flex-direction: column; gap: 0.1rem;",
                                    span {
                                        style: "color: white; font-size: 0.78rem;",
                                        "{entry.npc_name}"
                                    }
                                    span {
                                        style: "color: #6b7280; font-size: 0.68rem;",
                                        "#{entry.request_id}"
                                    }
                                }

                                span {
                                    style: "color: #93c5fd; font-size: 0.7rem; text-transform: capitalize;",
                                    "{entry.outcome}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


