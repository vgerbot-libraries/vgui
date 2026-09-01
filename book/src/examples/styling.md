# Styling Showcase Example

## Live Demo

<iframe src="../wasm/styling/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The styling showcase puts every `vgui` styling mechanism side by side in a single
scrollable panel. It demonstrates:

- `css!` macro with gradients, `box-shadow`, and arbitrary CSS properties.
- Tailwind utility classes (`tw!`) including gradients and arbitrary color values.
- Pseudo-state attributes: `hover`, `active`, and `focus`.
- Dynamic class composition with `twc!` and conditional `Option<&str>` arguments.
- Responsive breakpoints (`sm:` / `lg:`) that reflow on viewport resize.
- Runtime-constructed class strings via `tw_dynamic()`.

## Source Code

```rust
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (active, set_active) = create_signal(false);

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] text-white" style={css!{ width: 700px; height: 600px; overflow-y: auto; }}>
            <h2 class="text-lg font-bold">{"Styling Showcase"}</h2>

            // ── css! macro ───────────────────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"css! macro"}</span>
                <div style={css!{
                    display: flex;
                    gap: 12px;
                    padding: 16px;
                    background: linear-gradient(135deg, #2563ff, #6c757d);
                    border-radius: 8px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                }}>
                    <div style={css!{ background: #2563ff; padding: 12px; border-radius: 4px; }}>
                        {"Child A"}
                    </div>
                    <div style={css!{ background: #6c757d; padding: 12px; border-radius: 4px; }}>
                        {"Child B"}
                    </div>
                </div>
            </div>

            // ── Tailwind classes ─────────────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"tw! classes"}</span>
                <div class="flex gap-3 p-4 bg-gradient-to-br from-[#2563ff] to-[#6c757d] rounded-lg">
                    <div class="bg-[#2563ff] p-3 rounded">
                        {"Child A"}
                    </div>
                    <div class="bg-[#6c757d] p-3 rounded">
                        {"Child B"}
                    </div>
                </div>
            </div>

            // ── Pseudo-states ────────────────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Pseudo-states (hover / active / focus)"}</span>
                <div class="flex gap-2">
                    <button
                        class="px-4 py-2 bg-[#2563ff] text-white rounded"
                        hover={css!{ background: #0044cc; }}
                    >
                        {"Hover me"}
                    </button>
                    <button
                        class="px-4 py-2 bg-[#10b981] text-white rounded"
                        active={css!{ background: #34d399; }}
                    >
                        {"Active me"}
                    </button>
                    <button
                        class="px-4 py-2 bg-[#9933ff] text-white rounded"
                        focus={css!{ border: 2px solid #ffffff; }}
                    >
                        {"Focus me"}
                    </button>
                </div>
            </div>

            // ── Dynamic classes (twc!) ───────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Dynamic classes (twc!)"}</span>
                <button
                    class={twc!(
                        "p-3 rounded text-white transition-colors",
                        active.get().then_some("bg-[#2563ff]"),
                        (!active.get()).then_some("bg-[#6c757d]")
                    )}
                    on:click={click(move |cx| set_active.update(cx, |v| *v = !*v))}
                >
                    {if active.get() { "Active: ON" } else { "Active: OFF" }}
                </button>
            </div>

            // ── Responsive breakpoints ───────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Responsive breakpoints (resize window)"}</span>
                <div class="flex flex-col lg:flex-row gap-2">
                    <div class="bg-[#2563ff] p-3 rounded text-white">{"Box 1"}</div>
                    <div class="bg-[#10b981] p-3 rounded text-white">{"Box 2"}</div>
                    <div class="bg-[#9933ff] p-3 rounded text-white">{"Box 3"}</div>
                </div>
                <span class="sm:text-sm lg:text-lg text-[#aaa]">{"sm:text-sm lg:text-lg"}</span>
            </div>

            // ── Runtime classes (tw_dynamic) ─────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"tw_dynamic() runtime"}</span>
                <div class={tw_dynamic("p-4 bg-[#2563ff] text-white rounded-lg")}>
                    {"Runtime-constructed class string"}
                </div>
            </div>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(700.), px(600.0)), cx);
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

### `css!` macro vs Tailwind classes

The first two sections produce the same visual result — a gradient flex container
with two colored children — using two different mechanisms:

- **`css!` macro** writes raw CSS declarations as the element's inline `style`.
  Any valid CSS property works, including `linear-gradient`, `box-shadow`, and
  `rgba()` colors. Use it when you need a property that has no Tailwind
  equivalent or when you want pixel-precise control.
- **Tailwind classes** (`class="..."`) compile to the same CSS but via utility
  names: `bg-gradient-to-br from-[#2563ff] to-[#6c757d] rounded-lg`. Arbitrary
  values use the bracket syntax `bg-[#2563ff]`.

### Pseudo-state attributes (`hover` / `active` / `focus`)

Three buttons demonstrate the pseudo-state attributes. Each takes a `css!` block
that is applied only while the element is in that state:

- `hover={css!{ background: #0044cc; }}` — darkens the blue button on mouse-over.
- `active={css!{ background: #34d399; }}` — brightens the green button while pressed.
- `focus={css!{ border: 2px solid #ffffff; }}` — adds a white outline to the purple button when focused.

These map to the CSS `:hover`, `:active`, and `:focus` pseudo-classes without
needing a separate stylesheet.

### `twc!` for conditional classes

The dynamic-classes button uses `twc!` to compose a class string at render time
from a base plus conditional `Option<&str>` arguments:

```rust
class={twc!(
    "p-3 rounded text-white transition-colors",
    active.get().then_some("bg-[#2563ff]"),
    (!active.get()).then_some("bg-[#6c757d]")
)}
```

When `active` is true the button gets the blue background; when false it gets the
gray one. The label text also reacts via `if active.get() { "Active: ON" } else
{ "Active: OFF" }`. Clicking toggles the signal with
`set_active.update(cx, |v| *v = !*v)`.

### Responsive breakpoints

The breakpoints section uses `flex flex-col lg:flex-row` so the three colored
boxes stack vertically on narrow viewports and switch to a horizontal row at the
`lg` breakpoint. The text below uses `sm:text-sm lg:text-lg` to change font size
at the `sm` and `lg` breakpoints. Resize the window (or the iframe) to see the
layout reflow.

### `tw_dynamic()` for runtime class strings

`tw_dynamic("p-4 bg-[#2563ff] text-white rounded-lg")` accepts a plain `&str`
built at runtime — useful when class fragments are assembled from variables or
configuration. Unlike `twc!`, which takes compile-time-known fragments plus
conditional `Option<&str>`s, `tw_dynamic` parses an arbitrary string at runtime
and resolves it against the Tailwind engine.

## Running

**Native:**

    cargo run -p vgui-styling

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-styling --release
    wasm-bindgen --target web --out-dir examples/styling/dist \
        --no-typescript target/wasm32-unknown-unknown/release/styling.wasm
    python3 scripts/serve_plain.py 8080 examples/styling
