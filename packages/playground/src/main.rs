use dioxus::prelude::*;
use dsx::{AstView, NamespaceView, View};
use indoc::indoc;

fn main() {
    dioxus::launch(App);
}

#[allow(non_snake_case)]
pub fn App() -> Element {

    let eval = eval(indoc! {"
        setTimeout(() => {
            let editor = window.editor;
            editor.onDidChangeModelContent(function (_e) {
                let content = editor.getValue();
                dioxus.send(content);
            });
        }, 800);
    "});

    let editor_content = use_signal(|| {
        String::from("return div { \"hello dioscript!\" };")
    });
    let _ = use_resource(move || {
        to_owned![eval, editor_content];
        async move {
            #[allow(irrefutable_let_patterns)]
            while let v = eval.recv().await {
                match v {
                    Ok(v) => {
                        editor_content.set(v.as_str().unwrap().to_string());
                    },
                    Err(_e) => {},
                }
            }
        }
    });
   
    let mut display_result = use_signal(|| 0);

    rsx! {
        script {
            r#type: "module",
            src: "/editor.js"
        }
        div {
            class: "mt-4 mx-auto px-8",
            div {
                class: "flex flex-row gap-4 mb-4",
                div {
                    class: "basis-1/2",
                }
                div {
                    class: "basis-1/2",
                    button { 
                        class: "bg-cyan-500 hover:bg-cyan-700 text-white font-semibold text-sm py-2 px-3 rounded",
                        onclick: move |_| { display_result.set(0); },
                        "Result"
                    }
                    button {
                        class: "bg-emerald-500 hover:bg-emerald-700 text-white font-semibold text-sm ml-2 py-2 px-3 rounded",
                        onclick: move |_| { display_result.set(1); },
                        "AST Tree"   
                    }
                    button {
                        class: "bg-emerald-500 hover:bg-emerald-700 text-white font-semibold text-sm ml-2 py-2 px-3 rounded",
                        onclick: move |_| { display_result.set(2); },
                        "Using Namespace"   
                    }
                }
            }
            div {
                class: "flex flex-row gap-4",
                div {
                    class: "basis-1/2",
                    div {
                        id: "monaco",
                        class: "w-full h-[700px] border border-gray-400",
                    }
                }
                div {
                    class: "basis-1/2",
                    div {
                        class: "w-full h-[700px] border border-gray-400",
                        div {
                            class: "mt-1 px-4 py-4",    
                            if *display_result.read() == 0 {
                                View {
                                  code: editor_content.to_string(),  
                                }
                            } else if *display_result.read() == 1 {
                                AstView { code: editor_content.to_string() }
                            } else if *display_result.read() == 2 {
                                NamespaceView { code: editor_content.to_string() }
                            }
                        }
                    }
                }
            }
        }
    }
}
