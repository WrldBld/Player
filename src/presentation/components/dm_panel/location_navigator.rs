//! Location Navigator - DM tool to preview any location

use dioxus::prelude::*;

use crate::application::services::LocationService;
use crate::presentation::services::use_location_service;

/// Props for LocationNavigator
#[derive(Props, Clone, PartialEq)]
pub struct LocationNavigatorProps {
    pub world_id: String,
    pub on_preview: EventHandler<String>,
}

/// Location Navigator component
#[component]
pub fn LocationNavigator(props: LocationNavigatorProps) -> Element {
    let location_service = use_location_service();
    let mut locations: Signal<Vec<crate::application::services::LocationSummary>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Load locations on mount
    {
        let world_id = props.world_id.clone();
        let loc_svc = location_service.clone();
        use_effect(move || {
            let wid = world_id.clone();
            let svc = loc_svc.clone();
            loading.set(true);
            spawn(async move {
                match svc.list_locations(&wid).await {
                    Ok(loc_list) => {
                        locations.set(loc_list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load locations: {}", e)));
                        loading.set(false);
                    }
                }
            });
        });
    }

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 1rem; padding: 1rem; background: #1a1a2e; border-radius: 0.5rem;",
            
            h3 {
                style: "margin: 0; color: white; font-size: 1.125rem;",
                "Location Navigator"
            }

            if let Some(err) = error.read().as_ref() {
                div {
                    style: "padding: 0.75rem; background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; color: #ef4444; font-size: 0.875rem;",
                    "{err}"
                }
            }

            if *loading.read() {
                div {
                    style: "padding: 2rem; text-align: center; color: #9ca3af;",
                    "Loading locations..."
                }
            } else if locations.read().is_empty() {
                div {
                    style: "padding: 2rem; text-align: center; color: #9ca3af;",
                    "No locations in this world"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.75rem; max-height: 400px; overflow-y: auto;",
                    for location in locations.read().iter() {
                        LocationCard {
                            location: location.clone(),
                            on_preview: move |_| props.on_preview.call(location.id.clone()),
                        }
                    }
                }
            }
        }
    }
}

/// Location Card component
#[derive(Props, Clone, PartialEq)]
struct LocationCardProps {
    location: crate::application::services::LocationSummary,
    on_preview: EventHandler<()>,
}

#[component]
fn LocationCard(props: LocationCardProps) -> Element {
    rsx! {
        div {
            style: "padding: 1rem; background: #0f0f23; border-radius: 0.5rem; border: 1px solid #374151; display: flex; justify-content: space-between; align-items: center;",
            
            div {
                h4 {
                    style: "margin: 0 0 0.25rem 0; color: white; font-size: 1rem;",
                    "{props.location.name}"
                }
                if let Some(loc_type) = props.location.location_type.as_ref() {
                    div {
                        style: "color: #9ca3af; font-size: 0.75rem;",
                        "{loc_type}"
                    }
                }
            }
            button {
                onclick: move |_| props.on_preview.call(()),
                style: "padding: 0.5rem 1rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                "Preview"
            }
        }
    }
}

