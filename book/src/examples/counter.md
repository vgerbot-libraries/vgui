# Counter Example

## Live Demo

<iframe src="../wasm/counter/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The counter is the minimal `vgui` application. It demonstrates:

- `create_signal` for state management.
- `create_memo` for a derived value (`doubled`).
- `<Show>` for conditional rendering (positive/negative/zero/odd/even).
- Component functions taking `WriteSignal` as props.
- Tailwind classes via `class=` with `hover:` variants.

## Source Code

```rust
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn increment_button(set_count: WriteSignal<i32>) -> impl gpui::IntoElement {
    view! {
        <button
            class="p-2 bg-[#0000ff] hover:bg-[#000088] text-white rounded"
            on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
        >
            {"Increment"}
        </button>
    }
}

fn decrement_button(set_count: WriteSignal<i32>) -> impl gpui::IntoElement {
    view! {
        <button
            class="p-2 bg-[#FF0000] hover:bg-[#880000] text-white rounded"
            on:click={click(move |cx| set_count.update(cx, |n| *n -= 1))}
        >
            {"Decrement"}
        </button>
    }
}

fn app() -> impl gpui::IntoElement {
    let (count, set_count) = create_signal(0i32);
    let doubled = create_memo({
        let count = count.clone();
        move || count.get() * 2
    });
    view! {
        <div class="flex flex-col gap-3 p-4 bg-[#505050] w-[500px] h-[500px] justify-center items-center text-white">
            <span>{format!("Hello, {}!", count.get())}</span>
            <span>{format!("doubled {}", doubled.get())}</span>
            <Show when={count.get() > 0}>
                <span>{"positive"}</span>
            </Show>
            <Show when={count.get() < 0}>
                <span>{"negative"}</span>
            </Show>
            <Show when={count.get() == 0}>
                <span>{"zero"}</span>
            </Show>
            <Show when={count.get() & 1 == 1} fallback={view! { <span>{"even"}</span> }}>
                <span>{"odd"}</span>
            </Show>
            {increment_button(set_count.clone())}
            {decrement_button(set_count.clone())}
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
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

    // On WASM, run_embedded returns an ApplicationHandle that keeps the
    // app alive. mem::forget prevents it from being dropped when start()
    // returns, since WASM's run() is non-blocking.
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
    run();
}
```

## Key Concepts

### Signal + memo

`create_signal(0i32)` creates the count state. `create_memo` derives `doubled`
from it — the memo recomputes only when `count` changes, not on every render.

### `<Show>` with and without fallback

Four `<Show>` blocks demonstrate both forms. The first three have no
`fallback` (render nothing when false). The fourth uses `fallback` to show
"even" when the count is not odd.

### Component functions

`increment_button` and `decrement_button` are plain functions that take a
`WriteSignal<i32>` and return `impl IntoElement`. They are called via
interpolation `{increment_button(set_count.clone())}` rather than uppercase
tags, since they take a single non-attribute argument.

### `click` helper

The `click(move |cx| ...)` helper wraps a simple `Fn(&mut App)` closure into
the `gpui` event handler signature `Fn(&ClickEvent, &mut Window, &mut App)`.

### Tailwind classes with hover variants

`class="p-2 bg-[#0000ff] hover:bg-[#000088] text-white rounded"` uses arbitrary
color values, a `hover:` variant for the background color, and standard
spacing/typography utilities.

### Running

**Native:**

```bash
cargo run -p vgui-counter
```

**Web (WASM):**

```bash
# Build the WASM binary
cargo build --target wasm32-unknown-unknown -p vgui-counter --release

# Generate JS bindings
wasm-bindgen --target web --out-dir examples/counter/dist \
    --no-typescript target/wasm32-unknown-unknown/release/counter.wasm

# Serve and open in a browser
python3 scripts/serve_plain.py 8080 examples/counter
```
