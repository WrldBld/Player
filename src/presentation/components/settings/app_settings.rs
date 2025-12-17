//! Application Settings Panel - UI for managing Engine settings
//!
//! This component provides a form interface for viewing and editing
//! the Engine's application settings. Settings are grouped by category
//! for better organization.

use dioxus::prelude::*;
use crate::application::dto::AppSettings;
use crate::presentation::services::use_settings_service;

/// Application Settings Panel component
///
/// Loads settings from the Engine on mount, displays them in a categorized
/// form, and provides Save and Reset buttons for modification.
#[component]
pub fn AppSettingsPanel() -> Element {
    let settings_service = use_settings_service();

    // State for the form fields
    let mut settings = use_signal(|| AppSettings::default());
    let mut is_loading = use_signal(|| true);
    let mut is_saving = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut success_message = use_signal(|| None::<String>);

    // Clone service for closures
    let service_for_load = settings_service.clone();
    let service_for_save = settings_service.clone();
    let service_for_reset = settings_service.clone();

    // Load settings on mount
    use_effect(move || {
        let svc = service_for_load.clone();
        spawn(async move {
            is_loading.set(true);
            error.set(None);

            match svc.get().await {
                Ok(loaded_settings) => {
                    settings.set(loaded_settings);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load settings: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    });

    // Handler for saving settings
    let handle_save = move |_| {
        let svc = service_for_save.clone();
        let current_settings = settings.read().clone();
        spawn(async move {
            is_saving.set(true);
            error.set(None);
            success_message.set(None);

            match svc.update(&current_settings).await {
                Ok(updated_settings) => {
                    settings.set(updated_settings);
                    success_message.set(Some("Settings saved successfully!".to_string()));
                    is_saving.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to save settings: {}", e)));
                    is_saving.set(false);
                }
            }
        });
    };

    // Handler for resetting settings
    let handle_reset = move |_| {
        let svc = service_for_reset.clone();
        spawn(async move {
            is_saving.set(true);
            error.set(None);
            success_message.set(None);

            match svc.reset().await {
                Ok(reset_settings) => {
                    settings.set(reset_settings);
                    success_message.set(Some("Settings reset to defaults!".to_string()));
                    is_saving.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to reset settings: {}", e)));
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            class: "app-settings-panel h-full flex flex-col",

            // Header
            div {
                class: "flex justify-between items-center mb-4",

                h2 {
                    class: "text-white text-xl font-medium",
                    "Application Settings"
                }

                div {
                    class: "flex gap-2",

                    button {
                        class: "px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed",
                        onclick: handle_reset,
                        disabled: *is_loading.read() || *is_saving.read(),
                        "Reset to Defaults"
                    }

                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed",
                        onclick: handle_save,
                        disabled: *is_loading.read() || *is_saving.read(),
                        if *is_saving.read() { "Saving..." } else { "Save Changes" }
                    }
                }
            }

            // Success/Error messages
            if let Some(msg) = success_message.read().as_ref() {
                div {
                    class: "mb-4 p-3 bg-green-900 bg-opacity-30 text-green-400 rounded-md text-sm",
                    "{msg}"
                }
            }

            if let Some(err) = error.read().as_ref() {
                div {
                    class: "mb-4 p-3 bg-red-900 bg-opacity-30 text-red-400 rounded-md text-sm",
                    "{err}"
                }
            }

            // Loading state
            if *is_loading.read() {
                div {
                    class: "flex-1 flex items-center justify-center text-gray-400",
                    "Loading settings..."
                }
            } else {
                // Settings form
                div {
                    class: "flex-1 overflow-y-auto bg-gray-900 rounded-lg p-6 space-y-6",

                    // Session Settings
                    SettingsSection {
                        title: "Session Settings",
                        description: "Controls for conversation and session management",

                        NumberField {
                            label: "Max Conversation Turns",
                            description: "Maximum number of conversation turns before automatic summarization",
                            value: settings.read().max_conversation_turns,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.max_conversation_turns = val);
                                success_message.set(None);
                            }
                        }
                    }

                    // Circuit Breaker Settings
                    SettingsSection {
                        title: "Circuit Breaker",
                        description: "Fault tolerance configuration for external services",

                        NumberField {
                            label: "Failure Threshold",
                            description: "Number of consecutive failures before circuit breaker opens",
                            value: settings.read().circuit_breaker_failure_threshold as usize,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.circuit_breaker_failure_threshold = val as u32);
                                success_message.set(None);
                            }
                        }

                        NumberField {
                            label: "Open Duration (seconds)",
                            description: "Duration the circuit breaker remains open before attempting recovery",
                            value: settings.read().circuit_breaker_open_duration_secs as usize,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.circuit_breaker_open_duration_secs = val as u64);
                                success_message.set(None);
                            }
                        }

                        NumberField {
                            label: "Health Check Cache TTL (seconds)",
                            description: "Time-to-live for health check cache entries",
                            value: settings.read().health_check_cache_ttl_secs as usize,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.health_check_cache_ttl_secs = val as u64);
                                success_message.set(None);
                            }
                        }
                    }

                    // Animation Settings
                    SettingsSection {
                        title: "Typewriter Animation",
                        description: "Timing controls for text reveal animations",

                        NumberField {
                            label: "Sentence Delay (ms)",
                            description: "Delay between sentences in typewriter effect",
                            value: settings.read().typewriter_sentence_delay_ms as usize,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.typewriter_sentence_delay_ms = val as u64);
                                success_message.set(None);
                            }
                        }

                        NumberField {
                            label: "Pause Delay (ms)",
                            description: "Delay for pause punctuation in typewriter effect",
                            value: settings.read().typewriter_pause_delay_ms as usize,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.typewriter_pause_delay_ms = val as u64);
                                success_message.set(None);
                            }
                        }

                        NumberField {
                            label: "Character Delay (ms)",
                            description: "Delay between individual characters in typewriter effect",
                            value: settings.read().typewriter_char_delay_ms as usize,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.typewriter_char_delay_ms = val as u64);
                                success_message.set(None);
                            }
                        }
                    }

                    // Validation Settings
                    SettingsSection {
                        title: "Validation Limits",
                        description: "Maximum lengths and values for data validation",

                        NumberField {
                            label: "Max Name Length",
                            description: "Maximum allowed length for name fields",
                            value: settings.read().max_name_length,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.max_name_length = val);
                                success_message.set(None);
                            }
                        }

                        NumberField {
                            label: "Max Description Length",
                            description: "Maximum allowed length for description fields",
                            value: settings.read().max_description_length,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.max_description_length = val);
                                success_message.set(None);
                            }
                        }

                        NumberField {
                            label: "Default Max Stat Value",
                            description: "Default maximum value for character statistics",
                            value: settings.read().default_max_stat_value as usize,
                            onchange: move |val: usize| {
                                settings.with_mut(|s| s.default_max_stat_value = val as i32);
                                success_message.set(None);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Settings section component - groups related settings
#[derive(Props, Clone, PartialEq)]
struct SettingsSectionProps {
    title: &'static str,
    description: &'static str,
    children: Element,
}

#[component]
fn SettingsSection(props: SettingsSectionProps) -> Element {
    rsx! {
        div {
            class: "settings-section",

            div {
                class: "mb-3",

                h3 {
                    class: "text-white font-medium text-lg mb-1",
                    "{props.title}"
                }

                p {
                    class: "text-gray-400 text-sm",
                    "{props.description}"
                }
            }

            div {
                class: "space-y-4",
                {props.children}
            }
        }
    }
}

/// Number input field component
#[derive(Props, Clone, PartialEq)]
struct NumberFieldProps {
    label: &'static str,
    description: &'static str,
    value: usize,
    onchange: EventHandler<usize>,
}

#[component]
fn NumberField(props: NumberFieldProps) -> Element {
    // Format value as string for display
    let value_str = format!("{}", props.value);

    rsx! {
        div {
            class: "number-field",

            label {
                class: "block",

                div {
                    class: "flex justify-between items-baseline mb-1",

                    span {
                        class: "text-gray-300 text-sm font-medium",
                        "{props.label}"
                    }

                    span {
                        class: "text-gray-500 text-xs",
                        "{props.description}"
                    }
                }

                input {
                    r#type: "number",
                    class: "w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500",
                    value: "{value_str}",
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<usize>() {
                            props.onchange.call(val);
                        }
                    }
                }
            }
        }
    }
}
