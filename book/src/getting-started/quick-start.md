# Quick Start

## A Minimal App

This is the smallest useful `vgui` application — a window with a counter and
an increment button:

```rust
use gpui::{px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

fn app() -> impl gpui::IntoElement {
    let (count, set_count) = create_signal(0i32);
    view! {
        <div class="flex flex-col gap-3 p-4 w-[500px] h-[500px] justify-center items-center text-white">
            <span>{format!("count = {}", count.get())}</span>
            <button
                class="p-2 bg-[#0000ff] hover:bg-[#000088] text-white rounded"
                on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
            >
                {"Increment"}
            </button>
        </div>
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    });
}
```

### How it works

1. **`Application::new().run(...)`** — starts the `gpui` event loop.
2. **`cx.open_window(...)`** — opens a window with the given bounds.
3. **`vgui::mount(cx, app)`** — creates a `VguiRoot` entity that owns the
   reactive scope and calls `app()` on every render.
4. **`create_signal(0i32)`** — creates a reactive signal holding the count.
   Returns a `(ReadSignal, WriteSignal)` pair.
5. **`view! { ... }`** — expands the JSX-like markup into `gpui` element
   builders at compile time.
6. **`count.get()`** — reads the signal value and registers the current scope
   as a dependency, so this `<span>` re-renders when the count changes.
7. **`click(move |cx| set_count.update(cx, |n| *n += 1))`** — the `click`
   helper wraps a closure into the event handler signature `gpui` expects.
   `set_count.update` mutates the signal and notifies dependents.

## Running the Examples

The repository includes four end-to-end examples under `examples/`:

```bash
# Minimal counter with signals, memo, and <Show>
cargo run -p vgui-counter

# Todo list with <For>, filtering, css! styling
cargo run -p vgui-todolist

# All <input> types: text, password, checkbox, radio, range, file, etc.
cargo run -p vgui-inputs

# HTML tag coverage: headings, lists, tables, progress, details, dialog, etc.
cargo run -p vgui-tags-demo
```

## Project Layout

A typical `vgui` application has this structure:

```
my-app/
├── Cargo.toml
└── src/
    └── main.rs
```

`Cargo.toml`:

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
vgui = "0.1"
gpui = "0.2"
```

`src/main.rs` follows the pattern above: define an `app()` function that
returns `impl IntoElement`, then wire it into `Application::run` with
`vgui::mount`. As your app grows, extract component functions that take props
and return `impl IntoElement`, and compose them inside `view!` using uppercase
tags (see [Custom Components](../custom-components.md)).
