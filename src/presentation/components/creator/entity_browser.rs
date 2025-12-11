//! Entity Browser - Tree view of world entities

use dioxus::prelude::*;

use super::EntityTypeTab;

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

/// Character list with mock data (will be replaced with API calls)
#[component]
fn CharacterList(selected_id: Option<String>, on_select: EventHandler<String>) -> Element {
    // TODO: Replace with actual API call
    let characters = vec![
        ("char-1", "Bartender Jasper", "NPC"),
        ("char-2", "Hooded Figure", "NPC"),
        ("char-3", "Sir Aldric", "Player"),
    ];

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 0.25rem;",

            for (id, name, role) in characters {
                EntityListItem {
                    id: id.to_string(),
                    name: name.to_string(),
                    subtitle: role.to_string(),
                    selected: selected_id.as_deref() == Some(id),
                    on_click: move |_| on_select.call(id.to_string()),
                }
            }
        }
    }
}

/// Location list with mock data (will be replaced with API calls)
#[component]
fn LocationList(selected_id: Option<String>, on_select: EventHandler<String>) -> Element {
    // TODO: Replace with actual API call
    let locations = vec![
        ("loc-1", "Dragon's Rest Inn", "Interior"),
        ("loc-2", "Town Square", "Exterior"),
        ("loc-3", "Northern Pass", "Wilderness"),
    ];

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 0.25rem;",

            for (id, name, loc_type) in locations {
                EntityListItem {
                    id: id.to_string(),
                    name: name.to_string(),
                    subtitle: loc_type.to_string(),
                    selected: selected_id.as_deref() == Some(id),
                    on_click: move |_| on_select.call(id.to_string()),
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
