# Context / Provider Example

## Live Demo

<iframe src="../wasm/context/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

This example demonstrates vgui's Context / Provider pattern — the
SolidJS-equivalent of `createContext` / `useContext` / `<Provider>` for
dependency injection down the element tree.

The example is split across three files — `theme.rs` (the context type,
marker, and a leaf consumer), `panel.rs` (a sub-component that both consumes
and provides), and `main.rs` (the root provider and entry point) — to show
that a `Context<T>` marker crosses file/module boundaries for free.

It features:

- `Context::new()` — a zero-sized, `const`-constructable typed marker stored
  in a plain `static`, keyed by `TypeId`.
- `<Provider context={...} value={...}>` — pushes a value onto a thread-local
  stack before evaluating children and pops it after, so descendants observe
  the value during construction.
- `use_context` / `use_context_or` — read the nearest ancestor provider value
  (or a default fallback).
- Nearest-ancestor resolution: a nested `<Provider>` shadows an outer one for
  its subtree; after the inner provider closes, the outer value is visible
  again.

## Source Code

`theme.rs` — the context type, marker, and leaf consumer:

```rust
use vgui::prelude::*;

/// A theme mode propagated through the element tree via `<Provider>`.
#[derive(Clone, PartialEq)]
pub enum Mode {
    Light,
    Dark,
}

/// The context marker. Zero-sized, stored in a plain `static`.
pub static MODE: Context<Mode> = Context::new();

/// A box that reads the nearest `MODE` provider, falling back to `Light`
/// when no provider is active. `css!` takes literal CSS, so the `if`/`else`
/// picks one of two literal blocks — no dynamic interpolation needed.
///
/// This leaf consumer defines no provider of its own; it reads whatever
/// ancestor provider was pushed in another module, proving a `Context<T>`
/// marker crosses file boundaries for free.
pub fn themed_box(label: &str) -> impl gpui::IntoElement {
    let mode = use_context_or(&MODE, || Mode::Light);
    let style = if matches!(mode, Mode::Dark) {
        css! {
            background: #1a1a2a;
            color: #ffffff;
            padding: 16px;
            margin: 8px;
            border-radius: 8px;
        }
    } else {
        css! {
            background: #f5f5f5;
            color: #111111;
            padding: 16px;
            margin: 8px;
            border-radius: 8px;
        }
    };
    view! {
        <div style={style}>{label.to_string()}</div>
    }
}
```

`panel.rs` — a sub-component in its own module that imports the marker from
`theme.rs`, reads the ancestor provider, and nests its own override:

```rust
use crate::theme::{themed_box, Mode, MODE};
use vgui::prelude::*;

/// A sub-component living in its own module. It imports the `MODE` marker
/// from `theme.rs`, reads the ancestor provider set in `main.rs`, and nests
/// its own override provider — demonstrating that context resolution is
/// per-render, not per-module.
#[allow(non_snake_case)]
pub fn ThemePanel() -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-2">
            {themed_box("panel: inherits root context")}
            <Provider context={MODE} value={Mode::Dark}>
                {themed_box("panel: overridden to dark")}
            </Provider>
        </div>
    }
}
```

`main.rs` — the root provider and dual entry point:

```rust
#![cfg_attr(target_family = "wasm", no_main)]

mod panel;
mod theme;

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use panel::ThemePanel;
use theme::{themed_box, Mode, MODE};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (mode, set_mode) = create_signal(Mode::Light);
    view! {
        <Provider context={MODE} value={mode.get()}>
            <div class="flex flex-col p-4 gap-2 w-[400px] h-[400px]">
                {themed_box("root context (toggles)")}
                <ThemePanel />
                <button class="p-2 bg-[#0066cc] text-white rounded"
                    on:click={click(move |cx| set_mode.update(cx, |m|
                        *m = match *m { Mode::Light => Mode::Dark, Mode::Dark => Mode::Light }))}>
                    {"toggle root theme"}
                </button>
            </div>
        </Provider>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(400.), px(400.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    };

    #[cfg(not(target_family = "wasm"))]
    gpui_app.run(launch);

    #[cfg(target_family = "wasm")]
    std::mem::forget(gpui_app.run_embedded(launch));
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    vgui::intercept_keyboard_events();
    run();
}
```

## Key Concepts

### `Context::new()` static

`Context<T>` is a zero-sized, `const`-constructable typed marker. It carries
no value itself — it only identifies a context type, keyed by `TypeId` of
`T`. Declare it in a plain `static`:

```rust
static MODE: Context<Mode> = Context::new();
```

One context per type. If you need two contexts of the same logical type,
wrap the value in a newtype (`struct Alt(Mode);`) and declare a second
`Context<Alt>`.

### `<Provider context={} value={}>`

The `<Provider>` builtin pushes a value onto a thread-local stack *before*
evaluating its children and pops it *after*. Descendants constructed between
enter and exit observe the value via `use_context`. The `context` and `value`
attributes are both required; any other attribute is rejected.

The type of `value` must match the context's `T` — `__provider_scope_enter`
unifies `T` from both arguments, so a mismatch is a compile error.

### `use_context` / `use_context_or`

`use_context(&CTX)` walks the provider stack top-down and returns the nearest
matching entry, or `None` if no provider is active. `use_context_or(&CTX,
|| default)` falls back to the closure when no provider is present.

### Nearest-ancestor resolution & nested override

The stack is searched top-down, so a nested `<Provider>` shadows an outer
one within its subtree. In the example, the root provider binds `MODE` to a
signal-driven `Mode` (toggled by the button), while an inner provider
overrides it to `Mode::Dark` — so the "overridden to dark" box stays dark
regardless of the toggle, while the "root context" box follows the signal.

### Cross-module context

A `Context<T>` marker is a `static` keyed by `TypeId` on a thread-local
provider stack. The stack is per-render, not per-module: a provider pushed
in one file is visible to any descendant component constructed during that
render, regardless of which module defines it. So a `use crate::theme::MODE`
in `panel.rs` (or any other module) sees the nearest ancestor `<Provider>`
that was entered in `main.rs` — no wiring, re-export, or parameter passing
required. The three-file split in this example makes that flow visible in
source: `theme.rs` owns the marker and a leaf consumer, `panel.rs` imports
the marker and both consumes and provides, and `main.rs` pushes the root
provider.

### Programmatic provider (`provide_context`)

For advanced manual use and tests, `provide_context(&CTX, value)` returns a
`ProviderGuard` that pops the stack on drop (RAII). The `<Provider>` macro
builtin is the primary mechanism; `provide_context` is for cases where you
need to provide a value outside a `view!` tree.

## Running

**Native:**

    cargo run -p vgui-context

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-context --release
    wasm-bindgen --target web --out-dir examples/context/dist \
        --no-typescript target/wasm32-unknown-unknown/release/context.wasm
    python3 scripts/serve_plain.py 8080 examples/context
