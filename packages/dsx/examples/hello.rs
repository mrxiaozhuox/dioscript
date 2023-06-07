use dioxus::prelude::*;
use dsx::View;

fn main() {
    dioxus_desktop::launch(App);
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            View {
                code: "return a {{ href: \"https://github.com/mrxiaozhuox\", \"Hello World\" }}".to_string(),
            }
        }
    })
}
