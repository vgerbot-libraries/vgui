# Select Test

## Live Demo

<iframe src="../wasm/select-test/" width="100%" height="400" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The select test demonstrates `<select>` with a custom option content renderer.
Each option is rendered via a closure child that receives `(value, label)` and
returns a `view! {}` element — a colored value span alongside the label text.
The same closure renders both the popover rows and the trigger's display of the
selected option. The popover width always matches the trigger width.

## Source Code

```rust
#![cfg_attr(target_family="wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family="wasm"))]
use gpui_platform::application;

#[cfg(target_family="wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (val, set_val) = create_signal("1".to_string());
    view! {
        <div class="flex flex-col gap-2 p-4 bg-[#1a1a2e] w-[400px] h-[300px] text-white">
            <select
                options={vec![
                    ("1".to_string(), "One".to_string()),
                    ("2".to_string(), "Two".to_string()),
                    ("3".to_string(), "Three".to_string()),
                ]}
                value={val.get()}
                on:change={move |v: &str, cx: &mut App| set_val.set(cx, v.to_string())}
            >
                {move |value: &str, label: &str| view! {
                    <div class="flex items-center gap-2">
                        <span class="text-[#0f0]">{value.to_string()}</span>
                        <span>{label.to_string()}</span>
                    </div>
                }}
            </select>
            <span class="text-sm text-[#0f0]">{format!("selected: {}", val.get())}</span>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family="wasm"))]
    let gpui_app = application();

    #[cfg(target_family="wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(400.), px(300.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    };

    #[cfg(not(target_family="wasm"))]
    gpui_app.run(launch);

    #[cfg(target_family="wasm")]
    std::mem::forget(gpui_app.run_embedded(launch));
}

#[cfg(not(target_family="wasm"))]
fn main() {
    run();
}

#[cfg(target_family="wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    vgui::intercept_keyboard_events();
    run();
}
```

## Key Concepts

### Custom option content via closure child

`<select>` accepts an optional single closure child with the signature
`Fn(&str, &str) -> impl IntoElement`. The first argument is the option's value,
the second is its label. The closure renders both the popover rows and the
trigger's selected-option display. When no child closure is provided, options
render as plain label text.

### Popover width follows the trigger

The dropdown popover is an absolute child of the relative trigger and uses
`w_full()`, so its width always equals the trigger's content-box width.

### Running

**Native:**

```bash
cargo run -p vgui-select-test
```

**Web (WASM):**

```bash
cargo build --target wasm32-unknown-unknown -p vgui-select-test --release
wasm-bindgen --target web --out-dir examples/select-test/dist \
    --no-typescript target/wasm32-unknown-unknown/release/select-test.wasm
python3 scripts/serve_plain.py 8080 examples/select-test
```
