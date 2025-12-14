//! Narrative Event Library - Browse and manage future narrative events

use dioxus::prelude::*;

use crate::application::dto::{CreateNarrativeEventRequest, NarrativeEventData};
use crate::presentation::components::story_arc::narrative_event_card::NarrativeEventCard;
use crate::presentation::services::use_narrative_event_service;

#[derive(Props, Clone, PartialEq)]
pub struct NarrativeEventLibraryProps {
    pub world_id: String,
}

#[component]
pub fn NarrativeEventLibrary(props: NarrativeEventLibraryProps) -> Element {
    let mut events: Signal<Vec<NarrativeEventData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut search_text = use_signal(|| String::new());
    let mut filter_status = use_signal(|| "all".to_string());
    let mut show_favorites_only = use_signal(|| false);
    let mut selected_event: Signal<Option<NarrativeEventData>> = use_signal(|| None);
    let mut show_create_form = use_signal(|| false);

    // Get narrative event service
    let narrative_event_service = use_narrative_event_service();
    let narrative_event_service_for_effect = narrative_event_service.clone();

    // Load events
    let world_id = props.world_id.clone();
    use_effect(move || {
        let world_id = world_id.clone();
        let service = narrative_event_service_for_effect.clone();
        spawn(async move {
            is_loading.set(true);
            error.set(None);

            match service.list_narrative_events(&world_id).await {
                Ok(loaded) => events.set(loaded),
                Err(e) => error.set(Some(format!("Failed to load narrative events: {}", e))),
            }
            is_loading.set(false);
        });
    });

    // Filter events
    let filtered_events = {
        let search = search_text.read().to_lowercase();
        let status = filter_status.read().clone();
        let favorites_only = *show_favorites_only.read();
        let all_events = events.read().clone();

        all_events.into_iter().filter(|event| {
            // Filter by favorites
            if favorites_only && !event.is_favorite {
                return false;
            }

            // Filter by status
            match status.as_str() {
                "active" => {
                    if !event.is_active {
                        return false;
                    }
                }
                "triggered" => {
                    if !event.is_triggered {
                        return false;
                    }
                }
                "pending" => {
                    if event.is_triggered || !event.is_active {
                        return false;
                    }
                }
                _ => {}
            }

            // Filter by search
            if !search.is_empty() {
                let matches_name = event.name.to_lowercase().contains(&search);
                let matches_desc = event.description.to_lowercase().contains(&search);
                let matches_tags = event.tags.iter().any(|t| t.to_lowercase().contains(&search));
                if !matches_name && !matches_desc && !matches_tags {
                    return false;
                }
            }

            true
        }).collect::<Vec<_>>()
    };

    rsx! {
        div {
            class: "narrative-event-library",
            style: "height: 100%; display: flex; flex-direction: column; gap: 1rem; padding: 1rem;",

            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",

                h2 { style: "color: white; margin: 0; font-size: 1.25rem;", "Narrative Events" }

                button {
                    onclick: move |_| show_create_form.set(true),
                    style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; display: flex; align-items: center; gap: 0.5rem;",
                    span { "+" }
                    span { "New Event" }
                }
            }

            // Filters
            div {
                style: "background: #1a1a2e; border-radius: 0.5rem; padding: 0.75rem; display: flex; gap: 0.75rem; align-items: center; flex-wrap: wrap;",

                // Search
                input {
                    r#type: "text",
                    placeholder: "Search events...",
                    value: "{search_text}",
                    oninput: move |e| search_text.set(e.value()),
                    style: "flex: 1; min-width: 200px; padding: 0.5rem 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem;",
                }

                // Status filter
                select {
                    value: "{filter_status}",
                    onchange: move |e| filter_status.set(e.value()),
                    style: "padding: 0.5rem 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem;",

                    option { value: "all", "All Events" }
                    option { value: "active", "Active Only" }
                    option { value: "pending", "Pending" }
                    option { value: "triggered", "Triggered" }
                }

                // Favorites toggle
                {
                    let is_favorites = *show_favorites_only.read();
                    rsx! {
                        label {
                            style: "display: flex; align-items: center; gap: 0.375rem; color: #9ca3af; font-size: 0.875rem; cursor: pointer;",

                            input {
                                r#type: "checkbox",
                                checked: is_favorites,
                                onchange: move |_| show_favorites_only.set(!is_favorites),
                            }
                            "⭐ Favorites"
                        }
                    }
                }
            }

            // Stats bar
            div {
                style: "display: flex; gap: 1rem; color: #6b7280; font-size: 0.875rem;",

                span { "{events.read().len()} total" }
                span { "{events.read().iter().filter(|e| e.is_active).count()} active" }
                span { "{events.read().iter().filter(|e| e.is_triggered).count()} triggered" }
                span { "{events.read().iter().filter(|e| e.is_favorite).count()} favorites" }
            }

            // Event list
            div {
                style: "flex: 1; overflow-y: auto;",

                if *is_loading.read() {
                    div {
                        style: "display: flex; justify-content: center; align-items: center; padding: 3rem; color: #9ca3af;",
                        "Loading narrative events..."
                    }
                } else if let Some(err) = error.read().as_ref() {
                    div {
                        style: "background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; padding: 1rem; color: #ef4444;",
                        "Error: {err}"
                    }
                } else if filtered_events.is_empty() {
                    div {
                        style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 3rem; color: #6b7280;",

                        div { style: "font-size: 3rem; margin-bottom: 1rem;", "⭐" }

                        if events.read().is_empty() {
                            p { "No narrative events yet" }
                            p { style: "font-size: 0.875rem;", "Create events to set up story hooks and branching narratives" }
                        } else {
                            p { "No events match your filters" }
                        }
                    }
                } else {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1rem;",

                        for event in filtered_events.iter() {
                            NarrativeEventCard {
                                key: "{event.id}",
                                event: event.clone(),
                                on_click: {
                                    let event = event.clone();
                                    move |_| selected_event.set(Some(event.clone()))
                                },
                                on_toggle_favorite: {
                                    let event_id = event.id.clone();
                                    let world_id = props.world_id.clone();
                                    let service = narrative_event_service.clone();
                                    move |_| {
                                        let event_id = event_id.clone();
                                        let world_id = world_id.clone();
                                        let service = service.clone();
                                        spawn(async move {
                                            if let Err(e) = service.toggle_favorite(&event_id).await {
                                                tracing::error!("Failed to toggle favorite: {}", e);
                                            }
                                            if let Ok(reloaded) = service.list_narrative_events(&world_id).await {
                                                events.set(reloaded);
                                            }
                                        });
                                    }
                                },
                                on_toggle_active: {
                                    let event_id = event.id.clone();
                                    let is_active = event.is_active;
                                    let world_id = props.world_id.clone();
                                    let service = narrative_event_service.clone();
                                    move |_| {
                                        let event_id = event_id.clone();
                                        let world_id = world_id.clone();
                                        let service = service.clone();
                                        spawn(async move {
                                            if let Err(e) = service.set_active(&event_id, !is_active).await {
                                                tracing::error!("Failed to toggle active: {}", e);
                                            }
                                            if let Ok(reloaded) = service.list_narrative_events(&world_id).await {
                                                events.set(reloaded);
                                            }
                                        });
                                    }
                                },
                            }
                        }
                    }
                }
            }

            // Create form modal
            if *show_create_form.read() {
                NarrativeEventFormModal {
                    world_id: props.world_id.clone(),
                    on_save: {
                        let mut events = events.clone();
                        move |new_event: NarrativeEventData| {
                            events.write().push(new_event);
                            show_create_form.set(false);
                        }
                    },
                    on_close: move |_| show_create_form.set(false),
                }
            }
        }
    }
}

/// Modal form for creating a new narrative event
#[derive(Props, Clone, PartialEq)]
struct NarrativeEventFormModalProps {
    world_id: String,
    on_save: EventHandler<NarrativeEventData>,
    on_close: EventHandler<()>,
}

#[component]
fn NarrativeEventFormModal(props: NarrativeEventFormModalProps) -> Element {
    let narrative_event_service = use_narrative_event_service();

    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut scene_direction = use_signal(|| String::new());
    let mut is_saving = use_signal(|| false);
    let mut save_error: Signal<Option<String>> = use_signal(|| None);

    let save_event = {
        let world_id = props.world_id.clone();
        let service = narrative_event_service.clone();
        let on_save = props.on_save.clone();
        move |_| {
            let world_id = world_id.clone();
            let service = service.clone();
            let on_save = on_save.clone();
            let name_val = name.read().clone();
            let desc_val = description.read().clone();
            let direction_val = scene_direction.read().clone();

            if name_val.trim().is_empty() {
                save_error.set(Some("Name is required".to_string()));
                return;
            }

            is_saving.set(true);
            save_error.set(None);

            spawn(async move {
                let request = CreateNarrativeEventRequest {
                    name: name_val,
                    description: desc_val,
                    scene_direction: direction_val,
                    ..Default::default()
                };

                match service.create_narrative_event(&world_id, request).await {
                    Ok(new_event) => {
                        on_save.call(new_event);
                    }
                    Err(e) => {
                        save_error.set(Some(format!("Failed to create event: {}", e)));
                        is_saving.set(false);
                    }
                }
            });
        }
    };

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_close.call(()),

            div {
                style: "background: #1a1a2e; border-radius: 0.75rem; max-width: 500px; width: 90%; max-height: 90vh; overflow-y: auto;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem 1.5rem; border-bottom: 1px solid #374151;",
                    h3 { style: "color: white; margin: 0;", "New Narrative Event" }
                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #9ca3af; font-size: 1.5rem; cursor: pointer;",
                        "×"
                    }
                }

                // Form
                div {
                    style: "padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;",

                    // Name field
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;",
                            "Event Name *"
                        }
                        input {
                            r#type: "text",
                            placeholder: "Enter event name...",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                            style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; box-sizing: border-box;",
                        }
                    }

                    // Description field
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;",
                            "Description"
                        }
                        textarea {
                            placeholder: "What happens when this event triggers?",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                            style: "width: 100%; min-height: 80px; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; resize: vertical; box-sizing: border-box;",
                        }
                    }

                    // Scene direction field
                    div {
                        label {
                            style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;",
                            "Scene Direction"
                        }
                        textarea {
                            placeholder: "How should the DM/AI present this event?",
                            value: "{scene_direction}",
                            oninput: move |e| scene_direction.set(e.value()),
                            style: "width: 100%; min-height: 60px; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.5rem; color: white; resize: vertical; box-sizing: border-box;",
                        }
                    }

                    // Error message
                    if let Some(err) = save_error.read().as_ref() {
                        div {
                            style: "background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; padding: 0.75rem; color: #ef4444; font-size: 0.875rem;",
                            "{err}"
                        }
                    }
                }

                // Footer
                div {
                    style: "display: flex; justify-content: flex-end; gap: 0.75rem; padding: 1rem 1.5rem; border-top: 1px solid #374151;",

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                        "Cancel"
                    }

                    button {
                        onclick: save_event,
                        disabled: *is_saving.read(),
                        style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                        if *is_saving.read() { "Creating..." } else { "Create Event" }
                    }
                }
            }
        }
    }
}

