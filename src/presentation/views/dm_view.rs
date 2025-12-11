//! Dungeon Master View - Directorial control panel and Creator mode

use dioxus::prelude::*;

use crate::presentation::components::creator::{CreatorMode};

/// The active tab/mode in the DM View
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DMMode {
    #[default]
    Director,
    Creator,
}

#[component]
pub fn DMView(on_back: EventHandler<()>) -> Element {
    let mut active_mode = use_signal(|| DMMode::Director);

    rsx! {
        div {
            class: "dm-view",
            style: "height: 100%; display: flex; flex-direction: column; background: #0f0f23;",

            // Top bar with tabs and back button
            div {
                class: "dm-topbar",
                style: "display: flex; align-items: center; justify-content: space-between; padding: 0.75rem 1rem; background: #1a1a2e; border-bottom: 1px solid #374151;",

                // Tab buttons
                div {
                    style: "display: flex; gap: 0.5rem;",

                    TabButton {
                        label: "Director Mode",
                        active: *active_mode.read() == DMMode::Director,
                        on_click: move |_| active_mode.set(DMMode::Director),
                    }

                    TabButton {
                        label: "Creator Mode",
                        active: *active_mode.read() == DMMode::Creator,
                        on_click: move |_| active_mode.set(DMMode::Creator),
                    }
                }

                // Back button
                button {
                    onclick: move |_| on_back.call(()),
                    style: "padding: 0.5rem 1rem; background: rgba(0,0,0,0.5); color: white; border: 1px solid #374151; border-radius: 0.5rem; cursor: pointer;",
                    "← Back"
                }
            }

            // Content area
            div {
                class: "dm-content",
                style: "flex: 1; overflow: hidden;",

                match *active_mode.read() {
                    DMMode::Director => rsx! { DirectorModeContent {} },
                    DMMode::Creator => rsx! { CreatorMode {} },
                }
            }
        }
    }
}

#[component]
fn TabButton(label: &'static str, active: bool, on_click: EventHandler<()>) -> Element {
    let bg_color = if active { "#3b82f6" } else { "transparent" };
    let border = if active { "none" } else { "1px solid #374151" };

    rsx! {
        button {
            onclick: move |_| on_click.call(()),
            style: format!(
                "padding: 0.5rem 1rem; background: {}; color: white; border: {}; border-radius: 0.5rem; cursor: pointer; font-weight: {};",
                bg_color,
                border,
                if active { "600" } else { "400" }
            ),
            "{label}"
        }
    }
}

/// The original Director mode content (directing gameplay)
#[component]
fn DirectorModeContent() -> Element {
    let mut scene_notes = use_signal(|| "The party has just entered the Dragon's Rest Inn. Tension is high - the bartender knows more than he's letting on.".to_string());
    let mut current_tone = use_signal(|| "Mysterious".to_string());
    let mut show_approval_popup = use_signal(|| true);

    rsx! {
        div {
            style: "height: 100%; display: grid; grid-template-columns: 1fr 350px; gap: 1rem; padding: 1rem;",

            // Left panel - Scene preview and conversation
            div {
                class: "main-panel",
                style: "display: flex; flex-direction: column; gap: 1rem;",

                // Scene preview (smaller version of what players see)
                div {
                    class: "scene-preview",
                    style: "height: 200px; background: linear-gradient(to bottom, #1a1a2e, #2d1b3d); border-radius: 0.5rem; position: relative; overflow: hidden;",

                    div {
                        style: "position: absolute; bottom: 20%; left: 50%; transform: translateX(-50%); display: flex; gap: 2rem;",
                        div { style: "width: 80px; height: 120px; background: rgba(255,255,255,0.1); border-radius: 0.25rem;" }
                        div { style: "width: 80px; height: 120px; background: rgba(59,130,246,0.2); border-radius: 0.25rem;" }
                    }
                }

                // Conversation log
                div {
                    class: "conversation-log",
                    style: "flex: 1; background: #1a1a2e; border-radius: 0.5rem; padding: 1rem; overflow-y: auto;",

                    h3 { style: "color: #9ca3af; margin-bottom: 1rem; font-size: 0.875rem; text-transform: uppercase;", "Conversation Log" }

                    div {
                        style: "display: flex; flex-direction: column; gap: 0.75rem;",

                        LogEntry { speaker: "System", text: "Player 'Kira' connected as Fighter", is_system: true }
                        LogEntry { speaker: "Kira", text: "I approach the bartender and ask about the missing merchant.", is_system: false }
                        LogEntry { speaker: "System", text: "Awaiting LLM response...", is_system: true }
                    }
                }

                // Approval popup
                if *show_approval_popup.read() {
                    div {
                        class: "approval-popup",
                        style: "background: #1f2937; border: 2px solid #f59e0b; border-radius: 0.75rem; padding: 1.25rem;",

                        h4 { style: "color: #f59e0b; margin-bottom: 1rem;", "Approval Required" }

                        div { style: "margin-bottom: 1rem;",
                            p { style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Bartender will say:" }
                            p { style: "color: white; font-style: italic;",
                                "\"Ah, old Geralt? Haven't seen him in days. Word is he went north, but... *leans in* ...between you and me, I saw some shady characters following him.\""
                            }
                        }

                        div { style: "margin-bottom: 1rem;",
                            p { style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.5rem;", "Proposed Actions:" }
                            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                                ProposedAction { name: "Reveal Information", description: "The northern pass is dangerous", approved: true }
                                ProposedAction { name: "Give Item: Map Fragment", description: "A torn piece of a map", approved: false }
                            }
                        }

                        div { style: "display: flex; gap: 0.5rem;",
                            button {
                                onclick: move |_| show_approval_popup.set(false),
                                style: "flex: 1; padding: 0.75rem; background: #22c55e; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600;",
                                "Accept"
                            }
                            button {
                                style: "flex: 1; padding: 0.75rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600;",
                                "Modify"
                            }
                            button {
                                style: "flex: 1; padding: 0.75rem; background: #ef4444; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 600;",
                                "Reject"
                            }
                        }
                    }
                }
            }

            // Right panel - Directorial controls
            div {
                class: "control-panel",
                style: "display: flex; flex-direction: column; gap: 1rem; overflow-y: auto;",

                // Scene notes
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Scene Notes" }
                    textarea {
                        value: "{scene_notes}",
                        oninput: move |e| scene_notes.set(e.value()),
                        style: "width: 100%; height: 100px; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; resize: vertical; box-sizing: border-box;",
                    }
                }

                // Tone selection
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Tone" }
                    select {
                        value: "{current_tone}",
                        onchange: move |e| current_tone.set(e.value()),
                        style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white;",
                        option { value: "Serious", "Serious" }
                        option { value: "Lighthearted", "Lighthearted" }
                        option { value: "Tense", "Tense" }
                        option { value: "Mysterious", "Mysterious" }
                        option { value: "Comedic", "Comedic" }
                    }
                }

                // NPC Motivations
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "NPC Motivations" }

                    NPCMotivationCard {
                        name: "Bartender",
                        mood: "Suspicious",
                        goal: "Protect his regulars while getting information"
                    }

                    NPCMotivationCard {
                        name: "Hooded Figure",
                        mood: "Watchful",
                        goal: "Observe the newcomers without being noticed"
                    }
                }

                // Active NPCs
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Active NPCs" }

                    div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                        NPCToggle { name: "Bartender", active: true }
                        NPCToggle { name: "Hooded Figure", active: false }
                        NPCToggle { name: "Drunk Patron", active: false }
                    }
                }

                // Quick actions
                div {
                    class: "panel-section",
                    style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1rem;",

                    h3 { style: "color: #9ca3af; margin-bottom: 0.75rem; font-size: 0.875rem; text-transform: uppercase;", "Quick Actions" }

                    div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                        button { style: "padding: 0.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer;", "View Social Graph" }
                        button { style: "padding: 0.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer;", "View Timeline" }
                        button { style: "padding: 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.5rem; cursor: pointer;", "Start Combat" }
                    }
                }
            }
        }
    }
}

#[component]
fn LogEntry(speaker: &'static str, text: &'static str, is_system: bool) -> Element {
    rsx! {
        div {
            style: format!(
                "padding: 0.5rem; border-radius: 0.25rem; {}",
                if is_system { "background: rgba(59, 130, 246, 0.1); color: #60a5fa; font-size: 0.875rem;" }
                else { "color: white;" }
            ),
            if !is_system {
                span { style: "color: #3b82f6; font-weight: bold;", "{speaker}: " }
            }
            span { "{text}" }
        }
    }
}

#[component]
fn ProposedAction(name: &'static str, description: &'static str, approved: bool) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: rgba(0,0,0,0.2); border-radius: 0.25rem;",
            input {
                r#type: "checkbox",
                checked: approved,
            }
            div {
                span { style: "color: white; font-size: 0.875rem;", "{name}" }
                span { style: "color: #9ca3af; font-size: 0.75rem; margin-left: 0.5rem;", "- {description}" }
            }
        }
    }
}

#[component]
fn NPCMotivationCard(name: &'static str, mood: &'static str, goal: &'static str) -> Element {
    rsx! {
        div {
            style: "padding: 0.75rem; background: #0f0f23; border-radius: 0.5rem; margin-bottom: 0.5rem;",
            h4 { style: "color: #3b82f6; font-size: 0.875rem; margin-bottom: 0.25rem;", "{name}" }
            p { style: "color: #9ca3af; font-size: 0.75rem;", "Mood: {mood}" }
            p { style: "color: #9ca3af; font-size: 0.75rem;", "Goal: {goal}" }
        }
    }
}

#[component]
fn NPCToggle(name: &'static str, active: bool) -> Element {
    rsx! {
        label {
            style: "display: flex; align-items: center; gap: 0.5rem; color: white; cursor: pointer;",
            input {
                r#type: "checkbox",
                checked: active,
            }
            span { "{name}" }
        }
    }
}
