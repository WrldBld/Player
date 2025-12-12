//! Skills Display Component
//!
//! Displays player's skills with modifiers in a compact panel.

use dioxus::prelude::*;

/// A player skill with its properties
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerSkillData {
    /// Unique skill ID
    pub id: String,
    /// Human-readable skill name
    pub name: String,
    /// Skill category (e.g., "Combat", "Magic", "Social")
    pub category: String,
    /// Skill modifier (positive or negative)
    pub modifier: i32,
    /// Whether character is proficient in this skill
    pub proficient: bool,
}

/// Props for SkillsDisplay component
#[derive(Props, Clone, PartialEq)]
pub struct SkillsDisplayProps {
    /// List of player skills to display
    pub skills: Vec<PlayerSkillData>,
    /// Called when closing the skills panel
    pub on_close: EventHandler<()>,
}

/// Skills Display Component
///
/// Shows a compact panel of the player's skills, grouped by category,
/// with modifiers highlighted and proficient skills marked.
#[component]
pub fn SkillsDisplay(props: SkillsDisplayProps) -> Element {
    // Group skills by category
    let mut categories: std::collections::BTreeMap<String, Vec<PlayerSkillData>> =
        std::collections::BTreeMap::new();

    for skill in props.skills.iter() {
        categories
            .entry(skill.category.clone())
            .or_insert_with(Vec::new)
            .push(skill.clone());
    }

    rsx! {
        // Modal overlay
        div {
            id: "skills-overlay",
            style: "position: fixed; inset: 0; background: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| {
                props.on_close.call(());
            },

            // Modal content
            div {
                id: "skills-modal",
                style: "background: linear-gradient(135deg, #1a1a2e 0%, #0f0f23 100%); padding: 2rem; border-radius: 1rem; max-width: 600px; width: 90%; max-height: 80vh; overflow-y: auto; border: 2px solid #3b82f6; box-shadow: 0 20px 60px rgba(59, 130, 246, 0.2);",
                onclick: move |e: dioxus::prelude::MouseEvent| {
                    e.stop_propagation();
                },

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;",

                    h2 {
                        style: "color: #3b82f6; margin: 0; font-size: 1.5rem;",
                        "Skills"
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #9ca3af; cursor: pointer; font-size: 1.5rem; padding: 0;",
                        "×"
                    }
                }

                // Skills by category
                if categories.is_empty() {
                    p {
                        style: "color: #9ca3af; text-align: center; margin: 2rem 0;",
                        "No skills available"
                    }
                } else {
                    {
                        categories.iter().map(|(category, skills)| {
                            rsx! {
                                div {
                                    key: "{category}",
                                    style: "margin-bottom: 1.5rem;",

                                    // Category header
                                    h3 {
                                        style: "color: #f59e0b; font-size: 0.875rem; text-transform: uppercase; letter-spacing: 0.05em; margin: 0 0 0.75rem 0;",
                                        "{category}"
                                    }

                                    // Skills in this category
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 0.5rem;",

                                        {
                                            skills.iter().map(|skill| {
                                                let modifier_color = if skill.modifier >= 0 {
                                                    "#22c55e"
                                                } else {
                                                    "#ef4444"
                                                };

                                                let border_color = if skill.proficient { "#f59e0b" } else { "transparent" };

                                                rsx! {
                                                    div {
                                                        key: "{skill.id}",
                                                        style: "display: flex; align-items: center; justify-content: space-between; padding: 0.75rem; background: rgba(0, 0, 0, 0.3); border-radius: 0.5rem; border-left: 3px solid {border_color};",

                                                        // Skill name
                                                        div {
                                                            style: "display: flex; align-items: center; gap: 0.5rem;",

                                                            span {
                                                                style: "color: white; flex: 1;",
                                                                "{skill.name}"
                                                            }

                                                            // Proficiency indicator
                                                            if skill.proficient {
                                                                span {
                                                                    style: "color: #f59e0b; font-size: 0.75rem; font-weight: bold;",
                                                                    "Prof"
                                                                }
                                                            }
                                                        }

                                                        // Modifier
                                                        div {
                                                            style: "color: {modifier_color}; font-weight: bold; font-size: 1rem; min-width: 3rem; text-align: right;",
                                                            if skill.modifier >= 0 {
                                                                "+{skill.modifier}"
                                                            } else {
                                                                "{skill.modifier}"
                                                            }
                                                        }
                                                    }
                                                }
                                            })
                                        }
                                    }
                                }
                            }
                        })
                    }
                }

                // Footer
                div {
                    style: "margin-top: 1.5rem; padding-top: 1rem; border-top: 1px solid rgba(255, 255, 255, 0.1);",

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "width: 100%; padding: 0.75rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                        "Close"
                    }
                }
            }
        }
    }
}
