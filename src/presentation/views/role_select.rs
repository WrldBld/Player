//! Role selection view

use dioxus::prelude::*;

use crate::UserRole;

#[component]
pub fn RoleSelect(on_select_role: EventHandler<UserRole>) -> Element {
    rsx! {
        div {
            class: "role-select",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);",

            h2 {
                style: "color: white; margin-bottom: 2rem; font-size: 1.75rem;",
                "Select Your Role"
            }

            div {
                style: "display: flex; gap: 1.5rem; flex-wrap: wrap; justify-content: center;",

                // Dungeon Master card
                RoleCard {
                    title: "Dungeon Master",
                    description: "Direct the story, approve NPC responses, and manage the game world.",
                    icon: "🎭",
                    color: "#ef4444",
                    on_click: move |_| on_select_role.call(UserRole::DungeonMaster)
                }

                // Player card
                RoleCard {
                    title: "Player",
                    description: "Experience the story as a player character in a visual novel style.",
                    icon: "⚔️",
                    color: "#3b82f6",
                    on_click: move |_| on_select_role.call(UserRole::Player)
                }

                // Spectator card
                RoleCard {
                    title: "Spectator",
                    description: "Watch the game unfold without participating.",
                    icon: "👁️",
                    color: "#8b5cf6",
                    on_click: move |_| on_select_role.call(UserRole::Spectator)
                }
            }
        }
    }
}

#[component]
fn RoleCard(
    title: &'static str,
    description: &'static str,
    icon: &'static str,
    color: &'static str,
    on_click: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            onclick: move |_| on_click.call(()),
            style: format!(
                "background: #0f0f23; border: 2px solid {}; border-radius: 1rem; padding: 2rem; width: 250px; cursor: pointer; transition: transform 0.2s, box-shadow 0.2s;",
                color
            ),

            div {
                style: "font-size: 3rem; margin-bottom: 1rem;",
                "{icon}"
            }
            h3 {
                style: format!("color: {}; margin-bottom: 0.5rem; font-size: 1.25rem;", color),
                "{title}"
            }
            p {
                style: "color: #9ca3af; font-size: 0.875rem; line-height: 1.5;",
                "{description}"
            }
        }
    }
}
