//! Sheet Field Input - Dynamic input rendering for character sheet fields

use dioxus::prelude::*;
use std::collections::HashMap;

use crate::application::dto::{
    FieldType, FieldValue, SheetField, SheetSection, SheetTemplate,
};

/// Props for the sheet section renderer
#[derive(Props, Clone, PartialEq)]
pub struct SheetSectionProps {
    pub section: SheetSection,
    pub values: HashMap<String, FieldValue>,
    pub on_change: EventHandler<(String, FieldValue)>,
    #[props(default = false)]
    pub read_only: bool,
}

/// Renders a section of the character sheet
#[component]
pub fn SheetSectionInput(props: SheetSectionProps) -> Element {
    let mut is_collapsed = use_signal(|| props.section.collapsed_by_default);

    let section_style = match props.section.layout {
        crate::application::dto::SectionLayout::Vertical => {
            "display: flex; flex-direction: column; gap: 0.75rem;"
        }
        crate::application::dto::SectionLayout::Grid { columns } => {
            let _cols = columns.min(4).max(1);
            // We'll use inline grid
            "display: grid; gap: 0.75rem;"
        }
        crate::application::dto::SectionLayout::TwoColumn => {
            "display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem;"
        }
        crate::application::dto::SectionLayout::Flow => {
            "display: flex; flex-wrap: wrap; gap: 0.5rem;"
        }
    };

    // Sort fields by order
    let mut sorted_fields = props.section.fields.clone();
    sorted_fields.sort_by_key(|f| f.order);

    rsx! {
        div {
            class: "sheet-section",
            style: "background: rgba(0,0,0,0.2); border-radius: 0.5rem; overflow: hidden;",

            // Section header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1rem; background: rgba(0,0,0,0.3); cursor: pointer;",
                onclick: move |_| {
                    if props.section.collapsible {
                        let current = *is_collapsed.read();
                        is_collapsed.set(!current);
                    }
                },

                h4 {
                    style: "color: #e5e7eb; font-size: 0.875rem; margin: 0; font-weight: 600;",
                    "{props.section.name}"
                }

                if props.section.collapsible {
                    span {
                        style: "color: #6b7280; font-size: 0.75rem;",
                        if *is_collapsed.read() { "+" } else { "-" }
                    }
                }
            }

            // Section content
            if !*is_collapsed.read() {
                div {
                    style: "padding: 1rem; {section_style}",

                    for field in sorted_fields {
                        {
                            let field_id = field.id.clone();
                            let current_value = props.values.get(&field_id).cloned();
                            let on_change = props.on_change.clone();
                            rsx! {
                                SheetFieldInput {
                                    key: "{field_id}",
                                    field: field,
                                    value: current_value,
                                    on_change: move |value| on_change.call((field_id.clone(), value)),
                                    read_only: props.read_only,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Props for an individual field input
#[derive(Props, Clone, PartialEq)]
pub struct SheetFieldInputProps {
    pub field: SheetField,
    pub value: Option<FieldValue>,
    pub on_change: EventHandler<FieldValue>,
    #[props(default = false)]
    pub read_only: bool,
}

/// Renders a single field input based on its type
#[component]
pub fn SheetFieldInput(props: SheetFieldInputProps) -> Element {
    let field_style = "width: 100%; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; box-sizing: border-box;";
    let label_style = "color: #9ca3af; font-size: 0.75rem; margin-bottom: 0.25rem; display: block;";

    let is_read_only = props.read_only || props.field.read_only;

    rsx! {
        div {
            class: "sheet-field",
            style: "display: flex; flex-direction: column;",

            // Label
            label {
                style: "{label_style}",
                "{props.field.name}"
                if props.field.required {
                    span { style: "color: #ef4444; margin-left: 0.25rem;", "*" }
                }
            }

            // Input based on field type
            match &props.field.field_type {
                FieldType::Number { min, max, default } => {
                    let current = match &props.value {
                        Some(FieldValue::Number(n)) => *n,
                        _ => default.unwrap_or(0),
                    };
                    let min_val = *min;
                    let max_val = *max;
                    let on_change = props.on_change.clone();

                    rsx! {
                        input {
                            r#type: "number",
                            value: "{current}",
                            min: min_val.map(|v| v.to_string()),
                            max: max_val.map(|v| v.to_string()),
                            disabled: is_read_only,
                            style: "{field_style}",
                            oninput: move |e| {
                                if let Ok(n) = e.value().parse::<i32>() {
                                    on_change.call(FieldValue::Number(n));
                                }
                            },
                        }
                    }
                }

                FieldType::Text { multiline, max_length: _ } => {
                    let current = match &props.value {
                        Some(FieldValue::Text(s)) => s.clone(),
                        _ => String::new(),
                    };
                    let is_multiline = *multiline;
                    let on_change = props.on_change.clone();

                    if is_multiline {
                        rsx! {
                            textarea {
                                value: "{current}",
                                disabled: is_read_only,
                                style: "{field_style} min-height: 60px; resize: vertical;",
                                oninput: move |e| {
                                    on_change.call(FieldValue::Text(e.value()));
                                },
                            }
                        }
                    } else {
                        rsx! {
                            input {
                                r#type: "text",
                                value: "{current}",
                                disabled: is_read_only,
                                style: "{field_style}",
                                oninput: move |e| {
                                    on_change.call(FieldValue::Text(e.value()));
                                },
                            }
                        }
                    }
                }

                FieldType::Checkbox { default } => {
                    let current = match &props.value {
                        Some(FieldValue::Boolean(b)) => *b,
                        _ => *default,
                    };
                    let on_change = props.on_change.clone();

                    rsx! {
                        input {
                            r#type: "checkbox",
                            checked: current,
                            disabled: is_read_only,
                            style: "width: auto; margin: 0.25rem 0;",
                            onchange: move |e| {
                                on_change.call(FieldValue::Boolean(e.checked()));
                            },
                        }
                    }
                }

                FieldType::Select { options } => {
                    let current = match &props.value {
                        Some(FieldValue::Text(s)) => s.clone(),
                        _ => options.first().map(|o| o.value.clone()).unwrap_or_default(),
                    };
                    let opts = options.clone();
                    let on_change = props.on_change.clone();

                    rsx! {
                        select {
                            value: "{current}",
                            disabled: is_read_only,
                            style: "{field_style}",
                            onchange: move |e| {
                                on_change.call(FieldValue::Text(e.value()));
                            },

                            for opt in opts {
                                option {
                                    value: "{opt.value}",
                                    "{opt.label}"
                                }
                            }
                        }
                    }
                }

                FieldType::Resource { max_field: _, default_max } => {
                    let (current, max) = match &props.value {
                        Some(FieldValue::Resource { current, max }) => (*current, *max),
                        _ => (default_max.unwrap_or(10), default_max.unwrap_or(10)),
                    };
                    let on_change = props.on_change.clone();
                    let on_change2 = props.on_change.clone();

                    rsx! {
                        div {
                            style: "display: flex; align-items: center; gap: 0.5rem;",

                            input {
                                r#type: "number",
                                value: "{current}",
                                min: "0",
                                max: "{max}",
                                disabled: is_read_only,
                                style: "width: 60px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; text-align: center;",
                                oninput: move |e| {
                                    if let Ok(n) = e.value().parse::<i32>() {
                                        on_change.call(FieldValue::Resource { current: n, max });
                                    }
                                },
                            }

                            span { style: "color: #6b7280;", "/" }

                            input {
                                r#type: "number",
                                value: "{max}",
                                min: "1",
                                disabled: is_read_only,
                                style: "width: 60px; padding: 0.5rem; background: #0f0f23; border: 1px solid #374151; border-radius: 0.25rem; color: white; text-align: center;",
                                oninput: move |e| {
                                    if let Ok(n) = e.value().parse::<i32>() {
                                        on_change2.call(FieldValue::Resource { current, max: n });
                                    }
                                },
                            }
                        }
                    }
                }

                FieldType::Derived { formula: _, depends_on: _ } => {
                    // Derived fields are read-only and calculated
                    let display = match &props.value {
                        Some(FieldValue::Number(n)) => n.to_string(),
                        Some(FieldValue::Text(s)) => s.clone(),
                        _ => "—".to_string(),
                    };

                    rsx! {
                        div {
                            style: "padding: 0.5rem; background: rgba(0,0,0,0.3); border: 1px solid #374151; border-radius: 0.25rem; color: #9ca3af;",
                            "{display}"
                            span {
                                style: "color: #6b7280; font-size: 0.75rem; margin-left: 0.5rem;",
                                "(calculated)"
                            }
                        }
                    }
                }

                FieldType::SkillReference { categories: _, show_attribute: _ } => {
                    // TODO: Add skill picker
                    rsx! {
                        div {
                            style: "padding: 0.5rem; background: rgba(0,0,0,0.2); border: 1px dashed #374151; border-radius: 0.25rem; color: #6b7280; font-size: 0.875rem;",
                            "Skill reference (coming soon)"
                        }
                    }
                }

                FieldType::ItemList { item_type: _, max_items: _ } => {
                    // TODO: Add item list editor
                    rsx! {
                        div {
                            style: "padding: 0.5rem; background: rgba(0,0,0,0.2); border: 1px dashed #374151; border-radius: 0.25rem; color: #6b7280; font-size: 0.875rem;",
                            "Item list (coming soon)"
                        }
                    }
                }

                FieldType::SkillList { show_modifier: _, show_proficiency: _ } => {
                    // TODO: Add skill list display
                    rsx! {
                        div {
                            style: "padding: 0.5rem; background: rgba(0,0,0,0.2); border: 1px dashed #374151; border-radius: 0.25rem; color: #6b7280; font-size: 0.875rem;",
                            "Skill list (coming soon)"
                        }
                    }
                }
            }

            // Description/help text
            if let Some(desc) = &props.field.description {
                p {
                    style: "color: #6b7280; font-size: 0.75rem; margin-top: 0.25rem; margin-bottom: 0;",
                    "{desc}"
                }
            }
        }
    }
}

/// Props for the full sheet template renderer
#[derive(Props, Clone, PartialEq)]
pub struct CharacterSheetFormProps {
    pub template: SheetTemplate,
    pub values: HashMap<String, FieldValue>,
    pub on_change: EventHandler<(String, FieldValue)>,
    #[props(default = false)]
    pub read_only: bool,
}

/// Renders the entire character sheet form based on a template
#[component]
pub fn CharacterSheetForm(props: CharacterSheetFormProps) -> Element {
    // Sort sections by order
    let mut sorted_sections = props.template.sections.clone();
    sorted_sections.sort_by_key(|s| s.order);

    rsx! {
        div {
            class: "character-sheet-form",
            style: "display: flex; flex-direction: column; gap: 1rem;",

            for section in sorted_sections {
                {
                    let section_id = section.id.clone();
                    rsx! {
                        SheetSectionInput {
                            key: "{section_id}",
                            section: section,
                            values: props.values.clone(),
                            on_change: props.on_change.clone(),
                            read_only: props.read_only,
                        }
                    }
                }
            }
        }
    }
}
