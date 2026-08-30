# Focus Management Example

## Live Demo

<iframe src="../wasm/focus-management/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

This example demonstrates three focus management features in vgui:

- **Focus trap** in `<dialog>` — Tab/Shift+Tab cycles within dialog content.
- **Focus restore** in `<dialog>` — Focus returns to the element that had it
  before the dialog opened.
- **Roving tabindex** in `<radiogroup>` — Only the checked radio is a tab stop;
  arrow keys move between radios.

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
    let (dialog_open, set_dialog_open) = create_signal(false);
    let (radio_val, set_radio) = create_signal(0i32);
    let (field1, set_field1) = create_signal(String::new());
    let (field2, set_field2) = create_signal(String::new());

    // Individual setters for radio on:change closures (each needs its own
    // WriteSignal clone with a 'static lifetime).
    let sr0 = set_radio.clone();
    let sr1 = set_radio.clone();
    let sr2 = set_radio.clone();
    let set_dialog_open_btn = set_dialog_open.clone();
    let set_dialog_close = set_dialog_open.clone();
    let set_dialog_close_btn = set_dialog_open.clone();

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#505050] w-[600px] h-[500px] text-white">
            <h2 class="text-lg font-bold">{"Focus Management Demo"}</h2>

            // ── Dialog with focus trap + restore ──────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">
                    {"Click the button, then Tab/Shift+Tab to cycle within the dialog. Escape or click-outside closes it and restores focus."}
                </span>
                <button
                    class="px-3 py-2 bg-[#0066cc] hover:bg-[#004499] rounded text-sm"
                    on:click={click(move |cx| set_dialog_open_btn.set(cx, true))}
                >
                    {"Open Dialog"}
                </button>
            </div>

            // ── Radio group with roving tabindex ──────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">
                    {"Tab reaches only the checked radio. Arrow keys move between radios."}
                </span>
                <radiogroup>
                    <div class="flex flex-row gap-4 items-center">
                        <div class="flex flex-row gap-1 items-center">
                            <input type="radio" checked={radio_val.get() == 0} on:change={move |_v: bool, cx: &mut App| sr0.set(cx, 0)} />
                            <span class="text-sm">{"Option A"}</span>
                        </div>
                        <div class="flex flex-row gap-1 items-center">
                            <input type="radio" checked={radio_val.get() == 1} on:change={move |_v: bool, cx: &mut App| sr1.set(cx, 1)} />
                            <span class="text-sm">{"Option B"}</span>
                        </div>
                        <div class="flex flex-row gap-1 items-center">
                            <input type="radio" checked={radio_val.get() == 2} on:change={move |_v: bool, cx: &mut App| sr2.set(cx, 2)} />
                            <span class="text-sm">{"Option C"}</span>
                        </div>
                    </div>
                </radiogroup>
                <span class="text-sm text-[#0f0]">{format!("selected: {}", radio_val.get())}</span>
            </div>

            // ── Dialog content ────────────────────────────────────────
            <dialog open={dialog_open.get()} on:close={move |cx| set_dialog_close.set(cx, false)}>
                <div class="bg-white text-black p-5 rounded-lg flex flex-col gap-3 w-[350px]">
                    <h3 class="font-bold">{"Dialog with Focus Trap"}</h3>
                    <div class="flex flex-col gap-1">
                        <span class="text-sm text-[#666]">{"Field 1 (type text)"}</span>
                        <input
                            type="text"
                            placeholder="First field"
                            value={field1.get()}
                            on:input={move |v: &str, cx: &mut App| set_field1.set(cx, v.to_string())}
                        />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-sm text-[#666]">{"Field 2 (type text)"}</span>
                        <input
                            type="text"
                            placeholder="Second field"
                            value={field2.get()}
                            on:input={move |v: &str, cx: &mut App| set_field2.set(cx, v.to_string())}
                        />
                    </div>
                    <div class="flex flex-row gap-2 justify-end">
                        <button
                            class="px-3 py-2 bg-[#ccc] hover:bg-[#aaa] rounded text-sm"
                            on:click={click(move |cx| set_dialog_close_btn.set(cx, false))}
                        >
                            {"Close"}
                        </button>
                    </div>
                </div>
            </dialog>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(500.0)), cx);
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
    run();
}
```

## Key Concepts

### Focus Trap in `<dialog>`

The `<dialog>` component automatically traps focus: Tab and Shift+Tab cycle
within the dialog content and cannot escape to background elements. This is
built into `vgui::dialog()` — no extra configuration needed.

The trap works by intercepting Tab key events on the backdrop, calling
`focus_next`/`focus_prev`, and checking `contains_focused` on the dialog
content's `FocusHandle`. If focus would escape, it wraps around to the
first/last focusable element inside the dialog.

### Focus Restore in `<dialog>`

When the dialog opens, the currently focused element is saved. When it closes
(via Escape, click-outside, or `on:close`), focus is restored to that element.

The save/restore is deferred to after the render cycle using `cx.defer()` +
`cx.with_window()`, because the window is on the update stack during render
and `with_window` returns `None` if called directly.

### Roving Tabindex in `<radiogroup>`

`<radiogroup>` wraps radios in a container with `tab_group()` and an arrow-key
handler. Each `<input type="radio">` inside registers its `FocusHandle` with
the group during render via a thread_local scope stack (same pattern as
`<label>`).

- The checked radio gets `tab_index(0)` — it's a tab stop.
- Unchecked radios get `focusable().tab_stop(false)` — focusable via
  mouse/arrows but not via Tab.
- Arrow keys (←/↑/→/↓) move focus between radios in the group.

### Running

**Native:**

```bash
cargo run -p vgui-focus-management
```

**Web (WASM):**

```bash
# Build the WASM binary
cargo build --target wasm32-unknown-unknown -p vgui-focus-management --release

# Generate JS bindings
wasm-bindgen --target web --out-dir examples/focus-management/dist \
    --no-typescript target/wasm32-unknown-unknown/release/focus-management.wasm

# Serve and open in a browser
python3 scripts/serve_plain.py 8080 examples/focus-management
```
