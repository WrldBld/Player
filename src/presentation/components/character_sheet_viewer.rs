//! Character Sheet Viewer - Read-only display of character stats for players

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::application::dto::{
    FieldType, FieldValue, SheetField, SheetSection, SheetTemplate,
};

/// Props for the character sheet viewer
#[derive(Props, Clone, PartialEq)]
pub struct CharacterSheetViewerProps {
    /// The character's name
    pub character_name: String,
    /// The sheet template
    pub template: SheetTemplate,
    /// The character's values
    pub values: HashMap<String, FieldValue>,
    /// Handler for closing the viewer
    pub on_close: EventHandler<()>,
}

/// Character Sheet Viewer - modal overlay showing character stats
#[component]
pub fn CharacterSheetViewer(props: CharacterSheetViewerProps) -> Element {
    // Sort sections by order
    let mut sorted_sections = props.template.sections.clone();
    sorted_sections.sort_by_key(|s| s.order);

    rsx! {
        // Overlay background
        div {
            class: "character-sheet-overlay",
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.85); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 2rem;",
            onclick: move |_| props.on_close.call(()),

            // Sheet container (prevent click propagation)
            div {
                class: "character-sheet-modal",
                style: "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); border-radius: 1rem; width: 100%; max-width: 800px; max-height: 90vh; overflow: hidden; display: flex; flex-direction: column; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "sheet-header",
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 1.5rem; border-bottom: 2px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2);",

                    div {
                        h2 {
                            style: "color: #f3f4f6; font-size: 1.5rem; margin: 0; font-weight: 600;",
                            "{props.character_name}"
                        }
                        p {
                            style: "color: #9ca3af; font-size: 0.875rem; margin: 0.25rem 0 0 0;",
                            "{props.template.name}"
                        }
                    }

                    button {
                        onclick: move |_| props.on_close.call(()),
                        style: "width: 36px; height: 36px; background: rgba(255,255,255,0.1); border: none; border-radius: 0.5rem; color: #9ca3af; cursor: pointer; font-size: 1.25rem; display: flex; align-items: center; justify-content: center;",
                        "×"
                    }
                }

                // Scrollable content
                div {
                    class: "sheet-content",
                    style: "flex: 1; overflow-y: auto; padding: 1.5rem;",

                    div {
                        style: "display: flex; flex-direction: column; gap: 1.5rem;",

                        for section in sorted_sections {
                            SheetSectionViewer {
                                key: "{section.id}",
                                section: section,
                                values: props.values.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Props for a section viewer
#[derive(Props, Clone, PartialEq)]
struct SheetSectionViewerProps {
    section: SheetSection,
    values: HashMap<String, FieldValue>,
}

/// Renders a read-only section of the character sheet
#[component]
fn SheetSectionViewer(props: SheetSectionViewerProps) -> Element {
    let mut is_collapsed = use_signal(|| props.section.collapsed_by_default);

    // Determine layout style
    let content_style = match props.section.layout {
        crate::infrastructure::asset_loader::SectionLayout::Vertical => {
            "display: flex; flex-direction: column; gap: 0.5rem;"
        }
        crate::infrastructure::asset_loader::SectionLayout::Grid { columns: _ } => {
            "display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 0.75rem;"
        }
        crate::infrastructure::asset_loader::SectionLayout::TwoColumn => {
            "display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem;"
        }
        crate::infrastructure::asset_loader::SectionLayout::Flow => {
            "display: flex; flex-wrap: wrap; gap: 0.75rem;"
        }
    };

    // Sort fields by order
    let mut sorted_fields = props.section.fields.clone();
    sorted_fields.sort_by_key(|f| f.order);

    rsx! {
        div {
            class: "sheet-section",
            style: "background: rgba(0,0,0,0.2); border-radius: 0.75rem; overflow: hidden;",

            // Section header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1rem; background: rgba(0,0,0,0.3); cursor: pointer;",
                onclick: move |_| {
                    if props.section.collapsible {
                        let current = *is_collapsed.read();
                        is_collapsed.set(!current);
                    }
                },

                h3 {
                    style: "color: #e5e7eb; font-size: 0.875rem; margin: 0; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em;",
                    "{props.section.name}"
                }

                if props.section.collapsible {
                    span {
                        style: "color: #6b7280; font-size: 0.875rem;",
                        if *is_collapsed.read() { "+" } else { "−" }
                    }
                }
            }

            // Section content
            if !*is_collapsed.read() {
                div {
                    style: "padding: 1rem; {content_style}",

                    for field in sorted_fields {
                        SheetFieldViewer {
                            key: "{field.id}",
                            field: field.clone(),
                            value: props.values.get(&field.id).cloned(),
                        }
                    }
                }
            }
        }
    }
}

/// Props for a field viewer
#[derive(Props, Clone, PartialEq)]
struct SheetFieldViewerProps {
    field: SheetField,
    value: Option<FieldValue>,
}

/// Renders a single read-only field
#[component]
fn SheetFieldViewer(props: SheetFieldViewerProps) -> Element {
    let value_display = format_field_value(&props.field.field_type, &props.value);

    // Check if this is a "large" field type
    let is_resource = matches!(props.field.field_type, FieldType::Resource { .. });

    rsx! {
        div {
            class: "sheet-field",
            style: "display: flex; flex-direction: column;",

            // Label
            span {
                style: "color: #6b7280; font-size: 0.75rem; margin-bottom: 0.25rem;",
                "{props.field.name}"
            }

            // Value display
            if is_resource {
                // Resource bar display
                {
                    if let Some(FieldValue::Resource { current, max }) = &props.value {
                        let percentage = if *max > 0 { (*current as f32 / *max as f32) * 100.0 } else { 0.0 };
                        let bar_color = if percentage > 50.0 { "#22c55e" } else if percentage > 25.0 { "#f59e0b" } else { "#ef4444" };

                        rsx! {
                            div {
                                style: "display: flex; flex-direction: column; gap: 0.25rem;",

                                // Text value
                                span {
                                    style: "color: #f3f4f6; font-size: 1rem; font-weight: 500;",
                                    "{current} / {max}"
                                }

                                // Progress bar
                                div {
                                    style: "height: 6px; background: rgba(0,0,0,0.3); border-radius: 3px; overflow: hidden;",

                                    div {
                                        style: "height: 100%; width: {percentage}%; background: {bar_color}; transition: width 0.3s ease;",
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            span {
                                style: "color: #6b7280; font-size: 1rem;",
                                "—"
                            }
                        }
                    }
                }
            } else {
                // Regular value display
                span {
                    style: "color: #f3f4f6; font-size: 1.125rem; font-weight: 500;",
                    "{value_display}"
                }
            }
        }
    }
}

/// Format a field value for display
fn format_field_value(field_type: &FieldType, value: &Option<FieldValue>) -> String {
    match (field_type, value) {
        (FieldType::Number { .. }, Some(FieldValue::Number(n))) => {
            // Format with sign for modifiers
            if *n >= 0 {
                format!("+{}", n)
            } else {
                n.to_string()
            }
        }
        (FieldType::Number { default, .. }, None) => {
            default.map(|d| format!("+{}", d)).unwrap_or_else(|| "0".to_string())
        }
        (FieldType::Text { .. }, Some(FieldValue::Text(s))) => s.clone(),
        (FieldType::Text { .. }, None) => "—".to_string(),
        (FieldType::Checkbox { .. }, Some(FieldValue::Boolean(b))) => {
            if *b { "Yes" } else { "No" }.to_string()
        }
        (FieldType::Checkbox { default }, None) => {
            if *default { "Yes" } else { "No" }.to_string()
        }
        (FieldType::Select { options }, Some(FieldValue::Text(s))) => {
            // Find the label for the selected value
            options
                .iter()
                .find(|o| o.value == *s)
                .map(|o| o.label.clone())
                .unwrap_or_else(|| s.clone())
        }
        (FieldType::Select { options }, None) => {
            options.first().map(|o| o.label.clone()).unwrap_or_else(|| "—".to_string())
        }
        (FieldType::Resource { .. }, _) => {
            // Handled specially in the component
            "—".to_string()
        }
        (FieldType::Derived { .. }, Some(FieldValue::Number(n))) => n.to_string(),
        (FieldType::Derived { .. }, Some(FieldValue::Text(s))) => s.clone(),
        (FieldType::Derived { .. }, _) => "—".to_string(),
        (FieldType::SkillReference { .. }, _) => "—".to_string(),
        (FieldType::ItemList { .. }, _) => "—".to_string(),
        (FieldType::SkillList { .. }, _) => "—".to_string(),
        _ => "—".to_string(),
    }
}
