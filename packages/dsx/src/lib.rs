use dioscript_runtime::types::Value;
use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn View(cx: Scope, code: String) -> Element {
    let mut rt = dioscript_runtime::Runtime::new();
    let result = rt.execute(&code);
    match result {
        Ok(result) => {
            let html = match result {
                Value::String(s) => s,
                Value::Element(e) => e.to_html(),
                _ => String::new(),
            };
            cx.render(rsx! {
                div {
                    id: "dioscript",
                    dangerous_inner_html: "{html}"
                }
            })
        }
        Err(e) => {
            let message = e.to_string();
            cx.render(rsx! {
                div { class: "font-semibold", "Error: {message}" }
            })
        },
    }
}
