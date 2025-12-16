//! Approval popup component
//!
//! Shows proposed NPC dialogue and actions for DM approval before execution.
//! Supports challenge outcome regeneration and discard functionality.

use dioxus::prelude::*;
use crate::application::dto::{ChallengeSuggestionInfo, NarrativeEventSuggestionInfo, OutcomeDetailData};

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

/// Outcome details for display in the approval popup
#[derive(Clone, PartialEq, Default)]
pub struct ChallengeOutcomes {
    pub success: Option<OutcomeDetailData>,
    pub failure: Option<OutcomeDetailData>,
    pub critical_success: Option<OutcomeDetailData>,
    pub critical_failure: Option<OutcomeDetailData>,
}

/// Data for outcome regeneration request
#[derive(Clone, PartialEq)]
pub struct RegenerateOutcomeData {
    pub request_id: String,
    /// Which outcome to regenerate (None = all)
    pub outcome_type: Option<String>,
    pub guidance: Option<String>,
}

/// Data for discard challenge request
#[derive(Clone, PartialEq)]
pub struct DiscardChallengeData {
    pub request_id: String,
    pub feedback: Option<String>,
}

/// Props for the ApprovalPopup component
#[derive(Props, Clone, PartialEq)]
pub struct ApprovalPopupProps {
    /// Unique request ID for this approval item
    #[props(default)]
    pub request_id: String,
    /// The NPC that will perform the action
    pub npc_name: String,
    /// Proposed dialogue text from LLM
    pub dialogue: String,
    /// List of proposed actions/tool calls
    pub proposed_actions: Vec<ProposedAction>,
    /// Optional challenge suggestion from Engine
    pub challenge_suggestion: Option<ChallengeSuggestionInfo>,
    /// Optional challenge outcomes for detailed display
    #[props(default)]
    pub challenge_outcomes: Option<ChallengeOutcomes>,
    /// Optional narrative event suggestion from Engine
    pub narrative_event_suggestion: Option<NarrativeEventSuggestionInfo>,
    /// Handler when Accept is clicked
    pub on_accept: EventHandler<Vec<ProposedAction>>,
    /// Handler when Modify is clicked
    pub on_modify: EventHandler<()>,
    /// Handler when Reject is clicked
    pub on_reject: EventHandler<()>,
    /// Handler when regenerate outcome is requested (optional)
    #[props(default)]
    pub on_regenerate_outcome: Option<EventHandler<RegenerateOutcomeData>>,
    /// Handler when discard challenge is requested (optional)
    #[props(default)]
    pub on_discard_challenge: Option<EventHandler<DiscardChallengeData>>,
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

            // Challenge suggestion section
            if let Some(suggestion) = &props.challenge_suggestion {
                {
                    let request_id = props.request_id.clone();
                    let on_regenerate = props.on_regenerate_outcome.clone();
                    let on_discard = props.on_discard_challenge.clone();
                    let outcomes = props.challenge_outcomes.clone();

                    rsx! {
                        div {
                            style: "margin-bottom: 1rem; padding: 1rem; background: rgba(245, 158, 11, 0.1); border: 1px solid #f59e0b; border-radius: 0.5rem;",

                            // Header with title and discard button
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",

                                h4 {
                                    style: "color: #f59e0b; margin: 0; font-size: 0.875rem; display: flex; gap: 0.5rem; align-items: center;",
                                    "Challenge Suggested"
                                }

                                // Discard button
                                if on_discard.is_some() {
                                    {
                                        let request_id_discard = request_id.clone();
                                        let handler = on_discard.clone();
                                        rsx! {
                                            button {
                                                onclick: move |_| {
                                                    if let Some(h) = &handler {
                                                        h.call(DiscardChallengeData {
                                                            request_id: request_id_discard.clone(),
                                                            feedback: None,
                                                        });
                                                    }
                                                },
                                                style: "padding: 0.25rem 0.5rem; background: rgba(107, 114, 128, 0.5); color: #9ca3af; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.625rem; text-transform: uppercase;",
                                                "Discard"
                                            }
                                        }
                                    }
                                }
                            }

                            // Challenge info
                            div {
                                style: "margin-bottom: 0.75rem;",

                                div {
                                    style: "display: flex; justify-content: space-between; align-items: baseline;",

                                    span {
                                        style: "color: white; font-weight: bold; font-size: 0.875rem;",
                                        "{suggestion.challenge_name}"
                                    }

                                    span {
                                        style: "color: #9ca3af; margin-left: 0.5rem; font-size: 0.75rem;",
                                        "({suggestion.skill_name} - {suggestion.difficulty_display})"
                                    }
                                }
                            }

                            div {
                                style: "margin-bottom: 0.5rem;",

                                p {
                                    style: "color: #9ca3af; font-size: 0.75rem; margin: 0 0 0.25rem 0;",
                                    "Confidence: {suggestion.confidence}"
                                }
                            }

                            p {
                                style: "color: #9ca3af; font-size: 0.75rem; font-style: italic; margin: 0 0 0.75rem 0; line-height: 1.4;",
                                "\"{suggestion.reasoning}\""
                            }

                            // Outcome details (if available)
                            if let Some(ref outcomes) = outcomes {
                                OutcomeDetailsSection {
                                    outcomes: outcomes.clone(),
                                    request_id: request_id.clone(),
                                    on_regenerate: on_regenerate.clone(),
                                }
                            }

                            // Action buttons
                            div {
                                style: "display: flex; gap: 0.5rem;",

                                button {
                                    style: "flex: 1; padding: 0.5rem; background: rgba(34, 197, 94, 0.8); color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.75rem; font-weight: 600;",
                                    "Approve Challenge"
                                }

                                button {
                                    style: "flex: 1; padding: 0.5rem; background: rgba(239, 68, 68, 0.8); color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.75rem; font-weight: 600;",
                                    "Skip Challenge"
                                }
                            }
                        }
                    }
                }
            }

            // Narrative event suggestion section
            if let Some(suggestion) = &props.narrative_event_suggestion {
                div {
                    style: "margin-bottom: 1rem; padding: 1rem; background: rgba(139, 92, 246, 0.1); border: 1px solid #8b5cf6; border-radius: 0.5rem;",

                    h4 {
                        style: "color: #8b5cf6; margin: 0 0 0.75rem 0; font-size: 0.875rem; display: flex; gap: 0.5rem; align-items: center;",
                        "Narrative Event Suggested"
                    }

                    div {
                        style: "margin-bottom: 0.75rem;",

                        div {
                            style: "display: flex; justify-content: space-between; align-items: baseline;",

                            span {
                                style: "color: white; font-weight: bold; font-size: 0.875rem;",
                                "{suggestion.event_name}"
                            }
                        }

                        if let Some(outcome) = &suggestion.suggested_outcome {
                            p {
                                style: "color: #9ca3af; font-size: 0.75rem; margin: 0.5rem 0 0 0;",
                                "Suggested Outcome: {outcome}"
                            }
                        }
                    }

                    div {
                        style: "margin-bottom: 0.5rem;",

                        p {
                            style: "color: #9ca3af; font-size: 0.75rem; margin: 0 0 0.25rem 0;",
                            "Confidence: {suggestion.confidence}"
                        }
                    }

                    p {
                        style: "color: #9ca3af; font-size: 0.75rem; font-style: italic; margin: 0 0 0.75rem 0; line-height: 1.4;",
                        "\"{suggestion.reasoning}\""
                    }

                    div {
                        style: "display: flex; gap: 0.5rem;",

                        button {
                            style: "flex: 1; padding: 0.5rem; background: rgba(139, 92, 246, 0.8); color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.75rem; font-weight: 600;",
                            "Trigger Event"
                        }

                        button {
                            style: "flex: 1; padding: 0.5rem; background: rgba(107, 114, 128, 0.8); color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.75rem; font-weight: 600;",
                            "Skip Event"
                        }
                    }
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

/// Props for OutcomeDetailsSection
#[derive(Props, Clone, PartialEq)]
struct OutcomeDetailsSectionProps {
    outcomes: ChallengeOutcomes,
    request_id: String,
    on_regenerate: Option<EventHandler<RegenerateOutcomeData>>,
}

/// Section displaying challenge outcomes with regeneration buttons
#[component]
fn OutcomeDetailsSection(props: OutcomeDetailsSectionProps) -> Element {
    let mut expanded_outcome = use_signal(|| Option::<String>::None);

    rsx! {
        div {
            style: "margin-bottom: 0.75rem; border-top: 1px solid rgba(245, 158, 11, 0.3); padding-top: 0.75rem;",

            // Header with "Regenerate All" button
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;",

                p {
                    style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0;",
                    "Outcomes"
                }

                if props.on_regenerate.is_some() {
                    {
                        let request_id = props.request_id.clone();
                        let handler = props.on_regenerate.clone();
                        rsx! {
                            button {
                                onclick: move |_| {
                                    if let Some(h) = &handler {
                                        h.call(RegenerateOutcomeData {
                                            request_id: request_id.clone(),
                                            outcome_type: None, // None = regenerate all
                                            guidance: None,
                                        });
                                    }
                                },
                                style: "padding: 0.25rem 0.5rem; background: rgba(59, 130, 246, 0.5); color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.625rem; text-transform: uppercase;",
                                "Regenerate All"
                            }
                        }
                    }
                }
            }

            // Outcome tabs
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",

                // Success outcome
                if let Some(ref outcome) = props.outcomes.success {
                    OutcomeTab {
                        label: "Success",
                        color: "#22c55e",
                        outcome: outcome.clone(),
                        outcome_type: "success".to_string(),
                        request_id: props.request_id.clone(),
                        on_regenerate: props.on_regenerate.clone(),
                        is_expanded: *expanded_outcome.read() == Some("success".to_string()),
                        on_toggle: move |_| {
                            let current = expanded_outcome.read().clone();
                            if current == Some("success".to_string()) {
                                expanded_outcome.set(None);
                            } else {
                                expanded_outcome.set(Some("success".to_string()));
                            }
                        },
                    }
                }

                // Failure outcome
                if let Some(ref outcome) = props.outcomes.failure {
                    OutcomeTab {
                        label: "Failure",
                        color: "#ef4444",
                        outcome: outcome.clone(),
                        outcome_type: "failure".to_string(),
                        request_id: props.request_id.clone(),
                        on_regenerate: props.on_regenerate.clone(),
                        is_expanded: *expanded_outcome.read() == Some("failure".to_string()),
                        on_toggle: move |_| {
                            let current = expanded_outcome.read().clone();
                            if current == Some("failure".to_string()) {
                                expanded_outcome.set(None);
                            } else {
                                expanded_outcome.set(Some("failure".to_string()));
                            }
                        },
                    }
                }

                // Critical success outcome
                if let Some(ref outcome) = props.outcomes.critical_success {
                    OutcomeTab {
                        label: "Critical Success",
                        color: "#fbbf24",
                        outcome: outcome.clone(),
                        outcome_type: "critical_success".to_string(),
                        request_id: props.request_id.clone(),
                        on_regenerate: props.on_regenerate.clone(),
                        is_expanded: *expanded_outcome.read() == Some("critical_success".to_string()),
                        on_toggle: move |_| {
                            let current = expanded_outcome.read().clone();
                            if current == Some("critical_success".to_string()) {
                                expanded_outcome.set(None);
                            } else {
                                expanded_outcome.set(Some("critical_success".to_string()));
                            }
                        },
                    }
                }

                // Critical failure outcome
                if let Some(ref outcome) = props.outcomes.critical_failure {
                    OutcomeTab {
                        label: "Critical Failure",
                        color: "#dc2626",
                        outcome: outcome.clone(),
                        outcome_type: "critical_failure".to_string(),
                        request_id: props.request_id.clone(),
                        on_regenerate: props.on_regenerate.clone(),
                        is_expanded: *expanded_outcome.read() == Some("critical_failure".to_string()),
                        on_toggle: move |_| {
                            let current = expanded_outcome.read().clone();
                            if current == Some("critical_failure".to_string()) {
                                expanded_outcome.set(None);
                            } else {
                                expanded_outcome.set(Some("critical_failure".to_string()));
                            }
                        },
                    }
                }
            }
        }
    }
}

/// Props for OutcomeTab
#[derive(Props, Clone, PartialEq)]
struct OutcomeTabProps {
    label: &'static str,
    color: &'static str,
    outcome: OutcomeDetailData,
    outcome_type: String,
    request_id: String,
    on_regenerate: Option<EventHandler<RegenerateOutcomeData>>,
    is_expanded: bool,
    on_toggle: EventHandler<()>,
}

/// Individual outcome tab with expandable details and inline editing
#[component]
fn OutcomeTab(props: OutcomeTabProps) -> Element {
    let border_color = props.color;
    let label = props.label;
    let is_expanded = props.is_expanded;
    let mut editing = use_signal(|| false);
    let mut edited_flavor = use_signal(|| props.outcome.flavor_text.clone());
    let mut edited_direction = use_signal(|| props.outcome.scene_direction.clone());

    rsx! {
        div {
            style: "background: rgba(0, 0, 0, 0.2); border-radius: 0.375rem; border-left: 3px solid {border_color};",

            // Header (clickable to expand)
            button {
                onclick: move |_| props.on_toggle.call(()),
                style: "width: 100%; display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0.75rem; background: none; border: none; cursor: pointer; text-align: left;",

                span {
                    style: "color: {border_color}; font-size: 0.75rem; font-weight: 600;",
                    "{label}"
                }

                span {
                    style: "color: #6b7280; font-size: 0.75rem;",
                    if is_expanded { "v" } else { ">" }
                }
            }

            // Expanded content
            if is_expanded {
                div {
                    style: "padding: 0 0.75rem 0.75rem 0.75rem;",

                    // Flavor text
                    if !props.outcome.flavor_text.is_empty() || *editing.read() {
                        div {
                            style: "margin-bottom: 0.5rem;",

                            p {
                                style: "color: #6b7280; font-size: 0.625rem; text-transform: uppercase; margin: 0 0 0.25rem 0;",
                                "Flavor"
                            }
                            if *editing.read() {
                                textarea {
                                    value: "{edited_flavor}",
                                    oninput: move |e| edited_flavor.set(e.value()),
                                    style: "width: 100%; padding: 0.5rem; background: #020617; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.75rem; min-height: 60px; resize: vertical;",
                                }
                            } else {
                                p {
                                    style: "color: white; font-size: 0.75rem; font-style: italic; margin: 0; line-height: 1.4;",
                                    "\"{props.outcome.flavor_text}\""
                                }
                            }
                        }
                    }

                    // Scene direction
                    if !props.outcome.scene_direction.is_empty() || *editing.read() {
                        div {
                            style: "margin-bottom: 0.5rem;",

                            p {
                                style: "color: #6b7280; font-size: 0.625rem; text-transform: uppercase; margin: 0 0 0.25rem 0;",
                                "Scene Direction"
                            }
                            if *editing.read() {
                                textarea {
                                    value: "{edited_direction}",
                                    oninput: move |e| edited_direction.set(e.value()),
                                    style: "width: 100%; padding: 0.5rem; background: #020617; border: 1px solid #374151; border-radius: 0.375rem; color: #d1d5db; font-size: 0.75rem; min-height: 60px; resize: vertical;",
                                }
                            } else {
                                p {
                                    style: "color: #d1d5db; font-size: 0.75rem; margin: 0; line-height: 1.4;",
                                    "{props.outcome.scene_direction}"
                                }
                            }
                        }
                    }

                    // Tool calls (if any)
                    if !props.outcome.proposed_tools.is_empty() {
                        div {
                            style: "margin-bottom: 0.5rem;",

                            p {
                                style: "color: #6b7280; font-size: 0.625rem; text-transform: uppercase; margin: 0 0 0.25rem 0;",
                                "Tool Calls ({props.outcome.proposed_tools.len()})"
                            }
                            div {
                                style: "display: flex; flex-wrap: wrap; gap: 0.25rem;",

                                for tool in props.outcome.proposed_tools.iter() {
                                    span {
                                        key: "{tool.id}",
                                        style: "padding: 0.125rem 0.375rem; background: rgba(59, 130, 246, 0.2); color: #93c5fd; border-radius: 0.25rem; font-size: 0.625rem;",
                                        "{tool.name}"
                                    }
                                }
                            }
                        }
                    }

                    // Edit / Regenerate controls
                    if props.on_regenerate.is_some() {
                        div {
                            style: "display: flex; justify-content: flex-end; gap: 0.5rem;",

                            // Toggle edit/save
                            {
                                let editing_sig = editing.clone();
                                rsx! {
                                    button {
                                        onclick: move |_| {
                                            let current = *editing_sig.read();
                                            editing_sig.set(!current);
                                        },
                                        style: "padding: 0.25rem 0.5rem; background: rgba(148, 163, 184, 0.3); color: #e5e7eb; border: 1px solid #64748b; border-radius: 0.25rem; cursor: pointer; font-size: 0.625rem; text-transform: uppercase;",
                                        if *editing.read() { "Done Editing" } else { "Edit" }
                                    }
                                }
                            }

                            // Regenerate (optionally incorporating edited text as guidance)
                            {
                                let request_id = props.request_id.clone();
                                let outcome_type = props.outcome_type.clone();
                                let handler = props.on_regenerate.clone();
                                let edited_flavor_sig = edited_flavor.clone();
                                let edited_direction_sig = edited_direction.clone();

                                rsx! {
                                    button {
                                        onclick: move |_| {
                                            if let Some(h) = &handler {
                                                let mut guidance_parts = Vec::new();
                                                let flavor = edited_flavor_sig.read().trim().to_string();
                                                let direction = edited_direction_sig.read().trim().to_string();
                                                if !flavor.is_empty() {
                                                    guidance_parts.push(format!("Flavor: {}", flavor));
                                                }
                                                if !direction.is_empty() {
                                                    guidance_parts.push(format!("Scene: {}", direction));
                                                }
                                                let guidance = if guidance_parts.is_empty() {
                                                    None
                                                } else {
                                                    Some(guidance_parts.join(" | "))
                                                };

                                                h.call(RegenerateOutcomeData {
                                                    request_id: request_id.clone(),
                                                    outcome_type: Some(outcome_type.clone()),
                                                    guidance,
                                                });
                                            }
                                        },
                                        style: "padding: 0.25rem 0.5rem; background: rgba(59, 130, 246, 0.3); color: #93c5fd; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.625rem; text-transform: uppercase;",
                                        "Regenerate"
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
