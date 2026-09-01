# Overlays Example

## Live Demo

<iframe src="../wasm/overlays/" width="100%" height="500" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The overlays example demonstrates the three overlay primitives in `vgui` and how
to gate them with conditional rendering:

- `dialog()` — a modal with a focus trap, backdrop, and Escape / click-outside
  dismissal. The signature is `(open: bool, on_close: impl Fn(&mut App), content)`.
- `floating()` — renders content at a fixed `Point<Pixels>` position, detached
  from the normal flow.
- `portal()` — lifts content onto a high-priority deferred layer with a numeric
  priority argument.
- `show()` — conditionally renders one of two branches (`then` / `fallback`)
  based on a reactive boolean.
- `gpui::Empty` as an `IntoElement` fallback so `show()` renders nothing when the
  overlay is closed.

## Source Code

```rust
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{point, px, size, App, Bounds, Empty, WindowBounds, WindowOptions};
use vgui::{dialog, show};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (dialog_open, set_dialog_open) = create_signal(false);
    let (floating_open, set_floating_open) = create_signal(false);
    let (portal_open, set_portal_open) = create_signal(false);
    let set_dialog_close = set_dialog_open.clone();
    let set_dialog_confirm = set_dialog_open.clone();
    let set_dialog_cancel = set_dialog_open.clone();

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] text-white" style={css!{ width: 500px; height: 500px; }}>
            <h2 class="text-lg font-bold">{"Overlays Example"}</h2>

            // ── Dialog ───────────────────────────────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#888]">{"dialog() — modal with focus trap + backdrop"}</span>
                <button
                    class="px-4 py-2 bg-[#2563ff] rounded text-white"
                    on:click={click(move |cx| set_dialog_open.set(cx, true))}
                >
                    {"Open Dialog"}
                </button>
                {dialog(dialog_open.get(), move |cx| set_dialog_close.set(cx, false), view! {
                    <div class="bg-[#2d2d44] p-6 rounded-lg text-white" style={css!{ max-width: 300px; }}>
                        <h3 class="font-bold mb-2">{"Confirm Action"}</h3>
                        <p class="text-sm mb-4">{"Are you sure you want to proceed?"}</p>
                        <div class="flex gap-3 mt-4">
                            <button
                                class="px-4 py-2 bg-[#2563ff] rounded text-white"
                                on:click={click(move |cx| set_dialog_confirm.set(cx, false))}
                            >
                                {"Confirm"}
                            </button>
                            <button
                                class="px-4 py-2 bg-[#6c757d] rounded text-white"
                                on:click={click(move |cx| set_dialog_cancel.set(cx, false))}
                            >
                                {"Cancel"}
                            </button>
                        </div>
                    </div>
                })}
            </div>

            // ── Floating ─────────────────────────────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#888]">{"floating() — positioned at (150, 200)"}</span>
                <button
                    class="px-4 py-2 bg-[#10b981] rounded text-white"
                    on:click={click(move |cx| set_floating_open.update(cx, |v| *v = !*v))}
                >
                    {"Toggle Floating"}
                </button>
                {show(floating_open.get(), floating(point(px(150.), px(200.)), view! {
                    <div class="bg-[#2d2d44] p-3 rounded text-white text-sm">
                        {"This is a floating element positioned at (150, 200)"}
                    </div>
                }), gpui::Empty)}
            </div>

            // ── Portal ───────────────────────────────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#888]">{"portal() — high-priority deferred layer"}</span>
                <button
                    class="px-4 py-2 bg-[#9933ff] rounded text-white"
                    on:click={click(move |cx| set_portal_open.update(cx, |v| *v = !*v))}
                >
                    {"Toggle Portal"}
                </button>
                {show(portal_open.get(), portal(view! {
                    <div class="bg-[#2563ff] p-4 rounded text-white" style={css!{ position: absolute; top: 20px; right: 20px; }}>
                        {"Portaled content on a high-priority layer"}
                    </div>
                }, 50), gpui::Empty)}
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

### `dialog()` — modal with focus trap and backdrop

`dialog(open, on_close, content)` renders a modal when `open` is true. The
dialog is placed on a deferred layer above all normal content, paints a
semi-transparent backdrop, traps keyboard focus inside the dialog while it is
open, and restores focus to the previously focused element on close. It
dismisses on Escape and on backdrop click, calling `on_close` in both cases.
Here `on_close` is `move |cx| set_dialog_close.set(cx, false)`, so any dismissal
path flips the signal and the dialog disappears on the next render.

### `floating()` — positioned overlay

`floating(point, content)` renders `content` at an absolute `Point<Pixels>`
position. The point is constructed with `gpui::point(gpui::px(150.),
gpui::px(200.))`. Unlike `dialog()`, `floating()` has no backdrop or focus
trap — it simply detaches content from the layout flow and paints it at the
given coordinates.

### `portal()` — high-priority deferred layer

`portal(content, priority)` lifts `content` onto a deferred layer with a
numeric priority (here `50`). Higher priority layers paint above lower ones,
so portaled content always renders on top of the normal tree regardless of
where the `portal()` call appears. The inner `view!` uses
`css!{ position: absolute; top: 20px; right: 20px; }` to anchor itself in the
top-right corner of the window.

### `show()` for conditional overlays

`show(when, then, fallback)` renders `then` when `when` is true and `fallback`
otherwise. Both branches must be `impl IntoElement`. For the floating and
portal sections the fallback is `gpui::Empty`, which implements `IntoElement`
and renders nothing — so the overlay simply vanishes when its toggle signal is
false. The `dialog()` helper already handles its own visibility via the `open`
boolean argument, so it does not need a `show()` wrapper.

### `gpui::Empty` as an `IntoElement` fallback

`gpui::Empty` is a zero-sized type that implements `gpui::IntoElement` by
producing no element. It is the idiomatic "render nothing" value for the
`fallback` slot of `show()` when you only want content to appear conditionally
with no alternative.

### Running

**Native:**

    cargo run -p vgui-overlays

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-overlays --release
    wasm-bindgen --target web --out-dir examples/overlays/dist \
        --no-typescript target/wasm32-unknown-unknown/release/overlays.wasm
    python3 scripts/serve_plain.py 8080 examples/overlays
