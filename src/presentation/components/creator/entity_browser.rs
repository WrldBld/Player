//! Entity Browser - Tree view of world entities

use dioxus::prelude::*;

use super::EntityTypeTab;
use crate::application::services::character_service::CharacterSummary;
use crate::application::services::location_service::LocationSummary;
use crate::routes::Route;

/// Props for the EntityBrowser component
#[component]
pub fn EntityBrowser(
    world_id: String,
    selected_type: EntityTypeTab,
    selected_id: Option<String>,
    characters: Signal<Vec<CharacterSummary>>,
    locations: Signal<Vec<LocationSummary>>,
    characters_loading: Signal<bool>,
    locations_loading: Signal<bool>,
    characters_error: Signal<Option<String>>,
    locations_error: Signal<Option<String>>,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "entity-browser",
            style: "flex: 1; display: flex; flex-direction: column; background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

            // Tab buttons for entity types - uses router Links
            div {
                class: "browser-tabs",
                style: "display: flex; border-bottom: 1px solid #374151;",

                EntityTypeTabLink {
                    world_id: world_id.clone(),
                    tab: EntityTypeTab::Characters,
                    active: selected_type == EntityTypeTab::Characters,
                }
                EntityTypeTabLink {
                    world_id: world_id.clone(),
                    tab: EntityTypeTab::Locations,
                    active: selected_type == EntityTypeTab::Locations,
                }
                EntityTypeTabLink {
                    world_id: world_id.clone(),
                    tab: EntityTypeTab::Items,
                    active: selected_type == EntityTypeTab::Items,
                }
                EntityTypeTabLink {
                    world_id: world_id.clone(),
                    tab: EntityTypeTab::Maps,
                    active: selected_type == EntityTypeTab::Maps,
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
                            characters: characters,
                            selected_id: selected_id.clone(),
                            loading: characters_loading,
                            error: characters_error,
                            on_select: move |id| on_select.call(id),
                        }
                    },
                    EntityTypeTab::Locations => rsx! {
                        LocationList {
                            locations: locations,
                            selected_id: selected_id.clone(),
                            loading: locations_loading,
                            error: locations_error,
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

/// Tab link that uses router navigation
#[component]
fn EntityTypeTabLink(world_id: String, tab: EntityTypeTab, active: bool) -> Element {
    let bg = if active { "#3b82f6" } else { "transparent" };
    let short_label = match tab {
        EntityTypeTab::Characters => "Char",
        EntityTypeTab::Locations => "Loc",
        EntityTypeTab::Items => "Item",
        EntityTypeTab::Maps => "Map",
    };
    let subtab = match tab {
        EntityTypeTab::Characters => "characters",
        EntityTypeTab::Locations => "locations",
        EntityTypeTab::Items => "items",
        EntityTypeTab::Maps => "maps",
    };

    rsx! {
        Link {
            to: Route::DMCreatorSubTabRoute {
                world_id: world_id,
                subtab: subtab.to_string(),
            },
            style: format!(
                "flex: 1; padding: 0.5rem 0.25rem; background: {}; color: white; border: none; cursor: pointer; font-size: 0.75rem; text-decoration: none; text-align: center;",
                bg
            ),
            "{short_label}"
        }
    }
}

/// Character list - renders from reactive signal
#[component]
fn CharacterList(
    characters: Signal<Vec<CharacterSummary>>,
    selected_id: Option<String>,
    loading: Signal<bool>,
    error: Signal<Option<String>>,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        if *loading.read() {
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

/// Location list - renders from reactive signal
#[component]
fn LocationList(
    locations: Signal<Vec<LocationSummary>>,
    selected_id: Option<String>,
    loading: Signal<bool>,
    error: Signal<Option<String>>,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        if *loading.read() {
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
