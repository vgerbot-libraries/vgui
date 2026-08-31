# Inputs Demo

## Live Demo

<iframe src="../wasm/inputs/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The inputs demo showcases every `<input>` type in a single window. It
demonstrates:

- Text input with live echo via `on:input`.
- Password input with character count.
- Label association via `for=` and wrapping `<label>`.
- Checkbox with `on:change` (bool).
- Radio group with multiple buttons.
- Range slider with `min`/`max`/`step` and `on:change` (f64).
- Number input with `min`/`max`.
- Date input (text-entry v1).
- File picker with `on:change` (`Vec<PathBuf>`).
- Submit button.
- Select wrapped in a `<label>` with reactive `value` and `on:change`.
- `tabindex` for focus order.

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
    let (text, set_text) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (checked, set_checked) = create_signal(false);
    let (radio_val, set_radio) = create_signal(0i32);
    let (slider, set_slider) = create_signal(50.0f64);
    let (sel_val, set_sel_val) = create_signal("a".to_string());
    let (number_val, set_number) = create_signal(String::new());
    let (date_val, set_date) = create_signal(String::new());
    let sr0 = set_radio.clone();
    let sr0b = set_radio.clone();
    let scb = set_checked.clone();
    let sr1 = set_radio.clone();
    let sr2 = set_radio.clone();

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] w-[600px] h-[700px] text-white overflow-y-auto">
            <span class="text-lg font-bold">{"vgui <input> demo"}</span>

            // Text input with live mirror
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Text (on:input)"}</span>
                <input
                    type="text"
                    placeholder="Type here..."
                    on:input={move |v: &str, cx: &mut App| set_text.set(cx, v.to_string())}
                    tabindex={0}
                />
                <span class="text-sm text-[#0f0]">{format!("echo: \"{}\"", text.get())}</span>
            </div>

            // Password
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Password"}</span>
                <input
                    type="password"
                    placeholder="secret"
                    on:input={move |v: &str, cx: &mut App| set_password.set(cx, v.to_string())}
                    tabindex={0}
                />
                <span class="text-sm text-[#0f0]">{format!("len: {} chars", password.get().chars().count())}</span>
            </div>

            // Labeled text input (for attribute)
            <div class="flex flex-col gap-1">
                <label for="username" class="text-sm text-[#888]">{"Username"}</label>
                <input type="text" id="username" placeholder="Enter username" tabindex={0} />
            </div>

            // Wrapping label with text input
            <label class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Wrapped input"}</span>
                <input type="text" placeholder="Click label to focus" tabindex={0} />
            </label>

            // Checkbox
            <div class="flex flex-row gap-2 items-center">
                <input
                    type="checkbox"
                    checked={checked.get()}
                    on:change={move |v: bool, cx: &mut App| set_checked.set(cx, v)}
                    tabindex={-1}
                />
                <span class="text-sm">{format!("checkbox: {}", if checked.get() { "on" } else { "off" })}</span>
            </div>

            // Radio group
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Radio group"}</span>
                <div class="flex flex-row gap-4 items-center">
                    <div class="flex flex-row gap-1 items-center">
                        <input type="radio" checked={radio_val.get() == 0} on:change={move |_v: bool, cx: &mut App| sr0.set(cx, 0)} />
                        <span class="text-sm">{"A"}</span>
                    </div>
                    <div class="flex flex-row gap-1 items-center">
                        <input type="radio" checked={radio_val.get() == 1} on:change={move |_v: bool, cx: &mut App| sr1.set(cx, 1)} />
                        <span class="text-sm">{"B"}</span>
                    </div>
                    <div class="flex flex-row gap-1 items-center">
                        <input type="radio" checked={radio_val.get() == 2} on:change={move |_v: bool, cx: &mut App| sr2.set(cx, 2)} />
                        <span class="text-sm">{"C"}</span>
                    </div>
                </div>
                <span class="text-sm text-[#0f0]">{format!("selected: {}", radio_val.get())}</span>
            </div>

            // Range slider
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Range slider"}</span>
                <input
                    type="range"
                    min={0.0f64}
                    max={100.0f64}
                    step={1.0f64}
                    value={slider.get()}
                    on:change={move |v: f64, cx: &mut App| set_slider.set(cx, v)}
                    tabindex={1}
                />
                <span class="text-sm text-[#0f0]">{format!("value: {:.1}", slider.get())}</span>
            </div>

            // Number input
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Number (min 0, max 100)"}</span>
                <input
                    type="number"
                    min={0.0f64}
                    max={100.0f64}
                    placeholder="42"
                    on:input={move |v: &str, cx: &mut App| set_number.set(cx, v.to_string())}
                    tabindex={0}
                />
                <span class="text-sm text-[#0f0]">{format!("number: \"{}\"", number_val.get())}</span>
            </div>

            // Date input (text-entry v1)
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Date (YYYY-MM-DD)"}</span>
                <input
                    type="date"
                    placeholder="2026-01-15"
                    on:input={move |v: &str, cx: &mut App| set_date.set(cx, v.to_string())}
                    tabindex={0}
                />
                <span class="text-sm text-[#0f0]">{format!("date: \"{}\"", date_val.get())}</span>
            </div>

            // File input
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"File picker"}</span>
                <input
                    type="file"
                    value="Choose file..."
                    on:change={move |paths: Vec<std::path::PathBuf>, _cx: &mut App| {
                        if let Some(p) = paths.first() {
                            eprintln!("file selected: {:?}", p);
                        }
                    }}
                />
            </div>

            // Submit button
            <input type="submit" value="Submit" on:click={click(move |_cx| eprintln!("submit clicked"))} />

            // Hidden input (renders nothing)
            <input type="hidden" value="invisible" />

            // Wrapping label with checkbox (click label to focus checkbox)
            <label class="flex flex-row gap-2 items-center">
                <input type="checkbox" checked={checked.get()} on:change={move |v: bool, cx: &mut App| scb.set(cx, v)} />
                <span class="text-sm">{"Wrapped checkbox"}</span>
            </label>

            // Wrapping label with radio
            <label class="flex flex-row gap-2 items-center">
                <input type="radio" checked={radio_val.get() == 0} on:change={move |_v: bool, cx: &mut App| sr0b.set(cx, 0)} />
                <span class="text-sm">{"Wrapped radio A"}</span>
            </label>

            // Wrapping label with select
            <label class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Wrapped select"}</span>
                <select options={vec![("a".to_string(), "Apple".to_string()), ("b".to_string(), "Banana".to_string())]} value={sel_val.get()} on:change={move |v: &str, cx: &mut App| set_sel_val.set(cx, v.to_string())} />
            </label>

            <span class="text-sm text-[#0f0]">{format!("select: {}", sel_val.get())}</span>

            // Wrapping label with file input
            <label class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Wrapped file input"}</span>
                <input type="file" value="Choose file..." on:change={move |paths: Vec<std::path::PathBuf>, _cx: &mut App| { if let Some(p) = paths.first() { eprintln!("file: {:?}", p); } }} />
            </label>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family="wasm"))]
    let gpui_app = application();

    #[cfg(target_family="wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(700.0)), cx);
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

### Event handler signatures per input type

| Input type    | Event       | Handler signature                          |
| ------------- | ----------- | ------------------------------------------ |
| text/password/number/date | `on:input` | `FnMut(&str, &mut App)`          |
| checkbox/radio | `on:change` | `FnMut(bool, &mut App)`                  |
| range         | `on:change` | `FnMut(f64, &mut App)`                     |
| file          | `on:change` | `FnMut(Vec<PathBuf>, &mut App)`            |

### `tabindex` for focus navigation

`tabindex={0}` includes the element in the tab cycle; `tabindex={-1}` makes it
focusable but not tab-reachable; `tabindex={1}` gives priority. `VguiRoot`
handles `Tab`/`Shift+Tab` for focus cycling.

### Label association

Two forms are demonstrated: explicit `<label for="username">` with a matching
`id="username"` on the input, and a wrapping `<label>` that collects the first
focusable child.

### Running

**Native:**

```bash
cargo run -p vgui-inputs
```

**Web (WASM):**

```bash
cargo build --target wasm32-unknown-unknown -p vgui-inputs --release
wasm-bindgen --target web --out-dir examples/inputs/dist \
    --no-typescript target/wasm32-unknown-unknown/release/inputs.wasm
python3 scripts/serve_plain.py 8080 examples/inputs
```
