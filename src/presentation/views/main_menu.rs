//! Main menu view - Connect to a game server

use dioxus::prelude::*;

#[component]
pub fn MainMenu(on_connect: EventHandler<String>) -> Element {
    let mut server_url = use_signal(|| "ws://localhost:3000/ws".to_string());

    rsx! {
        div {
            class: "main-menu",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);",

            div {
                class: "menu-card",
                style: "background: #0f0f23; padding: 3rem; border-radius: 1rem; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5); max-width: 400px; width: 90%;",

                h1 {
                    style: "color: white; text-align: center; margin-bottom: 0.5rem; font-size: 2.5rem;",
                    "WrldBldr"
                }
                p {
                    style: "color: #9ca3af; text-align: center; margin-bottom: 2rem;",
                    "TTRPG Game Client"
                }

                div {
                    style: "margin-bottom: 1.5rem;",

                    label {
                        style: "display: block; color: #9ca3af; margin-bottom: 0.5rem; font-size: 0.875rem;",
                        "Server Address"
                    }
                    input {
                        r#type: "text",
                        value: "{server_url}",
                        oninput: move |e| server_url.set(e.value()),
                        style: "width: 100%; padding: 0.75rem; border: 1px solid #374151; border-radius: 0.5rem; background: #1f2937; color: white; font-size: 1rem; box-sizing: border-box;",
                        placeholder: "ws://localhost:3000/ws"
                    }
                }

                button {
                    onclick: move |_| on_connect.call(server_url.read().clone()),
                    style: "width: 100%; padding: 0.875rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; font-size: 1rem; font-weight: 600; cursor: pointer; transition: background 0.2s;",
                    "Connect to Server"
                }
            }
        }
    }
}
