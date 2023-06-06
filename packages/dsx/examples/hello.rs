use dioxus::prelude::*;
use dsx::View;

fn main() {
    dioxus_desktop::launch(App);
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            View {
                code: "return a {{ href: \"https://github.com/mrxiaozhuox\", \"Hello World\" }}".to_string(),
            }
        }
    })
}
