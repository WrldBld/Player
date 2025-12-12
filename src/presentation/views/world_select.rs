//! World Selection View - Select or create a world before entering gameplay
//!
//! This view appears after role selection and before the game views.
//! - DM: Can create new worlds or continue existing ones
//! - Player: Can join existing worlds
//! - Spectator: Can watch existing worlds

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::infrastructure::asset_loader::{
    DiceSystem, RuleSystemConfig, RuleSystemType, RuleSystemVariant, StatDefinition,
    SuccessComparison, WorldSnapshot,
};
use crate::presentation::state::GameState;
use crate::UserRole;

/// Summary of a world for the list view
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// Props for WorldSelectView
#[derive(Props, Clone, PartialEq)]
pub struct WorldSelectViewProps {
    /// The role the user selected
    pub role: UserRole,
    /// Called when a world is selected and loaded
    pub on_world_selected: EventHandler<String>,
    /// Called when user wants to go back to role selection
    pub on_back: EventHandler<()>,
}

/// World Selection View component
#[component]
pub fn WorldSelectView(props: WorldSelectViewProps) -> Element {
    let mut game_state = use_context::<GameState>();
    let mut worlds: Signal<Vec<WorldSummary>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut show_create_form = use_signal(|| false);
    let mut world_to_load: Signal<Option<String>> = use_signal(|| None);

    let is_dm = props.role == UserRole::DungeonMaster;

    // Fetch worlds on mount
    use_effect(move || {
        spawn(async move {
            match fetch_worlds().await {
                Ok(list) => {
                    worlds.set(list);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e));
                    is_loading.set(false);
                }
            }
        });
    });

    // Effect to load world when world_to_load is set
    use_effect(move || {
        if let Some(world_id) = world_to_load.read().clone() {
            let mut game_state = game_state.clone();
            let world_id_for_callback = world_id.clone();
            spawn(async move {
                is_loading.set(true);
                match fetch_world_snapshot(&world_id).await {
                    Ok(snapshot) => {
                        game_state.load_world(snapshot);
                        // Signal that world is loaded - parent will navigate
                        props.on_world_selected.call(world_id_for_callback);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load world: {}", e)));
                        is_loading.set(false);
                    }
                }
                world_to_load.set(None);
            });
        }
    });

    // Title based on role
    let title = match props.role {
        UserRole::DungeonMaster => "Select World",
        UserRole::Player => "Join a World",
        UserRole::Spectator => "Watch a World",
    };

    let subtitle = match props.role {
        UserRole::DungeonMaster => "Continue an existing campaign or create a new one",
        UserRole::Player => "Choose a world to join as a player",
        UserRole::Spectator => "Choose a world to watch",
    };

    let action_label = match props.role {
        UserRole::DungeonMaster => "Continue",
        UserRole::Player => "Join",
        UserRole::Spectator => "Watch",
    };

    rsx! {
        div {
            class: "world-select-view",
            style: "height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 2rem;",

            div {
                style: "max-width: 700px; width: 100%;",

                // Back button
                button {
                    onclick: move |_| props.on_back.call(()),
                    style: "margin-bottom: 1.5rem; padding: 0.5rem 1rem; background: transparent; color: #9ca3af; border: 1px solid #374151; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem;",
                    "← Back to Role Selection"
                }

                h1 {
                    style: "color: white; text-align: center; margin-bottom: 0.5rem; font-size: 2rem;",
                    "{title}"
                }
                p {
                    style: "color: #9ca3af; text-align: center; margin-bottom: 2rem;",
                    "{subtitle}"
                }

                // Error message
                if let Some(err) = error.read().as_ref() {
                    div {
                        style: "padding: 1rem; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 0.5rem; color: #ef4444; margin-bottom: 1rem;",
                        "{err}"
                    }
                }

                // Loading state
                if *is_loading.read() {
                    div {
                        style: "text-align: center; color: #6b7280; padding: 2rem;",
                        "Loading worlds..."
                    }
                } else if *show_create_form.read() && is_dm {
                    // Create form (DM only)
                    CreateWorldForm {
                        on_created: move |world_id: String| {
                            show_create_form.set(false);
                            world_to_load.set(Some(world_id));
                        },
                        on_cancel: move |_| show_create_form.set(false),
                    }
                } else {
                    // World list
                    div {
                        style: "background: #1a1a2e; border-radius: 0.5rem; overflow: hidden;",

                        // Header with create button (DM only)
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-bottom: 1px solid #374151;",

                            h2 { style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin: 0;",
                                if is_dm { "Your Worlds" } else { "Available Worlds" }
                            }

                            if is_dm {
                                button {
                                    onclick: move |_| show_create_form.set(true),
                                    style: "padding: 0.5rem 1rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.875rem;",
                                    "+ Create New World"
                                }
                            }
                        }

                        // World list
                        div {
                            style: "max-height: 400px; overflow-y: auto;",

                            if worlds.read().is_empty() {
                                div {
                                    style: "padding: 2rem; text-align: center; color: #6b7280;",
                                    if is_dm {
                                        p { "No worlds yet." }
                                        p { style: "font-size: 0.875rem;", "Create your first world to get started!" }
                                    } else {
                                        p { "No worlds available." }
                                        p { style: "font-size: 0.875rem;", "Ask your DM to create a world first." }
                                    }
                                }
                            }

                            for world in worlds.read().iter() {
                                WorldCard {
                                    key: "{world.id}",
                                    world: world.clone(),
                                    action_label: action_label,
                                    is_dm: is_dm,
                                    on_select: move |id: String| world_to_load.set(Some(id)),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// World card in the list
#[component]
fn WorldCard(
    world: WorldSummary,
    action_label: &'static str,
    is_dm: bool,
    on_select: EventHandler<String>,
) -> Element {
    let world_id = world.id.clone();

    rsx! {
        div {
            style: "padding: 1rem; border-bottom: 1px solid #374151; display: flex; justify-content: space-between; align-items: center;",

            div {
                style: "flex: 1;",
                h3 { style: "color: white; margin: 0 0 0.25rem 0; font-size: 1rem;", "{world.name}" }
                if let Some(desc) = &world.description {
                    p { style: "color: #9ca3af; margin: 0; font-size: 0.875rem; line-height: 1.4;", "{desc}" }
                }
            }

            button {
                onclick: move |_| on_select.call(world_id.clone()),
                style: "padding: 0.5rem 1rem; background: #3b82f6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.875rem; white-space: nowrap;",
                "{action_label} →"
            }
        }
    }
}

/// Form for creating a new world (DM only)
#[component]
fn CreateWorldForm(on_created: EventHandler<String>, on_cancel: EventHandler<()>) -> Element {
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut selected_type = use_signal(|| RuleSystemType::D20);
    let mut selected_variant: Signal<Option<RuleSystemVariant>> =
        use_signal(|| Some(RuleSystemVariant::Dnd5e));
    let mut is_creating = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut created_world_id: Signal<Option<String>> = use_signal(|| None);

    // Full rule system config (loaded when variant changes)
    let mut rule_config: Signal<Option<RuleSystemConfig>> = use_signal(|| None);
    let mut show_advanced = use_signal(|| false);
    let mut is_loading_preset = use_signal(|| false);

    // Update variant when type changes
    let current_type = *selected_type.read();
    let available_variants = RuleSystemVariant::variants_for_type(current_type);

    // Effect to call on_created when world is created
    use_effect(move || {
        let world_id = created_world_id.read().clone();
        if let Some(id) = world_id {
            on_created.call(id);
            created_world_id.set(None);
        }
    });

    // Effect to load preset when variant changes
    let variant_for_effect = selected_variant.read().clone();
    use_effect(move || {
        if let Some(variant) = variant_for_effect.clone() {
            spawn(async move {
                is_loading_preset.set(true);
                match fetch_preset(&variant).await {
                    Ok(config) => {
                        rule_config.set(Some(config));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load preset: {}", e)));
                    }
                }
                is_loading_preset.set(false);
            });
        } else {
            rule_config.set(None);
        }
    });

    let mut handle_type_change = move |type_str: String| {
        let new_type = match type_str.as_str() {
            "D20" => RuleSystemType::D20,
            "D100" => RuleSystemType::D100,
            "Narrative" => RuleSystemType::Narrative,
            _ => RuleSystemType::Custom,
        };
        selected_type.set(new_type);
        // Set default variant for the new type
        let variants = RuleSystemVariant::variants_for_type(new_type);
        selected_variant.set(variants.into_iter().next());
    };

    let mut handle_variant_change = move |variant_str: String| {
        let variant = match variant_str.as_str() {
            "Dnd5e" => Some(RuleSystemVariant::Dnd5e),
            "Pathfinder2e" => Some(RuleSystemVariant::Pathfinder2e),
            "GenericD20" => Some(RuleSystemVariant::GenericD20),
            "CallOfCthulhu7e" => Some(RuleSystemVariant::CallOfCthulhu7e),
            "RuneQuest" => Some(RuleSystemVariant::RuneQuest),
            "GenericD100" => Some(RuleSystemVariant::GenericD100),
            "KidsOnBikes" => Some(RuleSystemVariant::KidsOnBikes),
            "FateCore" => Some(RuleSystemVariant::FateCore),
            "PoweredByApocalypse" => Some(RuleSystemVariant::PoweredByApocalypse),
            _ => None,
        };
        selected_variant.set(variant);
    };

    let handle_create = move |_| {
        let name_val = name.read().clone();
        if name_val.trim().is_empty() {
            error.set(Some("World name is required".to_string()));
            return;
        }

        let desc_val = description.read().clone();
        let config = rule_config.read().clone();

        spawn(async move {
            is_creating.set(true);
            error.set(None);

            match create_world(
                &name_val,
                if desc_val.is_empty() { None } else { Some(&desc_val) },
                config,
            )
            .await
            {
                Ok(world_id) => {
                    created_world_id.set(Some(world_id));
                }
                Err(e) => {
                    error.set(Some(e));
                    is_creating.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            style: "background: #1a1a2e; border-radius: 0.5rem; padding: 1.5rem; max-height: 80vh; overflow-y: auto;",

            h2 { style: "color: white; margin: 0 0 1rem 0;", "Create New World" }

            // Error
            if let Some(err) = error.read().as_ref() {
                div {
                    style: "padding: 0.75rem; background: rgba(239, 68, 68, 0.1); border-radius: 0.25rem; color: #ef4444; margin-bottom: 1rem; font-size: 0.875rem;",
                    "{err}"
                }
            }

            // Name field
            div { style: "margin-bottom: 1rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Name *" }
                input {
                    r#type: "text",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    placeholder: "The Dragon's Bane",
                    disabled: *is_creating.read(),
                    style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                }
            }

            // Description field
            div { style: "margin-bottom: 1rem;",
                label { style: "display: block; color: #9ca3af; font-size: 0.875rem; margin-bottom: 0.25rem;", "Description" }
                textarea {
                    value: "{description}",
                    oninput: move |e| description.set(e.value()),
                    placeholder: "A dark fantasy campaign in the realm of Valdris...",
                    disabled: *is_creating.read(),
                    style: "width: 100%; min-height: 80px; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                }
            }

            // Rule System Selection
            div { style: "margin-bottom: 1rem; padding: 1rem; background: #0f0f23; border-radius: 0.5rem; border: 1px solid #374151;",
                h3 { style: "color: #9ca3af; font-size: 0.875rem; text-transform: uppercase; margin: 0 0 0.75rem 0;", "Rule System" }

                // System Type dropdown
                div { style: "margin-bottom: 0.75rem;",
                    label { style: "display: block; color: #6b7280; font-size: 0.75rem; margin-bottom: 0.25rem;", "System Type" }
                    select {
                        value: "{current_type:?}",
                        onchange: move |e| handle_type_change(e.value()),
                        disabled: *is_creating.read(),
                        style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                        option { value: "D20", "D20 System (D&D, Pathfinder)" }
                        option { value: "D100", "D100 System (Call of Cthulhu)" }
                        option { value: "Narrative", "Narrative (Kids on Bikes, FATE)" }
                        option { value: "Custom", "Custom" }
                    }
                }

                // Variant/Preset dropdown (only show if variants available)
                if !available_variants.is_empty() {
                    {
                        let current_variant_value = selected_variant.read()
                            .as_ref()
                            .map(|v| format!("{:?}", v))
                            .unwrap_or_default();

                        rsx! {
                            div { style: "margin-bottom: 0.5rem;",
                                label { style: "display: block; color: #6b7280; font-size: 0.75rem; margin-bottom: 0.25rem;", "Preset" }
                                select {
                                    value: "{current_variant_value}",
                                    onchange: move |e| handle_variant_change(e.value()),
                                    disabled: *is_creating.read() || *is_loading_preset.read(),
                                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                                    for variant in available_variants.iter() {
                                        option {
                                            value: "{variant:?}",
                                            "{variant.display_name()}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Show selected preset description
                if let Some(variant) = selected_variant.read().as_ref() {
                    p { style: "color: #6b7280; font-size: 0.75rem; margin: 0.5rem 0 0 0;",
                        "{variant.description()}"
                    }
                } else {
                    p { style: "color: #6b7280; font-size: 0.75rem; margin: 0.5rem 0 0 0;",
                        "{current_type.description()}"
                    }
                }

                // Loading indicator
                if *is_loading_preset.read() {
                    p { style: "color: #8b5cf6; font-size: 0.75rem; margin: 0.5rem 0 0 0;",
                        "Loading preset..."
                    }
                }
            }

            // Advanced Configuration (collapsible)
            if rule_config.read().is_some() {
                div { style: "margin-bottom: 1rem;",
                    {
                        let is_expanded = *show_advanced.read();
                        rsx! {
                            button {
                                onclick: move |_| show_advanced.set(!is_expanded),
                                style: "width: 100%; padding: 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: #9ca3af; cursor: pointer; text-align: left; display: flex; justify-content: space-between; align-items: center;",
                                span { "Advanced Configuration" }
                                span { if is_expanded { "▼" } else { "▶" } }
                            }
                        }
                    }

                    if *show_advanced.read() {
                        {
                            let config = rule_config.read().clone().unwrap();
                            rsx! {
                                RuleSystemConfigEditor {
                                    config: config,
                                    on_change: move |new_config| rule_config.set(Some(new_config)),
                                    disabled: *is_creating.read(),
                                }
                            }
                        }
                    }
                }
            }

            // Buttons
            div { style: "display: flex; gap: 0.75rem;",
                button {
                    onclick: handle_create,
                    disabled: *is_creating.read() || *is_loading_preset.read(),
                    style: "flex: 1; padding: 0.75rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-weight: 600;",
                    if *is_creating.read() { "Creating..." } else { "Create World" }
                }
                button {
                    onclick: move |_| on_cancel.call(()),
                    disabled: *is_creating.read(),
                    style: "padding: 0.75rem 1.5rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer;",
                    "Cancel"
                }
            }
        }
    }
}

/// Editor for RuleSystemConfig - shows all fields
#[component]
fn RuleSystemConfigEditor(
    config: RuleSystemConfig,
    on_change: EventHandler<RuleSystemConfig>,
    disabled: bool,
) -> Element {
    let mut local_config = use_signal(|| config.clone());

    // Update local config when prop changes
    let config_clone = config.clone();
    use_effect(move || {
        local_config.set(config_clone.clone());
    });

    let config_read = local_config.read();
    let dice_str = match &config_read.dice_system {
        DiceSystem::D20 => "D20",
        DiceSystem::D100 => "D100",
        DiceSystem::Fate => "Fate",
        DiceSystem::DicePool { .. } => "DicePool",
        DiceSystem::Custom(_) => "Custom",
    };
    let comparison_str = match &config_read.success_comparison {
        SuccessComparison::GreaterOrEqual => "GreaterOrEqual",
        SuccessComparison::LessOrEqual => "LessOrEqual",
        SuccessComparison::Narrative => "Narrative",
    };

    rsx! {
        div { style: "padding: 1rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0 0 0.25rem 0.25rem; border-top: none;",

            // Rule System Name
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #6b7280; font-size: 0.75rem; margin-bottom: 0.25rem;", "Rule System Name" }
                input {
                    r#type: "text",
                    value: "{config_read.name}",
                    oninput: move |e| {
                        let mut cfg = local_config.read().clone();
                        cfg.name = e.value();
                        local_config.set(cfg.clone());
                        on_change.call(cfg);
                    },
                    disabled: disabled,
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                }
            }

            // Rule System Description
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #6b7280; font-size: 0.75rem; margin-bottom: 0.25rem;", "Description" }
                textarea {
                    value: "{config_read.description}",
                    oninput: move |e| {
                        let mut cfg = local_config.read().clone();
                        cfg.description = e.value();
                        local_config.set(cfg.clone());
                        on_change.call(cfg);
                    },
                    disabled: disabled,
                    style: "width: 100%; min-height: 60px; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; resize: vertical; box-sizing: border-box;",
                }
            }

            // Dice System
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #6b7280; font-size: 0.75rem; margin-bottom: 0.25rem;", "Dice System" }
                select {
                    value: "{dice_str}",
                    onchange: move |e| {
                        let mut cfg = local_config.read().clone();
                        cfg.dice_system = match e.value().as_str() {
                            "D20" => DiceSystem::D20,
                            "D100" => DiceSystem::D100,
                            "Fate" => DiceSystem::Fate,
                            _ => DiceSystem::Custom("Custom".to_string()),
                        };
                        local_config.set(cfg.clone());
                        on_change.call(cfg);
                    },
                    disabled: disabled,
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                    option { value: "D20", "D20 (d20 + modifier)" }
                    option { value: "D100", "D100 (percentile)" }
                    option { value: "Fate", "Fate Dice (4dF)" }
                    option { value: "Custom", "Custom" }
                }
            }

            // Success Comparison
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #6b7280; font-size: 0.75rem; margin-bottom: 0.25rem;", "Success Comparison" }
                select {
                    value: "{comparison_str}",
                    onchange: move |e| {
                        let mut cfg = local_config.read().clone();
                        cfg.success_comparison = match e.value().as_str() {
                            "GreaterOrEqual" => SuccessComparison::GreaterOrEqual,
                            "LessOrEqual" => SuccessComparison::LessOrEqual,
                            _ => SuccessComparison::Narrative,
                        };
                        local_config.set(cfg.clone());
                        on_change.call(cfg);
                    },
                    disabled: disabled,
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                    option { value: "GreaterOrEqual", "Roll >= Target (D20 style)" }
                    option { value: "LessOrEqual", "Roll <= Target (D100 style)" }
                    option { value: "Narrative", "Narrative (story-driven)" }
                }
            }

            // Skill Check Formula
            div { style: "margin-bottom: 0.75rem;",
                label { style: "display: block; color: #6b7280; font-size: 0.75rem; margin-bottom: 0.25rem;", "Skill Check Formula" }
                input {
                    r#type: "text",
                    value: "{config_read.skill_check_formula}",
                    oninput: move |e| {
                        let mut cfg = local_config.read().clone();
                        cfg.skill_check_formula = e.value();
                        local_config.set(cfg.clone());
                        on_change.call(cfg);
                    },
                    disabled: disabled,
                    placeholder: "e.g., 1d20 + modifier vs DC",
                    style: "width: 100%; padding: 0.5rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;",
                }
            }

            // Stats Section
            div { style: "margin-top: 1rem;",
                h4 { style: "color: #9ca3af; font-size: 0.75rem; text-transform: uppercase; margin: 0 0 0.5rem 0;",
                    "Character Stats ({config_read.stat_definitions.len()})"
                }

                // Stats header row
                div { style: "display: grid; grid-template-columns: 1fr 60px 60px 60px 60px 30px; gap: 0.5rem; margin-bottom: 0.25rem; padding: 0 0.25rem;",
                    span { style: "color: #6b7280; font-size: 0.625rem; text-transform: uppercase;", "Name" }
                    span { style: "color: #6b7280; font-size: 0.625rem; text-transform: uppercase; text-align: center;", "Abbr" }
                    span { style: "color: #6b7280; font-size: 0.625rem; text-transform: uppercase; text-align: center;", "Min" }
                    span { style: "color: #6b7280; font-size: 0.625rem; text-transform: uppercase; text-align: center;", "Max" }
                    span { style: "color: #6b7280; font-size: 0.625rem; text-transform: uppercase; text-align: center;", "Default" }
                    span {}
                }

                for (i, stat) in config_read.stat_definitions.iter().enumerate() {
                    div {
                        key: "{i}",
                        style: "display: grid; grid-template-columns: 1fr 60px 60px 60px 60px 30px; gap: 0.5rem; align-items: center; margin-bottom: 0.5rem;",

                        input {
                            r#type: "text",
                            value: "{stat.name}",
                            oninput: move |e| {
                                let mut cfg = local_config.read().clone();
                                if let Some(s) = cfg.stat_definitions.get_mut(i) {
                                    s.name = e.value();
                                }
                                local_config.set(cfg.clone());
                                on_change.call(cfg);
                            },
                            disabled: disabled,
                            style: "padding: 0.375rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white;",
                        }
                        input {
                            r#type: "text",
                            value: "{stat.abbreviation}",
                            oninput: move |e| {
                                let mut cfg = local_config.read().clone();
                                if let Some(s) = cfg.stat_definitions.get_mut(i) {
                                    s.abbreviation = e.value();
                                }
                                local_config.set(cfg.clone());
                                on_change.call(cfg);
                            },
                            disabled: disabled,
                            style: "padding: 0.375rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; text-align: center;",
                        }
                        input {
                            r#type: "number",
                            value: "{stat.min_value}",
                            oninput: move |e| {
                                let mut cfg = local_config.read().clone();
                                if let Some(s) = cfg.stat_definitions.get_mut(i) {
                                    s.min_value = e.value().parse().unwrap_or(0);
                                }
                                local_config.set(cfg.clone());
                                on_change.call(cfg);
                            },
                            disabled: disabled,
                            style: "padding: 0.375rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; text-align: center;",
                        }
                        input {
                            r#type: "number",
                            value: "{stat.max_value}",
                            oninput: move |e| {
                                let mut cfg = local_config.read().clone();
                                if let Some(s) = cfg.stat_definitions.get_mut(i) {
                                    s.max_value = e.value().parse().unwrap_or(20);
                                }
                                local_config.set(cfg.clone());
                                on_change.call(cfg);
                            },
                            disabled: disabled,
                            style: "padding: 0.375rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; text-align: center;",
                        }
                        input {
                            r#type: "number",
                            value: "{stat.default_value}",
                            oninput: move |e| {
                                let mut cfg = local_config.read().clone();
                                if let Some(s) = cfg.stat_definitions.get_mut(i) {
                                    s.default_value = e.value().parse().unwrap_or(10);
                                }
                                local_config.set(cfg.clone());
                                on_change.call(cfg);
                            },
                            disabled: disabled,
                            style: "padding: 0.375rem; background: #1a1a2e; border: 1px solid #374151; border-radius: 0.25rem; color: white; text-align: center;",
                        }
                        button {
                            onclick: move |_| {
                                let mut cfg = local_config.read().clone();
                                cfg.stat_definitions.remove(i);
                                local_config.set(cfg.clone());
                                on_change.call(cfg);
                            },
                            disabled: disabled,
                            style: "padding: 0.25rem 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                            "X"
                        }
                    }
                }

                // Add stat button
                button {
                    onclick: move |_| {
                        let mut cfg = local_config.read().clone();
                        cfg.stat_definitions.push(StatDefinition {
                            name: "New Stat".to_string(),
                            abbreviation: "NEW".to_string(),
                            min_value: 1,
                            max_value: 20,
                            default_value: 10,
                        });
                        local_config.set(cfg.clone());
                        on_change.call(cfg);
                    },
                    disabled: disabled,
                    style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                    "+ Add Stat"
                }
            }
        }
    }
}

/// Fetch list of worlds from API
async fn fetch_worlds() -> Result<Vec<WorldSummary>, String> {
    let base_url = "http://localhost:3000";
    let url = format!("{}/api/worlds", base_url);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            return Err(format!("Server error: {}", response.status()));
        }

        let data: Vec<WorldSummary> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let data: Vec<WorldSummary> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data)
    }
}

/// Fetch a full world snapshot by ID
async fn fetch_world_snapshot(world_id: &str) -> Result<WorldSnapshot, String> {
    let base_url = "http://localhost:3000";
    let url = format!("{}/api/worlds/{}/export/raw", base_url, world_id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            return Err(format!("Server error: {}", response.status()));
        }

        let data: WorldSnapshot = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse world data: {}", e))?;

        Ok(data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let data: WorldSnapshot = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse world data: {}", e))?;

        Ok(data)
    }
}

/// Fetch a preset configuration from the API
async fn fetch_preset(variant: &RuleSystemVariant) -> Result<RuleSystemConfig, String> {
    let base_url = "http://localhost:3000";
    let system_type = match variant {
        RuleSystemVariant::Dnd5e | RuleSystemVariant::Pathfinder2e | RuleSystemVariant::GenericD20 => "D20",
        RuleSystemVariant::CallOfCthulhu7e | RuleSystemVariant::RuneQuest | RuleSystemVariant::GenericD100 => "D100",
        RuleSystemVariant::KidsOnBikes | RuleSystemVariant::FateCore | RuleSystemVariant::PoweredByApocalypse => "Narrative",
        RuleSystemVariant::Custom(_) => "Custom",
    };
    let variant_str = format!("{:?}", variant);
    let url = format!("{}/api/rule-systems/{}/presets/{}", base_url, system_type, variant_str);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            return Err(format!("Server error: {}", response.status()));
        }

        let data: RuleSystemConfig = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse preset: {}", e))?;

        Ok(data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let data: RuleSystemConfig = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse preset: {}", e))?;

        Ok(data)
    }
}

/// Create a new world with full rule system configuration
async fn create_world(
    name: &str,
    description: Option<&str>,
    rule_system: Option<RuleSystemConfig>,
) -> Result<String, String> {
    let base_url = "http://localhost:3000";
    let url = format!("{}/api/worlds", base_url);

    #[derive(Serialize)]
    struct CreateWorldRequest {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rule_system: Option<RuleSystemConfig>,
    }

    #[derive(Deserialize)]
    struct CreateWorldResponse {
        id: String,
    }

    let body = CreateWorldRequest {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        rule_system,
    };

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            return Err(format!("Server error: {}", response.status()));
        }

        let data: CreateWorldResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data.id)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let data: CreateWorldResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data.id)
    }
}
