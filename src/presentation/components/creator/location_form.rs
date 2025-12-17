//! Location Form - Create and edit locations

use dioxus::prelude::*;

use super::asset_gallery::AssetGallery;
use super::suggestion_button::{SuggestionButton, SuggestionContext, SuggestionType};
use crate::application::services::LocationData;
use crate::presentation::components::common::FormField;
use crate::presentation::services::use_location_service;

/// Location types
const LOCATION_TYPES: &[&str] = &[
    "Interior",
    "Exterior",
    "Wilderness",
    "Urban",
    "Dungeon",
    "Castle",
    "Village",
    "City",
    "Forest",
    "Mountain",
    "Cave",
    "Temple",
];

/// Location form for creating/editing locations
#[component]
pub fn LocationForm(
    location_id: String,
    world_id: String,
    locations_signal: Signal<Vec<crate::application::services::location_service::LocationSummary>>,
    on_close: EventHandler<()>,
) -> Element {
    let is_new = location_id.is_empty();
    let loc_service = use_location_service();

    // Form state
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut location_type = use_signal(|| "Interior".to_string());
    let mut atmosphere = use_signal(|| String::new());
    let mut notable_features = use_signal(|| String::new());
    let mut hidden_secrets = use_signal(|| String::new());
    let mut parent_location_id: Signal<Option<String>> = use_signal(|| None);
    let mut parent_locations: Signal<Vec<LocationData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| !is_new);
    let mut is_saving = use_signal(|| false);
    let mut success_message: Signal<Option<String>> = use_signal(|| None);
    let mut error_message: Signal<Option<String>> = use_signal(|| None);

    // Load location data if editing existing location
    {
        let loc_id_for_effect = location_id.clone();
        let loc_svc = loc_service.clone();
        let world_id_for_effect = world_id.clone();
        use_effect(move || {
            let loc_id = loc_id_for_effect.clone();
            let load_existing = !loc_id.is_empty();
            let world_id_clone = world_id_for_effect.clone();
            let svc = loc_svc.clone();

            spawn(async move {
                    // Load parent locations list
                if let Ok(parents) = svc.list_locations(&world_id_clone).await {
                        // Convert LocationSummary to LocationData for the dropdown
                        let parent_data: Vec<LocationData> = parents.iter().map(|summary| {
                            LocationData {
                                id: Some(summary.id.clone()),
                                name: summary.name.clone(),
                                description: None,
                                location_type: summary.location_type.clone(),
                                atmosphere: None,
                                notable_features: None,
                                hidden_secrets: None,
                                parent_location_id: None,
                                backdrop_asset: None,
                                backdrop_regions: Vec::new(),
                            }
                        }).collect();
                        parent_locations.set(parent_data);
                    }

                    // Load location data if editing
                    if load_existing {
                        match svc.get_location(&world_id_clone, &loc_id).await {
                        Ok(loc_data) => {
                            name.set(loc_data.name);
                            description.set(loc_data.description.unwrap_or_default());
                            location_type.set(loc_data.location_type.unwrap_or_else(|| "Interior".to_string()));
                            atmosphere.set(loc_data.atmosphere.unwrap_or_default());
                            notable_features.set(loc_data.notable_features.unwrap_or_default());
                            hidden_secrets.set(loc_data.hidden_secrets.unwrap_or_default());
                            parent_location_id.set(loc_data.parent_location_id);
                            is_loading.set(false);
                        }
                        Err(e) => {
                            error_message.set(Some(format!("Failed to load location: {}", e)));
                            is_loading.set(false);
                        }
                    }
                } else {
                    is_loading.set(false);
            }
            });
        });
    }

    rsx! {
        div {
            class: "location-form",
            style: "display: flex; flex-direction: column; height: 100%; background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

            // Header
            div {
                class: "form-header",
                style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-bottom: 1px solid #374151;",

                h2 {
                    style: "color: white; margin: 0; font-size: 1.25rem;",
                    if is_new { "New Location" } else { "Edit Location" }
                }

                button {
                    onclick: move |_| on_close.call(()),
                    style: "padding: 0.25rem 0.5rem; background: transparent; color: #9ca3af; border: none; cursor: pointer; font-size: 1.25rem;",
                    "×"
                }
            }

            // Error/Success messages
            if let Some(msg) = error_message.read().as_ref() {
                div {
                    style: "padding: 0.75rem 1rem; background: rgba(239, 68, 68, 0.1); border-bottom: 1px solid rgba(239, 68, 68, 0.3); color: #ef4444; font-size: 0.875rem;",
                    "{msg}"
                }
            }
            if let Some(msg) = success_message.read().as_ref() {
                div {
                    style: "padding: 0.75rem 1rem; background: rgba(34, 197, 94, 0.1); border-bottom: 1px solid rgba(34, 197, 94, 0.3); color: #22c55e; font-size: 0.875rem;",
                    "{msg}"
                }
            }

            // Form content (scrollable)
            div {
                class: "form-content",
                style: "flex: 1; overflow-y: auto; padding: 1rem; display: flex; flex-direction: column; gap: 1rem;",

                if *is_loading.read() {
                    div {
                        style: "display: flex; align-items: center; justify-content: center; padding: 2rem; color: #6b7280;",
                        "Loading location data..."
                    }
                } else {

                // Name field with suggest button
                FormField {
                    label: "Name",
                    required: true,
                    children: rsx! {
                        div { style: "display: flex; gap: 0.5rem;",
                            input {
                                r#type: "text",
                                value: "{name}",
                                oninput: move |e| name.set(e.value()),
                                placeholder: "Enter location name...",
                                style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                            }
                            SuggestionButton {
                                suggestion_type: SuggestionType::LocationName,
                                context: SuggestionContext {
                                    entity_type: Some(location_type.read().clone()),
                                    ..Default::default()
                                },
                                on_select: move |value| name.set(value),
                            }
                        }
                    }
                }

                // Location type dropdown
                FormField {
                    label: "Type",
                    required: false,
                    children: rsx! {
                        select {
                            value: "{location_type}",
                            onchange: move |e| location_type.set(e.value()),
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",

                            for lt in LOCATION_TYPES {
                                option { value: "{lt}", "{lt}" }
                            }
                        }
                    }
                }

                // Description field
                FormField {
                    label: "Description",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            textarea {
                                value: "{description}",
                                oninput: move |e| description.set(e.value()),
                                placeholder: "What does this place look like? What stands out?",
                                style: "width: 100%; min-height: 80px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                            }
                            div { style: "display: flex; justify-content: flex-end;",
                                SuggestionButton {
                                    suggestion_type: SuggestionType::LocationDescription,
                                    context: SuggestionContext {
                                        entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                        entity_type: Some(location_type.read().clone()),
                                        ..Default::default()
                                    },
                                    on_select: move |value| description.set(value),
                                }
                            }
                        }
                    }
                }

                // Atmosphere field
                FormField {
                    label: "Atmosphere",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; gap: 0.5rem;",
                            input {
                                r#type: "text",
                                value: "{atmosphere}",
                                oninput: move |e| atmosphere.set(e.value()),
                                placeholder: "The mood and feeling of this place...",
                                style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                            }
                            SuggestionButton {
                                suggestion_type: SuggestionType::LocationAtmosphere,
                                context: SuggestionContext {
                                    entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                    entity_type: Some(location_type.read().clone()),
                                    additional_context: if description.read().is_empty() { None } else { Some(description.read().clone()) },
                                    ..Default::default()
                                },
                                on_select: move |value| atmosphere.set(value),
                            }
                        }
                    }
                }

                // Notable features field
                FormField {
                    label: "Notable Features",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            textarea {
                                value: "{notable_features}",
                                oninput: move |e| notable_features.set(e.value()),
                                placeholder: "Points of interest, interactable objects...",
                                style: "width: 100%; min-height: 60px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                            }
                            div { style: "display: flex; justify-content: flex-end;",
                                SuggestionButton {
                                    suggestion_type: SuggestionType::LocationFeatures,
                                    context: SuggestionContext {
                                        entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                        entity_type: Some(location_type.read().clone()),
                                        hints: if atmosphere.read().is_empty() { None } else { Some(atmosphere.read().clone()) },
                                        ..Default::default()
                                    },
                                    on_select: move |value| notable_features.set(value),
                                }
                            }
                        }
                    }
                }

                // Hidden secrets field
                FormField {
                    label: "Hidden Secrets",
                    required: false,
                    children: rsx! {
                        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            textarea {
                                value: "{hidden_secrets}",
                                oninput: move |e| hidden_secrets.set(e.value()),
                                placeholder: "Things players might discover with investigation...",
                                style: "width: 100%; min-height: 60px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                            }
                            div { style: "display: flex; justify-content: flex-end;",
                                SuggestionButton {
                                    suggestion_type: SuggestionType::LocationSecrets,
                                    context: SuggestionContext {
                                        entity_name: if name.read().is_empty() { None } else { Some(name.read().clone()) },
                                        entity_type: Some(location_type.read().clone()),
                                        additional_context: if notable_features.read().is_empty() { None } else { Some(notable_features.read().clone()) },
                                        ..Default::default()
                                    },
                                    on_select: move |value| hidden_secrets.set(value),
                                }
                            }
                        }
                    }
                }

                    // Parent location section
                    FormField {
                        label: "Parent Location",
                        required: false,
                        children: rsx! {
                            select {
                                value: parent_location_id.read().as_deref().unwrap_or(""),
                                onchange: move |e| {
                                    let val = e.value();
                                    parent_location_id.set(if val.is_empty() { None } else { Some(val) });
                                },
                                style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white;",

                                option { value: "", "None" }
                                for parent in parent_locations.read().iter() {
                                    // Don't allow selecting self as parent
                                    if parent.id.as_ref() != Some(&location_id) {
                                        option {
                                            value: "{parent.id.as_ref().unwrap_or(&String::new())}",
                                            "{parent.name}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Asset Gallery section
                    div {
                        class: "assets-section",
                        style: "margin-top: 1rem;",

                        h3 { style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin-bottom: 0.75rem;", "Assets" }

                        AssetGallery {
                            entity_type: "location".to_string(),
                            entity_id: location_id.clone(),
                        }
                    }
                }
            }

            // Footer with action buttons
            div {
                class: "form-footer",
                style: "display: flex; justify-content: flex-end; gap: 0.5rem; padding: 1rem; border-top: 1px solid #374151;",

                button {
                    onclick: move |_| on_close.call(()),
                    style: "padding: 0.5rem 1rem; background: transparent; color: #9ca3af; border: 1px solid #374151; border-radius: 0.25rem; cursor: pointer;",
                    disabled: *is_saving.read(),
                    "Cancel"
                }

                button {
                    style: format!(
                        "padding: 0.5rem 1rem; background: #22c55e; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-weight: 500; opacity: {};",
                        if *is_saving.read() { "0.6" } else { "1" }
                    ),
                    disabled: *is_saving.read(),
                    onclick: {
                        let loc_svc = loc_service.clone();
                        move |_| {
                            let loc_name = name.read().clone();
                            if loc_name.is_empty() {
                                error_message.set(Some("Location name is required".to_string()));
                                return;
                            }

                            error_message.set(None);
                            success_message.set(None);
                            is_saving.set(true);

                            let loc_id = location_id.clone();
                            let on_close = on_close.clone();
                            let svc = loc_svc.clone();
                            let world_id_clone = world_id.clone();

                            spawn(async move {
                                    let loc_data = LocationData {
                                        id: if is_new { None } else { Some(loc_id.clone()) },
                                        name: name.read().clone(),
                                        description: {
                                            let desc = description.read().clone();
                                            if desc.is_empty() { None } else { Some(desc) }
                                        },
                                        location_type: {
                                            let lt = location_type.read().clone();
                                            if lt.is_empty() { None } else { Some(lt) }
                                        },
                                        atmosphere: {
                                            let atm = atmosphere.read().clone();
                                            if atm.is_empty() { None } else { Some(atm) }
                                        },
                                        notable_features: {
                                            let nf = notable_features.read().clone();
                                            if nf.is_empty() { None } else { Some(nf) }
                                        },
                                        hidden_secrets: {
                                            let hs = hidden_secrets.read().clone();
                                            if hs.is_empty() { None } else { Some(hs) }
                                        },
                                        parent_location_id: parent_location_id.read().clone(),
                                        backdrop_asset: None,
                                        backdrop_regions: Vec::new(),
                                    };

                                    match if is_new {
                                        svc.create_location(&world_id_clone, &loc_data).await
                                    } else {
                                        svc.update_location(&loc_id, &loc_data).await
                                    } {
                                        Ok(saved_location) => {
                                            // Update the locations signal reactively
                                            if is_new {
                                                // Add new location to list
                                                let summary = crate::application::services::location_service::LocationSummary {
                                                    id: saved_location.id.clone().unwrap_or_default(),
                                                    name: saved_location.name.clone(),
                                                    location_type: saved_location.location_type.clone(),
                                                };
                                                locations_signal.write().push(summary);
                                            } else {
                                                // Update existing location in list
                                                if let Some(id) = &saved_location.id {
                                                    let mut locs = locations_signal.write();
                                                    if let Some(existing) = locs.iter_mut().find(|l| l.id == *id) {
                                                        existing.name = saved_location.name.clone();
                                                        existing.location_type = saved_location.location_type.clone();
                                                    }
                                                }
                                            }
                                            
                                            success_message.set(Some(if is_new {
                                                "Location created successfully".to_string()
                                            } else {
                                                "Location saved successfully".to_string()
                                            }));
                                            is_saving.set(false);
                                            // Close form - let the user see the success message
                                            on_close.call(());
                                        }
                                        Err(e) => {
                                            error_message.set(Some(format!("Save failed: {}", e)));
                                            is_saving.set(false);
                                        }
                                }
                            });
                        }
                    },
                    if *is_saving.read() { "Saving..." } else { if is_new { "Create" } else { "Save" } }
                }
            }
        }
    }
}
