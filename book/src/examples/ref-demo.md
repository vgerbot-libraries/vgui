# ref Demo

## Live Demo

<iframe src="../wasm/ref-demo/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

This example demonstrates the SolidJS-style `ref` handle system. It shows how
to create a `NodeRef`, bind it to an element via `ref={...}`, and call
imperative methods (`scroll_to_bottom`, `scroll_to`, `focus`) from event
handlers.

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
    // Create NodeRefs before view! — they're empty shells until bound
    // during render by the `ref=` attribute.
    let scroll_ref = NodeRef::new();
    let focus_ref = NodeRef::new();
    let items: Vec<u32> = (0..20).collect();

    // Clone refs for the event-handler closures (the ref= attributes
    // below consume separate clones).
    let scroll_ref_btn1 = scroll_ref.clone();
    let scroll_ref_btn2 = scroll_ref.clone();
    let focus_ref_btn = focus_ref.clone();

    view! {
        <div class="flex flex-col gap-2 p-4 bg-[#505050] w-[400px] h-[500px] text-white">
            <h2 class="text-lg font-bold">{"ref Demo"}</h2>

            // Buttons that call imperative methods on the refs.
            <div class="flex gap-2">
                <button
                    class="p-2 bg-[#0000ff] hover:bg-[#000088] rounded text-white"
                    on:click={click(move |_cx| {
                        scroll_ref_btn1.scroll_to_bottom();
                    })}
                >
                    {"Scroll to bottom"}
                </button>
                <button
                    class="p-2 bg-[#006600] hover:bg-[#004400] rounded text-white"
                    on:click={click(move |_cx| {
                        scroll_ref_btn2.scroll_to(2);
                    })}
                >
                    {"Scroll to #2"}
                </button>
                <button
                    class="p-2 bg-[#660066] hover:bg-[#440044] rounded text-white"
                    on:click={move |_e, window, cx| {
                        focus_ref_btn.focus(window, cx);
                    }}
                >
                    {"Focus box"}
                </button>
            </div>

            // A scrollable list bound to scroll_ref via ref=.
            // ref= forces an auto-id and applies track_focus + track_scroll
            // so scroll_to/scroll_to_bottom/bounds all work.
            <div
                ref={scroll_ref.clone()}
                class="flex-1 overflow-y-scroll bg-[#3a3a3a] rounded p-2 gap-1 flex-col"
            >
                <For each={items}>
                    {move |i: u32, _idx: usize| view! {
                        <div class="p-2 bg-[#2a2a2a] rounded">
                            {format!("Item {}", i)}
                        </div>
                    }}
                </For>
            </div>

            // A focusable box bound to focus_ref.
            <div
                ref={focus_ref.clone()}
                class="p-3 bg-[#2a2a2a] rounded border-2 border-[#666] focus:border-[#0f0]"
                tabindex={0}
            >
                {"Click 'Focus box' to focus me."}
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
        let bounds = Bounds::centered(None, size(px(400.), px(500.0)), cx);
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

### Creating a NodeRef

`NodeRef::new()` creates an empty shell handle. It's safe to call outside a
render scope (e.g. in `app()` before `view!`). The handle stays unbound until
the `view!` macro binds it during render.

### Binding with `ref=`

The `ref={node_ref}` attribute on any built-in element:

1. Forces an auto-generated element id if none is specified (required for
   `track_focus` / `track_scroll`).
2. Calls `__bind_ref` to cache a `FocusHandle` + `ScrollHandle` in the reactive
   scope slot, populating the `NodeRef`.
3. Applies `track_focus` and `track_scroll` to the element so gpui keeps the
   handles in sync across frames.

### Imperative methods

Once bound, `NodeRef` exposes:

| Method | Description |
|--------|-------------|
| `focus(window, cx)` | Move keyboard focus to the element. |
| `is_focused(window)` | Check if the element is focused. |
| `bounds()` | Painted bounds from the previous frame. |
| `scroll_to(ix)` | Scroll child `ix` into view. |
| `scroll_to_top(ix)` | Scroll child `ix` to the top. |
| `scroll_to_bottom()` | Scroll to the bottom of content. |
| `scroll_offset()` | Current scroll offset. |
| `set_scroll_offset(off)` | Set scroll offset explicitly. |
| `child_bounds(ix)` | Painted bounds of child `ix`. |
| `child_count()` | Number of tracked children. |

### Cloning refs for closures

`NodeRef` is `Clone` (uses `Rc<RefCell<...>>` internally). Clone it before
passing into event-handler closures so the `ref=` attribute in `view!` can
consume its own clone.

### Running

**Native:**

```bash
cargo run -p vgui-ref-demo
```

**Web (WASM):**

```bash
# Build the WASM binary
cargo build --target wasm32-unknown-unknown -p vgui-ref-demo --release

# Generate JS bindings
wasm-bindgen --target web --out-dir examples/ref-demo/dist \
    --no-typescript target/wasm32-unknown-unknown/release/ref-demo.wasm

# Serve and open in a browser
python3 scripts/serve_plain.py 8080 examples/ref-demo
```
