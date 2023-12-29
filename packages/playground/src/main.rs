use dioxus::prelude::*;
use dsx::View;
use indoc::indoc;

fn main() {
    dioxus_web::launch(App);
}

#[allow(non_snake_case)]
pub fn App(cx: Scope) -> Element {

        let create_eval = use_eval(&cx);
    let eval = use_state(cx, || {

        create_eval(indoc! {"
            setTimeout(() => {
                let editor = window.editor;
                editor.onDidChangeModelContent(function (_e) {
                    let content = editor.getValue();
                    dioxus.send(content);
                });
            }, 800);
        "}).unwrap()
    });
    let editor_content = use_state(cx, String::new);
    use_coroutine(cx,|rx: UnboundedReceiver<String>| {
        to_owned![eval, editor_content];
        async move {
            while let v = eval.recv().await {
                match v {
                    Ok(v) => {
                        editor_content.set(v.as_str().unwrap().to_string());
                    },
                    Err(e) => {},
                }
            }
        }
    });
    

    cx.render(rsx! {
        script {
            r#type: "module",
            src: "/editor.js"
        }
        div {
            class: "mt-4 mx-auto px-8",
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
                            View {
                              code: editor_content.to_string(),  
                            }
                        }
                    }
                }
            }
        }
    })
}
