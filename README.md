<div align="center">
    <h1>DioScript</h1>
</div>

> Dioscript is a script language use for generate web elements and embed into rust program.

```dioscript
let username = "YuKun Liu";
let login = false;

let display_navbar = true;
if display_navbar == false {
    return "unavailable navbar.";
}

return div {
    class: "navbar",
    h1 { username },
    if login {
        return h2 { "Hello User!" };
    } else {
        return h2 { "Hello Guest! };
    }
};

```



## Function Bind

Currently, You can bind **Rust** function to dioscript-runtime, and call it in dioscript code.

```rust
fn element_to_html(args: Vec<Value>) -> Value {
    let v = args.get(0).unwrap();
    if let Value::Element(e) = v {
        return Value::String(e.to_html());
    }
    Value::None
}

// import function
runtime.bind_module("html", {
    let mut module = ModuleGenerator::new();
    module.insert_rusty_function("to_html", element_to_html, 1);
    module
});

```

```dioscript
e = div {
    class: "main",
    p { "Hello Dioscropt" }
};

return html::to_html(e);

// result
// <div class="main"><p>Hello Dioscript</p></div>

```



## Runtime

Dioscript includes a very simple runtime, it will work for `conditional statements`, `variable scope`, `element internal statements`, and `value calculate`.

## Usage

Currently, I am working on a library that can convert `Dioxscript` to `Dioxus` components, and that can help Dioxus Web to dynamically generate & display page components.
