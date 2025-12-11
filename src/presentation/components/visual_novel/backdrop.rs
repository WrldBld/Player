//! Backdrop component for visual novel scenes
//!
//! Displays the background image for the current scene.

use dioxus::prelude::*;

/// Props for the Backdrop component
#[derive(Props, Clone, PartialEq)]
pub struct BackdropProps {
    /// URL or asset path for the backdrop image
    #[props(default)]
    pub image_url: Option<String>,
    /// Whether to show fade transition animation
    #[props(default = false)]
    pub transitioning: bool,
    /// Optional children to render on top of the backdrop
    #[props(default)]
    pub children: Element,
}

/// Backdrop component - displays the scene background
///
/// Uses the `.vn-backdrop` Tailwind class for styling.
/// Falls back to a gradient if no image is provided.
#[component]
pub fn Backdrop(props: BackdropProps) -> Element {
    let bg_style = match &props.image_url {
        Some(url) => format!(
            "background-image: url('{}'); background-size: cover; background-position: center;",
            url
        ),
        None => "background: linear-gradient(to bottom, #1a1a2e, #2d1b3d);".to_string(),
    };

    rsx! {
        div {
            class: "vn-backdrop",
            style: "position: absolute; inset: 0; {bg_style}",

            // Fade overlay for scene transitions
            if props.transitioning {
                div {
                    class: "backdrop-fade",
                    style: "position: absolute; inset: 0; background: black; animation: fadeOut 0.5s ease-out forwards;",
                }
            }

            // Vignette effect
            div {
                class: "backdrop-vignette",
                style: "position: absolute; inset: 0; box-shadow: inset 0 0 150px rgba(0, 0, 0, 0.5); pointer-events: none;",
            }

            // Children (character sprites, etc.)
            {props.children}
        }
    }
}

/// A simple loading backdrop shown while assets are loading
#[component]
pub fn LoadingBackdrop() -> Element {
    rsx! {
        div {
            class: "vn-backdrop",
            style: "position: absolute; inset: 0; background: linear-gradient(to bottom, #1a1a2e, #2d1b3d); display: flex; align-items: center; justify-content: center;",

            div {
                style: "text-align: center; color: #9ca3af;",

                div {
                    style: "font-size: 1.5rem; margin-bottom: 1rem;",
                    "Loading..."
                }

                div {
                    class: "loading-spinner",
                    style: "width: 40px; height: 40px; border: 3px solid #374151; border-top-color: #d4af37; border-radius: 50%; animation: spin 1s linear infinite; margin: 0 auto;",
                }
            }
        }
    }
}
