//! Suggestion Button - LLM-powered content suggestions
//!
//! A reusable button component that fetches suggestions from the Engine
//! and displays them in a dropdown for selection.

use dioxus::prelude::*;

pub use crate::application::services::SuggestionContext;
use crate::presentation::services::use_suggestion_service;

/// Types of suggestions that can be requested
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestionType {
    CharacterName,
    CharacterDescription,
    CharacterWants,
    CharacterFears,
    CharacterBackstory,
    LocationName,
    LocationDescription,
    LocationAtmosphere,
    LocationFeatures,
    LocationSecrets,
}

/// Suggestion button component with dropdown
///
/// Fetches suggestions from the API when clicked and displays them
/// in a dropdown. When a suggestion is selected, it calls the on_select handler.
#[component]
pub fn SuggestionButton(
    suggestion_type: SuggestionType,
    context: SuggestionContext,
    on_select: EventHandler<String>,
) -> Element {
    let suggestion_service = use_suggestion_service();
    let mut loading = use_signal(|| false);
    let mut suggestions: Signal<Vec<String>> = use_signal(Vec::new);
    let mut show_dropdown = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Close dropdown when clicking outside
    let close_dropdown = move |_| {
        show_dropdown.set(false);
    };

    let fetch_suggestions = {
        let svc = suggestion_service.clone();
        move |_| {
            let context = context.clone();
            let suggestion_type = suggestion_type;
            let service = svc.clone();

            spawn(async move {
                loading.set(true);
                error.set(None);
                suggestions.set(Vec::new());

                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("Fetching suggestions for {:?}", suggestion_type).into());

                let result = match suggestion_type {
                    SuggestionType::CharacterName => service.suggest_character_name(&context).await,
                    SuggestionType::CharacterDescription => service.suggest_character_description(&context).await,
                    SuggestionType::CharacterWants => service.suggest_character_wants(&context).await,
                    SuggestionType::CharacterFears => service.suggest_character_fears(&context).await,
                    SuggestionType::CharacterBackstory => service.suggest_character_backstory(&context).await,
                    SuggestionType::LocationName => service.suggest_location_name(&context).await,
                    SuggestionType::LocationDescription => service.suggest_location_description(&context).await,
                    SuggestionType::LocationAtmosphere => service.suggest_location_atmosphere(&context).await,
                    SuggestionType::LocationFeatures => service.suggest_location_features(&context).await,
                    SuggestionType::LocationSecrets => service.suggest_location_secrets(&context).await,
                };

                match result {
                    Ok(results) => {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(&format!("Got {} suggestions: {:?}", results.len(), results).into());

                        if results.is_empty() {
                            error.set(Some("No suggestions returned".to_string()));
                        } else {
                            suggestions.set(results);
                            show_dropdown.set(true);
                        }
                    }
                    Err(e) => {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(&format!("Suggestion error: {}", e).into());

                        error.set(Some(e.to_string()));
                    }
                }

                loading.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "suggestion-button-container",
            style: "position: relative; display: inline-block;",

            // The button
            button {
                onclick: fetch_suggestions,
                disabled: *loading.read(),
                style: "padding: 0.5rem 0.75rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem; white-space: nowrap; transition: background 0.2s;",
                onmouseenter: move |_| {},  // Could add hover state
                if *loading.read() {
                    "..."
                } else {
                    "Suggest"
                }
            }

            // Error tooltip
            if let Some(err) = error.read().as_ref() {
                div {
                    style: "position: absolute; top: 100%; left: 0; margin-top: 0.25rem; padding: 0.5rem; background: #ef4444; color: white; border-radius: 0.25rem; font-size: 0.75rem; white-space: nowrap; z-index: 100;",
                    "{err}"
                }
            }

            // Dropdown with suggestions
            if *show_dropdown.read() && !suggestions.read().is_empty() {
                // Backdrop to catch outside clicks
                div {
                    onclick: close_dropdown,
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 99;",
                }

                // Dropdown menu
                div {
                    class: "suggestion-dropdown",
                    style: "position: absolute; top: 100%; right: 0; margin-top: 0.25rem; min-width: 200px; max-width: 400px; max-height: 300px; overflow-y: auto; background: #1f2937; border: 1px solid #374151; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.3); z-index: 100;",

                    for (i, suggestion) in suggestions.read().iter().enumerate() {
                        SuggestionItem {
                            key: "{i}",
                            text: suggestion.clone(),
                            on_click: {
                                let suggestion = suggestion.clone();
                                move |_| {
                                    on_select.call(suggestion.clone());
                                    show_dropdown.set(false);
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Individual suggestion item in the dropdown
#[component]
fn SuggestionItem(text: String, on_click: EventHandler<()>) -> Element {
    rsx! {
        div {
            onclick: move |_| on_click.call(()),
            style: "padding: 0.75rem 1rem; color: #e5e7eb; cursor: pointer; border-bottom: 1px solid #374151; transition: background 0.15s;",
            onmouseenter: move |evt| {
                // Would be nice to highlight on hover, but we can't easily change style here
                // In real app, would use a class and CSS hover state
            },
            "{text}"
        }
    }
}

/// Compact suggestion button for inline use (smaller, icon-style)
#[component]
pub fn SuggestIcon(
    suggestion_type: SuggestionType,
    context: SuggestionContext,
    on_select: EventHandler<String>,
) -> Element {
    // Wrapper that uses the full SuggestionButton but with compact styling
    rsx! {
        SuggestionButton {
            suggestion_type,
            context,
            on_select,
        }
    }
}
