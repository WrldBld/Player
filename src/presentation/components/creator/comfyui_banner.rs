//! ComfyUI Connection Status Banner
//!
//! Displays a banner when ComfyUI is disconnected or experiencing issues.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ComfyUIBannerProps {
    pub state: String, // "connected", "degraded", "disconnected", "circuit_open"
    pub message: Option<String>,
    pub retry_in_seconds: Option<u32>,
}

/// Banner component showing ComfyUI connection status
#[component]
pub fn ComfyUIBanner(props: ComfyUIBannerProps) -> Element {
    // Only show banner if not connected
    if props.state == "connected" {
        return rsx! {};
    }

    let bg_color = match props.state.as_str() {
        "degraded" => "#f59e0b",
        "disconnected" | "circuit_open" => "#ef4444",
        _ => "#6b7280",
    };

    let icon = match props.state.as_str() {
        "degraded" => "⚠️",
        "disconnected" | "circuit_open" => "🔴",
        _ => "❓",
    };

    rsx! {
        div {
            style: format!(
                "background: {}; color: white; padding: 0.75rem 1rem; display: flex; align-items: center; justify-content: space-between; border-radius: 0.375rem; margin-bottom: 1rem;",
                bg_color
            ),
            div {
                style: "display: flex; align-items: center; gap: 0.5rem;",
                span { style: "font-size: 1.25rem;", "{icon}" }
                div {
                    div {
                        style: "font-weight: 500; font-size: 0.875rem;",
                        if let Some(msg) = props.message.as_ref() {
                            "{msg}"
                        } else {
                            "ComfyUI Connection Issue"
                        }
                    }
                    if let Some(seconds) = props.retry_in_seconds {
                        div {
                            style: "font-size: 0.75rem; opacity: 0.9; margin-top: 0.25rem;",
                            "Reconnecting in {seconds}s..."
                        }
                    }
                }
            }
            button {
                onclick: move |_| {
                    // TODO: Trigger manual retry
                },
                style: "background: rgba(255, 255, 255, 0.2); border: 1px solid rgba(255, 255, 255, 0.3); color: white; padding: 0.375rem 0.75rem; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                "Retry Now"
            }
        }
    }
}

