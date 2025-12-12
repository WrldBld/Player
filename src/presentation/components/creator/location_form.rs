//! Location Form - Create and edit locations

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::asset_gallery::AssetGallery;
use super::suggestion_button::{SuggestionButton, SuggestionContext, SuggestionType};
use crate::presentation::state::GameState;

/// Location data structure for API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationData {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub location_type: Option<String>,
    pub atmosphere: Option<String>,
    pub notable_features: Option<String>,
    pub hidden_secrets: Option<String>,
    pub parent_location_id: Option<String>,
}

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
pub fn LocationForm(location_id: String, on_close: EventHandler<()>) -> Element {
    let is_new = location_id.is_empty();
    let game_state = use_context::<GameState>();

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
        use_effect(move || {
            let loc_id = loc_id_for_effect.clone();
            let load_existing = !loc_id.is_empty();
            let world_id = game_state.world.read().as_ref().map(|w| w.world.id.clone());

            spawn(async move {
                if let Some(world_id) = world_id {
                    // Load parent locations list
                    if let Ok(parents) = fetch_locations(&world_id).await {
                        parent_locations.set(parents);
                    }

                    // Load location data if editing
                    if load_existing {
                        match fetch_location(&world_id, &loc_id).await {
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
                    onclick: move |_| {
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

                        spawn(async move {
                            let world_id = game_state.world.read().as_ref().map(|w| w.world.id.clone());

                            if let Some(world_id) = world_id {
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
                                };

                                match if is_new {
                                    save_location(&world_id, loc_data).await
                                } else {
                                    update_location(&world_id, &loc_id, loc_data).await
                                } {
                                    Ok(_) => {
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
                            } else {
                                error_message.set(Some("No world loaded".to_string()));
                                is_saving.set(false);
                            }
                        });
                    },
                    if *is_saving.read() { "Saving..." } else { if is_new { "Create" } else { "Save" } }
                }
            }
        }
    }
}

/// Reusable form field wrapper
#[component]
fn FormField(label: &'static str, required: bool, children: Element) -> Element {
    rsx! {
        div {
            class: "form-field",
            style: "display: flex; flex-direction: column; gap: 0.25rem;",

            label {
                style: "color: #9ca3af; font-size: 0.875rem;",
                "{label}"
                if required {
                    span { style: "color: #ef4444; margin-left: 0.25rem;", "*" }
                }
            }

            {children}
        }
    }
}

/// Fetch all locations from the API (for parent location dropdown)
async fn fetch_locations(world_id: &str) -> Result<Vec<LocationData>, String> {
    let base_url = get_engine_http_url();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, Response};

        let opts = RequestInit::new();
        opts.set_method("GET");

        let url = format!("{}/api/worlds/{}/locations", base_url, world_id);
        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;

        let window = web_sys::window().ok_or("No window object")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let resp: Response = resp_value
            .dyn_into()
            .map_err(|_| "Response cast failed")?;

        if !resp.ok() {
            return Err(format!("Server error: {}", resp.status()));
        }

        let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
            .await
            .map_err(|e| format!("JSON await failed: {:?}", e))?;

        let locations: Vec<LocationData> = serde_wasm_bindgen::from_value(json)
            .map_err(|e| format!("Deserialize error: {:?}", e))?;

        Ok(locations)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let url = format!("{}/api/worlds/{}/locations", base_url, world_id);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let locations: Vec<LocationData> = response
            .json()
            .await
            .map_err(|e| format!("Deserialize error: {}", e))?;

        Ok(locations)
    }
}

/// Fetch a single location from the API
async fn fetch_location(world_id: &str, location_id: &str) -> Result<LocationData, String> {
    let base_url = get_engine_http_url();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, Response};

        let opts = RequestInit::new();
        opts.set_method("GET");

        let url = format!("{}/api/worlds/{}/locations/{}", base_url, world_id, location_id);
        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;

        let window = web_sys::window().ok_or("No window object")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let resp: Response = resp_value
            .dyn_into()
            .map_err(|_| "Response cast failed")?;

        if !resp.ok() {
            return Err(format!("Server error: {}", resp.status()));
        }

        let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
            .await
            .map_err(|e| format!("JSON await failed: {:?}", e))?;

        let location: LocationData = serde_wasm_bindgen::from_value(json)
            .map_err(|e| format!("Deserialize error: {:?}", e))?;

        Ok(location)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let url = format!("{}/api/worlds/{}/locations/{}", base_url, world_id, location_id);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let location: LocationData = response
            .json()
            .await
            .map_err(|e| format!("Deserialize error: {}", e))?;

        Ok(location)
    }
}

/// Save a new location via the API
async fn save_location(world_id: &str, location: LocationData) -> Result<LocationData, String> {
    let base_url = get_engine_http_url();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, Response};

        let body = serde_json::to_string(&location)
            .map_err(|e| format!("JSON serialize error: {}", e))?;

        let mut opts = RequestInit::new();
        opts.method("POST");
        let body_js = wasm_bindgen::JsValue::from_str(&body);
        opts.body(Some(&body_js));

        let url = format!("{}/api/worlds/{}/locations", base_url, world_id);
        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;

        request.headers().set("Content-Type", "application/json")
            .map_err(|e| format!("Failed to set header: {:?}", e))?;

        let window = web_sys::window().ok_or("No window object")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let resp: Response = resp_value
            .dyn_into()
            .map_err(|_| "Response cast failed")?;

        if !resp.ok() {
            return Err(format!("Server error: {}", resp.status()));
        }

        let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
            .await
            .map_err(|e| format!("JSON await failed: {:?}", e))?;

        let saved: LocationData = serde_wasm_bindgen::from_value(json)
            .map_err(|e| format!("Deserialize error: {:?}", e))?;

        Ok(saved)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let url = format!("{}/api/worlds/{}/locations", base_url, world_id);

        let response = client
            .post(&url)
            .json(&location)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let saved: LocationData = response
            .json()
            .await
            .map_err(|e| format!("Deserialize error: {}", e))?;

        Ok(saved)
    }
}

/// Update an existing location via the API
async fn update_location(_world_id: &str, location_id: &str, location: LocationData) -> Result<LocationData, String> {
    let base_url = get_engine_http_url();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, Response};

        let body = serde_json::to_string(&location)
            .map_err(|e| format!("JSON serialize error: {}", e))?;

        let mut opts = RequestInit::new();
        opts.method("PUT");
        let body_js = wasm_bindgen::JsValue::from_str(&body);
        opts.body(Some(&body_js));

        let url = format!("{}/api/locations/{}", base_url, location_id);
        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;

        request.headers().set("Content-Type", "application/json")
            .map_err(|e| format!("Failed to set header: {:?}", e))?;

        let window = web_sys::window().ok_or("No window object")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let resp: Response = resp_value
            .dyn_into()
            .map_err(|_| "Response cast failed")?;

        if !resp.ok() {
            return Err(format!("Server error: {}", resp.status()));
        }

        let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
            .await
            .map_err(|e| format!("JSON await failed: {:?}", e))?;

        let saved: LocationData = serde_wasm_bindgen::from_value(json)
            .map_err(|e| format!("Deserialize error: {:?}", e))?;

        Ok(saved)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let url = format!("{}/api/locations/{}", base_url, location_id);

        let response = client
            .put(&url)
            .json(&location)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let saved: LocationData = response
            .json()
            .await
            .map_err(|e| format!("Deserialize error: {}", e))?;

        Ok(saved)
    }
}

/// Get the HTTP URL for the Engine API
fn get_engine_http_url() -> String {
    "http://localhost:3000".to_string()
}
