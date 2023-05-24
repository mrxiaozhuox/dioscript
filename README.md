<div align="center">
    <h1>DioScript</h1>
</div>

> Dioscript is a script language use for generate web element.

```dioscript
@username = "YuKun Liu";
@login = false;

@display_navbar = true;
if !@display_navbar {
    return "navbar unspport!";
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

Dioscript include a very simple runtime, it will work for `conditional statement`, `reference scope`, `element internal statement`, `value calculate`.

## Usage

Currently I am working for a library which can convert `Dioxscript` to `Dioxus` components, and that can help Dioxus Web to dynmaic generate & display page component.