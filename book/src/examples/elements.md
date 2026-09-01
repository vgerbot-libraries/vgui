# HTML Elements Example

## Live Demo

<iframe src="../wasm/elements/" width="100%" height="700" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The HTML elements example renders a broad swath of the HTML tag surface that
`vgui` supports, side by side on one page. It demonstrates:

- `h1`–`h6` headings.
- Text formatting: `strong`, `em`, `u`, `s`, `mark`, `code`, `small`.
- Lists: `ul`, `ol`, `dl` (with `dt`/`dd`).
- Semantic tags: `header`, `nav`, `main`, `section`, `article`, `aside`, `footer`.
- Links via `<a>` with `on:click`.
- CSS properties applied through the `css!` macro.
- Tailwind utility classes for text overflow, font families, leading, and
  decoration.
- `progress` and `meter` gauges.
- `textarea` with `on:input`.
- `select` with `options` and `on:change`.
- `details`/`summary` toggling.
- `dialog` with `open` and `on:close`.
- Tables with `thead`/`tbody` and `colspan`.

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
    let (open, set_open) = create_signal(false);
    let (text, set_text) = create_signal("Hello".to_string());
    let (show_dialog, set_show_dialog) = create_signal(false);
    let (sel_val, set_sel_val) = create_signal("1".to_string());
    let dismiss_dialog = set_show_dialog.clone();
    let close_dialog_btn = set_show_dialog.clone();

    view! {
        <div class="flex flex-col gap-2 p-4 bg-[#1a1a2e] w-[600px] h-[700px] text-white overflow-y-auto">
            <h1>{"Heading 1"}</h1>
            <h2>{"Heading 2"}</h2>
            <h3>{"Heading 3"}</h3>
            <h4>{"Heading 4"}</h4>
            <h5>{"Heading 5"}</h5>
            <h6>{"Heading 6"}</h6>

            <hr />

            <p>{"Normal paragraph"}</p>
            <strong>{"Bold text"}</strong>
            <em>{"Italic text"}</em>
            <u>{"Underlined"}</u>
            <s>{"Strikethrough"}</s>
            <mark>{"Highlighted"}</mark>
            <code>{"monospace"}</code>
            <small>{"small text"}</small>

            <hr />

            <ul>
                <li>{"Item 1"}</li>
                <li>{"Item 2"}</li>
            </ul>
            <ol>
                <li>{"First"}</li>
                <li>{"Second"}</li>
            </ol>
            <dl>
                <dt>{"Term"}</dt>
                <dd>{"Definition"}</dd>
            </dl>

            <hr />

            <header>{"Header"}</header>
            <nav>{"Nav"}</nav>
            <main>{"Main content"}</main>
            <section>{"Section"}</section>
            <article>{"Article"}</article>
            <aside>{"Aside"}</aside>
            <footer>{"Footer"}</footer>

            <hr />

            <a on:click={click(move |_cx| {})}>{"Click link"}</a>

            <hr />

            <div style={css! {
                font-family: monospace;
                text-overflow: ellipsis;
                text-decoration-color: #ff0000;
                text-decoration-thickness: 2px;
                text-decoration-style: wavy;
                text-background: #ffff00;
                scrollbar-width: thin;
                background: linear-gradient(90deg, #ff0000, #0000ff);
                white-space: nowrap;
                overflow: hidden;
            }}>
                <span>{"CSS properties test"}</span>
            </div>

            <hr />

            <div class="truncate font-mono leading-none">{"Truncated monospace text with leading-none"}</div>
            <div class="text-ellipsis font-serif leading-loose">{"Ellipsis serif with leading-loose"}</div>
            <div class="underline decoration-wavy decoration-2">{"Wavy underline thickness 2"}</div>

            <hr />

            <progress value={0.5f64} max={1.0f64} />

            <meter value={0.7f64} max={1.0f64} />

            <hr />

            <textarea
                placeholder="Enter text"
                value={text.get()}
                on:input={move |v: &str, cx: &mut App| set_text.set(cx, v.to_string())}
            />

            <hr />

            <select
                options={vec![("1".to_string(), "One".to_string()), ("2".to_string(), "Two".to_string())]}
                value={sel_val.get()}
                on:change={move |v: &str, cx: &mut App| set_sel_val.set(cx, v.to_string())}
            />
            <hr />

            <details open={open.get()}>
                <summary on:click={click(move |cx| set_open.update(cx, |v| *v = !*v))}>
                    {"Click to toggle"}
                </summary>
                <div>{"Hidden content"}</div>
            </details>

            <hr />

            <button on:click={click(move |cx| set_show_dialog.set(cx, true))}>
                {"Open Dialog"}
            </button>
            <dialog open={show_dialog.get()} on:close={move |cx| dismiss_dialog.set(cx, false)}>
                <div class="bg-white p-4 rounded text-black">
                    <p>{"Dialog content — click outside or press Escape to close."}</p>
                    <button on:click={click(move |cx| close_dialog_btn.set(cx, false))}>
                        {"Close"}
                    </button>
                </div>
            </dialog>

            <div on:modifiers_changed={move |_e, _w, _cx| {}} on:any_mouse_down={move |_e, _w, _cx| {}}>
                {"Events test"}
            </div>

            <hr />

            <table class="w-full">
                <thead>
                    <tr class="bg-[#333]">
                        <th class="p-2 text-white">{"Name"}</th>
                        <th class="p-2 text-white">{"Age"}</th>
                        <th class="p-2 text-white">{"City"}</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td class="p-2">{"Alice"}</td>
                        <td class="p-2">{"30"}</td>
                        <td class="p-2">{"Beijing"}</td>
                    </tr>
                    <tr>
                        <td class="p-2" colspan={2u32}>{"Bob (spanned 2 cols)"}</td>
                        <td class="p-2">{"Shanghai"}</td>
                    </tr>
                </tbody>
            </table>
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

### HTML tag coverage

`vgui` maps a large subset of HTML elements directly to `gpui` primitives. This
example exercises them all in one scrollable column:

- **Headings** — `<h1>` through `<h6>` render at decreasing sizes.
- **Text formatting** — `<strong>`, `<em>`, `<u>`, `<s>`, `<mark>`, `<code>`,
  and `<small>` apply their conventional visual styling inline.
- **Lists** — `<ul>`/`<li>` for bulleted lists, `<ol>`/`<li>` for numbered
  lists, and `<dl>` with `<dt>`/`<dd>` for description lists.
- **Semantic structure** — `<header>`, `<nav>`, `<main>`, `<section>`,
  `<article>`, `<aside>`, and `<footer>` render as block containers.
- **Links** — `<a>` takes an `on:click` handler; the example wires it to a
  no-op closure demonstrating the pattern.
- **Tables** — `<table>` with `<thead>`/`<tbody>`/`<tr>`/`<th>`/`<td>`. The
  `colspan={2u32}` attribute on a `<td>` spans it across two columns.

### CSS properties via the `css!` macro

The `style={css! { ... }}` attribute applies raw CSS declarations to an element.
The example sets `font-family`, `text-overflow`, `text-decoration-*`, gradient
`background`, `white-space`, and `overflow` — properties that are awkward or
impossible to express as Tailwind utilities. The `css!` macro emits a typed
`StyleRefinement` that merges with any `class=` utilities on the same element.

### Tailwind utility classes

Three `<div>` elements demonstrate utility-class equivalents of the CSS above:
`truncate` (`overflow-hidden` + `text-ellipsis` + `whitespace-nowrap`),
`font-mono`/`font-serif` for font families, `leading-none`/`leading-loose` for
line height, and `underline decoration-wavy decoration-2` for text decoration.
Arbitrary values and standard utilities compose freely in a single `class=`
string.

### Interactive elements

- **`<details>`/`<summary>`** — The `open` prop is bound to a signal. Clicking
  the `<summary>` toggles the signal via `on:click`, which reactively opens or
  closes the hidden content block.
- **`<dialog>`** — The `open` prop is signal-driven. The `on:close` handler
  fires when the dialog is dismissed (Escape key or click-outside), resetting
  the signal to `false`. A manual close button sets the same signal directly.

### Running

**Native:**

    cargo run -p vgui-elements

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-elements --release
    wasm-bindgen --target web --out-dir examples/elements/dist \
        --no-typescript target/wasm32-unknown-unknown/release/elements.wasm
    python3 scripts/serve_plain.py 8080 examples/elements
