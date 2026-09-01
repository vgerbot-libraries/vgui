# Inputs Example

## Live Demo

<iframe src="../wasm/inputs/" width="100%" height="700" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The inputs example is a comprehensive tour of every form control `vgui` supports. It demonstrates:

- Text, password, number, and date inputs with `on:input` handlers.
- Checkbox and radio inputs with `on:change` handlers.
- Range slider with `on:change` returning an `f64`.
- File picker with `on:change` returning `Vec<PathBuf>`.
- `<select>` with grouped options via the `groups` prop.
- Multiple select with comma-separated values.
- `<select>` with a custom child closure for per-option rendering.
- `tabindex` for focus ordering.
- Label association via the `for` attribute and wrapping `<label>` elements.

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
    let (text, set_text) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (checked, set_checked) = create_signal(false);
    let (radio_val, set_radio) = create_signal(0i32);
    let (slider, set_slider) = create_signal(50.0f64);
    let (sel_val, set_sel_val) = create_signal("a".to_string());
    let (number_val, set_number) = create_signal(String::new());
    let (date_val, set_date) = create_signal(String::new());
    let (group_sel, set_group_sel) = create_signal("apple".to_string());
    let (multi_sel, set_multi_sel) = create_signal("1,3".to_string());
    let (custom_sel, set_custom_sel) = create_signal("1".to_string());
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

            <hr />

            // ── Select with grouped options ──────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Select with groups"}</span>
                <select
                    groups={vec![
                        ("Fruits".to_string(), vec![
                            ("apple".to_string(), "Apple".to_string()),
                            ("banana".to_string(), "Banana".to_string()),
                        ]),
                        ("Vegetables".to_string(), vec![
                            ("carrot".to_string(), "Carrot".to_string()),
                            ("daikon".to_string(), "Daikon".to_string()),
                        ]),
                    ]}
                    value={group_sel.get()}
                    on:change={move |v: &str, cx: &mut App| set_group_sel.set(cx, v.to_string())}
                />
                <span class="text-sm text-[#0f0]">{format!("grouped select: {}", group_sel.get())}</span>
            </div>

            // ── Multiple select ──────────────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Multiple select (comma-separated values)"}</span>
                <select
                    options={vec![
                        ("1".to_string(), "One".to_string()),
                        ("2".to_string(), "Two".to_string()),
                        ("3".to_string(), "Three".to_string()),
                    ]}
                    value={multi_sel.get()}
                    multiple={true}
                    on:change={move |v: &str, cx: &mut App| set_multi_sel.set(cx, v.to_string())}
                />
                <span class="text-sm text-[#0f0]">{format!("multi select: {}", multi_sel.get())}</span>
            </div>

            // ── Select with custom child closure ─────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Select with custom option rendering"}</span>
                <select
                    options={vec![
                        ("1".to_string(), "One".to_string()),
                        ("2".to_string(), "Two".to_string()),
                        ("3".to_string(), "Three".to_string()),
                    ]}
                    value={custom_sel.get()}
                    on:change={move |v: &str, cx: &mut App| set_custom_sel.set(cx, v.to_string())}
                >
                    {move |value: &str, label: &str| view! {
                        <div class="flex items-center gap-2">
                            <span class="text-[#0f0]">{value.to_string()}</span>
                            <span>{label.to_string()}</span>
                        </div>
                    }}
                </select>
                <span class="text-sm text-[#0f0]">{format!("custom select: {}", custom_sel.get())}</span>
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

### Event handler signatures per input type

Each input type delivers a different value type to its event handler. The closure
signature must match the value type the element emits:

| Input type | Event | Handler value type | Example |
|---|---|---|---|
| `text` / `password` / `number` / `date` | `on:input` | `&str` | `move \|v: &str, cx: &mut App\| set_text.set(cx, v.to_string())` |
| `checkbox` / `radio` | `on:change` | `bool` | `move \|v: bool, cx: &mut App\| set_checked.set(cx, v)` |
| `range` | `on:change` | `f64` | `move \|v: f64, cx: &mut App\| set_slider.set(cx, v)` |
| `file` | `on:change` | `Vec<PathBuf>` | `move \|paths: Vec<PathBuf>, _cx: &mut App\| { ... }` |
| `select` | `on:change` | `&str` | `move \|v: &str, cx: &mut App\| set_sel.set(cx, v.to_string())` |
| `submit` | `on:click` | — (use `click`) | `click(move \|_cx\| { ... })` |

Text-entry inputs (`text`, `password`, `number`, `date`) fire `on:input` on every
keystroke with the current string value. Toggle inputs (`checkbox`, `radio`) and
`range` fire `on:change` with the new typed value. The `file` input fires
`on:change` with the full list of selected paths.

### `tabindex`

The `tabindex` attribute controls focus order. `tabindex={0}` places the element
in the natural tab order. `tabindex={1}` (the range slider) moves it ahead of
default-order elements. `tabindex={-1}` (the standalone checkbox) removes it from
sequential tab navigation while keeping it focusable programmatically.

### Label association

Two patterns associate a `<label>` with its control:

1. **`for` attribute** — `<label for="username">` references an input by its
   `id="username"`. Clicking the label focuses the input.
2. **Wrapping label** — The control is nested inside `<label>...</label>`. The
   example wraps text inputs, checkboxes, radios, selects, and file inputs this
   way. Clicking anywhere on the label text focuses or toggles the wrapped
   control.

### Select with grouped options

The `groups` prop accepts a `Vec<(String, Vec<(String, String)>)>` — a list of
`(group_label, options)` pairs. Each group renders as an `<optgroup>` with its
label and nested options. The `value` prop selects the active option and
`on:change` fires with the chosen option's value string.

### Multiple select

Setting `multiple={true}` enables multi-selection. The `value` prop holds the
currently selected values as a comma-separated string (e.g. `"1,3"`), and
`on:change` delivers the updated comma-separated string.

### Select with custom child closure

A `<select>` may take a child closure `{move |value: &str, label: &str| view! { ... }}`
that renders each option. The closure receives the option's value and label and
returns an element, allowing rich per-option layouts (icons, badges, multi-line
content) instead of plain text.

### Running

**Native:**

    cargo run -p vgui-inputs

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-inputs --release
    wasm-bindgen --target web --out-dir examples/inputs/dist \
        --no-typescript target/wasm32-unknown-unknown/release/inputs.wasm
    python3 scripts/serve_plain.py 8080 examples/inputs
