<div align="center">
    <h1>DioScript</h1>
</div>

> Dioscript is a script language use for generate web elements.

```dioscript
@username = "YuKun Liu";
@login = false;

@display_navbar = true;
if !@display_navbar {
    return "unavailable navbar.";
}

return div {
    class: "navbar",
    h1 { @username },
    if login {
        return h2 { "Hello User!" };
    } else {
        return h2 { "Hello Guest! };
    }
};

```

## Runtime

Dioscript includes a very simple runtime, it will work for `conditional statements`, `reference scope`, `element internal statements`, and `value calculate`.

## Usage

Currently, I am working on a library that can convert `Dioxscript` to `Dioxus` components, and that can help Dioxus Web to dynamically generate & display page components.
