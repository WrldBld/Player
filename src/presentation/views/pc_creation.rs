//! PC Creation View - Multi-step form for creating a player character

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::application::dto::{FieldValue, SheetTemplate};
use crate::application::ports::outbound::Platform;
use crate::application::services::{
    LocationService, PlayerCharacterService, WorldService,
    CreatePlayerCharacterRequest,
};
use crate::application::services::player_character_service::CharacterSheetDataApi;
use crate::presentation::services::{
    use_location_service, use_player_character_service, use_world_service,
};
use crate::presentation::state::{use_session_state, ConnectionStatus};

/// Wizard step enum
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum CreationStep {
    #[default]
    Basics,
    CharacterSheet,
    StartingLocation,
    Review,
}

/// Props for PC creation view
#[derive(Props, Clone, PartialEq)]
pub struct PCCreationProps {
    pub session_id: String,
    pub world_id: String,
}

/// PC Creation View - Multi-step form for creating a player character
#[component]
pub fn PCCreationView(props: PCCreationProps) -> Element {
    let navigator = use_navigator();
    let platform = use_context::<Platform>();
    let session_state = use_session_state();
    let pc_service = use_player_character_service();
    let location_service = use_location_service();
    let world_service = use_world_service();

    // Step tracking
    let mut current_step = use_signal(|| CreationStep::Basics);

    // Form state - Step 1: Basics
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());

    // Form state - Step 2: Character Sheet
    let mut sheet_template: Signal<Option<SheetTemplate>> = use_signal(|| None);
    let mut sheet_values: Signal<HashMap<String, FieldValue>> = use_signal(HashMap::new);
    let mut sheet_loading = use_signal(|| false);

    // Form state - Step 3: Starting Location
    let mut available_locations: Signal<Vec<crate::application::services::LocationSummary>> = use_signal(Vec::new);
    let mut selected_location_id: Signal<Option<String>> = use_signal(|| None);
    let mut locations_loading = use_signal(|| false);

    // General state
    let mut is_creating = use_signal(|| false);
    let mut error_message: Signal<Option<String>> = use_signal(|| None);

    // Load sheet template
    {
        let world_id = props.world_id.clone();
        let world_svc = world_service.clone();
        let platform_clone = platform.clone();
        use_effect(move || {
            let svc = world_svc.clone();
            let world_id_clone = world_id.clone();
            let plat = platform_clone.clone();
            sheet_loading.set(true);
            spawn(async move {
                match svc.get_sheet_template(&world_id_clone).await {
                    Ok(template_json) => {
                        match serde_json::from_value::<SheetTemplate>(template_json) {
                            Ok(template) => {
                                sheet_template.set(Some(template));
                            }
                            Err(e) => {
                                plat.log_warn(&format!("Failed to parse sheet template: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        plat.log_warn(&format!("Failed to load sheet template: {}", e));
                    }
                }
                sheet_loading.set(false);
            });
        });
    }

    // Load available locations
    {
        let world_id = props.world_id.clone();
        let loc_svc = location_service.clone();
        let platform_clone = platform.clone();
        use_effect(move || {
            let svc = loc_svc.clone();
            let world_id_clone = world_id.clone();
            let plat = platform_clone.clone();
            locations_loading.set(true);
            spawn(async move {
                match svc.list_locations(&world_id_clone).await {
                    Ok(locations) => {
                        available_locations.set(locations);
                    }
                    Err(e) => {
                        plat.log_error(&format!("Failed to load locations: {}", e));
                    }
                }
                locations_loading.set(false);
            });
        });
    }

    // Navigation handlers
    let go_next = move |_| {
        let step = *current_step.read();
        match step {
            CreationStep::Basics => {
                if name.read().trim().is_empty() {
                    error_message.set(Some("Character name is required".to_string()));
                    return;
                }
                current_step.set(CreationStep::CharacterSheet);
            }
            CreationStep::CharacterSheet => {
                current_step.set(CreationStep::StartingLocation);
            }
            CreationStep::StartingLocation => {
                if selected_location_id.read().is_none() {
                    error_message.set(Some("Please select a starting location".to_string()));
                    return;
                }
                current_step.set(CreationStep::Review);
            }
            CreationStep::Review => {
                // Will be handled by create button
            }
        }
        error_message.set(None);
    };

    let go_back = move |_| {
        let step = *current_step.read();
        match step {
            CreationStep::Basics => {
                navigator.go_back();
            }
            CreationStep::CharacterSheet => {
                current_step.set(CreationStep::Basics);
            }
            CreationStep::StartingLocation => {
                current_step.set(CreationStep::CharacterSheet);
            }
            CreationStep::Review => {
                current_step.set(CreationStep::StartingLocation);
            }
        }
        error_message.set(None);
    };

    let create_character = move |_| {
        let name_val = name.read().clone();
        let desc_val = description.read().clone();
        let location_id = selected_location_id.read().clone();
        let sheet_vals = sheet_values.read().clone();
        let session_id = props.session_id.clone();
        let pc_svc = pc_service.clone();
        let nav = navigator.clone();
        let world_id = props.world_id.clone();

        if name_val.trim().is_empty() {
            error_message.set(Some("Character name is required".to_string()));
            return;
        }

        if location_id.is_none() {
            error_message.set(Some("Please select a starting location".to_string()));
            return;
        }

        is_creating.set(true);
        error_message.set(None);

        spawn(async move {
            let sheet_data = if sheet_vals.is_empty() {
                None
            } else {
                Some(CharacterSheetDataApi { values: sheet_vals })
            };

            let request = CreatePlayerCharacterRequest {
                name: name_val,
                description: if desc_val.trim().is_empty() {
                    None
                } else {
                    Some(desc_val)
                },
                starting_location_id: location_id.unwrap(),
                sheet_data,
                sprite_asset: None,
                portrait_asset: None,
            };

            match pc_svc.create_pc(&session_id, &request).await {
                Ok(_pc) => {
                    // Navigate to PC View
                    nav.push(crate::routes::Route::PCViewRoute {
                        world_id: world_id.clone(),
                    });
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to create character: {}", e)));
                    is_creating.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            style: "height: 100vh; display: flex; flex-direction: column; background: #0f0f23; color: white;",
            
            // Header
            div {
                style: "padding: 1.5rem; border-bottom: 1px solid #374151;",
                h1 {
                    style: "margin: 0; font-size: 1.5rem; color: white;",
                    "Create Your Character"
                }
                p {
                    style: "margin: 0.5rem 0 0 0; color: #9ca3af; font-size: 0.875rem;",
                    "Set up your character to join the adventure"
                }
            }

            // Progress indicator
            div {
                style: "display: flex; gap: 0.5rem; padding: 1rem 1.5rem; background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid #374151;",
                StepIndicator {
                    number: 1,
                    label: "Basics",
                    is_active: *current_step.read() == CreationStep::Basics,
                    is_complete: *current_step.read() != CreationStep::Basics,
                }
                StepIndicator {
                    number: 2,
                    label: "Character Sheet",
                    is_active: *current_step.read() == CreationStep::CharacterSheet,
                    is_complete: matches!(*current_step.read(), CreationStep::StartingLocation | CreationStep::Review),
                }
                StepIndicator {
                    number: 3,
                    label: "Starting Location",
                    is_active: *current_step.read() == CreationStep::StartingLocation,
                    is_complete: *current_step.read() == CreationStep::Review,
                }
                StepIndicator {
                    number: 4,
                    label: "Review",
                    is_active: *current_step.read() == CreationStep::Review,
                    is_complete: false,
                }
            }

            // Content area
            div {
                style: "flex: 1; overflow-y: auto; padding: 2rem;",
                
                // Error message
                if let Some(err) = error_message.read().as_ref() {
                    div {
                        style: "padding: 0.75rem 1rem; background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 0.5rem; color: #ef4444; margin-bottom: 1rem;",
                        "{err}"
                    }
                }

                match *current_step.read() {
                    CreationStep::Basics => rsx! {
                        BasicsStep {
                            name: name.read().clone(),
                            description: description.read().clone(),
                            on_name_change: move |n| name.set(n),
                            on_description_change: move |d| description.set(d),
                        }
                    },
                    CreationStep::CharacterSheet => rsx! {
                        CharacterSheetStep {
                            template: sheet_template.read().clone(),
                            values: sheet_values.read().clone(),
                            loading: *sheet_loading.read(),
                            on_values_change: move |v| sheet_values.set(v),
                        }
                    },
                    CreationStep::StartingLocation => rsx! {
                        StartingLocationStep {
                            locations: available_locations.read().clone(),
                            selected: selected_location_id.read().clone(),
                            loading: *locations_loading.read(),
                            on_select: move |id| selected_location_id.set(Some(id)),
                        }
                    },
                    CreationStep::Review => rsx! {
                        ReviewStep {
                            name: name.read().clone(),
                            description: description.read().clone(),
                            location: available_locations.read().iter()
                                .find(|l| l.id == selected_location_id.read().as_ref().map(|s| s.as_str()).unwrap_or(""))
                                .map(|l| l.name.clone())
                                .unwrap_or_default(),
                            has_sheet: !sheet_values.read().is_empty(),
                        }
                    },
                }
            }

            // Footer with navigation
            div {
                style: "padding: 1rem 1.5rem; border-top: 1px solid #374151; display: flex; justify-content: space-between;",
                button {
                    onclick: go_back,
                    style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                    if *current_step.read() == CreationStep::Basics {
                        "Cancel"
                    } else {
                        "Back"
                    }
                }
                div {
                    style: "display: flex; gap: 0.5rem;",
                    if *current_step.read() == CreationStep::Review {
                        button {
                            onclick: create_character,
                            disabled: *is_creating.read(),
                            style: "padding: 0.5rem 1.5rem; background: #22c55e; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 500;",
                            if *is_creating.read() {
                                "Creating..."
                            } else {
                                "Create Character"
                            }
                        }
                    } else {
                        button {
                            onclick: go_next,
                            style: "padding: 0.5rem 1.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-weight: 500;",
                            "Next"
                        }
                    }
                }
            }
        }
    }
}

/// Step indicator component
#[component]
fn StepIndicator(number: u8, label: &'static str, is_active: bool, is_complete: bool) -> Element {
    let bg_color = if is_active || is_complete {
        "#3b82f6"
    } else {
        "#374151"
    };
    let text_color = if is_active {
        "white"
    } else if is_complete {
        "#9ca3af"
    } else {
        "#6b7280"
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem;",
            div {
                style: format!(
                    "width: 32px; height: 32px; border-radius: 50%; background: {}; display: flex; align-items: center; justify-content: center; font-size: 0.875rem; color: white; font-weight: 600;",
                    bg_color
                ),
                if is_complete { "✓" } else { "{number}" }
            }
            span {
                style: format!("color: {}; font-size: 0.875rem; font-weight: 500;", text_color),
                "{label}"
            }
            if number < 4 {
                div {
                    style: "width: 60px; height: 2px; background: #374151; margin: 0 0.5rem;",
                }
            }
        }
    }
}

/// Step 1: Character Basics
#[derive(Props, Clone, PartialEq)]
struct BasicsStepProps {
    name: String,
    description: String,
    on_name_change: EventHandler<String>,
    on_description_change: EventHandler<String>,
}

#[component]
fn BasicsStep(props: BasicsStepProps) -> Element {
    rsx! {
        div {
            style: "max-width: 600px; margin: 0 auto;",
            h2 {
                style: "margin-bottom: 1.5rem; font-size: 1.25rem; color: white;",
                "Character Basics"
            }
            div {
                style: "display: flex; flex-direction: column; gap: 1.5rem;",
                div {
                    label {
                        style: "display: block; margin-bottom: 0.5rem; color: #9ca3af; font-size: 0.875rem; font-weight: 500;",
                        "Character Name *"
                    }
                    input {
                        r#type: "text",
                        value: "{props.name}",
                        oninput: move |e| props.on_name_change.call(e.value()),
                        placeholder: "Enter your character's name",
                        style: "width: 100%; padding: 0.75rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 1rem;",
                    }
                }
                div {
                    label {
                        style: "display: block; margin-bottom: 0.5rem; color: #9ca3af; font-size: 0.875rem; font-weight: 500;",
                        "Description (Optional)"
                    }
                    textarea {
                        value: "{props.description}",
                        oninput: move |e| props.on_description_change.call(e.value()),
                        placeholder: "Describe your character...",
                        rows: 4,
                        style: "width: 100%; padding: 0.75rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.5rem; color: white; font-size: 1rem; resize: vertical;",
                    }
                }
            }
        }
    }
}

/// Step 2: Character Sheet
#[derive(Props, Clone, PartialEq)]
struct CharacterSheetStepProps {
    template: Option<SheetTemplate>,
    values: HashMap<String, FieldValue>,
    loading: bool,
    on_values_change: EventHandler<HashMap<String, FieldValue>>,
}

#[component]
fn CharacterSheetStep(props: CharacterSheetStepProps) -> Element {
    rsx! {
        div {
            style: "max-width: 800px; margin: 0 auto;",
            h2 {
                style: "margin-bottom: 1.5rem; font-size: 1.25rem; color: white;",
                "Character Sheet"
            }
            if props.loading {
                div {
                    style: "text-align: center; padding: 2rem; color: #9ca3af;",
                    "Loading character sheet template..."
                }
            } else if let Some(template) = props.template.as_ref() {
                crate::presentation::components::creator::sheet_field_input::CharacterSheetForm {
                    template: template.clone(),
                    values: props.values.clone(),
                    on_values_change: move |v| props.on_values_change.call(v),
                }
            } else {
                div {
                    style: "padding: 1.5rem; background: #1a1a2e; border-radius: 0.5rem; border: 1px solid #374151; text-align: center; color: #9ca3af;",
                    "No character sheet template available for this world. You can skip this step."
                }
            }
        }
    }
}

/// Step 3: Starting Location
#[derive(Props, Clone, PartialEq)]
struct StartingLocationStepProps {
    locations: Vec<crate::application::services::LocationSummary>,
    selected: Option<String>,
    loading: bool,
    on_select: EventHandler<String>,
}

#[component]
fn StartingLocationStep(props: StartingLocationStepProps) -> Element {
    let locations = props.locations.clone();
    let selected = props.selected.clone();

    rsx! {
        div {
            style: "max-width: 800px; margin: 0 auto;",
            h2 {
                style: "margin-bottom: 1.5rem; font-size: 1.25rem; color: white;",
                "Choose Starting Location"
            }
            if props.loading {
                div {
                    style: "text-align: center; padding: 2rem; color: #9ca3af;",
                    "Loading locations..."
                }
            } else if locations.is_empty() {
                div {
                    style: "padding: 1.5rem; background: #1a1a2e; border-radius: 0.5rem; border: 1px solid #374151; text-align: center; color: #9ca3af;",
                    "No locations available. Please contact your DM."
                }
            } else {
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 1rem;",
                    {locations.into_iter().map(|location| {
                        let loc_id = location.id.clone();
                        let sel = selected.as_ref().map(|s| s == &loc_id).unwrap_or(false);
                        rsx! {
                            LocationCard {
                                location,
                                is_selected: sel,
                                on_select: move |_| props.on_select.call(loc_id.clone()),
                            }
                        }
                    })}
                }
            }
        }
    }
}

/// Location card component
#[derive(Props, Clone, PartialEq)]
struct LocationCardProps {
    location: crate::application::services::LocationSummary,
    is_selected: bool,
    on_select: EventHandler<()>,
}

#[component]
fn LocationCard(props: LocationCardProps) -> Element {
    let border_color = if props.is_selected {
        "#3b82f6"
    } else {
        "#374151"
    };
    let bg_color = if props.is_selected {
        "rgba(59, 130, 246, 0.1)"
    } else {
        "#1a1a2e"
    };

    rsx! {
        div {
            onclick: move |_| props.on_select.call(()),
            style: format!(
                "padding: 1.5rem; background: {}; border: 2px solid {}; border-radius: 0.5rem; cursor: pointer; transition: all 0.2s;",
                bg_color, border_color
            ),
            h3 {
                style: "margin: 0 0 0.5rem 0; color: white; font-size: 1rem;",
                "{props.location.name}"
            }
            if let Some(loc_type) = props.location.location_type.as_ref() {
                p {
                    style: "margin: 0; color: #9ca3af; font-size: 0.875rem; line-height: 1.5;",
                    "{loc_type}"
                }
            }
            if props.is_selected {
                div {
                    style: "margin-top: 0.75rem; color: #3b82f6; font-size: 0.875rem; font-weight: 500;",
                    "✓ Selected"
                }
            }
        }
    }
}

/// Step 4: Review
#[derive(Props, Clone, PartialEq)]
struct ReviewStepProps {
    name: String,
    description: String,
    location: String,
    has_sheet: bool,
}

#[component]
fn ReviewStep(props: ReviewStepProps) -> Element {
    rsx! {
        div {
            style: "max-width: 600px; margin: 0 auto;",
            h2 {
                style: "margin-bottom: 1.5rem; font-size: 1.25rem; color: white;",
                "Review Your Character"
            }
            div {
                style: "display: flex; flex-direction: column; gap: 1.5rem; padding: 1.5rem; background: #1a1a2e; border-radius: 0.5rem; border: 1px solid #374151;",
                div {
                    div {
                        style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;",
                        "Name"
                    }
                    div {
                        style: "color: white; font-size: 1rem; font-weight: 500;",
                        "{props.name}"
                    }
                }
                if !props.description.is_empty() {
                    div {
                        div {
                            style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;",
                            "Description"
                        }
                        div {
                            style: "color: white; font-size: 0.875rem; line-height: 1.5;",
                            "{props.description}"
                        }
                    }
                }
                div {
                    div {
                        style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;",
                        "Starting Location"
                    }
                    div {
                        style: "color: white; font-size: 1rem; font-weight: 500;",
                        "{props.location}"
                    }
                }
                div {
                    div {
                        style: "color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;",
                        "Character Sheet"
                    }
                    div {
                        style: "color: white; font-size: 0.875rem;",
                        if props.has_sheet {
                            "✓ Configured"
                        } else {
                            "Not configured"
                        }
                    }
                }
            }
            div {
                style: "margin-top: 1.5rem; padding: 1rem; background: rgba(59, 130, 246, 0.1); border: 1px solid #3b82f6; border-radius: 0.5rem; color: #9ca3af; font-size: 0.875rem;",
                "Ready to create your character! Click 'Create Character' to begin your adventure."
            }
        }
    }
}

