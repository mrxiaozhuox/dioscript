use dioscript_runtime::types::Value;
use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn View(code: String) -> Element {
    let mut rt = dioscript_runtime::Runtime::new();
    let result = rt.execute(&code);
    match result {
        Ok(result) => {
            let html = match result {
                Value::String(s) => s,
                Value::Element(e) => e.to_html(),
                _ => String::new(),
            };
            rsx! {
                div {
                    id: "dioscript",
                    dangerous_inner_html: "{html}"
                }
            }
        }
        Err(e) => {
            let message = e.to_string();
            let message = message.split_once(": ").map(|v| v.1).unwrap_or(&message);
            let errors = message.split("\n").map(|v| {
                rsx! {
                    p { "{v}" }
                }
            });
            rsx! {
                div {
                    class: "font-semibold text-red-700 whitespace-pre font-mono",
                    { errors }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn AstView(code: String) -> Element {
    let ast = dioscript_parser::ast::DioscriptAst::from_string(&code);
    match ast {
        Ok(result) => {
            rsx! {
                div {
                    class: "text-xs font-semibold w-[550px] h-[670px] overflow-scroll",
                    id: "dioscript",
                    dangerous_inner_html: "<pre>{result:#?}</pre>"
                }
            }
        }
        Err(e) => {
            let message = e.to_string();
            let errors = message.split("\n").map(|v| {
                rsx! {
                    p { "{v}" }
                }
            });
            rsx! {
                div {
                    class: "font-semibold text-red-700 whitespace-pre font-mono",
                    { errors }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn NamespaceView(code: String) -> Element {
    let mut rt = dioscript_runtime::Runtime::new();
    let result = rt.execute(&code);
    match result {
        Ok(_r) => {
            let result = rt.using_namespace();
            rsx! {
                div {
                    class: "text-xs font-semibold w-[550px] h-[670px] overflow-scroll",
                    id: "dioscript",
                    dangerous_inner_html: "<pre>{result:#?}</pre>"
                }
            }
        }
        Err(e) => {
            let message = e.to_string();
            let message = message.split_once(": ").map(|v| v.1).unwrap_or(&message);
            let errors = message.split("\n").map(|v| {
                rsx! {
                    p { "{v}" }
                }
            });
            rsx! {
                div {
                    class: "font-semibold text-red-700 whitespace-pre font-mono",
                    { errors }
                }
            }
        }
    }
}
