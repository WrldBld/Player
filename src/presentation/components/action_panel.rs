//! Action panel component
//!
//! Displays available interactions and system buttons (inventory, character sheet, etc.)

use dioxus::prelude::*;

use crate::infrastructure::websocket::InteractionData;

/// Props for the ActionPanel component
#[derive(Props, Clone, PartialEq)]
pub struct ActionPanelProps {
    /// Available scene interactions
    #[props(default)]
    pub interactions: Vec<InteractionData>,
    /// Handler for interaction selection
    pub on_interaction: EventHandler<InteractionData>,
    /// Handler for inventory button
    #[props(default)]
    pub on_inventory: Option<EventHandler<()>>,
    /// Handler for character sheet button
    #[props(default)]
    pub on_character: Option<EventHandler<()>>,
    /// Handler for map button
    #[props(default)]
    pub on_map: Option<EventHandler<()>>,
    /// Handler for log button
    #[props(default)]
    pub on_log: Option<EventHandler<()>>,
}

/// Action panel - displays system buttons and scene interactions
#[component]
pub fn ActionPanel(props: ActionPanelProps) -> Element {
    let available_interactions: Vec<_> = props
        .interactions
        .iter()
        .filter(|i| i.is_available)
        .collect();

    rsx! {
        div {
            class: "action-panel",
            style: "position: absolute; bottom: 1rem; left: 1rem; display: flex; flex-wrap: wrap; gap: 0.5rem; z-index: 20;",

            // System buttons
            if let Some(ref handler) = props.on_inventory {
                SystemButton {
                    label: "Inventory",
                    icon: "bag",
                    on_click: handler.clone(),
                }
            }

            if let Some(ref handler) = props.on_character {
                SystemButton {
                    label: "Character",
                    icon: "person",
                    on_click: handler.clone(),
                }
            }

            if let Some(ref handler) = props.on_map {
                SystemButton {
                    label: "Map",
                    icon: "map",
                    on_click: handler.clone(),
                }
            }

            if let Some(ref handler) = props.on_log {
                SystemButton {
                    label: "Log",
                    icon: "scroll",
                    on_click: handler.clone(),
                }
            }

            // Divider between system and scene actions
            if !available_interactions.is_empty() {
                div {
                    class: "action-divider",
                    style: "width: 1px; height: 32px; background: #374151; margin: 0 0.25rem;",
                }
            }

            // Scene-specific interactions
            for interaction in available_interactions {
                InteractionButton {
                    key: "{interaction.id}",
                    interaction: interaction.clone(),
                    on_click: props.on_interaction.clone(),
                }
            }
        }
    }
}

/// Props for SystemButton
#[derive(Props, Clone, PartialEq)]
pub struct SystemButtonProps {
    /// Button label
    pub label: &'static str,
    /// Icon name (for future icon system)
    pub icon: &'static str,
    /// Click handler
    pub on_click: EventHandler<()>,
}

/// System button (inventory, character, etc.)
#[component]
pub fn SystemButton(props: SystemButtonProps) -> Element {
    let icon_char = match props.icon {
        "bag" => "🎒",
        "person" => "📋",
        "map" => "🗺️",
        "scroll" => "📜",
        _ => "⚙️",
    };

    rsx! {
        button {
            class: "btn btn-secondary",
            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 0.75rem;",
            onclick: move |_| props.on_click.call(()),

            span { "{icon_char}" }
            span { "{props.label}" }
        }
    }
}

/// Props for InteractionButton
#[derive(Props, Clone, PartialEq)]
pub struct InteractionButtonProps {
    /// The interaction to display
    pub interaction: InteractionData,
    /// Click handler
    pub on_click: EventHandler<InteractionData>,
}

/// Scene interaction button
#[component]
pub fn InteractionButton(props: InteractionButtonProps) -> Element {
    let icon = get_interaction_icon(&props.interaction.interaction_type);
    let interaction = props.interaction.clone();

    rsx! {
        button {
            class: "btn btn-secondary",
            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 0.75rem;",
            onclick: move |_| props.on_click.call(interaction.clone()),

            span { "{icon}" }
            span { "{props.interaction.name}" }

            // Show target if available
            if let Some(ref target) = props.interaction.target_name {
                span {
                    style: "color: #9ca3af; font-size: 0.75rem;",
                    "({target})"
                }
            }
        }
    }
}

/// Get an emoji icon for the interaction type
fn get_interaction_icon(interaction_type: &str) -> &'static str {
    match interaction_type.to_lowercase().as_str() {
        "talk" | "dialogue" | "speak" => "💬",
        "examine" | "look" | "inspect" => "🔍",
        "use" | "interact" => "✋",
        "travel" | "go" | "move" => "🚪",
        "take" | "pickup" | "grab" => "📥",
        "give" | "offer" => "📤",
        "attack" | "fight" | "combat" => "⚔️",
        "buy" | "purchase" | "shop" => "💰",
        "sell" => "🏷️",
        "rest" | "sleep" => "😴",
        "read" => "📖",
        "open" => "📦",
        "close" => "📪",
        "lock" | "unlock" => "🔐",
        _ => "✨",
    }
}

/// Compact action panel for mobile or smaller views
#[derive(Props, Clone, PartialEq)]
pub struct CompactActionPanelProps {
    /// Handler for menu button
    pub on_menu: EventHandler<()>,
}

#[component]
pub fn CompactActionPanel(props: CompactActionPanelProps) -> Element {
    rsx! {
        div {
            class: "compact-action-panel",
            style: "position: absolute; bottom: 1rem; right: 1rem; z-index: 20;",

            button {
                class: "btn btn-primary",
                style: "width: 48px; height: 48px; border-radius: 50%; font-size: 1.5rem;",
                onclick: move |_| props.on_menu.call(()),

                "≡"
            }
        }
    }
}
