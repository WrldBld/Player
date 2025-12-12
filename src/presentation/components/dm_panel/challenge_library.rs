//! Challenge Library - Browse and manage challenges for a world
//!
//! Displays challenges organized by type with options to:
//! - View all challenges with filtering
//! - Quick access to favorites
//! - Create, edit, and delete challenges
//! - Toggle active/favorite status

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::infrastructure::asset_loader::{
    ChallengeData, ChallengeType, ChallengeDifficulty, SkillData,
};

/// Props for ChallengeLibrary
#[derive(Props, Clone, PartialEq)]
pub struct ChallengeLibraryProps {
    /// The world ID to manage challenges for
    pub world_id: String,
    /// Skills for this world (for skill name lookup)
    pub skills: Vec<SkillData>,
    /// Called when the panel should close
    pub on_close: EventHandler<()>,
    /// Called when a challenge should be triggered
    pub on_trigger_challenge: Option<EventHandler<ChallengeData>>,
}

/// Challenge Library component
#[component]
pub fn ChallengeLibrary(props: ChallengeLibraryProps) -> Element {
    let mut challenges: Signal<Vec<ChallengeData>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut filter_type: Signal<Option<ChallengeType>> = use_signal(|| None);
    let mut search_query = use_signal(String::new);
    let mut show_only_favorites = use_signal(|| false);
    let mut show_only_active = use_signal(|| false);
    let mut show_create_form = use_signal(|| false);
    let mut editing_challenge: Signal<Option<ChallengeData>> = use_signal(|| None);
    let mut show_delete_confirmation: Signal<Option<String>> = use_signal(|| None);
    let mut is_deleting = use_signal(|| false);

    // Build skill lookup map
    let skills_map: HashMap<String, String> = props
        .skills
        .iter()
        .map(|s| (s.id.clone(), s.name.clone()))
        .collect();

    let world_id = props.world_id.clone();
    let world_id_for_effect = world_id.clone();

    // Load challenges on mount
    use_effect(move || {
        let world_id = world_id_for_effect.clone();
        spawn(async move {
            match fetch_challenges(&world_id).await {
                Ok(list) => {
                    challenges.set(list);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e));
                    is_loading.set(false);
                }
            }
        });
    });

    // Filter challenges based on current filters
    let filtered_challenges: Vec<ChallengeData> = {
        let all_challenges = challenges.read();
        let search = search_query.read().to_lowercase();
        let type_filter = *filter_type.read();
        let favorites_only = *show_only_favorites.read();
        let active_only = *show_only_active.read();

        all_challenges
            .iter()
            .filter(|c| {
                // Type filter
                if let Some(t) = type_filter {
                    if c.challenge_type != t {
                        return false;
                    }
                }
                // Favorites filter
                if favorites_only && !c.is_favorite {
                    return false;
                }
                // Active filter
                if active_only && !c.active {
                    return false;
                }
                // Search filter
                if !search.is_empty() {
                    let name_match = c.name.to_lowercase().contains(&search);
                    let desc_match = c.description.to_lowercase().contains(&search);
                    let tag_match = c.tags.iter().any(|t| t.to_lowercase().contains(&search));
                    if !name_match && !desc_match && !tag_match {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    };

    // Group challenges by type for display
    let challenges_by_type: HashMap<ChallengeType, Vec<ChallengeData>> = {
        let mut grouped: HashMap<ChallengeType, Vec<ChallengeData>> = HashMap::new();
        for challenge in filtered_challenges.iter() {
            grouped
                .entry(challenge.challenge_type)
                .or_default()
                .push(challenge.clone());
        }
        // Sort by order within each type
        for challenges_vec in grouped.values_mut() {
            challenges_vec.sort_by(|a, b| {
                // Favorites first, then by order
                b.is_favorite.cmp(&a.is_favorite).then(a.order.cmp(&b.order))
            });
        }
        grouped
    };

    let handle_toggle_favorite = move |challenge_id: String| {
        let id = challenge_id.clone();
        spawn(async move {
            // Save original state for rollback
            let mut challenges_write = challenges.write();
            let original_state = challenges_write.iter().find(|c| c.id == id).map(|c| c.is_favorite);

            if let Some(c) = challenges_write.iter_mut().find(|c| c.id == id) {
                c.is_favorite = !c.is_favorite;
            }
            drop(challenges_write);

            // Call API
            match toggle_challenge_favorite(&id).await {
                Ok(is_favorite) => {
                    // Update with confirmed state from server
                    let mut challenges_write = challenges.write();
                    if let Some(c) = challenges_write.iter_mut().find(|c| c.id == id) {
                        c.is_favorite = is_favorite;
                    }
                }
                Err(_) => {
                    // Rollback on error
                    let mut challenges_write = challenges.write();
                    if let Some(c) = challenges_write.iter_mut().find(|c| c.id == id) {
                        if let Some(original) = original_state {
                            c.is_favorite = original;
                        }
                    }
                }
            }
        });
    };

    let handle_toggle_active = move |challenge_id: String| {
        let id = challenge_id.clone();
        spawn(async move {
            // Save original state for rollback
            let mut challenges_write = challenges.write();
            let original_active = challenges_write.iter().find(|c| c.id == id).map(|c| c.active);

            if let Some(c) = challenges_write.iter_mut().find(|c| c.id == id) {
                c.active = !c.active;
            }
            drop(challenges_write);

            let new_active = match original_active {
                Some(was_active) => !was_active,
                None => true,
            };

            // Call API
            match set_challenge_active(&id, new_active).await {
                Ok(()) => {
                    // State already updated optimistically, confirmed by server
                }
                Err(_) => {
                    // Rollback on error
                    let mut challenges_write = challenges.write();
                    if let Some(c) = challenges_write.iter_mut().find(|c| c.id == id) {
                        if let Some(original) = original_active {
                            c.active = original;
                        }
                    }
                }
            }
        });
    };

    let handle_delete = move |challenge_id: String| {
        show_delete_confirmation.set(Some(challenge_id));
    };

    let do_delete = move |_| {
        if let Some(challenge_id) = show_delete_confirmation.read().clone() {
            let id = challenge_id.clone();
            spawn(async move {
                is_deleting.set(true);
                if delete_challenge(&id).await.is_ok() {
                    challenges.write().retain(|c| c.id != id);
                    show_delete_confirmation.set(None);
                } else {
                    is_deleting.set(false);
                }
            });
        }
    };

    let cancel_delete = move |_| {
        show_delete_confirmation.set(None);
        is_deleting.set(false);
    };

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.85); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_close.call(()),

            div {
                style: "background: #1a1a2e; border-radius: 0.75rem; width: 95%; max-width: 1000px; max-height: 90vh; overflow: hidden; display: flex; flex-direction: column;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem 1.5rem; border-bottom: 1px solid #374151; background: rgba(0,0,0,0.2);",

                    h2 { style: "color: white; margin: 0; font-size: 1.25rem;", "Challenge Library" }

                    div { style: "display: flex; gap: 0.75rem; align-items: center;",
                        button {
                            onclick: move |_| show_create_form.set(true),
                            style: "padding: 0.5rem 1rem; background: #10b981; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                            "+ New Challenge"
                        }

                        button {
                            onclick: move |_| props.on_close.call(()),
                            style: "padding: 0.5rem; background: transparent; border: none; color: #9ca3af; cursor: pointer; font-size: 1.5rem;",
                            "×"
                        }
                    }
                }

                // Filters bar
                div {
                    style: "padding: 0.75rem 1.5rem; border-bottom: 1px solid #374151; display: flex; flex-wrap: wrap; gap: 0.75rem; align-items: center;",

                    // Search input
                    input {
                        r#type: "text",
                        placeholder: "Search challenges...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                        style: "padding: 0.5rem 0.75rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; flex: 1; min-width: 200px;",
                    }

                    // Type filter dropdown
                    {
                        let type_value = match *filter_type.read() {
                            Some(t) => format!("{:?}", t),
                            None => String::new(),
                        };
                        rsx! {
                    select {
                        value: "{type_value}",
                        onchange: move |e| {
                            let val = e.value();
                            filter_type.set(if val.is_empty() {
                                None
                            } else {
                                match val.as_str() {
                                    "SkillCheck" => Some(ChallengeType::SkillCheck),
                                    "AbilityCheck" => Some(ChallengeType::AbilityCheck),
                                    "SavingThrow" => Some(ChallengeType::SavingThrow),
                                    "OpposedCheck" => Some(ChallengeType::OpposedCheck),
                                    "ComplexChallenge" => Some(ChallengeType::ComplexChallenge),
                                    _ => None,
                                }
                            });
                        },
                        style: "padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white;",
                        option { value: "", "All Types" }
                        for challenge_type in ChallengeType::all() {
                            option { value: "{challenge_type:?}", "{challenge_type.display_name()}" }
                        }
                    }
                        }
                    }

                    // Toggle buttons
                    label { style: "display: flex; align-items: center; gap: 0.375rem; color: #9ca3af; font-size: 0.75rem; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: *show_only_favorites.read(),
                            onchange: move |e| show_only_favorites.set(e.checked()),
                        }
                        "Favorites"
                    }

                    label { style: "display: flex; align-items: center; gap: 0.375rem; color: #9ca3af; font-size: 0.75rem; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: *show_only_active.read(),
                            onchange: move |e| show_only_active.set(e.checked()),
                        }
                        "Active Only"
                    }
                }

                // Error message
                if let Some(err) = error.read().as_ref() {
                    div {
                        style: "padding: 0.75rem 1.5rem; background: rgba(239, 68, 68, 0.1); color: #ef4444; font-size: 0.875rem;",
                        "{err}"
                    }
                }

                // Content
                div {
                    style: "flex: 1; overflow-y: auto; padding: 1rem 1.5rem;",

                    if *is_loading.read() {
                        div {
                            style: "display: flex; align-items: center; justify-content: center; padding: 3rem; color: #9ca3af;",
                            "Loading challenges..."
                        }
                    } else if filtered_challenges.is_empty() {
                        div {
                            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 3rem; color: #6b7280; text-align: center;",
                            div { style: "font-size: 2rem; margin-bottom: 0.5rem;", "🎲" }
                            p { style: "margin: 0;", "No challenges found" }
                            if !search_query.read().is_empty() || filter_type.read().is_some() {
                                p { style: "margin: 0.5rem 0 0 0; font-size: 0.875rem;", "Try adjusting your filters" }
                            } else {
                                button {
                                    onclick: move |_| show_create_form.set(true),
                                    style: "margin-top: 1rem; padding: 0.5rem 1rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                                    "Create Your First Challenge"
                                }
                            }
                        }
                    } else {
                        div { style: "display: flex; flex-direction: column; gap: 1.5rem;",
                            // Render by type
                            for challenge_type in ChallengeType::all() {
                                if let Some(type_challenges) = challenges_by_type.get(&challenge_type) {
                                    if !type_challenges.is_empty() {
                                        ChallengeTypeSection {
                                            key: "{challenge_type:?}",
                                            challenge_type: challenge_type,
                                            challenges: type_challenges.clone(),
                                            skills_map: skills_map.clone(),
                                            on_toggle_favorite: handle_toggle_favorite.clone(),
                                            on_toggle_active: handle_toggle_active.clone(),
                                            on_edit: {
                                                let mut editing = editing_challenge.clone();
                                                move |c: ChallengeData| editing.set(Some(c))
                                            },
                                            on_delete: handle_delete.clone(),
                                            on_trigger: props.on_trigger_challenge.clone(),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Create form modal (overlay on top)
            if *show_create_form.read() {
                ChallengeFormModal {
                    world_id: world_id.clone(),
                    challenge: None,
                    skills: props.skills.clone(),
                    on_save: {
                        let mut challenges = challenges.clone();
                        move |challenge: ChallengeData| {
                            challenges.write().push(challenge);
                            show_create_form.set(false);
                        }
                    },
                    on_close: move |_| show_create_form.set(false),
                }
            }

            // Edit form modal
            if let Some(challenge) = editing_challenge.read().clone() {
                ChallengeFormModal {
                    world_id: world_id.clone(),
                    challenge: Some(challenge.clone()),
                    skills: props.skills.clone(),
                    on_save: {
                        let mut challenges = challenges.clone();
                        let challenge_id = challenge.id.clone();
                        move |updated: ChallengeData| {
                            let mut write = challenges.write();
                            if let Some(c) = write.iter_mut().find(|c| c.id == challenge_id) {
                                *c = updated;
                            }
                            editing_challenge.set(None);
                        }
                    },
                    on_close: move |_| editing_challenge.set(None),
                }
            }

            // Delete confirmation modal
            if let Some(challenge_id) = show_delete_confirmation.read().clone() {
                if let Some(challenge) = challenges.read().iter().find(|c| c.id == challenge_id).cloned() {
                    ConfirmDeleteChallengeModal {
                        challenge_name: challenge.name.clone(),
                        is_deleting: *is_deleting.read(),
                        on_confirm: do_delete,
                        on_cancel: cancel_delete,
                    }
                }
            }
        }
    }
}

/// Section for a challenge type
#[derive(Props, Clone, PartialEq)]
struct ChallengeTypeSectionProps {
    challenge_type: ChallengeType,
    challenges: Vec<ChallengeData>,
    skills_map: HashMap<String, String>,
    on_toggle_favorite: EventHandler<String>,
    on_toggle_active: EventHandler<String>,
    on_edit: EventHandler<ChallengeData>,
    on_delete: EventHandler<String>,
    on_trigger: Option<EventHandler<ChallengeData>>,
}

#[component]
fn ChallengeTypeSection(props: ChallengeTypeSectionProps) -> Element {
    let mut is_collapsed = use_signal(|| false);

    rsx! {
        div {
            style: "background: rgba(0,0,0,0.2); border-radius: 0.5rem; overflow: hidden;",

            // Section header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1rem; background: rgba(0,0,0,0.3); cursor: pointer;",
                onclick: move |_| {
                    let current = *is_collapsed.read();
                    is_collapsed.set(!current);
                },

                div { style: "display: flex; align-items: center; gap: 0.5rem;",
                    h3 { style: "color: #e5e7eb; margin: 0; font-size: 0.875rem; font-weight: 600;",
                        "{props.challenge_type.display_name()}"
                    }
                    span { style: "color: #6b7280; font-size: 0.75rem;",
                        "({props.challenges.len()})"
                    }
                }

                span { style: "color: #6b7280;",
                    if *is_collapsed.read() { "▶" } else { "▼" }
                }
            }

            // Challenge cards
            if !*is_collapsed.read() {
                div { style: "padding: 0.75rem; display: flex; flex-direction: column; gap: 0.5rem;",
                    for challenge in props.challenges.iter() {
                        ChallengeCard {
                            key: "{challenge.id}",
                            challenge: challenge.clone(),
                            skill_name: props.skills_map.get(&challenge.skill_id).cloned().unwrap_or_else(|| "Unknown".to_string()),
                            on_toggle_favorite: props.on_toggle_favorite.clone(),
                            on_toggle_active: props.on_toggle_active.clone(),
                            on_edit: props.on_edit.clone(),
                            on_delete: props.on_delete.clone(),
                            on_trigger: props.on_trigger.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Individual challenge card
#[derive(Props, Clone, PartialEq)]
struct ChallengeCardProps {
    challenge: ChallengeData,
    skill_name: String,
    on_toggle_favorite: EventHandler<String>,
    on_toggle_active: EventHandler<String>,
    on_edit: EventHandler<ChallengeData>,
    on_delete: EventHandler<String>,
    on_trigger: Option<EventHandler<ChallengeData>>,
}

#[component]
fn ChallengeCard(props: ChallengeCardProps) -> Element {
    let challenge = props.challenge.clone();
    let id = challenge.id.clone();
    let id_for_favorite = id.clone();
    let id_for_active = id.clone();
    let id_for_delete = id.clone();
    let challenge_for_edit = challenge.clone();
    let challenge_for_trigger = challenge.clone();

    let opacity = if challenge.active { "1" } else { "0.6" };
    let border_color = if challenge.is_favorite { "#f59e0b" } else { "#374151" };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem; background: #0f0f23; border: 1px solid {border_color}; border-radius: 0.375rem; opacity: {opacity};",

            // Favorite star
            button {
                onclick: move |_| props.on_toggle_favorite.call(id_for_favorite.clone()),
                style: "background: none; border: none; cursor: pointer; font-size: 1rem; padding: 0;",
                if challenge.is_favorite { "⭐" } else { "☆" }
            }

            // Main info
            div { style: "flex: 1; min-width: 0;",
                div { style: "display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.25rem;",
                    span { style: "color: white; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{challenge.name}"
                    }
                    span { style: "color: #9ca3af; font-size: 0.75rem;",
                        "{challenge.difficulty.display()}"
                    }
                }
                div { style: "display: flex; gap: 0.5rem; flex-wrap: wrap;",
                    span { style: "color: #60a5fa; font-size: 0.75rem;",
                        "{props.skill_name}"
                    }
                    if !challenge.description.is_empty() {
                        span { style: "color: #6b7280; font-size: 0.75rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 200px;",
                            "{challenge.description}"
                        }
                    }
                }
            }

            // Tags
            div { style: "display: flex; gap: 0.25rem; flex-wrap: wrap;",
                for tag in challenge.tags.iter().take(2) {
                    span {
                        style: "padding: 0.125rem 0.375rem; background: #374151; color: #9ca3af; font-size: 0.625rem; border-radius: 0.25rem;",
                        "{tag}"
                    }
                }
                if challenge.tags.len() > 2 {
                    {
                        let extra = challenge.tags.len() - 2;
                        rsx! { span { style: "color: #6b7280; font-size: 0.625rem;", "+{extra}" } }
                    }
                }
            }

            // Active toggle
            {
                let active_bg = if challenge.active { "#10b981" } else { "#374151" };
                let active_text = if challenge.active { "Active" } else { "Inactive" };
                rsx! {
            button {
                onclick: move |_| props.on_toggle_active.call(id_for_active.clone()),
                style: "padding: 0.25rem 0.5rem; background: {active_bg}; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.625rem;",
                "{active_text}"
            }
                }
            }

            // Actions
            div { style: "display: flex; gap: 0.25rem;",
                // Trigger button (only if handler provided)
                if let Some(ref on_trigger) = props.on_trigger {
                    button {
                        onclick: {
                            let trigger = on_trigger.clone();
                            let c = challenge_for_trigger.clone();
                            move |_| trigger.call(c.clone())
                        },
                        style: "padding: 0.375rem 0.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                        "▶"
                    }
                }

                button {
                    onclick: move |_| props.on_edit.call(challenge_for_edit.clone()),
                    style: "padding: 0.375rem 0.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                    "Edit"
                }

                button {
                    onclick: move |_| props.on_delete.call(id_for_delete.clone()),
                    style: "padding: 0.375rem 0.5rem; background: #ef4444; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;",
                    "×"
                }
            }
        }
    }
}

/// Props for ChallengeFormModal
#[derive(Props, Clone, PartialEq)]
struct ChallengeFormModalProps {
    world_id: String,
    challenge: Option<ChallengeData>,
    skills: Vec<SkillData>,
    on_save: EventHandler<ChallengeData>,
    on_close: EventHandler<()>,
}

/// Modal for creating/editing a challenge
#[component]
fn ChallengeFormModal(props: ChallengeFormModalProps) -> Element {
    let is_edit = props.challenge.is_some();
    let initial = props.challenge.clone().unwrap_or_default_challenge(&props.world_id);

    let mut name = use_signal(|| initial.name.clone());
    let mut description = use_signal(|| initial.description.clone());
    let mut skill_id = use_signal(|| initial.skill_id.clone());
    let mut challenge_type = use_signal(|| initial.challenge_type);
    let mut difficulty = use_signal(|| initial.difficulty.clone());
    let mut success_desc = use_signal(|| initial.outcomes.success.description.clone());
    let mut failure_desc = use_signal(|| initial.outcomes.failure.description.clone());
    let mut tags_str = use_signal(|| initial.tags.join(", "));
    let mut is_saving = use_signal(|| false);
    let mut save_error: Signal<Option<String>> = use_signal(|| None);
    let mut validation_errors: Signal<Vec<String>> = use_signal(Vec::new);

    let challenge_id = initial.id.clone();
    let world_id = props.world_id.clone();

    let handle_save = move |_| {
        // Validate inputs
        let mut errors = Vec::new();

        if name.read().trim().is_empty() {
            errors.push("Challenge name is required".to_string());
        }

        if skill_id.read().is_empty() {
            errors.push("Skill is required".to_string());
        }

        // Validate difficulty-specific values
        match &*difficulty.read() {
            ChallengeDifficulty::Dc { value } => {
                if *value <= 0 {
                    errors.push("DC value must be greater than 0".to_string());
                }
            }
            ChallengeDifficulty::Percentage { value } => {
                if *value < 0 || *value > 100 {
                    errors.push("Percentage must be between 0 and 100".to_string());
                }
            }
            _ => {}
        }

        if !errors.is_empty() {
            validation_errors.set(errors);
            return;
        }

        validation_errors.set(Vec::new());
        is_saving.set(true);
        save_error.set(None);

        let challenge_data = ChallengeData {
            id: challenge_id.clone(),
            world_id: world_id.clone(),
            scene_id: None,
            name: name.read().clone(),
            description: description.read().clone(),
            challenge_type: *challenge_type.read(),
            skill_id: skill_id.read().clone(),
            difficulty: difficulty.read().clone(),
            outcomes: ChallengeOutcomes {
                success: crate::infrastructure::asset_loader::Outcome {
                    description: success_desc.read().clone(),
                    triggers: vec![],
                },
                failure: crate::infrastructure::asset_loader::Outcome {
                    description: failure_desc.read().clone(),
                    triggers: vec![],
                },
                partial: None,
                critical_success: None,
                critical_failure: None,
            },
            trigger_conditions: vec![],
            prerequisite_challenges: vec![],
            active: true,
            order: 0,
            is_favorite: false,
            tags: tags_str
                .read()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        };

        let on_save = props.on_save.clone();
        let is_edit = is_edit;

        spawn(async move {
            let result = if is_edit {
                update_challenge(&challenge_data).await
            } else {
                create_challenge(&challenge_data).await
            };

            match result {
                Ok(saved) => {
                    on_save.call(saved);
                }
                Err(e) => {
                    save_error.set(Some(e));
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.9); display: flex; align-items: center; justify-content: center; z-index: 1100;",
            onclick: move |_| props.on_close.call(()),

            div {
                style: "background: #1a1a2e; border-radius: 0.75rem; width: 90%; max-width: 600px; max-height: 90vh; overflow-y: auto;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 1rem 1.5rem; border-bottom: 1px solid #374151;",
                    h3 { style: "color: white; margin: 0;",
                        if is_edit { "Edit Challenge" } else { "New Challenge" }
                    }
                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "background: none; border: none; color: #9ca3af; font-size: 1.5rem; cursor: pointer;",
                        "×"
                    }
                }

                // Form
                div {
                    style: "padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;",

                    // Validation errors
                    if !validation_errors.read().is_empty() {
                        div {
                            style: "padding: 0.75rem 1rem; background: rgba(239, 68, 68, 0.1); border-left: 3px solid #ef4444; border-radius: 0.375rem; color: #ef4444; font-size: 0.875rem;",
                            for error in validation_errors.read().iter() {
                                div { style: "margin-bottom: 0.25rem;", "• {error}" }
                            }
                        }
                    }

                    // Name
                    div {
                        label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Name *" }
                        input {
                            r#type: "text",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                            placeholder: "e.g., Investigate the Crime Scene",
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; box-sizing: border-box;",
                        }
                    }

                    // Description
                    div {
                        label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Description" }
                        textarea {
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                            placeholder: "What this challenge represents...",
                            rows: "2",
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; resize: vertical; box-sizing: border-box;",
                        }
                    }

                    // Type and Skill row
                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;",
                        // Type
                        div {
                            label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Type" }
                            select {
                                value: "{challenge_type.read():?}",
                                onchange: move |e| {
                                    let val = e.value();
                                    challenge_type.set(match val.as_str() {
                                        "SkillCheck" => ChallengeType::SkillCheck,
                                        "AbilityCheck" => ChallengeType::AbilityCheck,
                                        "SavingThrow" => ChallengeType::SavingThrow,
                                        "OpposedCheck" => ChallengeType::OpposedCheck,
                                        "ComplexChallenge" => ChallengeType::ComplexChallenge,
                                        _ => ChallengeType::SkillCheck,
                                    });
                                },
                                style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white;",
                                for ct in ChallengeType::all() {
                                    option { value: "{ct:?}", "{ct.display_name()}" }
                                }
                            }
                        }

                        // Skill
                        div {
                            label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Skill *" }
                            select {
                                value: "{skill_id}",
                                onchange: move |e| skill_id.set(e.value()),
                                style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white;",
                                option { value: "", "Select a skill..." }
                                for skill in props.skills.iter() {
                                    option { value: "{skill.id}", "{skill.name}" }
                                }
                            }
                        }
                    }

                    // Difficulty
                    div {
                        label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Difficulty" }
                        div { style: "display: flex; gap: 0.5rem;",
                            select {
                                value: match &*difficulty.read() {
                                    ChallengeDifficulty::Dc { .. } => "dc",
                                    ChallengeDifficulty::Percentage { .. } => "percentage",
                                    ChallengeDifficulty::Descriptor { .. } => "descriptor",
                                    ChallengeDifficulty::Opposed => "opposed",
                                    ChallengeDifficulty::Custom { .. } => "custom",
                                },
                                onchange: move |e| {
                                    difficulty.set(match e.value().as_str() {
                                        "dc" => ChallengeDifficulty::Dc { value: 10 },
                                        "percentage" => ChallengeDifficulty::Percentage { value: 50 },
                                        "descriptor" => ChallengeDifficulty::Descriptor { value: "Moderate".to_string() },
                                        "opposed" => ChallengeDifficulty::Opposed,
                                        "custom" => ChallengeDifficulty::Custom { value: "".to_string() },
                                        _ => ChallengeDifficulty::Dc { value: 10 },
                                    });
                                },
                                style: "padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white;",
                                option { value: "dc", "DC" }
                                option { value: "percentage", "Percentage" }
                                option { value: "descriptor", "Descriptor" }
                                option { value: "opposed", "Opposed" }
                                option { value: "custom", "Custom" }
                            }

                            // Value input (for DC/Percentage/Descriptor/Custom)
                            match &*difficulty.read() {
                                ChallengeDifficulty::Dc { value } => rsx! {
                                    input {
                                        r#type: "number",
                                        value: "{value}",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse() {
                                                difficulty.set(ChallengeDifficulty::Dc { value: v });
                                            }
                                        },
                                        style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white;",
                                    }
                                },
                                ChallengeDifficulty::Percentage { value } => rsx! {
                                    input {
                                        r#type: "number",
                                        value: "{value}",
                                        min: "0",
                                        max: "100",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse() {
                                                difficulty.set(ChallengeDifficulty::Percentage { value: v });
                                            }
                                        },
                                        style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white;",
                                    }
                                },
                                ChallengeDifficulty::Descriptor { value } => rsx! {
                                    select {
                                        value: "{value}",
                                        onchange: move |e| {
                                            difficulty.set(ChallengeDifficulty::Descriptor { value: e.value() });
                                        },
                                        style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white;",
                                        option { value: "Trivial", "Trivial" }
                                        option { value: "Easy", "Easy" }
                                        option { value: "Routine", "Routine" }
                                        option { value: "Moderate", "Moderate" }
                                        option { value: "Challenging", "Challenging" }
                                        option { value: "Hard", "Hard" }
                                        option { value: "Very Hard", "Very Hard" }
                                        option { value: "Extreme", "Extreme" }
                                    }
                                },
                                ChallengeDifficulty::Custom { value } => rsx! {
                                    input {
                                        r#type: "text",
                                        value: "{value}",
                                        placeholder: "Custom difficulty...",
                                        oninput: move |e| {
                                            difficulty.set(ChallengeDifficulty::Custom { value: e.value() });
                                        },
                                        style: "flex: 1; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white;",
                                    }
                                },
                                ChallengeDifficulty::Opposed => rsx! {
                                    span { style: "color: #6b7280; padding: 0.5rem;", "Roll vs opponent" }
                                },
                            }
                        }
                    }

                    // Success outcome
                    div {
                        label { style: "display: block; color: #10b981; font-size: 0.75rem; margin-bottom: 0.25rem;", "Success Outcome" }
                        textarea {
                            value: "{success_desc}",
                            oninput: move |e| success_desc.set(e.value()),
                            placeholder: "What happens on success...",
                            rows: "2",
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #10b981; border-radius: 0.375rem; color: white; resize: vertical; box-sizing: border-box;",
                        }
                    }

                    // Failure outcome
                    div {
                        label { style: "display: block; color: #ef4444; font-size: 0.75rem; margin-bottom: 0.25rem;", "Failure Outcome" }
                        textarea {
                            value: "{failure_desc}",
                            oninput: move |e| failure_desc.set(e.value()),
                            placeholder: "What happens on failure...",
                            rows: "2",
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #ef4444; border-radius: 0.375rem; color: white; resize: vertical; box-sizing: border-box;",
                        }
                    }

                    // Tags
                    div {
                        label { style: "display: block; color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem;", "Tags (comma-separated)" }
                        input {
                            r#type: "text",
                            value: "{tags_str}",
                            oninput: move |e| tags_str.set(e.value()),
                            placeholder: "investigation, social, combat",
                            style: "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.375rem; color: white; box-sizing: border-box;",
                        }
                    }

                    // Error
                    if let Some(err) = save_error.read().as_ref() {
                        div { style: "color: #ef4444; font-size: 0.875rem;", "{err}" }
                    }

                    // Actions
                    div { style: "display: flex; gap: 0.75rem; justify-content: flex-end; margin-top: 0.5rem;",
                        button {
                            onclick: move |_| props.on_close.call(()),
                            style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer;",
                            "Cancel"
                        }
                        {
                            let is_disabled = *is_saving.read() || name.read().is_empty() || skill_id.read().is_empty();
                            let opacity = if is_disabled { "0.5" } else { "1" };
                            let button_text = if *is_saving.read() { "Saving..." } else if is_edit { "Update" } else { "Create" };
                            rsx! {
                        button {
                            onclick: handle_save,
                            disabled: is_disabled,
                            style: "padding: 0.5rem 1rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; opacity: {opacity};",
                            "{button_text}"
                        }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Confirmation dialog for challenge deletion
#[derive(Props, Clone, PartialEq)]
struct ConfirmDeleteChallengeModalProps {
    challenge_name: String,
    is_deleting: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
fn ConfirmDeleteChallengeModal(props: ConfirmDeleteChallengeModalProps) -> Element {
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.75); display: flex; align-items: center; justify-content: center; z-index: 1101;",
            onclick: move |_| props.on_cancel.call(()),

            div {
                style: "background: #1a1a2e; border-radius: 0.75rem; width: 90%; max-width: 400px; padding: 1.5rem; overflow: hidden;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem;",

                    div {
                        style: "color: #dc2626; font-size: 1.5rem;",
                        "!"
                    }

                    h2 {
                        style: "color: #dc2626; font-size: 1.125rem; margin: 0;",
                        "Delete Challenge"
                    }
                }

                // Message
                p {
                    style: "color: #9ca3af; margin: 1rem 0;",
                    "Are you sure you want to delete \"{props.challenge_name}\"? This action cannot be undone."
                }

                // Buttons
                div {
                    style: "display: flex; gap: 0.75rem; justify-content: flex-end; margin-top: 1.5rem;",

                    button {
                        onclick: move |_| props.on_cancel.call(()),
                        disabled: props.is_deleting,
                        style: "padding: 0.5rem 1rem; background: #374151; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem;",
                        "Cancel"
                    }

                    button {
                        onclick: move |_| props.on_confirm.call(()),
                        disabled: props.is_deleting,
                        style: "padding: 0.5rem 1rem; background: #dc2626; color: white; border: none; border-radius: 0.5rem; cursor: pointer; font-size: 0.875rem; font-weight: 500;",
                        if props.is_deleting { "Deleting..." } else { "Delete Challenge" }
                    }
                }
            }
        }
    }
}

// Helper trait for default challenge
trait DefaultChallenge {
    fn unwrap_or_default_challenge(self, world_id: &str) -> ChallengeData;
}

impl DefaultChallenge for Option<ChallengeData> {
    fn unwrap_or_default_challenge(self, world_id: &str) -> ChallengeData {
        self.unwrap_or(ChallengeData {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            scene_id: None,
            name: String::new(),
            description: String::new(),
            challenge_type: ChallengeType::SkillCheck,
            skill_id: String::new(),
            difficulty: ChallengeDifficulty::default(),
            outcomes: ChallengeOutcomes::default(),
            trigger_conditions: vec![],
            prerequisite_challenges: vec![],
            active: true,
            order: 0,
            is_favorite: false,
            tags: vec![],
        })
    }
}

// ============================================================================
// API Configuration
// ============================================================================

use crate::infrastructure::asset_loader::ChallengeOutcomes;

const API_BASE_URL: &str = "http://localhost:3000";

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_challenges(world_id: &str) -> Result<Vec<ChallengeData>, String> {
    let url = format!("/api/worlds/{}/challenges", world_id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let resp = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.ok() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<Vec<ChallengeData>>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let resp = client
            .get(&format!("{}{}", API_BASE_URL, url))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<Vec<ChallengeData>>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
}

async fn create_challenge(challenge: &ChallengeData) -> Result<ChallengeData, String> {
    let url = format!("/api/worlds/{}/challenges", challenge.world_id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let resp = Request::post(&url)
            .json(challenge)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.ok() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<ChallengeData>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let resp = client
            .post(&format!("{}{}", API_BASE_URL, url))
            .json(challenge)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<ChallengeData>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
}

async fn update_challenge(challenge: &ChallengeData) -> Result<ChallengeData, String> {
    let url = format!("/api/challenges/{}", challenge.id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let resp = Request::put(&url)
            .json(challenge)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.ok() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<ChallengeData>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let resp = client
            .put(&format!("{}{}", API_BASE_URL, url))
            .json(challenge)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<ChallengeData>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
}

async fn delete_challenge(challenge_id: &str) -> Result<(), String> {
    let url = format!("/api/challenges/{}", challenge_id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let resp = Request::delete(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.ok() {
            return Err(format!("API error: {}", resp.status()));
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let resp = client
            .delete(&format!("{}{}", API_BASE_URL, url))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        Ok(())
    }
}

async fn toggle_challenge_favorite(challenge_id: &str) -> Result<bool, String> {
    let url = format!("/api/challenges/{}/favorite", challenge_id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let resp = Request::put(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.ok() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<bool>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let resp = client
            .put(&format!("{}{}", API_BASE_URL, url))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<bool>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
}

async fn set_challenge_active(challenge_id: &str, active: bool) -> Result<(), String> {
    let url = format!("/api/challenges/{}/active", challenge_id);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;
        let resp = Request::put(&url)
            .json(&active)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.ok() {
            return Err(format!("API error: {}", resp.status()));
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = reqwest::Client::new();
        let resp = client
            .put(&format!("{}{}", API_BASE_URL, url))
            .json(&active)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        Ok(())
    }
}
