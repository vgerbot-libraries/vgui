# Variants Example

## Live Demo

<iframe src="../wasm/variants/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Source Code

```rust
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;
use vgui::for_each;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

// Define the `Button` variant system: a base style plus two dimensions
// (`variant` for color and `size` for padding/font-size). The macro generates
// `ButtonVariant`, `ButtonSize`, and `ButtonVariants` (a `Copy` struct that
// implements `ApplyStyle`).
variants! {
    Button {
        base => css! {
            border-radius: 6px;
            border-width: 0px;
            cursor: pointer;
            font-family: inherit;
        },

        variant {
            primary => css! { background: #2563ff; color: #fff; },
            secondary => css! { background: #6c757d; color: #fff; },
            danger => css! { background: #dc2626; color: #fff; },
            outline => css! { background: #1a1a2e; color: #2563ff; border: 2px solid #2563ff; },
        },

        size {
            sm => css! { padding: 4px 10px; font-size: 12px; },
            md => css! { padding: 8px 16px; font-size: 14px; },
            lg => css! { padding: 12px 22px; font-size: 16px; },
        },
    }
}

/// A reusable button component driven by the generated variant types.
pub struct Button {
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Button {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        // Compose the selected options into a single `ButtonVariants` value;
        // `style={variants}` applies base + each dimension sequentially.
        let variants = ButtonVariants::default()
            .variant(self.variant)
            .size(self.size);
        let on_click = self.on_click;
        let children = self.children;
        view! {
            <button style={variants} on:click={click(move |cx| on_click(cx))}>
                {for_each(children, |c, _| c)}
            </button>
        }
        .into_any_element()
    }
}

fn btn(
    variant: ButtonVariant,
    size: ButtonSize,
    label: &str,
    on_click: impl Fn(&mut gpui::App) + 'static,
) -> Button {
    Button {
        variant,
        size,
        on_click: Box::new(on_click),
        children: vec![label.to_string().into_any_element()],
    }
}

fn label_name(variant: ButtonVariant, size: ButtonSize) -> String {
    let v = match variant {
        ButtonVariant::Primary => "Primary",
        ButtonVariant::Secondary => "Secondary",
        ButtonVariant::Danger => "Danger",
        ButtonVariant::Outline => "Outline",
    };
    let s = match size {
        ButtonSize::Sm => "sm",
        ButtonSize::Md => "md",
        ButtonSize::Lg => "lg",
    };
    format!("{v} · {s}")
}

fn row(size: ButtonSize, set: WriteSignal<u32>) -> impl gpui::IntoElement {
    let s1 = set.clone();
    let s2 = set.clone();
    let s3 = set.clone();
    let s4 = set.clone();
    view! {
        <div style={css! {
            display: flex;
            flex-direction: row;
            gap: 12px;
            align-items: center;
        }}>
            {btn(ButtonVariant::Primary, size, &label_name(ButtonVariant::Primary, size), move |cx| s1.update(cx, |n| *n += 1))}
            {btn(ButtonVariant::Secondary, size, &label_name(ButtonVariant::Secondary, size), move |cx| s2.update(cx, |n| *n += 1))}
            {btn(ButtonVariant::Danger, size, &label_name(ButtonVariant::Danger, size), move |cx| s3.update(cx, |n| *n += 1))}
            {btn(ButtonVariant::Outline, size, &label_name(ButtonVariant::Outline, size), move |cx| s4.update(cx, |n| *n += 1))}
        </div>
    }
}

fn app() -> impl gpui::IntoElement {
    let (clicks, set_clicks) = create_signal(0u32);
    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            gap: 16px;
            padding: 24px;
            background: #1a1a2e;
        }}>
            <h2 style={css! {
                color: #fff;
                font-size: 18px;
                margin: 0;
            }}>
                {format!("Component variants — total clicks: {}", clicks.get())}
            </h2>
            {row(ButtonSize::Sm, set_clicks.clone())}
            {row(ButtonSize::Md, set_clicks.clone())}
            {row(ButtonSize::Lg, set_clicks.clone())}
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(720.), px(420.0)), cx);
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

### `variants!` macro

The `variants!` macro declares a component variant system with a `base` style
plus one or more dimensions. Each dimension is an enum whose variants carry
`css!` styles. The macro generates:

- `ButtonVariant` — enum for the `variant` dimension (primary, secondary,
  danger, outline).
- `ButtonSize` — enum for the `size` dimension (sm, md, lg).
- `ButtonVariants` — a `Copy` struct that implements `ApplyStyle`, composing
  the base style with one option per dimension.

### Custom `Button` component

`Button` is a plain struct implementing `gpui::IntoElement`. Its `into_element`
method composes a `ButtonVariants` value from the selected `variant` and `size`,
then passes it as `style={variants}` — the `ApplyStyle` trait applies the base
style followed by each dimension's style sequentially.

### `btn()` helper

The `btn()` function constructs a `Button` with a label string and click
handler, returning the `Button` struct directly (which implements
`IntoElement`).

### `for_each` for children

`for_each(children, |c, _| c)` iterates the children vector inside the
`view!` macro, rendering each child element.

### Click counter signal

A `create_signal(0u32)` tracks total clicks across all buttons. Each button's
`on_click` closure increments the shared signal, and the header displays the
running count reactively.

### Running

**Native:**

```bash
cargo run -p vgui-variants
```

**Web (WASM):**

```bash
cargo build --target wasm32-unknown-unknown -p vgui-variants --release
wasm-bindgen --target web --out-dir examples/variants/dist \
    --no-typescript target/wasm32-unknown-unknown/release/variants.wasm
python3 scripts/serve_plain.py 8080 examples/variants
```
