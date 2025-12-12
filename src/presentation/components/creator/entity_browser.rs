//! Entity Browser - Tree view of world entities

use dioxus::prelude::*;

use super::EntityTypeTab;
use crate::presentation::state::GameState;

/// Entity data structures
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct EntityCharacter {
    pub id: String,
    pub name: String,
    pub archetype: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct EntityLocation {
    pub id: String,
    pub name: String,
    pub location_type: Option<String>,
}

/// Props for the EntityBrowser component
#[component]
pub fn EntityBrowser(
    selected_type: EntityTypeTab,
    on_type_change: EventHandler<EntityTypeTab>,
    selected_id: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "entity-browser",
            style: "flex: 1; display: flex; flex-direction: column; background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

            // Tab buttons for entity types
            div {
                class: "browser-tabs",
                style: "display: flex; border-bottom: 1px solid #374151;",

                EntityTypeTabButton {
                    tab: EntityTypeTab::Characters,
                    active: selected_type == EntityTypeTab::Characters,
                    on_click: move |_| on_type_change.call(EntityTypeTab::Characters),
                }
                EntityTypeTabButton {
                    tab: EntityTypeTab::Locations,
                    active: selected_type == EntityTypeTab::Locations,
                    on_click: move |_| on_type_change.call(EntityTypeTab::Locations),
                }
                EntityTypeTabButton {
                    tab: EntityTypeTab::Items,
                    active: selected_type == EntityTypeTab::Items,
                    on_click: move |_| on_type_change.call(EntityTypeTab::Items),
                }
                EntityTypeTabButton {
                    tab: EntityTypeTab::Maps,
                    active: selected_type == EntityTypeTab::Maps,
                    on_click: move |_| on_type_change.call(EntityTypeTab::Maps),
                }
            }

            // Search/filter bar
            div {
                class: "browser-search",
                style: "padding: 0.5rem;",

                input {
                    r#type: "text",
                    placeholder: "Search...",
                    style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                }
            }

            // Entity list
            div {
                class: "browser-list",
                style: "flex: 1; overflow-y: auto; padding: 0.5rem;",

                match selected_type {
                    EntityTypeTab::Characters => rsx! {
                        CharacterList {
                            selected_id: selected_id.clone(),
                            on_select: move |id| on_select.call(id),
                        }
                    },
                    EntityTypeTab::Locations => rsx! {
                        LocationList {
                            selected_id: selected_id.clone(),
                            on_select: move |id| on_select.call(id),
                        }
                    },
                    EntityTypeTab::Items => rsx! {
                        div { style: "color: #6b7280; text-align: center; padding: 1rem;",
                            "No items yet"
                        }
                    },
                    EntityTypeTab::Maps => rsx! {
                        div { style: "color: #6b7280; text-align: center; padding: 1rem;",
                            "No maps yet"
                        }
                    },
                }
            }

            // New entity button
            div {
                class: "browser-actions",
                style: "padding: 0.5rem; border-top: 1px solid #374151;",

                button {
                    style: "width: 100%; padding: 0.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-weight: 500;",
                    onclick: move |_| on_select.call(String::new()),
                    "+ New {selected_type.label()}"
                }
            }
        }
    }
}

#[component]
fn EntityTypeTabButton(tab: EntityTypeTab, active: bool, on_click: EventHandler<()>) -> Element {
    let bg = if active { "#3b82f6" } else { "transparent" };
    let short_label = match tab {
        EntityTypeTab::Characters => "Char",
        EntityTypeTab::Locations => "Loc",
        EntityTypeTab::Items => "Item",
        EntityTypeTab::Maps => "Map",
    };

    rsx! {
        button {
            onclick: move |_| on_click.call(()),
            style: format!(
                "flex: 1; padding: 0.5rem 0.25rem; background: {}; color: white; border: none; cursor: pointer; font-size: 0.75rem;",
                bg
            ),
            "{short_label}"
        }
    }
}

/// Character list with API data
#[component]
fn CharacterList(selected_id: Option<String>, on_select: EventHandler<String>) -> Element {
    let game_state = use_context::<GameState>();

    // Track loading and error states
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut characters: Signal<Vec<EntityCharacter>> = use_signal(Vec::new);

    // Fetch characters on mount
    use_effect(move || {
        spawn(async move {
            let world_id = game_state.world.read().as_ref().map(|w| w.world.id.clone());

            if let Some(world_id) = world_id {
                match fetch_characters(&world_id).await {
                    Ok(fetched) => {
                        characters.set(fetched);
                        is_loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        is_loading.set(false);
                    }
                }
            } else {
                error.set(Some("No world loaded".to_string()));
                is_loading.set(false);
            }
        });
    });

    rsx! {
        if *is_loading.read() {
            div {
                style: "display: flex; align-items: center; justify-content: center; padding: 2rem; color: #6b7280;",
                "Loading characters..."
            }
        } else if let Some(err) = error.read().as_ref() {
            div {
                style: "padding: 1rem; background: rgba(239, 68, 68, 0.1); border-radius: 0.5rem; color: #ef4444; font-size: 0.875rem;",
                "Error: {err}"
            }
        } else {
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",

                for character in characters.read().iter() {
                    EntityListItem {
                        id: character.id.clone(),
                        name: character.name.clone(),
                        subtitle: character.archetype.clone().unwrap_or_else(|| "Unknown".to_string()),
                        selected: selected_id.as_deref() == Some(&character.id),
                        on_click: {
                            let char_id = character.id.clone();
                            move |_| on_select.call(char_id.clone())
                        },
                    }
                }

                if characters.read().is_empty() {
                    div {
                        style: "color: #6b7280; text-align: center; padding: 1rem; font-size: 0.875rem;",
                        "No characters yet"
                    }
                }
            }
        }
    }
}

/// Location list with API data
#[component]
fn LocationList(selected_id: Option<String>, on_select: EventHandler<String>) -> Element {
    let game_state = use_context::<GameState>();

    // Track loading and error states
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut locations: Signal<Vec<EntityLocation>> = use_signal(Vec::new);

    // Fetch locations on mount
    use_effect(move || {
        spawn(async move {
            let world_id = game_state.world.read().as_ref().map(|w| w.world.id.clone());

            if let Some(world_id) = world_id {
                match fetch_locations(&world_id).await {
                    Ok(fetched) => {
                        locations.set(fetched);
                        is_loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        is_loading.set(false);
                    }
                }
            } else {
                error.set(Some("No world loaded".to_string()));
                is_loading.set(false);
            }
        });
    });

    rsx! {
        if *is_loading.read() {
            div {
                style: "display: flex; align-items: center; justify-content: center; padding: 2rem; color: #6b7280;",
                "Loading locations..."
            }
        } else if let Some(err) = error.read().as_ref() {
            div {
                style: "padding: 1rem; background: rgba(239, 68, 68, 0.1); border-radius: 0.5rem; color: #ef4444; font-size: 0.875rem;",
                "Error: {err}"
            }
        } else {
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",

                for location in locations.read().iter() {
                    EntityListItem {
                        id: location.id.clone(),
                        name: location.name.clone(),
                        subtitle: location.location_type.clone().unwrap_or_else(|| "Unknown".to_string()),
                        selected: selected_id.as_deref() == Some(&location.id),
                        on_click: {
                            let loc_id = location.id.clone();
                            move |_| on_select.call(loc_id.clone())
                        },
                    }
                }

                if locations.read().is_empty() {
                    div {
                        style: "color: #6b7280; text-align: center; padding: 1rem; font-size: 0.875rem;",
                        "No locations yet"
                    }
                }
            }
        }
    }
}

/// Reusable entity list item
#[component]
fn EntityListItem(
    id: String,
    name: String,
    subtitle: String,
    selected: bool,
    on_click: EventHandler<()>,
) -> Element {
    let bg = if selected { "rgba(59, 130, 246, 0.2)" } else { "transparent" };
    let border = if selected { "1px solid #3b82f6" } else { "1px solid transparent" };

    rsx! {
        div {
            onclick: move |_| on_click.call(()),
            style: format!(
                "padding: 0.5rem; background: {}; border: {}; border-radius: 0.25rem; cursor: pointer;",
                bg, border
            ),

            div { style: "color: white; font-size: 0.875rem;", "{name}" }
            div { style: "color: #6b7280; font-size: 0.75rem;", "{subtitle}" }
        }
    }
}

/// Fetch characters from the Engine API
async fn fetch_characters(world_id: &str) -> Result<Vec<EntityCharacter>, String> {
    let base_url = get_engine_http_url();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, Response};

        let opts = RequestInit::new();
        opts.set_method("GET");

        let url = format!("{}/api/worlds/{}/characters", base_url, world_id);
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

        let characters: Vec<EntityCharacter> = serde_wasm_bindgen::from_value(json)
            .map_err(|e| format!("Deserialize error: {:?}", e))?;

        Ok(characters)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let url = format!("{}/api/worlds/{}/characters", base_url, world_id);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let characters: Vec<EntityCharacter> = response
            .json()
            .await
            .map_err(|e| format!("Deserialize error: {}", e))?;

        Ok(characters)
    }
}

/// Fetch locations from the Engine API
async fn fetch_locations(world_id: &str) -> Result<Vec<EntityLocation>, String> {
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

        let locations: Vec<EntityLocation> = serde_wasm_bindgen::from_value(json)
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

        let locations: Vec<EntityLocation> = response
            .json()
            .await
            .map_err(|e| format!("Deserialize error: {}", e))?;

        Ok(locations)
    }
}

/// Get the HTTP URL for the Engine API
fn get_engine_http_url() -> String {
    // Default to localhost:3000
    "http://localhost:3000".to_string()
}
