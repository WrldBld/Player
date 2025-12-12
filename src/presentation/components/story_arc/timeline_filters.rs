//! Timeline Filters - Filter controls for the timeline view

use dioxus::prelude::*;

use crate::presentation::components::story_arc::timeline_view::TimelineFilterState;

#[derive(Props, Clone, PartialEq)]
pub struct TimelineFiltersProps {
    pub filters: Signal<TimelineFilterState>,
    pub on_filter_change: EventHandler<TimelineFilterState>,
}

#[component]
pub fn TimelineFilters(props: TimelineFiltersProps) -> Element {
    let mut expanded = use_signal(|| false);
    let current_filters = props.filters.read().clone();

    // Event type options
    let event_types = vec![
        ("", "All Events"),
        ("Location Change", "Location Change"),
        ("Dialogue", "Dialogue"),
        ("Combat", "Combat"),
        ("Challenge", "Challenge"),
        ("Item Acquired", "Item Acquired"),
        ("Relationship", "Relationship"),
        ("Scene Transition", "Scene Transition"),
        ("Information", "Information"),
        ("DM Marker", "DM Marker"),
        ("Narrative Event", "Narrative Event"),
        ("Session Start", "Session Start"),
        ("Session End", "Session End"),
    ];

    rsx! {
        div {
            class: "timeline-filters",
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 0.75rem;",

            // Compact filter bar
            div {
                style: "display: flex; gap: 0.75rem; align-items: center; flex-wrap: wrap;",

                // Search input
                div {
                    style: "flex: 1; min-width: 200px;",
                    input {
                        r#type: "text",
                        placeholder: "Search events...",
                        value: "{current_filters.search_text}",
                        oninput: {
                            let mut filters = current_filters.clone();
                            move |e: Event<FormData>| {
                                filters.search_text = e.value().clone();
                                props.on_filter_change.call(filters.clone());
                            }
                        },
                        style: "width: 100%; padding: 0.5rem 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem;",
                    }
                }

                // Event type dropdown
                {
                    let selected_type = current_filters.event_type.clone().unwrap_or_default();
                    rsx! {
                        select {
                            value: "{selected_type}",
                            onchange: {
                                let mut filters = current_filters.clone();
                                move |e: Event<FormData>| {
                                    let val = e.value();
                                    filters.event_type = if val.is_empty() { None } else { Some(val) };
                                    props.on_filter_change.call(filters.clone());
                                }
                            },
                            style: "padding: 0.5rem 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; font-size: 0.875rem;",

                            for (value, label) in event_types.iter() {
                                option { value: "{value}", "{label}" }
                            }
                        }
                    }
                }

                // Show hidden toggle
                label {
                    style: "display: flex; align-items: center; gap: 0.375rem; color: #9ca3af; font-size: 0.875rem; cursor: pointer; white-space: nowrap;",

                    input {
                        r#type: "checkbox",
                        checked: current_filters.show_hidden,
                        onchange: {
                            let mut filters = current_filters.clone();
                            move |_| {
                                filters.show_hidden = !filters.show_hidden;
                                props.on_filter_change.call(filters.clone());
                            }
                        },
                        style: "cursor: pointer;",
                    }
                    "Show hidden"
                }

                // Expand/collapse advanced filters
                {
                    let is_expanded = *expanded.read();
                    rsx! {
                        button {
                            onclick: move |_| expanded.set(!is_expanded),
                            style: "background: none; border: none; color: #60a5fa; cursor: pointer; font-size: 0.875rem; display: flex; align-items: center; gap: 0.25rem;",
                            if is_expanded { "Less ▲" } else { "More ▼" }
                        }
                    }
                }

                // Clear filters button (only show if filters active)
                if has_active_filters(&current_filters) {
                    button {
                        onclick: {
                            move |_| props.on_filter_change.call(TimelineFilterState::default())
                        },
                        style: "padding: 0.375rem 0.75rem; background: #374151; color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.75rem;",
                        "Clear"
                    }
                }
            }

            // Advanced filters (expandable)
            if *expanded.read() {
                div {
                    style: "margin-top: 0.75rem; padding-top: 0.75rem; border-top: 1px solid #374151; display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 0.75rem;",

                    // Date from
                    {
                        let date_from = current_filters.date_from.clone().unwrap_or_default();
                        rsx! {
                            div {
                                label {
                                    style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;",
                                    "From Date"
                                }
                                input {
                                    r#type: "date",
                                    value: "{date_from}",
                                    onchange: {
                                        let mut filters = current_filters.clone();
                                        move |e: Event<FormData>| {
                                            let val = e.value();
                                            filters.date_from = if val.is_empty() { None } else { Some(val) };
                                            props.on_filter_change.call(filters.clone());
                                        }
                                    },
                                    style: "width: 100%; padding: 0.375rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; font-size: 0.8125rem;",
                                }
                            }
                        }
                    }

                    // Date to
                    {
                        let date_to = current_filters.date_to.clone().unwrap_or_default();
                        rsx! {
                            div {
                                label {
                                    style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;",
                                    "To Date"
                                }
                                input {
                                    r#type: "date",
                                    value: "{date_to}",
                                    onchange: {
                                        let mut filters = current_filters.clone();
                                        move |e: Event<FormData>| {
                                            let val = e.value();
                                            filters.date_to = if val.is_empty() { None } else { Some(val) };
                                            props.on_filter_change.call(filters.clone());
                                        }
                                    },
                                    style: "width: 100%; padding: 0.375rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; font-size: 0.8125rem;",
                                }
                            }
                        }
                    }

                    // Character filter (TODO: populate from world data)
                    {
                        let char_id = current_filters.character_id.clone().unwrap_or_default();
                        rsx! {
                            div {
                                label {
                                    style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;",
                                    "Character"
                                }
                                select {
                                    value: "{char_id}",
                                    onchange: {
                                        let mut filters = current_filters.clone();
                                        move |e: Event<FormData>| {
                                            let val = e.value();
                                            filters.character_id = if val.is_empty() { None } else { Some(val) };
                                            props.on_filter_change.call(filters.clone());
                                        }
                                    },
                                    style: "width: 100%; padding: 0.375rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; font-size: 0.8125rem;",

                                    option { value: "", "All Characters" }
                                    // TODO: Populate with actual characters from world
                                }
                            }
                        }
                    }

                    // Location filter (TODO: populate from world data)
                    {
                        let loc_id = current_filters.location_id.clone().unwrap_or_default();
                        rsx! {
                            div {
                                label {
                                    style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;",
                                    "Location"
                                }
                                select {
                                    value: "{loc_id}",
                                    onchange: {
                                        let mut filters = current_filters.clone();
                                        move |e: Event<FormData>| {
                                            let val = e.value();
                                            filters.location_id = if val.is_empty() { None } else { Some(val) };
                                            props.on_filter_change.call(filters.clone());
                                        }
                                    },
                                    style: "width: 100%; padding: 0.375rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; font-size: 0.8125rem;",

                                    option { value: "", "All Locations" }
                                    // TODO: Populate with actual locations from world
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Check if any non-default filters are active
fn has_active_filters(filters: &TimelineFilterState) -> bool {
    !filters.search_text.is_empty()
        || filters.event_type.is_some()
        || filters.character_id.is_some()
        || filters.location_id.is_some()
        || filters.show_hidden
        || filters.date_from.is_some()
        || filters.date_to.is_some()
}
