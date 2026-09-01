# Forms Example

## Live Demo

<iframe src="../wasm/forms/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The forms example demonstrates end-to-end form handling in `vgui`. It shows:

- `<form>` with `on:submit` and `on:reset` handlers.
- Text, email, and number `<input>` types with `on:input` value binding.
- `<select>` with an `options` vector of `(value, label)` pairs.
- Checkbox `<input>` with a boolean `on:change` handler.
- Submit and reset `<input>` buttons.
- `<Show>` for conditional display of the submitted data.
- Enter-to-submit: pressing Enter in a text input inside `<form>` fires `on:submit`.

## Source Code

```rust
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

#[derive(Clone, PartialEq)]
struct FormData {
    name: String,
    email: String,
    age: String,
    country: String,
    subscribe: bool,
}

fn app() -> impl gpui::IntoElement {
    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (age, set_age) = create_signal(String::new());
    let (country, set_country) = create_signal("cn".to_string());
    let (subscribe, set_subscribe) = create_signal(false);
    let (submitted, set_submitted) = create_signal(Option::<FormData>::None);

    // Clones for the on:submit closure (the view! attributes below
    // consume separate clones).
    let s_name = name.clone();
    let s_email = email.clone();
    let s_age = age.clone();
    let s_country = country.clone();
    let s_subscribe = subscribe.clone();
    let r_name = set_name.clone();
    let r_email = set_email.clone();
    let r_age = set_age.clone();
    let r_country = set_country.clone();
    let r_subscribe = set_subscribe.clone();
    let r_submitted = set_submitted.clone();
    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] text-white" style={css!{ width: 500px; height: 600px; overflow-y: auto; }}>
            <h2 class="text-lg font-bold">{"Form Example"}</h2>
            <form
                on:submit={move |cx: &mut App| {
                    set_submitted.set(cx, Some(FormData {
                        name: s_name.get(),
                        email: s_email.get(),
                        age: s_age.get(),
                        country: s_country.get(),
                        subscribe: s_subscribe.get(),
                    }));
                }}
                on:reset={move |cx: &mut App| {
                    r_name.set(cx, String::new());
                    r_email.set(cx, String::new());
                    r_age.set(cx, String::new());
                    r_country.set(cx, "cn".to_string());
                    r_subscribe.set(cx, false);
                    r_submitted.set(cx, None);
                }}
            >
                <div class="flex flex-col gap-3">
                    <label class="flex flex-col gap-1">
                        <span class="text-sm text-[#888]">{"Name"}</span>
                        <input
                            type="text"
                            placeholder="Enter your name"
                            value={name.get()}
                            on:input={move |v: &str, cx: &mut App| set_name.set(cx, v.to_string())}
                        />
                    </label>

                    <label class="flex flex-col gap-1">
                        <span class="text-sm text-[#888]">{"Email"}</span>
                        <input
                            type="email"
                            placeholder="you@example.com"
                            value={email.get()}
                            on:input={move |v: &str, cx: &mut App| set_email.set(cx, v.to_string())}
                        />
                    </label>

                    <label class="flex flex-col gap-1">
                        <span class="text-sm text-[#888]">{"Age"}</span>
                        <input
                            type="number"
                            min={0.0f64}
                            max={150.0f64}
                            placeholder="30"
                            value={age.get()}
                            on:input={move |v: &str, cx: &mut App| set_age.set(cx, v.to_string())}
                        />
                    </label>

                    <div class="flex flex-col gap-1">
                        <span class="text-sm text-[#888]">{"Country"}</span>
                        <select
                            options={vec![
                                ("cn".to_string(), "China".to_string()),
                                ("us".to_string(), "United States".to_string()),
                                ("jp".to_string(), "Japan".to_string()),
                            ]}
                            value={country.get()}
                            on:change={move |v: &str, cx: &mut App| set_country.set(cx, v.to_string())}
                        />
                    </div>

                    <label class="flex flex-row gap-2 items-center">
                        <input
                            type="checkbox"
                            checked={subscribe.get()}
                            on:change={move |v: bool, cx: &mut App| set_subscribe.set(cx, v)}
                        />
                        <span class="text-sm">{"Subscribe to newsletter"}</span>
                    </label>

                    <div class="flex flex-row gap-3">
                        <input type="submit" value="Submit" class="px-4 py-2 bg-[#2563ff] text-white rounded cursor-pointer" />
                        <input type="reset" value="Reset" class="px-4 py-2 bg-[#6c757d] text-white rounded cursor-pointer" />
                    </div>
                </div>
            </form>

            <Show when={submitted.get().is_some()}>
                <div class="bg-[#2d2d44] p-4 rounded-lg">
                    <span class="text-sm font-bold">{"Submitted data:"}</span>
                    {if let Some(data) = submitted.get() {
                        view! {
                            <div class="text-sm text-[#0f0] mt-2">
                                <div>{format!("Name: {}", data.name)}</div>
                                <div>{format!("Email: {}", data.email)}</div>
                                <div>{format!("Age: {}", data.age)}</div>
                                <div>{format!("Country: {}", data.country)}</div>
                                <div>{format!("Subscribed: {}", data.subscribe)}</div>
                            </div>
                        }.into_any_element()
                    } else {
                        gpui::div().into_any_element()
                    }}
                </div>
            </Show>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(600.0)), cx);
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

### `on:submit` and `on:reset` handlers

The `<form>` element carries two event handlers:

- `on:submit` fires when the form is submitted — either by clicking the
  `<input type="submit">` button or by pressing Enter inside a text input. The
  handler reads every field signal and packs them into a `FormData` struct,
  storing it in the `submitted` signal.
- `on:reset` fires when the `<input type="reset">` button is clicked. It resets
  every field signal back to its default and clears `submitted` to `None`.

Both handlers take `move |cx: &mut App| { ... }`. Because `view!` attributes
consume the closures, the read and write signals are cloned into `s_*` and `r_*`
bindings before the `view!` macro so each closure captures its own owned clone.

### Enter-to-submit

Text inputs (`type="text"`, `type="email"`, `type="number"`) placed inside a
`<form>` automatically submit the form when the user presses Enter. No extra
keydown handler is needed — the platform form semantics handle it. The
`on:submit` handler then collects the current signal values into `FormData`.

### `<Show>` with `is_some()`

The submitted-data card is wrapped in `<Show when={submitted.get().is_some()}>`.
When `submitted` is `None` (before the first submit, or after a reset) the card
is not rendered at all. Inside the `<Show>`, an `if let Some(data) =
submitted.get()` block extracts the struct and renders each field; the `else`
branch returns `gpui::div().into_any_element()` as a type-compatible fallback
(never reached because `<Show>` already gated on `is_some()`).

### `FormData` with `Clone` + `PartialEq`

The `FormData` struct derives `Clone` and `PartialEq`. `Clone` is required so the
struct can be moved into signals and read back; `PartialEq` lets the reactive
system skip re-renders when the submitted data is unchanged. The struct holds
plain `String` and `bool` fields — one per form field — making it a faithful
snapshot of the form state at submit time.

### Input types and value binding

Each input binds its `value` to a signal read (`value={name.get()}`) and updates
the signal on every keystroke via `on:input={move |v: &str, cx: &mut App|
set_name.set(cx, v.to_string())}`. The `<select>` uses the `options` prop with a
`Vec<(String, String)>` of `(value, label)` pairs and `on:change` for selection
changes. The checkbox uses `checked={subscribe.get()}` with a boolean
`on:change={move |v: bool, cx: &mut App| set_subscribe.set(cx, v)}`.

## Running

**Native:**

    cargo run -p vgui-forms

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-forms --release
    wasm-bindgen --target web --out-dir examples/forms/dist \
        --no-typescript target/wasm32-unknown-unknown/release/forms.wasm
    python3 scripts/serve_plain.py 8080 examples/forms
