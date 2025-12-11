//! Tone selector component
//!
//! Dropdown selector for scene tone/mood influencing NPC behavior.

use dioxus::prelude::*;

/// Available tone options for the scene
const TONE_OPTIONS: &[&str] = &[
    "Serious",
    "Lighthearted",
    "Tense",
    "Mysterious",
    "Comedic",
    "Romantic",
    "Tragic",
    "Suspenseful",
    "Custom",
];

/// Props for the ToneSelector component
#[derive(Props, Clone, PartialEq)]
pub struct ToneSelectorProps {
    /// Currently selected tone
    pub selected: String,
    /// Handler called when tone is changed
    pub on_change: EventHandler<String>,
    /// Optional custom tone description
    #[props(default)]
    pub custom_tone: Option<String>,
}

/// Get a color for a tone for visual feedback
fn get_tone_color(tone: &str) -> &'static str {
    match tone {
        "Serious" => "#ef4444",
        "Lighthearted" => "#fbbf24",
        "Tense" => "#f97316",
        "Mysterious" => "#8b5cf6",
        "Comedic" => "#ec4899",
        "Romantic" => "#ec4899",
        "Tragic" => "#6366f1",
        "Suspenseful" => "#dc2626",
        _ => "#3b82f6",
    }
}

/// ToneSelector component - Select scene tone/atmosphere
///
/// Provides a dropdown to select the current scene's tone,
/// which influences how NPCs should behave and respond.
#[component]
pub fn ToneSelector(props: ToneSelectorProps) -> Element {
    let tone_color = get_tone_color(&props.selected);

    rsx! {
        div {
            class: "tone-selector",
            style: "display: flex; flex-direction: column;",

            // Label
            label {
                style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.5rem; font-weight: 600;",
                "Scene Tone"
            }

            // Dropdown with color indicator
            div {
                style: "display: flex; align-items: center; gap: 0.75rem;",

                // Color indicator
                div {
                    style: format!(
                        "width: 12px; height: 12px; border-radius: 50%; background: {}; flex-shrink: 0;",
                        tone_color
                    ),
                }

                // Select dropdown
                select {
                    value: "{props.selected}",
                    onchange: move |e| props.on_change.call(e.value()),
                    style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem; cursor: pointer; transition: border-color 0.2s;",

                    for tone in TONE_OPTIONS.iter() {
                        option {
                            value: "{tone}",
                            "{tone}"
                        }
                    }
                }
            }

            // Tone description
            div {
                style: "margin-top: 0.5rem; padding: 0.5rem; background: rgba(0, 0, 0, 0.2); border-radius: 0.375rem; color: #9ca3af; font-size: 0.75rem;",

                match props.selected.as_str() {
                    "Serious" => rsx! { "A grave and solemn atmosphere. NPCs are thoughtful and careful." },
                    "Lighthearted" => rsx! { "A playful and fun atmosphere. NPCs are cheerful and relaxed." },
                    "Tense" => rsx! { "An edge-of-your-seat atmosphere. NPCs are suspicious and alert." },
                    "Mysterious" => rsx! { "A puzzle-like atmosphere. NPCs are secretive and cryptic." },
                    "Comedic" => rsx! { "A humorous atmosphere. NPCs are witty and absurd." },
                    "Romantic" => rsx! { "An emotional atmosphere. NPCs are passionate and genuine." },
                    "Tragic" => rsx! { "A mournful atmosphere. NPCs are sorrowful and resigned." },
                    "Suspenseful" => rsx! { "A thrilling atmosphere. NPCs are nervous and guarded." },
                    _ => {
                        if let Some(custom) = &props.custom_tone {
                            rsx! { "{custom}" }
                        } else {
                            rsx! { "Custom tone selected." }
                        }
                    }
                }
            }
        }
    }
}
