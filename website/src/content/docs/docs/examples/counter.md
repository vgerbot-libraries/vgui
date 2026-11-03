---
title: Counter
description: Walk through the maintained native vgui counter and its signal-driven update cycle.
---

This example uses the same complete program as the [Quick Start](../../quick-start/) and the maintained `crates/vgui/examples/counter.rs` native example. It does not introduce a second counter implementation.

## Complete native example

The following is the complete `src/main.rs`. For the required manifest, project setup, and run instructions, follow the [Quick Start](../../quick-start/).

```rust
use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use gpui_platform::application;
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
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| vgui::mount(window, cx, app),
        )
        .unwrap();
    });
}
```

## Follow the reactive update

1. `create_signal(0i32)` returns the `count` read handle and `set_count` write handle.
2. `count.get()` reads the current integer while constructing the label. In the reactive render scope, that read establishes the view's dependency on the signal.
3. `click` adapts the closure to a GPUI click listener. The closure calls `set_count.update` with the application context and increments the integer in place.
4. A changed signal notifies its dependents and requests a render of the mounted root view. The next render reads the new count and constructs the updated label.

The signal is created inside the mounted reactive view, not in `main`. Repeated renders reuse its positional state instead of resetting the count to zero. Keep reactive primitive creation order and types stable across renders.

Writing an equal `PartialEq` value does not trigger a signal notification. That differs from the store behavior described in [Store and Lists](../store-and-lists/).

## Native mounting and styling

`application()` creates the native GPUI application, and `run` starts its event loop. `open_window` configures the native window, while `vgui::mount` installs `app` as its root view.

The `class` attributes use vgui-supported Tailwind-style utilities that style GPUI elements. They do not require a browser stylesheet, and the markup does not create browser DOM nodes.

## Verification boundary and next steps

The maintained program is compile-checked as a native workspace example. That check does not demonstrate that a window was displayed or that the Increment button was clicked, and it is not browser interaction coverage.

Continue with [Your First Reactive View](../../first-reactive-view/) for dependency tracking, memos, effects, and update troubleshooting. See [Native & Web](../../native-and-web/) for the separate native and browser verification requirements.
