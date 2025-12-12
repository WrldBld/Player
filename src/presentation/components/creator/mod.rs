//! Creator Mode components - World-building interface
//!
//! Components for the Creator Mode in the DM View, providing
//! entity creation, editing, asset generation, and LLM suggestions.

pub mod entity_browser;
pub mod character_form;
pub mod location_form;
pub mod asset_gallery;
pub mod generation_queue;
pub mod suggestion_button;
pub mod sheet_field_input;

pub use entity_browser::EntityBrowser;
pub use character_form::CharacterForm;
pub use location_form::LocationForm;
pub use asset_gallery::AssetGallery;
pub use generation_queue::GenerationQueuePanel;
pub use suggestion_button::{SuggestionButton, SuggestionContext, SuggestionType};
pub use sheet_field_input::{CharacterSheetForm, SheetFieldInput, SheetSectionInput};

use dioxus::prelude::*;
use crate::routes::Route;

/// Props for CreatorMode
#[derive(Props, Clone, PartialEq)]
pub struct CreatorModeProps {
    /// World ID from the route
    pub world_id: String,
    /// Currently selected tab from route (characters, locations, items, maps)
    #[props(default)]
    pub selected_tab: Option<String>,
}

/// The main Creator Mode container component
#[component]
pub fn CreatorMode(props: CreatorModeProps) -> Element {
    // Parse selected tab from URL, default to Characters
    let selected_entity_type = match props.selected_tab.as_deref() {
        Some("characters") | None => EntityTypeTab::Characters,
        Some("locations") => EntityTypeTab::Locations,
        Some("items") => EntityTypeTab::Items,
        Some("maps") => EntityTypeTab::Maps,
        _ => EntityTypeTab::Characters,
    };

    // Track the currently selected entity ID for editing
    let mut selected_entity_id: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div {
            class: "creator-mode",
            style: "height: 100%; display: grid; grid-template-columns: 280px 1fr; gap: 1rem; padding: 1rem;",

            // Left panel - Entity browser and generation queue
            div {
                class: "left-panel",
                style: "display: flex; flex-direction: column; gap: 1rem; overflow: hidden;",

                // Entity browser (tree view) - now uses router for tab changes
                EntityBrowser {
                    world_id: props.world_id.clone(),
                    selected_type: selected_entity_type,
                    selected_id: selected_entity_id.read().clone(),
                    on_select: move |id| selected_entity_id.set(Some(id)),
                }

                // Generation queue panel
                GenerationQueuePanel {}
            }

            // Right panel - Editor/Form area
            div {
                class: "editor-panel",
                style: "display: flex; flex-direction: column; gap: 1rem; overflow: hidden;",

                match (selected_entity_type, selected_entity_id.read().clone()) {
                    (EntityTypeTab::Characters, Some(id)) => rsx! {
                        CharacterForm {
                            character_id: id,
                            on_close: move |_| selected_entity_id.set(None),
                        }
                    },
                    (EntityTypeTab::Characters, None) => rsx! {
                        CharacterForm {
                            character_id: String::new(),
                            on_close: move |_| {},
                        }
                    },
                    (EntityTypeTab::Locations, Some(id)) => rsx! {
                        LocationForm {
                            location_id: id,
                            on_close: move |_| selected_entity_id.set(None),
                        }
                    },
                    (EntityTypeTab::Locations, None) => rsx! {
                        LocationForm {
                            location_id: String::new(),
                            on_close: move |_| {},
                        }
                    },
                    (EntityTypeTab::Items, _) => rsx! {
                        PlaceholderPanel { title: "Item Editor", message: "Item editing coming soon" }
                    },
                    (EntityTypeTab::Maps, _) => rsx! {
                        PlaceholderPanel { title: "Map Editor", message: "Map editing coming soon" }
                    },
                }
            }
        }
    }
}

/// Entity type tabs for the browser
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityTypeTab {
    #[default]
    Characters,
    Locations,
    Items,
    Maps,
}

impl EntityTypeTab {
    pub fn label(&self) -> &'static str {
        match self {
            EntityTypeTab::Characters => "Characters",
            EntityTypeTab::Locations => "Locations",
            EntityTypeTab::Items => "Items",
            EntityTypeTab::Maps => "Maps",
        }
    }
}

/// Placeholder panel for unimplemented features
#[component]
fn PlaceholderPanel(title: &'static str, message: &'static str) -> Element {
    rsx! {
        div {
            class: "placeholder-panel",
            style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; background: #1a1a2e; border-radius: 0.5rem; color: #6b7280;",

            h2 { style: "color: #9ca3af; margin-bottom: 0.5rem;", "{title}" }
            p { "{message}" }
        }
    }
}
