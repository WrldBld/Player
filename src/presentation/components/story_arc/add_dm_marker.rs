//! Add DM Marker Modal - Create DM notes/markers on the timeline

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Props, Clone, PartialEq)]
pub struct AddDmMarkerModalProps {
    pub world_id: String,
    #[props(default)]
    pub session_id: Option<String>,
    pub on_close: EventHandler<()>,
    pub on_created: EventHandler<()>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateDmMarkerRequest {
    title: String,
    note: String,
    importance: String,
    marker_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

#[component]
pub fn AddDmMarkerModal(props: AddDmMarkerModalProps) -> Element {
    let mut title = use_signal(|| String::new());
    let mut note = use_signal(|| String::new());
    let mut importance = use_signal(|| "normal".to_string());
    let mut marker_type = use_signal(|| "note".to_string());
    let mut tags_input = use_signal(|| String::new());
    let mut is_saving = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let importance_options = vec![
        ("minor", "Minor", "Low priority note"),
        ("normal", "Normal", "Standard note"),
        ("major", "Major", "Important plot point"),
        ("critical", "Critical", "Critical story moment"),
    ];

    let marker_type_options = vec![
        ("note", "Note", "📝"),
        ("plot_point", "Plot Point", "⭐"),
        ("foreshadowing", "Foreshadowing", "🔮"),
        ("session_break", "Session Break", "⏸️"),
        ("chapter_break", "Chapter Break", "📖"),
        ("recap", "Recap", "📋"),
    ];

    let can_save = !title.read().trim().is_empty();

    rsx! {
        div {
            class: "modal-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_close.call(()),

            div {
                class: "modal-content",
                style: "background: #1a1a2e; border-radius: 0.75rem; padding: 1.5rem; max-width: 500px; width: 90%;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;",

                    h2 { style: "color: white; margin: 0; font-size: 1.25rem;", "📝 Add DM Marker" }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #9ca3af; font-size: 1.5rem; cursor: pointer;",
                        "×"
                    }
                }

                // Form
                div {
                    style: "display: flex; flex-direction: column; gap: 1rem;",

                    // Title
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.375rem;",
                            "Title *"
                        }
                        input {
                            r#type: "text",
                            placeholder: "Enter marker title...",
                            value: "{title}",
                            oninput: move |e| title.set(e.value()),
                            style: "width: 100%; padding: 0.625rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.9375rem; box-sizing: border-box;",
                        }
                    }

                    // Note
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.375rem;",
                            "Note"
                        }
                        textarea {
                            placeholder: "Add details, context, or reminders...",
                            value: "{note}",
                            oninput: move |e| note.set(e.value()),
                            style: "width: 100%; min-height: 100px; padding: 0.625rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.9375rem; resize: vertical; box-sizing: border-box;",
                        }
                    }

                    // Marker Type
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.375rem;",
                            "Marker Type"
                        }
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",

                            for (value, label, icon) in marker_type_options.iter() {
                                {
                                    let is_selected = *marker_type.read() == *value;
                                    let value = *value;
                                    rsx! {
                                        button {
                                            onclick: move |_| marker_type.set(value.to_string()),
                                            style: format!(
                                                "padding: 0.5rem 0.75rem; border-radius: 0.375rem; cursor: pointer; display: flex; align-items: center; gap: 0.375rem; font-size: 0.8125rem; {}",
                                                if is_selected {
                                                    "background: #3b82f6; color: white; border: 1px solid #3b82f6;"
                                                } else {
                                                    "background: #0f0f23; color: #9ca3af; border: 1px solid #374151;"
                                                }
                                            ),
                                            span { "{icon}" }
                                            span { "{label}" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Importance
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.375rem;",
                            "Importance"
                        }
                        div {
                            style: "display: flex; gap: 0.5rem;",

                            for (value, label, description) in importance_options.iter() {
                                {
                                    let is_selected = *importance.read() == *value;
                                    let value = *value;
                                    let color = get_importance_color(value);
                                    rsx! {
                                        button {
                                            onclick: move |_| importance.set(value.to_string()),
                                            title: "{description}",
                                            style: format!(
                                                "flex: 1; padding: 0.5rem; border-radius: 0.375rem; cursor: pointer; font-size: 0.8125rem; {}",
                                                if is_selected {
                                                    format!("background: {}; color: white; border: 1px solid {};", color, color)
                                                } else {
                                                    "background: #0f0f23; color: #9ca3af; border: 1px solid #374151;".to_string()
                                                }
                                            ),
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Tags
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.375rem;",
                            "Tags (comma separated)"
                        }
                        input {
                            r#type: "text",
                            placeholder: "e.g., act1, villain, mystery",
                            value: "{tags_input}",
                            oninput: move |e| tags_input.set(e.value()),
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem; box-sizing: border-box;",
                        }
                    }

                    // Error display
                    if let Some(err) = error.read().as_ref() {
                        div {
                            style: "background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.375rem; padding: 0.75rem; color: #ef4444; font-size: 0.875rem;",
                            "{err}"
                        }
                    }

                    // Buttons
                    div {
                        style: "display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem;",

                        button {
                            onclick: move |_| props.on_close.call(()),
                            style: "padding: 0.625rem 1.25rem; background: #374151; color: white; border: none; border-radius: 0.375rem; cursor: pointer;",
                            "Cancel"
                        }

                        button {
                            onclick: {
                                let world_id = props.world_id.clone();
                                let session_id = props.session_id.clone();
                                move |_| {
                                    if !can_save { return; }

                                    let title_val = title.read().trim().to_string();
                                    let note_val = note.read().trim().to_string();
                                    let importance_val = importance.read().clone();
                                    let marker_type_val = marker_type.read().clone();
                                    let tags: Vec<String> = tags_input.read()
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();

                                    let world_id = world_id.clone();
                                    let session_id = session_id.clone();
                                    spawn(async move {
                                        is_saving.set(true);
                                        error.set(None);

                                        let request = CreateDmMarkerRequest {
                                            title: title_val,
                                            note: note_val,
                                            importance: importance_val,
                                            marker_type: marker_type_val,
                                            tags,
                                        };

                                        match create_dm_marker(&world_id, session_id.as_deref(), &request).await {
                                            Ok(_) => {
                                                props.on_created.call(());
                                            }
                                            Err(e) => {
                                                error.set(Some(e));
                                            }
                                        }

                                        is_saving.set(false);
                                    });
                                }
                            },
                            disabled: !can_save || *is_saving.read(),
                            style: format!(
                                "padding: 0.625rem 1.25rem; color: white; border: none; border-radius: 0.375rem; cursor: {}; {}",
                                if can_save && !*is_saving.read() { "pointer" } else { "not-allowed" },
                                if can_save { "background: #8b5cf6;" } else { "background: #4b5563; opacity: 0.5;" }
                            ),
                            if *is_saving.read() { "Saving..." } else { "Create Marker" }
                        }
                    }
                }
            }
        }
    }
}

fn get_importance_color(importance: &str) -> &'static str {
    match importance {
        "critical" => "#ef4444",
        "major" => "#f59e0b",
        "normal" => "#3b82f6",
        "minor" => "#6b7280",
        _ => "#6b7280",
    }
}

async fn create_dm_marker(
    world_id: &str,
    session_id: Option<&str>,
    request: &CreateDmMarkerRequest,
) -> Result<(), String> {
    let url = if let Some(sid) = session_id {
        format!("/api/sessions/{}/story-events", sid)
    } else {
        format!("/api/worlds/{}/story-events", world_id)
    };

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(request).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            Ok(())
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let full_url = format!("http://localhost:3000{}", url);
        let response = client
            .post(&full_url)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }
}
