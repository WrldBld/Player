//! Directorial notes component
//!
//! Provides an editable textarea for DM's live notes and scene observations.

use dioxus::prelude::*;

/// Props for the DirectorialNotes component
#[derive(Props, Clone, PartialEq)]
pub struct DirectorialNotesProps {
    /// Current notes text
    pub notes: String,
    /// Handler called when notes are updated
    pub on_change: EventHandler<String>,
    /// Optional placeholder text
    #[props(default = "Scene notes, observations, plot hooks...")]
    pub placeholder: &'static str,
}

/// DirectorialNotes component - Editable textarea for DM notes
///
/// Allows DMs to write and edit live notes during gameplay.
/// Useful for tracking scene observations, player reactions, and plot developments.
#[component]
pub fn DirectorialNotes(props: DirectorialNotesProps) -> Element {
    rsx! {
        div {
            class: "directorial-notes",
            style: "display: flex; flex-direction: column; height: 100%;",

            // Header
            div {
                style: "display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.5rem;",

                h3 {
                    style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin: 0;",
                    "Directorial Notes"
                }

                // Word count indicator
                span {
                    style: "color: #6b7280; font-size: 0.75rem; margin-left: auto;",
                    "{props.notes.len()} characters"
                }
            }

            // Textarea
            textarea {
                class: "notes-textarea",
                value: "{props.notes}",
                placeholder: props.placeholder,
                oninput: move |e| props.on_change.call(e.value()),
                style: "flex: 1; width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace; font-size: 0.875rem; line-height: 1.5; resize: none; box-sizing: border-box; transition: border-color 0.2s;",
            }

            // Footer info
            div {
                style: "color: #6b7280; font-size: 0.75rem; margin-top: 0.5rem;",
                "Last modified: just now"
            }
        }
    }
}
