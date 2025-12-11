//! Approval popup component
//!
//! Shows proposed NPC dialogue and actions for DM approval before execution.

use dioxus::prelude::*;

/// A proposed action/tool call from the LLM
#[derive(Clone, PartialEq)]
pub struct ProposedAction {
    /// Action name
    pub name: String,
    /// Action description
    pub description: String,
    /// Whether this action is checked/approved
    pub checked: bool,
}

/// Props for the ApprovalPopup component
#[derive(Props, Clone, PartialEq)]
pub struct ApprovalPopupProps {
    /// The NPC that will perform the action
    pub npc_name: String,
    /// Proposed dialogue text from LLM
    pub dialogue: String,
    /// List of proposed actions/tool calls
    pub proposed_actions: Vec<ProposedAction>,
    /// Handler when Accept is clicked
    pub on_accept: EventHandler<Vec<ProposedAction>>,
    /// Handler when Modify is clicked
    pub on_modify: EventHandler<()>,
    /// Handler when Reject is clicked
    pub on_reject: EventHandler<()>,
}

/// ApprovalPopup component - Shows LLM response for approval
///
/// Displays proposed NPC dialogue and tool calls with checkboxes.
/// DM can approve, modify, or reject the proposed actions.
#[component]
pub fn ApprovalPopup(props: ApprovalPopupProps) -> Element {
    let mut actions = use_signal(|| props.proposed_actions.clone());

    rsx! {
        div {
            class: "approval-popup",
            style: "background: #1f2937; border: 2px solid #f59e0b; border-radius: 0.75rem; padding: 1.25rem; box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);",

            // Header
            h3 {
                style: "color: #f59e0b; margin: 0 0 1rem 0; font-size: 1rem;",
                "Approval Required"
            }

            // NPC and action info
            div {
                style: "margin-bottom: 1rem; padding: 0.75rem; background: rgba(0, 0, 0, 0.2); border-radius: 0.5rem;",

                p {
                    style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0 0 0.25rem 0;",
                    "Action from: {props.npc_name}"
                }
            }

            // Dialogue box
            div {
                style: "margin-bottom: 1rem; padding: 1rem; background: rgba(0, 0, 0, 0.3); border-left: 3px solid #3b82f6; border-radius: 0.5rem;",

                p {
                    style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0 0 0.5rem 0;",
                    "Proposed Dialogue"
                }

                p {
                    style: "color: white; font-style: italic; margin: 0; line-height: 1.5;",
                    "\"{props.dialogue}\""
                }
            }

            // Proposed actions list
            if !props.proposed_actions.is_empty() {
                div {
                    style: "margin-bottom: 1rem;",

                    p {
                        style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0 0 0.5rem 0;",
                        "Proposed Actions"
                    }

                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem;",

                        for (idx, action) in actions.read().iter().enumerate() {
                            ProposedActionCheckbox {
                                action: action.clone(),
                                on_toggle: move |_| {
                                    let mut updated_actions = actions.read().to_vec();
                                    if idx < updated_actions.len() {
                                        updated_actions[idx].checked = !updated_actions[idx].checked;
                                    }
                                    actions.set(updated_actions);
                                }
                            }
                        }
                    }
                }
            }

            // Action buttons
            div {
                style: "display: flex; gap: 0.75rem;",

                button {
                    onclick: move |_| props.on_accept.call(actions.read().to_vec()),
                    style: "flex: 1; padding: 0.75rem; background: #22c55e; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600; font-size: 0.875rem; transition: background 0.2s;",
                    onmouseover: move |_| {},
                    "Accept"
                }

                button {
                    onclick: move |_| props.on_modify.call(()),
                    style: "flex: 1; padding: 0.75rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600; font-size: 0.875rem; transition: background 0.2s;",
                    "Modify"
                }

                button {
                    onclick: move |_| props.on_reject.call(()),
                    style: "flex: 1; padding: 0.75rem; background: #ef4444; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600; font-size: 0.875rem; transition: background 0.2s;",
                    "Reject"
                }
            }
        }
    }
}

/// Individual action checkbox
#[component]
fn ProposedActionCheckbox(
    action: ProposedAction,
    on_toggle: EventHandler<()>,
) -> Element {
    rsx! {
        label {
            style: "display: flex; align-items: flex-start; gap: 0.75rem; padding: 0.75rem; background: rgba(0, 0, 0, 0.2); border-radius: 0.375rem; cursor: pointer; transition: background 0.2s;",

            input {
                r#type: "checkbox",
                checked: action.checked,
                onchange: move |_| on_toggle.call(()),
                style: "margin-top: 0.25rem; cursor: pointer;",
            }

            div {
                style: "flex: 1;",

                div {
                    style: "color: white; font-size: 0.875rem; font-weight: 500;",
                    "{action.name}"
                }

                div {
                    style: "color: #9ca3af; font-size: 0.75rem; margin-top: 0.25rem;",
                    "{action.description}"
                }
            }
        }
    }
}
