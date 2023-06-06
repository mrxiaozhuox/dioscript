use dioscript_parser::types::Value;
use dioxus::prelude::*;

#[inline_props]
pub fn View(cx: Scope, code: String) -> Element {
    let mut rt = dioscript_runtime::Runtime::new();
    let result = rt.execute(&code).unwrap();
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
