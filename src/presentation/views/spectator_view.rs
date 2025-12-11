//! Spectator View - Watch the game without participating

use dioxus::prelude::*;

#[component]
pub fn SpectatorView(on_back: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "spectator-view",
            style: "height: 100%; display: flex; flex-direction: column; position: relative; background: linear-gradient(to bottom, #1a1a2e, #2d1b3d);",

            // Back button
            button {
                onclick: move |_| on_back.call(()),
                style: "position: absolute; top: 1rem; left: 1rem; z-index: 100; padding: 0.5rem 1rem; background: rgba(0,0,0,0.5); color: white; border: 1px solid #374151; border-radius: 0.5rem; cursor: pointer;",
                "← Back"
            }

            // Spectator badge
            div {
                style: "position: absolute; top: 1rem; right: 1rem; z-index: 100; padding: 0.5rem 1rem; background: rgba(139, 92, 246, 0.2); color: #a78bfa; border: 1px solid #8b5cf6; border-radius: 0.5rem;",
                "👁️ Spectating"
            }

            // Main content - similar to PC view but read-only
            div {
                style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;",

                div {
                    style: "text-align: center; color: white;",
                    h2 { style: "margin-bottom: 1rem;", "Spectator Mode" }
                    p { style: "color: #9ca3af;", "You are watching the game unfold." }
                    p { style: "color: #9ca3af; margin-top: 0.5rem;", "The scene will appear here once the game begins." }
                }
            }
        }
    }
}
