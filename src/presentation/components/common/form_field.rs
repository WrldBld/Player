use dioxus::prelude::*;

#[component]
pub fn FormField(label: &'static str, required: bool, children: Element) -> Element {
    rsx! {
        div {
            class: "form-field",
            style: "display: flex; flex-direction: column; gap: 0.25rem;",
            label {
                style: "color: #9ca3af; font-size: 0.875rem;",
                "{label}"
                if required {
                    span { style: "color: #ef4444; margin-left: 0.25rem;", "*" }
                }
            }
            {children}
        }
    }
}
